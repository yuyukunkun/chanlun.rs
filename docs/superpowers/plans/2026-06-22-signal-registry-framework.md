# 子项目1 信号注册框架 实现计划

> **面向 AI 代理的工作者：** 必需子技能：使用 superpowers:subagent-driven-development（推荐）或 superpowers:executing-plans 逐任务实现此计划。步骤使用复选框（`- [ ]`）语法来跟踪进度。

**目标：** 用 `#[signal]` proc-macro + `inventory` 编译期注册表替代 Python 的 `import_by_name` 动态导入和 `SignalsParser` docstring 解析，提供「信号名 → 函数指针」O(1) 查表。

**架构：** 新建独立 proc-macro crate `chanlun-signal-macros`（`#[signal(name, template)]` 属性宏，emit `crate::signal::registry::` 路径）；核心 crate `chanlun` 新增 `signal/registry.rs`（描述符类型 + `inventory` 归并 + 查询 API），并依赖宏 crate + `inventory`。信号函数签名 `fn(&观察者, &HashMap<String, Value>) -> Vec<Signal>`，无 TaCache（核心层 K线已挂指标）。

**技术栈：** Rust（edition 2024 / 宏 crate 2021）、`syn` 2 + `quote` + `proc-macro2`、`inventory` 0.3、`serde_json`。

**设计文档：** `docs/superpowers/specs/2026-06-22-signal-registry-framework-design.md`

---

## 文件结构

| 文件 | 职责 |
|---|---|
| `chanlun-signal-macros/Cargo.toml` | proc-macro crate 清单（`proc-macro = true` + syn/quote/proc-macro2） |
| `chanlun-signal-macros/src/lib.rs` | `#[signal(name, template)]` 属性宏 |
| `chanlun/Cargo.toml` | 新增 `inventory` + path 依赖 `chanlun-signal-macros` |
| `chanlun/src/signal/registry.rs` | `SignalFn`/`SignalDescriptor`/`SignalMeta`/`归并`/`SIGNAL_REGISTRY`/查询 API + 探针单测 |
| `chanlun/src/signal/mod.rs` | 增 `pub mod registry;` |
| `chanlun/tests/test_signal_registry.rs` | 端到端集成测试：`#[signal]` 贴探针函数 → 注册表命中（在 chanlun crate 内，因宏 emit `crate::` 路径） |

**测试归属说明**：`#[signal]` 宏 emit `crate::signal::registry::SignalDescriptor`，仅在 `chanlun` crate 内解析得了，故**宏的端到端测试放 `chanlun/tests/`，不放宏 crate**（放宏 crate 会循环依赖 chanlun）。宏 crate 自身只验证「能编译」。

---

## 任务 0：脚手架——proc-macro crate + 依赖接线

**文件：**
- 创建：`chanlun-signal-macros/Cargo.toml`、`chanlun-signal-macros/src/lib.rs`
- 修改：`chanlun/Cargo.toml`

- [ ] **步骤 1：创建宏 crate 清单**

创建 `chanlun-signal-macros/Cargo.toml`：

```toml
[package]
name = "chanlun-signal-macros"
version = "0.1.0"
edition = "2021"
license = "MIT"
description = "chanlun 信号注册 proc-macro（#[signal]）"

[lib]
proc-macro = true

[dependencies]
syn = { version = "2", features = ["full"] }
quote = "1"
proc-macro2 = "1"
```

- [ ] **步骤 2：创建宏 crate 占位实现**

创建 `chanlun-signal-macros/src/lib.rs`（占位，任务 2 填充真实逻辑）：

```rust
//! chanlun 信号注册 proc-macro。
//!
//! 第三方代码声明：`#[signal]` 注册机制参考 czsc 项目
//! （https://github.com/waditu/czsc，Apache License 2.0），已简化适配。

use proc_macro::TokenStream;

/// 占位——任务 2 实现真实的 #[signal] 属性宏。
#[proc_macro_attribute]
pub fn signal(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
```

- [ ] **步骤 3：chanlun 接线依赖**

修改 `chanlun/Cargo.toml` 的 `[dependencies]`，追加两行（放在 `sha2 = "0.10"` 之后）：

```toml
inventory = "0.3"
chanlun-signal-macros = { path = "../chanlun-signal-macros" }
```

- [ ] **步骤 4：验证两个 crate 都能构建**

运行：`cd /home/moscow/chanlun.rs/chanlun-signal-macros && cargo build`
预期：编译通过（占位宏）。

运行：`cd /home/moscow/chanlun.rs/chanlun && cargo build`
预期：编译通过（新增依赖，尚未使用，unused-dep 不会报错）。

- [ ] **步骤 5：Commit**

```bash
cd /home/moscow/chanlun.rs
git add chanlun-signal-macros chanlun/Cargo.toml
git commit -m "feat(signal-registry): 脚手架 — proc-macro crate + inventory 依赖"
```

---

## 任务 1：registry.rs —— 描述符类型 + 归并 + 查询 API

**文件：**
- 创建：`chanlun/src/signal/registry.rs`
- 修改：`chanlun/src/signal/mod.rs`

- [ ] **步骤 1：mod.rs 注册子模块**

修改 `chanlun/src/signal/mod.rs`，在 `pub mod signal;`（第 13 行）之后加一行：

```rust
pub mod registry;
```

- [ ] **步骤 2：编写 registry.rs（含 cargo 单测）**

创建 `chanlun/src/signal/registry.rs`（一字不差）：

```rust
//! 信号注册表 —— 编译期收集 `#[signal]` 注册的信号函数，运行时按名查表。
//!
//! 第三方代码声明：注册机制参考 czsc（https://github.com/waditu/czsc，
//! Apache License 2.0），已简化适配（无 category / TaCache）。

use crate::business::observer::观察者;
use crate::signal::Signal;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::LazyLock;

/// 信号函数签名 —— 读观察者状态（含 K线已挂指标）+ 参数 → 信号列表。无 TaCache。
pub type SignalFn = fn(&观察者, &HashMap<String, Value>) -> Vec<Signal>;

/// 信号描述符（编译期元数据，由 `#[signal]` 宏生成、`inventory` 收集）。
#[derive(Clone, Copy)]
pub struct SignalDescriptor {
    /// 信号函数名，如 "youwukuncheng_中枢第三买卖点_V230602"
    pub name: &'static str,
    /// 参数模板，如 "{freq}_D1MO{max_overlap}_中枢第三买卖点V230602"
    pub template: &'static str,
    /// 函数指针
    pub func: SignalFn,
}

inventory::collect!(SignalDescriptor);

/// 运行时信号元信息。
pub struct SignalMeta {
    pub func: SignalFn,
    pub template: &'static str,
}

/// 归并描述符为注册表；重名返回 Err（纯函数，便于单测）。
fn 归并(
    descs: impl Iterator<Item = SignalDescriptor>,
) -> Result<HashMap<&'static str, SignalMeta>, String> {
    let mut m: HashMap<&'static str, SignalMeta> = HashMap::new();
    for d in descs {
        if m
            .insert(d.name, SignalMeta { func: d.func, template: d.template })
            .is_some()
        {
            return Err(format!("信号重名：{}", d.name));
        }
    }
    Ok(m)
}

/// 全局注册表视图（由 inventory 归并；重名 panic，fail-fast）。
pub static SIGNAL_REGISTRY: LazyLock<HashMap<&'static str, SignalMeta>> = LazyLock::new(|| {
    归并(inventory::iter::<SignalDescriptor>.into_iter().copied())
        .unwrap_or_else(|e| panic!("{e}"))
});

/// 按名查信号元信息。
pub fn get_signal(name: &str) -> Option<&'static SignalMeta> {
    SIGNAL_REGISTRY.get(name)
}

/// 按名查参数模板。
pub fn get_template(name: &str) -> Option<&'static str> {
    SIGNAL_REGISTRY.get(name).map(|m| m.template)
}

/// 列出所有已注册信号名（排序）。
pub fn list_signal_names() -> Vec<&'static str> {
    let mut v: Vec<_> = SIGNAL_REGISTRY.keys().copied().collect();
    v.sort();
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 探针信号函数（最小签名实现，仅供测试归并/查表）。
    fn __probe(_obs: &观察者, _p: &HashMap<String, Value>) -> Vec<Signal> {
        Vec::new()
    }

    fn 描述符(name: &'static str) -> SignalDescriptor {
        SignalDescriptor { name, template: "{freq}_D1_probe", func: __probe }
    }

    #[test]
    fn test_归并_正常() {
        let m = 归并([描述符("a_V000001"), 描述符("b_V000001")].into_iter()).unwrap();
        assert_eq!(m.len(), 2);
        assert!(m.contains_key("a_V000001"));
        assert_eq!(m["a_V000001"].template, "{freq}_D1_probe");
    }

    #[test]
    fn test_归并_重名_返回Err() {
        let r = 归并([描述符("dup_V000001"), 描述符("dup_V000001")].into_iter());
        assert!(r.is_err());
        assert!(r.unwrap_err().contains("信号重名"));
    }
}

/// 测试用：通过 inventory 提交一个探针描述符，验证全局注册表能收到。
#[cfg(test)]
fn __probe_for_inventory(_obs: &观察者, _p: &HashMap<String, Value>) -> Vec<Signal> {
    Vec::new()
}

#[cfg(test)]
inventory::submit! {
    SignalDescriptor {
        name: "__probe_inventory_V000000",
        template: "{freq}_D1_probe_inventory",
        func: __probe_for_inventory as SignalFn,
    }
}

#[cfg(test)]
mod inventory_tests {
    use super::*;

    #[test]
    fn test_全局注册表收到inventory探针() {
        assert!(get_signal("__probe_inventory_V000000").is_some());
        assert_eq!(
            get_template("__probe_inventory_V000000"),
            Some("{freq}_D1_probe_inventory")
        );
        assert!(list_signal_names().contains(&"__probe_inventory_V000000"));
    }
}
```

- [ ] **步骤 3：运行测试**

运行：`cd /home/moscow/chanlun.rs/chanlun && cargo test signal::registry`
预期：3 个测试全 PASS（`test_归并_正常`、`test_归并_重名_返回Err`、`test_全局注册表收到inventory探针`）。

> 注：若 `inventory::iter::<SignalDescriptor>.into_iter().copied()` 因 inventory 0.3 API 细节编译报错，改为 `inventory::iter::<SignalDescriptor>().copied()` 或 `inventory::iter::<SignalDescriptor> {}`（参考 `/home/moscow/czsc/crates/czsc-signals/src/registry.rs:136` 的 `inventory::iter::<...>.into_iter().copied().collect()` 写法）。

- [ ] **步骤 4：Commit**

```bash
cd /home/moscow/chanlun.rs
git add chanlun/src/signal/registry.rs chanlun/src/signal/mod.rs
git commit -m "feat(signal-registry): registry.rs — 描述符/归并/查询 API + 探针测试"
```

---

## 任务 2：`#[signal]` 属性宏

**文件：**
- 修改：`chanlun-signal-macros/src/lib.rs`

- [ ] **步骤 1：实现 #[signal] 宏**

把 `chanlun-signal-macros/src/lib.rs` 全部内容替换为（一字不差）：

```rust
//! chanlun 信号注册 proc-macro。
//!
//! 第三方代码声明：`#[signal]` 注册机制参考 czsc 项目
//! （https://github.com/waditu/czsc，Apache License 2.0），已简化适配
//! （无 category / TaCache，签名固定为 fn(&观察者, &HashMap<String, Value>) -> Vec<Signal>）。

use proc_macro::TokenStream;
use quote::quote;
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::{Expr, ExprLit, ItemFn, Lit, Meta, Token};

/// `#[signal(name = "foo_V230101", template = "{freq}_D1_foo")]`
///
/// 校验：函数名含 `_V<数字>`；`name` 与函数名一致；`name`/`template` 非空。
/// 生成：一个 `static` SignalDescriptor + `inventory::submit!`，路径用 `crate::signal::registry::`。
#[proc_macro_attribute]
pub fn signal(attr: TokenStream, item: TokenStream) -> TokenStream {
    let parser = Punctuated::<Meta, Token![,]>::parse_terminated;
    let metas = match parser.parse(attr) {
        Ok(m) => m,
        Err(e) => return e.to_compile_error().into(),
    };

    let mut name: Option<String> = None;
    let mut template: Option<String> = None;
    for m in metas {
        if let Meta::NameValue(nv) = m
            && let Some(ident) = nv.path.get_ident()
            && let Expr::Lit(ExprLit { lit: Lit::Str(v), .. }) = nv.value
        {
            match ident.to_string().as_str() {
                "name" => name = Some(v.value()),
                "template" => template = Some(v.value()),
                _ => {}
            }
        }
    }

    let f: ItemFn = match syn::parse(item) {
        Ok(v) => v,
        Err(e) => return e.to_compile_error().into(),
    };

    let name = name.unwrap_or_default();
    let template = template.unwrap_or_default();
    let fn_ident = &f.sig.ident;
    let fn_name = fn_ident.to_string();

    let mut errors = Vec::new();
    if name.is_empty() || template.is_empty() {
        errors.push(quote! { compile_error!("#[signal] name/template 不能为空"); });
    }
    if name != fn_name {
        errors.push(quote! { compile_error!("#[signal] name 必须与函数名一致"); });
    }
    // 函数名须含 _V<数字>
    let 有版本 = fn_name
        .rsplit_once("_V")
        .map(|(_, v)| !v.is_empty() && v.chars().all(|c| c.is_ascii_digit()))
        .unwrap_or(false);
    if !有版本 {
        errors.push(quote! { compile_error!("#[signal] 函数名必须含 _V<版本号>，如 foo_V230101"); });
    }

    if !errors.is_empty() {
        let errs = errors.into_iter();
        return quote! { #(#errs)* }.into();
    }

    let descriptor_ident = syn::Ident::new(
        &format!("__SIG_DESC_{}", fn_name).to_uppercase(),
        fn_ident.span(),
    );

    let expanded = quote! {
        #f

        #[allow(non_upper_case_globals)]
        static #descriptor_ident: crate::signal::registry::SignalDescriptor =
            crate::signal::registry::SignalDescriptor {
                name: #name,
                template: #template,
                func: #fn_ident as crate::signal::registry::SignalFn,
            };

        inventory::submit! { #descriptor_ident }
    };
    expanded.into()
}
```

- [ ] **步骤 2：验证宏 crate 编译**

运行：`cd /home/moscow/chanlun.rs/chanlun-signal-macros && cargo build`
预期：编译通过。

- [ ] **步骤 3：Commit**

```bash
cd /home/moscow/chanlun.rs
git add chanlun-signal-macros/src/lib.rs
git commit -m "feat(signal-registry): #[signal] 属性宏 — 校验+生成描述符+提交"
```

---

## 任务 3：端到端集成测试（chanlun 内用 #[signal]）

**文件：**
- 创建：`chanlun/tests/test_signal_registry.rs`

- [ ] **步骤 1：编写集成测试**

创建 `chanlun/tests/test_signal_registry.rs`（一字不差）。它在 chanlun crate 内用 `#[signal]` 贴一个探针函数，验证宏 + 注册表端到端：

```rust
//! 端到端：#[signal] 宏 + inventory 注册表协同。
//! 放在 chanlun crate 内，因 #[signal] emit 的是 `crate::signal::registry::` 路径。

use std::collections::HashMap;

use chanlun::business::observer::观察者;
use chanlun::signal::registry::{get_signal, get_template, list_signal_names};
use chanlun::signal::Signal;
use chanlun_signal_macros::signal;
use serde_json::Value;

/// 探针信号函数：贴 #[signal] 后应被自动注册。
#[signal(
    name = "test_probe_signal_V230101",
    template = "{freq}_D1MO{max_overlap}_test_probe_signalV230101"
)]
fn test_probe_signal_V230101(_obs: &观察者, _params: &HashMap<String, Value>) -> Vec<Signal> {
    Vec::new()
}

#[test]
fn test_signal_宏自动注册到全局表() {
    // get_signal 命中
    assert!(
        get_signal("test_probe_signal_V230101").is_some(),
        "#[signal] 应把探针函数注册进 SIGNAL_REGISTRY"
    );
    // 模板正确
    assert_eq!(
        get_template("test_probe_signal_V230101"),
        Some("{freq}_D1MO{max_overlap}_test_probe_signalV230101")
    );
    // 列表含它
    assert!(list_signal_names().contains(&"test_probe_signal_V230101"));
}

#[test]
fn test_未注册信号返回None() {
    assert!(get_signal("不存在的信号_V999999").is_none());
}
```

- [ ] **步骤 2：运行集成测试**

运行：`cd /home/moscow/chanlun.rs/chanlun && cargo test --test test_signal_registry`
预期：2 个测试全 PASS。

> 注：本测试与 registry.rs 的 `#[cfg(test)]` inventory 探针不冲突——集成测试是独立编译单元，`__probe_inventory_V000000` 仅在 lib 单测时提交，集成测试时只有 `test_probe_signal_V230101`。

- [ ] **步骤 3：跑全量 signal 测试确认无回归**

运行：`cd /home/moscow/chanlun.rs/chanlun && cargo test signal`
预期：原 23 个原语单测 + registry 3 个 + 集成 2 个，全 PASS。

- [ ] **步骤 4：Commit**

```bash
cd /home/moscow/chanlun.rs
git add chanlun/tests/test_signal_registry.rs
git commit -m "test(signal-registry): 端到端——#[signal] 宏自动注册 + 查表"
```

---

## 自检结论

- **规格覆盖**：设计 §4 crate 结构 → 任务 0；§5 描述符/注册表/查询 API → 任务 1；§6 `#[signal]` 宏 → 任务 2；§7 测试（归并重名/inventory 探针/宏端到端）→ 任务 1（单测）+ 任务 3（集成）；§9 错误处理（编译期 compile_error、启动期重名 panic、运行期 None）→ 任务 2（compile_error）+ 任务 1（归并 Err→panic / get_signal None）。全覆盖。
- **类型一致**：`SignalFn`/`SignalDescriptor`/`SignalMeta`/`归并`/`get_signal`/`get_template`/`list_signal_names` 在 registry.rs 定义，任务 2 宏 emit `crate::signal::registry::{SignalDescriptor, SignalFn}`、任务 3 集成测试 import `chanlun::signal::registry::{get_signal, get_template, list_signal_names}`，命名贯穿一致。
- **占位符**：任务 0 步骤 2 的占位宏是**有意的脚手架**（任务 2 替换为真实实现），非计划缺陷；其余步骤均含完整可编译代码。
- **风险提示**：任务 1 步骤 3 标注了 `inventory::iter` API 细节的 fallback（参考 czsc registry.rs 实际写法）。
