/*
 * MIT License
 *
 * Copyright (c) 2026 YuYuKunKun
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

//! 信号注册表 —— 编译期收集 + 运行时动态注册。
//!
//! 第三方代码声明：注册机制参考 czsc（https://github.com/waditu/czsc，
//! Apache License 2.0），已简化适配（无 category / TaCache）。
//!
//! # 双注册表架构
//!
//! - `SIGNAL_REGISTRY`: 编译时，`#[signal]` 宏 + `inventory`，不可变。
//! - `DYNAMIC_REGISTRY`: 运行时，`register_signal()` / `unregister_signal()`，`RwLock`。
//!
//! 查找时先查编译时，再查动态。同名时编译时优先（动态注册被遮蔽）。

use crate::business::observer::观察者;
use crate::signal::Signal;
use parking_lot::RwLock;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::LazyLock;

/// 信号函数签名 —— 读观察者状态（含 K线已挂指标）+ 参数 → 信号列表。无 TaCache。
pub type SignalFn = fn(&观察者, &HashMap<String, Value>) -> Vec<Signal>;

/// 信号描述符（编译期元数据，由 `#[signal]` 宏生成、`inventory` 收集）。
#[derive(Clone, Copy)]
pub struct SignalDescriptor {
    pub name: &'static str,
    pub template: &'static str,
    pub func: SignalFn,
}

inventory::collect!(SignalDescriptor);

/// 运行时信号元信息（编译时和动态共用）。
#[derive(Clone, Debug)]
pub struct SignalMeta {
    pub func: SignalFn,
    pub template: String,
}

/// 归并描述符为注册表；重名返回 Err。
fn 归并(
    descs: impl Iterator<Item = SignalDescriptor>,
) -> Result<HashMap<&'static str, SignalMeta>, String> {
    let mut m: HashMap<&'static str, SignalMeta> = HashMap::new();
    for d in descs {
        if m.insert(
            d.name,
            SignalMeta {
                func: d.func,
                template: d.template.to_string(),
            },
        )
        .is_some()
        {
            return Err(format!("信号重名：{}", d.name));
        }
    }
    Ok(m)
}

/// 编译时注册表（`#[signal]` 宏，inventory 收集，不可变）。
pub static SIGNAL_REGISTRY: LazyLock<HashMap<&'static str, SignalMeta>> = LazyLock::new(|| {
    归并(inventory::iter::<SignalDescriptor>.into_iter().copied()).unwrap_or_else(|e| panic!("{e}"))
});

/// 动态注册表（运行时注册，RwLock）。
pub static DYNAMIC_REGISTRY: LazyLock<RwLock<HashMap<String, SignalMeta>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

// ============================================================================
// 查询 API
// ============================================================================

/// 按名查信号元信息。先查编译时，再查动态。
pub fn get_signal(name: &str) -> Option<SignalMeta> {
    if let Some(m) = SIGNAL_REGISTRY.get(name) {
        return Some(m.clone());
    }
    DYNAMIC_REGISTRY.read().get(name).cloned()
}

/// 按名查参数模板。
pub fn get_template(name: &str) -> Option<String> {
    get_signal(name).map(|m| m.template)
}

/// 列出所有已注册信号名（编译时 + 动态，排序）。
pub fn list_signal_names() -> Vec<String> {
    let mut v: Vec<String> = SIGNAL_REGISTRY.keys().map(|k| k.to_string()).collect();
    for k in DYNAMIC_REGISTRY.read().keys() {
        if !SIGNAL_REGISTRY.contains_key(k.as_str()) {
            v.push(k.clone());
        }
    }
    v.sort();
    v
}

// ============================================================================
// 动态注册 API
// ============================================================================

/// 运行时动态注册信号函数。
///
/// - `name`: 信号名（必须全局唯一）
/// - `template`: 参数模板，如 `"{freq}_D{di}_涨跌停V230331"`
/// - `func`: 信号函数指针
///
/// 返回 `Err` 如果同名信号已存在于编译时或动态注册表中。
pub fn register_signal(name: &str, template: &str, func: SignalFn) -> Result<(), String> {
    if SIGNAL_REGISTRY.contains_key(name) {
        return Err(format!("信号 '{name}' 已在编译时注册表中，无法覆盖"));
    }
    let mut dyn_reg = DYNAMIC_REGISTRY.write();
    if dyn_reg.contains_key(name) {
        return Err(format!("信号 '{name}' 已在动态注册表中"));
    }
    dyn_reg.insert(
        name.to_string(),
        SignalMeta {
            func,
            template: template.to_string(),
        },
    );
    Ok(())
}

/// 从动态注册表中移除信号。返回 `Err` 如果信号不存在或属于编译时注册表。
pub fn unregister_signal(name: &str) -> Result<(), String> {
    if SIGNAL_REGISTRY.contains_key(name) {
        return Err(format!("信号 '{name}' 属于编译时注册表，无法动态移除"));
    }
    let mut dyn_reg = DYNAMIC_REGISTRY.write();
    if dyn_reg.remove(name).is_none() {
        return Err(format!("信号 '{name}' 不在动态注册表中"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn __probe(_obs: &观察者, _p: &HashMap<String, Value>) -> Vec<Signal> {
        Vec::new()
    }

    fn 描述符(name: &'static str) -> SignalDescriptor {
        SignalDescriptor {
            name,
            template: "{freq}_D1_probe",
            func: __probe,
        }
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

    // ── 动态注册测试 ──

    #[test]
    fn test_动态注册成功() {
        assert!(register_signal("__dyn_test_V000001", "{freq}_D1_test", __probe).is_ok());
        let meta = get_signal("__dyn_test_V000001").unwrap();
        assert_eq!(meta.template, "{freq}_D1_test");
        // 清理
        unregister_signal("__dyn_test_V000001").unwrap();
    }

    #[test]
    fn test_动态重名_报错() {
        register_signal("__dyn_dup_V000001", "{freq}_D1_a", __probe).unwrap();
        let r = register_signal("__dyn_dup_V000001", "{freq}_D1_b", __probe);
        assert!(r.is_err());
        assert!(r.unwrap_err().contains("已在动态注册表中"));
        unregister_signal("__dyn_dup_V000001").unwrap();
    }

    #[test]
    fn test_动态覆盖编译时_报错() {
        // 编译时已注册的信号不允许动态覆盖
        let r = register_signal("__probe_inventory_V000000", "{freq}_test", __probe);
        assert!(r.is_err());
        assert!(r.unwrap_err().contains("已"));
    }

    #[test]
    fn test_注销_成功() {
        register_signal("__dyn_rm_V000001", "{freq}_D1_rm", __probe).unwrap();
        assert!(get_signal("__dyn_rm_V000001").is_some());
        unregister_signal("__dyn_rm_V000001").unwrap();
        assert!(get_signal("__dyn_rm_V000001").is_none());
    }

    #[test]
    fn test_注销编译时_报错() {
        let r = unregister_signal("__probe_inventory_V000000");
        assert!(r.is_err());
    }

    #[test]
    fn test_list_包含动态信号() {
        register_signal("__list_dyn_V000001", "{freq}_test", __probe).unwrap();
        let names = list_signal_names();
        assert!(names.contains(&"__list_dyn_V000001".to_string()));
        // 编译时信号也在
        assert!(names.contains(&"__probe_inventory_V000000".to_string()));
        unregister_signal("__list_dyn_V000001").unwrap();
    }

    #[test]
    fn test_动态优先_编译时不遮蔽() {
        // 编译时信号正常返回
        let meta = get_signal("__probe_inventory_V000000").unwrap();
        assert_eq!(meta.template, "{freq}_D1_probe_inventory");
    }
}

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
            Some("{freq}_D1_probe_inventory".to_string())
        );
        assert!(list_signal_names().contains(&"__probe_inventory_V000000".to_string()));
    }
}
