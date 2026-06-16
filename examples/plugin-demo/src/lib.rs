//! chanlun 信号插件示例 — 两种动态注册方式。
//!
//! 编译: `cargo build` → `target/debug/libplugin_demo.so`
//! 加载: Python → `ctypes.CDLL(...)` → `init_plugin_manual()` / `init_plugin_macro()`
//!
//! # 方式 A: 手动 C-ABI 注册
//!   - 实现 `fn(SignalFn)` 信号函数
//!   - `init` 中调用 `chanlun_register_signal(name, template, func_ptr)`
//!
//! # 方式 B: #[signal] 宏 + inventory 批量提交
//!   - 用 `#[signal]` 宏写信号函数（与宿主内写法完全一致）
//!   - `init` 中遍历 `inventory::iter::<SignalDescriptor>` 批量调用 `chanlun_register_signal`

use std::collections::HashMap;
use std::os::raw::c_char;

use chanlun::business::observer::观察者;
use chanlun::signal::registry::SignalFn;
use chanlun::signal::Signal;
use serde_json::Value;

// ── C-ABI: 宿主导出的符号（由 dlopen 的动态链接器解析）──

unsafe extern "C" {
    fn chanlun_register_signal(
        name: *const c_char,
        template: *const c_char,
        func: SignalFn,
    ) -> i32;
    fn chanlun_unregister_signal(name: *const c_char) -> i32;
}

// ═══════════════════════════════════════════════════════════
// 方式 A: 手动 C-ABI 注册
// ═══════════════════════════════════════════════════════════

/// 插件信号 A：MACD 零上强势（DIF > 0 && DIF > DEA）。
///
/// 手动注册 — 不依赖 `#[signal]` 宏，不依赖 `chanlun-signal-macros`。
fn 插件MACD强势_V999999(obs: &观察者, params: &HashMap<String, Value>) -> Vec<Signal> {
    obs.确保指标已计算();
    let freq = params.get("freq").and_then(|v| v.as_str()).unwrap_or("日线");
    let di = params.get("di").and_then(|v| v.as_i64()).unwrap_or(1) as usize;

    let k1 = freq.to_string();
    let k2 = format!("D{di}");
    let k3 = "插件MACD强势V999999";

    let klines = &obs.普通K线序列;
    if klines.len() < di + 1 {
        return vec![Signal::new_empty(&k1, &k2, k3)];
    }
    let k线 = &klines[klines.len() - di];
    let macd = match k线.macd() {
        Some(m) => m,
        None => return vec![Signal::new_empty(&k1, &k2, k3)],
    };
    let dif = match macd.DIF { Some(v) => v, None => return vec![Signal::new_empty(&k1, &k2, k3)] };
    let dea = match macd.DEA { Some(v) => v, None => return vec![Signal::new_empty(&k1, &k2, k3)] };

    if dif > 0.0 && dif > dea {
        let score = ((dif - dea).abs() * 1000.0) as i32;
        vec![Signal::new(&k1, &k2, k3, "多头强势", "MACD零上", "DIF>DEA", score)]
    } else if dif < 0.0 && dif < dea {
        vec![Signal::new(&k1, &k2, k3, "空头强势", "MACD零下", "DIF<DEA", 0)]
    } else {
        vec![Signal::new_empty(&k1, &k2, k3)]
    }
}

/// 方式 A 入口：Python 调用 `plugin.init_plugin_manual()`。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn init_plugin_manual() -> i32 {
    let name = c"插件MACD强势_V999999";
    let template = c"{freq}_D{di}_插件MACD强势V999999";
    let func: SignalFn = 插件MACD强势_V999999;

    let ret = chanlun_register_signal(name.as_ptr(), template.as_ptr(), func);
    eprintln!("[plugin/manual] 注册: {name:?} → 返回码 {ret}");
    ret
}

/// 方式 A 清理。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn deinit_plugin_manual() -> i32 {
    chanlun_unregister_signal(c"插件MACD强势_V999999".as_ptr())
}

// ═══════════════════════════════════════════════════════════
// 方式 B: #[signal] 宏 + inventory::iter 批量提交
// ═══════════════════════════════════════════════════════════

// 依赖: Cargo.toml 中添加 chanlun-signal-macros
use chanlun::signal::registry::SignalDescriptor;
use chanlun_signal_macros::signal;

/// 插件信号 B1：MACD 金叉（与宿主内 `macd_金叉_V260601` 等价）。
#[signal(
    name = "插件MACD金叉_V999999",
    template = "{freq}_D{di}#MACD#{fast}#{slow}#{signal}_插件MACD金叉V999999",
    crate_path = "::chanlun"
)]
fn 插件MACD金叉_V999999(obs: &观察者, params: &HashMap<String, Value>) -> Vec<Signal> {
    obs.确保指标已计算();
    let di = params.get("di").and_then(|v| v.as_i64()).unwrap_or(1) as usize;
    let freq = params.get("freq").and_then(|v| v.as_str()).unwrap_or("日线");

    let k1 = freq.to_string();
    let k2 = "D1#MACD#13#31#11".to_string();
    let k3 = "插件MACD金叉V999999";

    let klines = &obs.普通K线序列;
    if klines.len() < di + 2 {
        return vec![Signal::new_empty(&k1, &k2, k3)];
    }
    let cur = klines[klines.len() - di].macd();
    let prev = klines[klines.len() - di - 1].macd();
    let (cur, prev) = match (cur, prev) {
        (Some(c), Some(p)) => (c, p),
        _ => return vec![Signal::new_empty(&k1, &k2, k3)],
    };
    let cd = match cur.DIF { Some(v) => v, None => return vec![Signal::new_empty(&k1, &k2, k3)] };
    let ce = match cur.DEA { Some(v) => v, None => return vec![Signal::new_empty(&k1, &k2, k3)] };
    let pd = match prev.DIF { Some(v) => v, None => return vec![Signal::new_empty(&k1, &k2, k3)] };
    let pe = match prev.DEA { Some(v) => v, None => return vec![Signal::new_empty(&k1, &k2, k3)] };

    if pd <= pe && cd > ce {
        vec![Signal::new(&k1, &k2, k3, "金叉", "插件", "[signal]宏", 80)]
    } else if pd >= pe && cd < ce {
        vec![Signal::new(&k1, &k2, k3, "死叉", "插件", "[signal]宏", 0)]
    } else {
        vec![Signal::new_empty(&k1, &k2, k3)]
    }
}

/// 插件信号 B2：涨跌停（与宿主内 `bar_zdt_V230331` 等价）。
#[signal(
    name = "插件涨跌停_V999999",
    template = "{freq}_D{di}_插件涨跌停V999999",
    crate_path = "::chanlun"
)]
fn 插件涨跌停_V999999(obs: &观察者, params: &HashMap<String, Value>) -> Vec<Signal> {
    let di = params.get("di").and_then(|v| v.as_i64()).unwrap_or(1) as usize;
    let freq = params.get("freq").and_then(|v| v.as_str()).unwrap_or("15分钟");

    let k1 = freq.to_string();
    let k2 = format!("D{di}");
    let k3 = "插件涨跌停V999999";

    let klines = &obs.普通K线序列;
    if klines.len() < di + 2 {
        return vec![Signal::new_empty(&k1, &k2, k3)];
    }
    let 当前 = &klines[klines.len() - di];
    let 前 = &klines[klines.len() - di - 1];

    let v1 = if 当前.收盘价 == 当前.高 && 当前.收盘价 >= 前.收盘价 {
        "涨停"
    } else if 当前.收盘价 == 当前.低 && 当前.收盘价 <= 前.收盘价 {
        "跌停"
    } else {
        "任意"
    };

    if v1 == "任意" {
        vec![Signal::new_empty(&k1, &k2, k3)]
    } else {
        vec![Signal::new(&k1, &k2, k3, v1, "插件", "[signal]宏", 0)]
    }
}

/// 方式 B 入口：Python 调用 `plugin.init_plugin_macro()`。
///
/// 遍历 `inventory::iter::<SignalDescriptor>`，将 `#[signal]` 宏注册的
/// 所有信号一次性提交到宿主 `DYNAMIC_REGISTRY`。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn init_plugin_macro() -> i32 {
    let mut count = 0i32;
    for desc in inventory::iter::<SignalDescriptor> {
        let Ok(name_c) = std::ffi::CString::new(desc.name) else { continue };
        let Ok(tpl_c) = std::ffi::CString::new(desc.template) else { continue };
        let ret = chanlun_register_signal(name_c.as_ptr(), tpl_c.as_ptr(), desc.func);
        if ret == 0 {
            count += 1;
            eprintln!("[plugin/macro] ✅ {} → {}", desc.name, desc.template);
        } else {
            eprintln!("[plugin/macro] ❌ {} (err {ret})", desc.name);
        }
    }
    eprintln!("[plugin/macro] 批量注册完成: {count} 个信号");
    count
}

/// 方式 B 清理：遍历 inventory 逐个调用 `chanlun_unregister_signal`。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn deinit_plugin_macro() -> i32 {
    for desc in inventory::iter::<SignalDescriptor> {
        let Ok(name_c) = std::ffi::CString::new(desc.name) else { continue };
        chanlun_unregister_signal(name_c.as_ptr());
    }
    0
}
