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

//! PyO3 绑定：将信号引擎和 call_signal 暴露给 Python。
//!
//! 第三方代码声明：引擎架构参考 czsc 的 `信号计算器`
//!（https://github.com/waditu/czsc，Apache License 2.0）。

use std::collections::HashMap;

use chanlun::signal::engine::{self, SignalConfig, SignalEngine as 核心SignalEngine};

use pyo3::exceptions::{PyKeyError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::business_py::{立体分析器Py, 观察者Py};
use crate::signal_py::{SignalPy, 时间戳转datetime};

// ======== 工具函数 ========

/// 将 PyAny 转换为 `serde_json::Value`。
/// 尝试顺序：i64 → f64 → String → bool → 兜底转为 String。
fn py_any_to_json_value(obj: &Bound<'_, PyAny>) -> PyResult<serde_json::Value> {
    // i64
    if let Ok(i) = obj.extract::<i64>() {
        return Ok(serde_json::Value::Number(i.into()));
    }
    // f64
    if let Ok(f) = obj.extract::<f64>() {
        if let Some(n) = serde_json::Number::from_f64(f) {
            return Ok(serde_json::Value::Number(n));
        }
        return Ok(serde_json::Value::String(f.to_string()));
    }
    // String
    if let Ok(s) = obj.extract::<String>() {
        return Ok(serde_json::Value::String(s));
    }
    // bool
    if let Ok(b) = obj.extract::<bool>() {
        return Ok(serde_json::Value::Bool(b));
    }
    // fallback: Python repr as string
    Ok(serde_json::Value::String(obj.to_string()))
}

/// 将 `PyDict` 转换为 `HashMap<String, serde_json::Value>`。
pub(crate) fn py_dict_to_params(
    dict: &Bound<'_, PyDict>,
) -> PyResult<HashMap<String, serde_json::Value>> {
    let mut params = HashMap::new();
    for (k, v) in dict.iter() {
        let key: String = k.extract()?;
        let value = py_any_to_json_value(&v)?;
        params.insert(key, value);
    }
    Ok(params)
}

// ======== 自由函数 ========

/// 通过 Rust 注册表按名调用单个信号函数。
///
/// Args:
///     name: 注册的信号名，如 ``"youwukuncheng_中枢第三买卖点_V230602"``
///     obs: 观察者Py 实例
///     params: 信号参数字典（不含 name）
///
/// Returns:
///     SignalPy 对象列表
///
/// Raises:
///     PyValueError: 信号名未注册
#[pyfunction]
pub fn call_signal(
    name: &str,
    obs: &观察者Py,
    params: &Bound<'_, PyDict>,
) -> PyResult<Vec<SignalPy>> {
    let obs_ref = obs.obs();
    let params_map = py_dict_to_params(params)?;

    let inner =
        engine::call_signal(name, &obs_ref, &params_map).map_err(|e| PyValueError::new_err(e))?;

    Ok(inner.into_iter().map(|s| SignalPy { inner: s }).collect())
}

/// 列出所有已注册的信号名（编译时 + 动态）。
#[pyfunction]
pub fn list_signals() -> Vec<String> {
    chanlun::signal::registry::list_signal_names()
}

/// 按名获取信号参数模板（编译时 + 动态）。
#[pyfunction]
pub fn get_signal_template(name: &str) -> Option<String> {
    chanlun::signal::registry::get_template(name)
}

// ======== 动态注册 API ========

/// 从动态注册表中移除信号。
#[pyfunction]
fn unregister_signal(name: &str) -> PyResult<()> {
    chanlun::signal::registry::unregister_signal(name).map_err(|e| PyValueError::new_err(e))
}

// ======== 信号引擎 pyclass ========

/// Rust 信号计算引擎的 Python 绑定。
///
/// 用法::
///
///     from chanlun._chanlun import 信号引擎
///
///     引擎 = 信号引擎([
///         {"name": "youwukuncheng_中枢第三买卖点_V230602",
///          "freq": 86400, "max_overlap": 3,
///          "本级完整性": "实", "同级完整性": "合"},
///     ])
///     引擎.自动挂载指标(分析器)
///     结果 = 引擎.更新(分析器)  # dict[str, str]
#[pyclass(name = "信号引擎", module = "chanlun._chanlun")]
pub struct SignalEnginePy {
    inner: 核心SignalEngine,
}

#[pymethods]
impl SignalEnginePy {
    /// 创建信号引擎。
    ///
    /// Args:
    ///     信号配置: 信号配置字典列表，每项必须含 ``"name"`` 和 ``"freq"``。
    #[new]
    #[pyo3(signature = (信号配置=None))]
    fn new(信号配置: Option<Vec<Bound<'_, PyDict>>>) -> PyResult<Self> {
        let configs = match 信号配置 {
            Some(list) => {
                let mut configs = Vec::with_capacity(list.len());
                for d in &list {
                    let name: String = d
                        .get_item("name")?
                        .ok_or_else(|| PyValueError::new_err("信号配置缺少 'name'"))?
                        .extract()?;

                    let freq_raw = d
                        .get_item("freq")?
                        .ok_or_else(|| PyKeyError::new_err("信号配置缺少 'freq'"))?;

                    // freq 可以是 int 或 str
                    let freq: i64 = if let Ok(i) = freq_raw.extract::<i64>() {
                        i
                    } else if let Ok(s) = freq_raw.extract::<String>() {
                        s.parse::<i64>().map_err(|_| {
                            PyValueError::new_err(format!("freq 无法解析为整数: {s}"))
                        })?
                    } else {
                        return Err(PyValueError::new_err(format!(
                            "freq 类型无效: {}",
                            freq_raw.get_type().name()?
                        )));
                    };

                    // 构建 params（排除 "name"，保留 "freq" 为字符串格式）
                    let mut params = HashMap::new();
                    for (k, v) in d.iter() {
                        let key: String = k.extract()?;
                        if key == "name" {
                            continue;
                        }
                        if key == "freq" {
                            // 统一为字符串，便于 Rust 信号函数通过 params::get_string 读取
                            params.insert(key, serde_json::Value::String(freq.to_string()));
                            continue;
                        }
                        let value = py_any_to_json_value(&v)?;
                        params.insert(key, value);
                    }

                    configs.push(SignalConfig {
                        signal_name: name,
                        freq,
                        params,
                    });
                }
                configs
            }
            None => Vec::new(),
        };
        Ok(Self {
            inner: 核心SignalEngine::new(configs),
        })
    }

    /// 扫描所有配置中的 MACD / 均线关键字，为各周期 observer 的配置添加缺失的指标参数。
    fn 自动挂载指标(&self, analyzer: &立体分析器Py) {
        self.inner.自动挂载指标(&analyzer.inner);
    }

    /// 遍历所有配置，执行信号函数，收集非空结果。
    ///
    /// Returns:
    ///     ``dict[str, str]`` — 信号 key → 信号 value（已过滤 "任意_任意_任意_0"）
    fn 更新(&self, analyzer: &立体分析器Py) -> HashMap<String, String> {
        self.inner.更新(&analyzer.inner)
    }

    /// 更新信号并返回完整结果（信号 + 行情）。
    /// 返回 dict: ``{"signals": {...}, "market": {...}}``，若无基础周期 K 线则 market 为 None。
    fn 更新_完整<'py>(
        &self, py: Python<'py>, analyzer: &立体分析器Py
    ) -> PyResult<Py<PyAny>> {
        let result = self.inner.更新_完整(&analyzer.inner);
        let d = PyDict::new(py);

        let signals_dict = PyDict::new(py);
        for (k, v) in &result.signals {
            signals_dict.set_item(k, v)?;
        }
        d.set_item("signals", signals_dict)?;

        if let Some(m) = &result.market {
            let md = PyDict::new(py);
            md.set_item("symbol", &m.symbol)?;
            md.set_item("dt", 时间戳转datetime(py, m.dt)?)?;
            md.set_item("id", m.id)?;
            md.set_item("open", m.open)?;
            md.set_item("high", m.high)?;
            md.set_item("low", m.low)?;
            md.set_item("close", m.close)?;
            md.set_item("vol", m.vol)?;
            d.set_item("market", md)?;
        } else {
            d.set_item("market", py.None())?;
        }

        Ok(d.into())
    }

    /// 返回配置数量
    fn __len__(&self) -> usize {
        self.inner.len()
    }

    fn __repr__(&self) -> String {
        format!("信号引擎(configs={})", self.inner.len())
    }
}

/// 注册模块。
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<SignalEnginePy>()?;
    m.add_function(wrap_pyfunction!(call_signal, m)?)?;
    m.add_function(wrap_pyfunction!(list_signals, m)?)?;
    m.add_function(wrap_pyfunction!(get_signal_template, m)?)?;
    m.add_function(wrap_pyfunction!(unregister_signal, m)?)?;
    Ok(())
}
