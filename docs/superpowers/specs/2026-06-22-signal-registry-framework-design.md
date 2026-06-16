# 子项目 1：信号注册框架 — 设计文档

- 日期：2026-06-22
- 所属：「全 Rust 信号计算迁移」第 1 个子项目（共 4 个）
- 参考：czsc（`/home/moscow/czsc`）的 `czsc-signal-macros` + `czsc-signals/{registry,types}.rs`
- 前置：原语层已完成（`chanlun/src/signal/` 的 Signal/Factor/Event/Position/Operate）

## 1. 背景与目标

「全 Rust 信号计算迁移」把信号函数、注册/解析、计算引擎、持仓状态机全部移到 Rust。拆为 4 个子项目（依赖序 1→2→3→4）：

1. **信号注册框架**（本文档）
2. 信号函数 API 暴露 + 移植 youwukuncheng
3. 信号计算引擎 + PyO3 分发器
4. Position.update 状态机

本子项目交付**编译期信号注册机制**：一个 `#[signal]` 属性宏 + `inventory` 注册表 + 描述符类型 + 一个探针信号验证机制。

**它消灭什么**：Python 的 `import_by_name`（动态导入，曾导致「找不到模块」「跨模块枚举 `is` 不等」）和 `SignalsParser` 的 docstring 正则解析（曾导致「多 pattern sig_pats_map」「get_function_name v[0]」「sys 未导入」等脆弱 bug）。注册变成编译期完成、查表 O(1)。

## 2. 范围

### 纳入
- 新 proc-macro crate `chanlun-signal-macros`：`#[signal(name, template)]` 属性宏
- `chanlun/src/signal/registry.rs`：`SignalDescriptor` / `SignalFn` / `SignalMeta` / `SIGNAL_REGISTRY` + 只读查询 API
- `chanlun/Cargo.toml` 新增 `inventory` 依赖 + path 依赖 `chanlun-signal-macros`
- 一个探针信号 + 测试（验证注册→查表→重名检测）

### 不纳入（后续子项目）
- 真实信号函数移植（子项目 2）
- 「确保指标按需增量计算」API（子项目 2，移植 youwukuncheng 读 MACD 时落地）
- 信号计算引擎 + `call_signal` PyO3 分发器（子项目 3）
- Position.update 状态机（子项目 4）

## 3. 关键设计决策

| 决策 | 选择 | 理由 |
|---|---|---|
| SignalFn 是否带 TaCache | **否** | 核心层 K线已挂载指标（`指标计算器::计算并挂载`），信号函数直接读 `标的K线.指标.macd(..)`，无需 czsc 式 TaCache |
| 注册表位置 | **chanlun 核心 crate** `signal/` 模块 | 信号函数直接读 observer（同 crate）、指标在 K线上，无需独立 signals crate |
| params 类型 | `HashMap<String, serde_json::Value>` | 灵活，对应 Python dict 来源（PyO3 层自然转换） |
| 描述符是否含 indicators/category 字段 | **否，保持最小 `{name, template, func}`** | 指标由「信号内识别 + 管线增量算」处理，不在描述符声明；本项目信号皆 observer 级，无需 category |

## 4. Crate 结构

```
chanlun-signal-macros/        ← 新建 proc-macro crate（Rust 强制独立）
├── Cargo.toml                ← [lib] proc-macro = true；deps: syn, quote, proc-macro2
└── src/lib.rs                ← #[signal] 属性宏

chanlun/                      ← 现有核心 crate
├── Cargo.toml                ← 新增 inventory="0.3" + path 依赖 chanlun-signal-macros
└── src/signal/
    ├── mod.rs                ← pub mod registry;
    └── registry.rs           ← 描述符类型 + 注册表 + 探针信号（cfg(test)）
```

`chanlun` 通过 path 依赖 `chanlun-signal-macros`（无需引入 workspace；Cargo path 依赖即可。如愿统一可后续加 `[workspace]`）。

## 5. 描述符类型与签名（`chanlun/src/signal/registry.rs`）

```rust
use crate::business::observer::观察者;
use crate::signal::Signal;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::LazyLock;

/// 信号函数签名 — 读观察者状态（含 K线已挂指标）+ 参数 → 信号列表。无 TaCache。
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
        if m.insert(d.name, SignalMeta { func: d.func, template: d.template }).is_some() {
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
```

## 6. `#[signal]` 宏（`chanlun-signal-macros/src/lib.rs`）

属性宏贴在信号函数上，做三件事：

1. **校验**：函数名必须含 `_V<数字版本>`；`name` 属性须与函数名一致；`name`/`template` 非空。不符 → `compile_error!`。
2. **保留原函数**不变。
3. **生成** 一个 `static` 描述符 + `inventory::submit!` 提交：

宏输入 `#[signal(name = "foo_V230101", template = "{freq}_D1_foo")]` 贴在 `fn foo_V230101(...)` 上，展开为（概念示意）：
```rust
fn foo_V230101(观: &观察者, p: &HashMap<String, Value>) -> Vec<Signal> { /* 原体 */ }
inventory::submit! {
    crate::signal::registry::SignalDescriptor {
        name: "foo_V230101",
        template: "{freq}_D1_foo",
        func: foo_V230101 as crate::signal::registry::SignalFn,
    }
}
```

**路径约定**：宏 emit `crate::signal::registry::...`，即假定信号函数住在 `chanlun` crate 内（本迁移的既定结构）。

## 7. 测试

1. **宏 crate**（`chanlun-signal-macros/tests/test_signal_macro.rs`）：普通集成测试——定义一个符合签名的探针函数并贴 `#[signal(name="probe_macro_V000000", template="{freq}_D1_probe")]`，断言它能编译且 `inventory::iter` 能收到对应描述符（name/template 正确）。编译失败用例（name 与函数名不一致、缺版本号）作为**可选** trybuild compile-fail 测试，非必须。
2. **核心注册表**（`registry.rs` 内 `#[cfg(test)]`）：
   - 用 `inventory::submit!` 提交一个探针 `SignalDescriptor`（name `__probe_V000000`）；
   - `get_signal("__probe_V000000")` 命中、`get_template` 返回模板、`list_signal_names()` 含它；
   - 重名场景：把归并逻辑抽成一个可独立调用的纯函数 `fn 归并(descs: impl Iterator<Item=SignalDescriptor>) -> Result<HashMap<..>, String>`，单测对重复 name 返回 Err（`SIGNAL_REGISTRY` 的 LazyLock 内部调用它并对 Err `panic!`），避免污染全局 inventory。

## 8. 数据流

```
编译期：  #[signal] 宏  →  SignalDescriptor 常量  →  inventory::submit!
启动时：  SIGNAL_REGISTRY (LazyLock)  ←  inventory::iter 归并（重名 panic）
运行时：  get_signal(name) -> &SignalMeta { func, template }   （O(1) 查表）
          后续子项目 3 的计算引擎用 func 调用、用 template 反向生成信号 key
```

## 9. 错误处理

- **编译期**：宏校验失败 → `compile_error!`（带清晰中文消息）。
- **启动期**：重名信号 → `panic!("信号重名：{name}")`（fail-fast，对应 czsc 的 normalize 重名检测）。
- **运行期**：`get_signal` 未命中返回 `None`（调用方——子项目 3——决定如何处理，对应旧「未找到解析函数」告警）。

## 10. 已知取舍与后续

- **无运行时可扩展性**：信号在编译期注册，新增信号需重编译（`maturin build`）。这是「全 Rust」方案的既定取舍，用户已确认。
- **指标按需机制不在本子项目**：信号函数读指标 + 管线增量计算的「确保指标」API 在子项目 2 落地。
- **category（kline/trader）暂不引入**：若子项目 4 的 Position.update 引入 trader 级信号，届时再扩描述符。

## 11. 许可证

新增 Rust 文件沿用项目 MIT 头。注册/宏机制参考 czsc（Apache 2.0），在 `registry.rs` 与 macro crate 顶部加第三方代码声明。
