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
