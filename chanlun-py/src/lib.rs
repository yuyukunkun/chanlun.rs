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
use std::sync::atomic::Ordering;

mod algorithm_py;
mod business_py;
mod config_py;
mod indicators_py;
mod kline_py;
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

/// 缠论技术分析库 — Rust 高性能实现
#[pymodule]
/// 缠论技术分析库 — Rust 高性能实现
fn _chanlun(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(get_分型模式, m)?)?;
    m.add_function(wrap_pyfunction!(set_分型模式, m)?)?;
    // 阶段 1: 枚举和基础类型
    types_py::register(m)?;
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
