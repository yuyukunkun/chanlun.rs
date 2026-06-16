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

#![allow(non_snake_case, clippy::too_many_arguments)]

use pyo3::prelude::*;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Mutex, Once, OnceLock};

/// 日志级别: 0=trace, 1=debug, 2=info, 3=warn, 4=error, 5=off
static LOG_LEVEL: AtomicU8 = AtomicU8::new(2); // 默认 info

type 过滤器句柄 =
    tracing_subscriber::reload::Handle<tracing_subscriber::EnvFilter, tracing_subscriber::Registry>;
static 过滤器句柄锁: OnceLock<Mutex<过滤器句柄>> = OnceLock::new();
static TRACING_INIT: Once = Once::new();

fn 级别数字转名称(n: u8) -> &'static str {
    match n {
        0 => "trace",
        1 => "debug",
        2 => "info",
        3 => "warn",
        4 => "error",
        5 => "off",
        _ => "unknown",
    }
}

fn 级别名称转数字(name: &str) -> Option<u8> {
    match name.to_lowercase().as_str() {
        "trace" => Some(0),
        "debug" => Some(1),
        "info" => Some(2),
        "warn" => Some(3),
        "error" => Some(4),
        "off" => Some(5),
        _ => None,
    }
}

fn init_tracing() {
    TRACING_INIT.call_once(|| {
        use chrono::Local;
        use std::fmt;
        use tracing_subscriber::fmt::format::Format;
        use tracing_subscriber::fmt::format::Writer;
        use tracing_subscriber::fmt::time::FormatTime;
        use tracing_subscriber::layer::SubscriberExt;
        use tracing_subscriber::util::SubscriberInitExt;

        struct 本地时间;
        impl FormatTime for 本地时间 {
            fn format_time(&self, w: &mut Writer<'_>) -> fmt::Result {
                write!(w, "{}", Local::now().format("%Y-%m-%d %H:%M:%S%.3f"))
            }
        }

        let format = Format::default()
            .with_timer(本地时间)
            .with_target(false)
            .with_file(true)
            .with_line_number(true)
            .with_ansi(true)
            .compact();

        let 初始级别 = tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

        let (过滤器层, 句柄) = tracing_subscriber::reload::Layer::new(初始级别);
        过滤器句柄锁
            .set(Mutex::new(句柄))
            .expect("过滤器句柄锁只能设置一次");

        tracing_subscriber::registry()
            .with(过滤器层)
            .with(tracing_subscriber::fmt::layer().event_format(format))
            .init();
    });
}

mod algorithm_py;
mod business_py;
pub(crate) mod cache;
mod config_py;
mod equality_py;
mod indicators_py;
mod kline_py;
mod signal_engine_py;
mod signal_py;
mod structure_py;
mod types_py;

/// 分型模式 — True 时使用构造时缓存值，False 时从 中 缠K 实时读取
#[pyfunction]
fn get_分型模式() -> bool {
    chanlun::structure::fractal_obj::分型模式.load(Ordering::Relaxed)
}

/// 设置 分型模式
#[pyfunction]
fn set_分型模式(value: bool) {
    chanlun::structure::fractal_obj::分型模式.store(value, Ordering::Relaxed);
}

/// 扩展线段模式 — 控制虚线高低取值方式，默认 False
#[pyfunction]
fn get_扩展线段模式() -> bool {
    chanlun::structure::dash_line::扩展线段模式.load(Ordering::Relaxed)
}

/// 设置 扩展线段模式
#[pyfunction]
fn set_扩展线段模式(value: bool) {
    chanlun::structure::dash_line::扩展线段模式.store(value, Ordering::Relaxed);
}

/// 获取当前日志级别 ("trace" / "debug" / "info" / "warn" / "error" / "off")
#[pyfunction]
fn get_log_level() -> &'static str {
    级别数字转名称(LOG_LEVEL.load(Ordering::Relaxed))
}

/// 设置日志级别 — 自动启用日志，同步更新 tracing subscriber
#[pyfunction]
fn set_log_level(level: &str) -> PyResult<()> {
    let 数字 = 级别名称转数字(level).ok_or_else(|| {
        pyo3::exceptions::PyValueError::new_err(format!(
            "无效日志级别 '{}'，有效值: trace, debug, info, warn, error, off",
            level
        ))
    })?;
    LOG_LEVEL.store(数字, Ordering::Relaxed);
    chanlun::log::日志启用.store(数字 < 5, Ordering::Relaxed);
    // 同步更新 tracing subscriber
    if let Some(guard) = 过滤器句柄锁.get() {
        let handle = guard.lock().unwrap();
        let 名称 = 级别数字转名称(数字);
        let filter = tracing_subscriber::EnvFilter::new(名称);
        let _ = handle.reload(filter);
    }
    Ok(())
}

/// 获取日志输出模式 ("off", "simple", "tracing")
#[pyfunction]
fn get_log_mode() -> &'static str {
    match chanlun::log::get_log_mode() {
        0 => "off",
        1 => "simple",
        2 => "tracing",
        _ => "unknown",
    }
}

/// 设置日志输出模式（必须在任何日志输出之前调用）
/// - "off": 不输出
/// - "simple": 直接 eprintln/println（默认）
/// - "tracing": 带时间戳和格式化的 tracing subscriber
#[pyfunction]
fn set_log_mode(mode: &str) -> PyResult<()> {
    let m = match mode.to_lowercase().as_str() {
        "off" | "0" => 0u8,
        "simple" | "on" | "1" => 1u8,
        "tracing" | "2" => 2u8,
        _ => {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "无效日志模式，有效值: 'off', 'simple', 'tracing'",
            ));
        }
    };
    if m == 2 {
        init_tracing();
    }
    chanlun::log::set_log_mode(m);
    Ok(())
}

/// 获取缓存模式 ("thread_local" 或 "global")
#[pyfunction]
fn get_cache_mode() -> &'static str {
    match crate::cache::peek_mode().unwrap_or(&crate::cache::CacheMode::ThreadLocal) {
        crate::cache::CacheMode::ThreadLocal => "thread_local",
        crate::cache::CacheMode::Global => "global",
    }
}

/// 设置缓存模式（必须在创建任何观察者之前调用）
#[pyfunction]
fn set_cache_mode(mode: &str) -> PyResult<()> {
    let m = match mode.to_lowercase().as_str() {
        "thread_local" | "local" => crate::cache::CacheMode::ThreadLocal,
        "global" => crate::cache::CacheMode::Global,
        _ => {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "无效缓存模式，有效值: 'thread_local', 'global'",
            ));
        }
    };
    crate::cache::set_mode(m).map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e))
}

/// 缠论技术分析库 — Rust 高性能实现
#[pymodule]
/// 缠论技术分析库 — Rust 高性能实现
fn _chanlun(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    chanlun::log::init_from_env();
    m.add_function(wrap_pyfunction!(get_分型模式, m)?)?;
    m.add_function(wrap_pyfunction!(set_分型模式, m)?)?;
    m.add_function(wrap_pyfunction!(get_扩展线段模式, m)?)?;
    m.add_function(wrap_pyfunction!(set_扩展线段模式, m)?)?;
    m.add_function(wrap_pyfunction!(get_log_level, m)?)?;
    m.add_function(wrap_pyfunction!(set_log_level, m)?)?;
    m.add_function(wrap_pyfunction!(get_log_mode, m)?)?;
    m.add_function(wrap_pyfunction!(set_log_mode, m)?)?;
    m.add_function(wrap_pyfunction!(get_cache_mode, m)?)?;
    m.add_function(wrap_pyfunction!(set_cache_mode, m)?)?;
    // 阶段 1: 枚举和基础类型
    types_py::register(m)?;
    // 阶段 1.5: 信号原语
    signal_py::register(m)?;
    // 阶段 2: 配置
    config_py::register(m)?;
    // 阶段 3: 技术指标
    indicators_py::register(m)?;
    // 阶段 4: K线
    kline_py::register(m)?;
    // 阶段 5: 结构
    structure_py::register(m)?;
    // 阶段 6: 算法
    algorithm_py::register(m)?;
    // 阶段 7: 业务
    business_py::register(m)?;
    // 阶段 7.5: 信号引擎
    signal_engine_py::register(m)?;
    // 阶段 8: 相等校验函数
    equality_py::register(m)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_分型模式_get_set() {
        // 手动初始化 Python 解释器（cargo test 环境下 auto-initialize 不一定生效）
        unsafe {
            if pyo3::ffi::Py_IsInitialized() == 0 {
                pyo3::ffi::Py_Initialize();
            }
        }
        pyo3::Python::try_attach(|py| {
            let module = PyModule::new(py, "test_module").unwrap();
            module
                .add_function(wrap_pyfunction!(get_分型模式, &module).unwrap())
                .unwrap();
            module
                .add_function(wrap_pyfunction!(set_分型模式, &module).unwrap())
                .unwrap();

            // 默认 true
            let getter = module.getattr("get_分型模式").unwrap();
            let result: bool = getter.call0().unwrap().extract().unwrap();
            assert!(result, "分型模式 默认应为 True");

            // 设置为 false
            let setter = module.getattr("set_分型模式").unwrap();
            setter.call1((false,)).unwrap();
            let result: bool = getter.call0().unwrap().extract().unwrap();
            assert!(!result, "分型模式 应为 False");

            // 恢复 true
            setter.call1((true,)).unwrap();
            let result: bool = getter.call0().unwrap().extract().unwrap();
            assert!(result, "分型模式 应为 True");
        })
        .expect("Python 解释器初始化后 attach 仍失败");
    }
}
