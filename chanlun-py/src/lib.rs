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

use pyo3::prelude::*;

mod algorithm_py;
mod business_py;
mod config_py;
mod indicators_py;
mod kline_py;
mod structure_py;
mod types_py;

/// 缠论技术分析库 — Rust 高性能实现
#[pymodule]
/// 缠论技术分析库 — Rust 高性能实现
fn _chanlun(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
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
    use pyo3::prelude::*;

    #[test]
    fn test_rc_pointer_across_getters() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let module = PyModule::new(py, "test_module").unwrap();
            module.add_class::<business_py::观察者Py>().unwrap();
            module.add_class::<business_py::基础买卖点Py>().unwrap();
            module.add_class::<business_py::买卖点Py>().unwrap();
            module.add_class::<kline_py::K线Py>().unwrap();
            module.add_class::<kline_py::缠论K线Py>().unwrap();
            module.add_class::<structure_py::分型Py>().unwrap();
            module.add_class::<structure_py::虚线Py>().unwrap();
            module.add_class::<config_py::缠论配置Py>().unwrap();

            let config = config_py::缠论配置Py::from_rust_config(&Default::default()).unwrap();
            let obs = business_py::观察者Py::new_impl("btcusd".into(), 300, config, py).unwrap();

            // Feed one K line
            let kline = kline_py::K线Py::new_impl(
                "btcusd".into(),
                1000,
                100.0,
                105.0,
                99.0,
                103.0,
                1000.0,
                0,
                300,
            );
            let kline_ref = kline.into_ref(py);
            // ... this is too complex
        });
    }
}
