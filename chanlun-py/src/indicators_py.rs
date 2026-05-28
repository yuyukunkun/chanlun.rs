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
use pyo3::types::PyType;

// ========== 平滑异同移动平均线 ==========

/// MACD 技术指标 — 指数平滑异同移动平均线。
///
/// 属性:
///   时间戳: str / 收盘价: float / 快线周期: int / 慢线周期: int / 信号周期: int
///   DIF: float|None — 快线EMA - 慢线EMA
///   DEA: float|None — DIF 的 EMA 平滑（信号线）
///   MACD柱: float — (DIF - DEA) * 2（柱状值）
///   快线EMA: float|None / 慢线EMA: float|None / DEA_EMA: float|None
///
/// 方法（均为 classmethod，直接构造实例）:
///   首次计算(初始收盘价, 初始时间, 快线周期=12, 慢线周期=26, 信号周期=9) -> 平滑异同移动平均线
///   首次计算_K线(k线, 计算方式, ...) -> 平滑异同移动平均线
///      — 计算方式: "收盘价"/"高"/"低"/"开盘价"
///   增量计算(前一个MACD, 当前收盘价, 当前时间) -> 平滑异同移动平均线
///      — 基于前一根的 EMA 状态增量更新，用于流式计算
///   增量计算_K线(前一个MACD, 当前K线, 计算方式) -> 平滑异同移动平均线
#[pyclass(
    name = "平滑异同移动平均线",
    module = "chanlun._chanlun",
    from_py_object
)]
#[derive(Clone)]
pub struct 平滑异同移动平均线Py {
    pub(crate) inner: chanlun::indicators::平滑异同移动平均线,
}

#[pymethods]
impl 平滑异同移动平均线Py {
    #[new]
    fn new() -> Self {
        unimplemented!("使用 首次计算 或 增量计算 创建")
    }

    #[getter]
    fn 时间戳(&self) -> String {
        self.inner.时间戳.to_string()
    }

    #[getter]
    fn 收盘价(&self) -> f64 {
        self.inner.收盘价
    }

    #[getter]
    fn 快线周期(&self) -> i64 {
        self.inner.快线周期
    }

    #[getter]
    fn 慢线周期(&self) -> i64 {
        self.inner.慢线周期
    }

    #[getter]
    fn 信号周期(&self) -> i64 {
        self.inner.信号周期
    }

    #[getter]
    fn DIF(&self) -> Option<f64> {
        self.inner.DIF
    }

    #[getter]
    fn DEA(&self) -> Option<f64> {
        self.inner.DEA
    }

    #[getter]
    fn MACD柱(&self) -> f64 {
        self.inner.MACD柱
    }

    #[getter]
    fn 快线EMA(&self) -> Option<f64> {
        self.inner.快线EMA
    }

    #[getter]
    fn 慢线EMA(&self) -> Option<f64> {
        self.inner.慢线EMA
    }

    #[getter]
    fn DEA_EMA(&self) -> Option<f64> {
        self.inner.DEA_EMA
    }

    fn __str__(&self) -> String {
        format!(
            "平滑异同移动平均线(DIF={:?}, DEA={:?}, BAR={:?})",
            self.inner.DIF, self.inner.DEA, self.inner.MACD柱
        )
    }

    fn __repr__(&self) -> String {
        self.__str__()
    }

    #[classmethod]
    #[pyo3(signature = (初始收盘价, 初始时间, 快线周期 = None, 慢线周期 = None, 信号周期 = None))]
    /// 首次计算MACD指标（没有历史数据时使用）
    fn 首次计算(
        _cls: &Bound<'_, PyType>,
        初始收盘价: f64,
        初始时间: i64,
        快线周期: Option<i64>,
        慢线周期: Option<i64>,
        信号周期: Option<i64>,
    ) -> Self {
        let inner = chanlun::indicators::平滑异同移动平均线::首次计算(
            初始收盘价,
            初始时间,
            快线周期.unwrap_or(12),
            慢线周期.unwrap_or(26),
            信号周期.unwrap_or(9),
        );
        Self { inner }
    }

    #[classmethod]
    #[pyo3(signature = (k线, 计算方式, 快线周期 = None, 慢线周期 = None, 信号周期 = None))]
    /// :param k线: 原始K线
    fn 首次计算_K线(
        _cls: &Bound<'_, PyType>,
        k线: &Bound<'_, PyAny>,
        计算方式: &str,
        快线周期: Option<i64>,
        慢线周期: Option<i64>,
        信号周期: Option<i64>,
    ) -> PyResult<Self> {
        let 计算值 = crate::indicators_py::K线取值(k线, 计算方式)?;
        Ok(Self {
            inner: chanlun::indicators::平滑异同移动平均线::首次计算(
                计算值,
                获取时间戳(k线)?,
                快线周期.unwrap_or(12),
                慢线周期.unwrap_or(26),
                信号周期.unwrap_or(9),
            ),
        })
    }

    #[classmethod]
    /// 基于前一个MACD指标增量计算当前MACD指标
    fn 增量计算(
        _cls: &Bound<'_, PyType>,
        前一个MACD: &Bound<'_, 平滑异同移动平均线Py>,
        当前收盘价: f64,
        当前时间: i64,
    ) -> Self {
        let inner = chanlun::indicators::平滑异同移动平均线::增量计算(
            &前一个MACD.borrow().inner,
            当前收盘价,
            当前时间,
        );
        Self { inner }
    }

    #[classmethod]
    /// :param 前一个MACD: 前一个MACD指标对象
    fn 增量计算_K线(
        _cls: &Bound<'_, PyType>,
        前一个MACD: &Bound<'_, 平滑异同移动平均线Py>,
        当前K线: &Bound<'_, PyAny>,
        计算方式: &str,
    ) -> PyResult<Self> {
        let 计算值 = K线取值(当前K线, 计算方式)?;
        Ok(Self {
            inner: chanlun::indicators::平滑异同移动平均线::增量计算(
                &前一个MACD.borrow().inner,
                计算值,
                获取时间戳(当前K线)?,
            ),
        })
    }
}

// ========== 相对强弱指数 ==========

/// RSI 技术指标 — 相对强弱指数（Wilder SMA 平滑）。
///
/// 属性:
///   时间戳: str / 收盘价: float / 周期: int / 超买阈值: float / 超卖阈值: float
///   RSI: float|None — 当前 RSI 值
///   RSI_SMA: float|None — RSI 的 SMA 平滑值
///   平均上涨: float|None / 平均下跌: float|None
///   上涨幅度: float / 下跌幅度: float / 平滑系数: float
///   RSI历史队列: list[float]
///
/// 方法（均为 classmethod，直接构造实例）:
///   首次计算(初始收盘价, 初始时间, 周期=14, 超买阈值=70, 超卖阈值=30, RSI_SMA周期=None)
///   首次计算_K线(k线, 计算方式, ...)
///   增量计算(前一个RSI, 当前收盘价, 当前时间)
///   增量计算_K线(前一个RSI, 当前K线, 计算方式)
#[pyclass(name = "相对强弱指数", module = "chanlun._chanlun", from_py_object)]
#[derive(Clone)]
pub struct 相对强弱指数Py {
    pub(crate) inner: chanlun::indicators::相对强弱指数,
}

#[pymethods]
impl 相对强弱指数Py {
    #[new]
    fn new() -> Self {
        unimplemented!("使用 首次计算 或 增量计算 创建")
    }

    #[getter]
    fn 时间戳(&self) -> String {
        self.inner.时间戳.to_string()
    }
    #[getter]
    fn 收盘价(&self) -> f64 {
        self.inner.收盘价
    }
    #[getter]
    fn 周期(&self) -> i64 {
        self.inner.周期
    }
    #[getter]
    fn 超买阈值(&self) -> f64 {
        self.inner.超买阈值
    }
    #[getter]
    fn 超卖阈值(&self) -> f64 {
        self.inner.超卖阈值
    }
    #[getter]
    fn RSI_SMA周期(&self) -> Option<i64> {
        self.inner.RSI_SMA周期
    }
    #[getter]
    fn RSI(&self) -> Option<f64> {
        self.inner.RSI
    }
    #[getter]
    fn 平均上涨(&self) -> Option<f64> {
        self.inner.平均上涨
    }
    #[getter]
    fn 平均下跌(&self) -> Option<f64> {
        self.inner.平均下跌
    }
    #[getter]
    fn 上涨幅度(&self) -> f64 {
        self.inner.上涨幅度
    }
    #[getter]
    fn 下跌幅度(&self) -> f64 {
        self.inner.下跌幅度
    }
    #[getter]
    fn 平滑系数(&self) -> f64 {
        self.inner.平滑系数
    }
    #[getter]
    fn RSI_SMA(&self) -> Option<f64> {
        self.inner.RSI_SMA
    }
    #[getter]
    fn RSI历史队列(&self) -> Vec<f64> {
        self.inner.RSI历史队列.clone()
    }

    fn __str__(&self) -> String {
        format!("相对强弱指数(RSI={:?})", self.inner.RSI)
    }

    fn __repr__(&self) -> String {
        self.__str__()
    }

    #[classmethod]
    #[pyo3(signature = (初始收盘价, 初始时间, 周期 = None, 超买阈值 = None, 超卖阈值 = None, RSI_SMA周期 = None))]
    /// 首次计算RSI（没有足够历史数据时使用）
    fn 首次计算(
        _cls: &Bound<'_, PyType>,
        初始收盘价: f64,
        初始时间: i64,
        周期: Option<i64>,
        超买阈值: Option<f64>,
        超卖阈值: Option<f64>,
        RSI_SMA周期: Option<i64>,
    ) -> Self {
        Self {
            inner: chanlun::indicators::相对强弱指数::首次计算(
                初始收盘价,
                初始时间,
                周期.unwrap_or(14),
                超买阈值.unwrap_or(70.0),
                超卖阈值.unwrap_or(30.0),
                RSI_SMA周期,
            ),
        }
    }

    #[classmethod]
    #[pyo3(signature = (k线, 计算方式, 周期 = None, 超买阈值 = None, 超卖阈值 = None, RSI_SMA周期 = None))]
    /// :param k线: 原始K线
    fn 首次计算_K线(
        _cls: &Bound<'_, PyType>,
        k线: &Bound<'_, PyAny>,
        计算方式: &str,
        周期: Option<i64>,
        超买阈值: Option<f64>,
        超卖阈值: Option<f64>,
        RSI_SMA周期: Option<i64>,
    ) -> PyResult<Self> {
        let 计算值 = K线取值(k线, 计算方式)?;
        Ok(Self {
            inner: chanlun::indicators::相对强弱指数::首次计算(
                计算值,
                获取时间戳(k线)?,
                周期.unwrap_or(14),
                超买阈值.unwrap_or(70.0),
                超卖阈值.unwrap_or(30.0),
                RSI_SMA周期,
            ),
        })
    }

    #[classmethod]
    /// 基于前一个RSI指标增量计算当前RSI
    fn 增量计算(
        _cls: &Bound<'_, PyType>,
        前一个RSI: &Bound<'_, 相对强弱指数Py>,
        当前收盘价: f64,
        当前时间: i64,
    ) -> Self {
        Self {
            inner: chanlun::indicators::相对强弱指数::增量计算(
                &前一个RSI.borrow().inner,
                当前收盘价,
                当前时间,
            ),
        }
    }

    #[classmethod]
    /// :param 前一个RSI: 前一个RSI指标对象
    fn 增量计算_K线(
        _cls: &Bound<'_, PyType>,
        前一个RSI: &Bound<'_, 相对强弱指数Py>,
        当前K线: &Bound<'_, PyAny>,
        计算方式: &str,
    ) -> PyResult<Self> {
        let 计算值 = K线取值(当前K线, 计算方式)?;
        Ok(Self {
            inner: chanlun::indicators::相对强弱指数::增量计算(
                &前一个RSI.borrow().inner,
                计算值,
                获取时间戳(当前K线)?,
            ),
        })
    }
}

// ========== 随机指标 ==========

/// KDJ 技术指标 — 随机指标（Stochastic Oscillator）。
///
/// 属性:
///   时间戳: str / 最高价: float / 最低价: float / 收盘价: float
///   N: int (RSV周期) / M1: int (K值平滑) / M2: int (D值平滑) / 超买阈值 / 超卖阈值
///   RSV: float|None — 未成熟随机值
///   K: float|None / D: float|None / J: float|None — KDJ 三线值
///   历史最高价队列 / 历史最低价队列: list[float]
///   前一个RSV / 前一个K / 前一个D: float|None
///
/// 方法（均为 classmethod，直接构造实例）:
///   首次计算(最高价, 最低价, 收盘价, 时间, N=9, M1=3, M2=3, 超买=80, 超卖=20)
///   首次计算_K线(k线, 计算方式, ...)
///   增量计算(前一个KDJ, 最高价, 最低价, 收盘价, 时间)
///   增量计算_K线(前一个KDJ, 当前K线, 计算方式)
#[pyclass(name = "随机指标", module = "chanlun._chanlun", from_py_object)]
#[derive(Clone)]
pub struct 随机指标Py {
    pub(crate) inner: chanlun::indicators::随机指标,
}

#[pymethods]
impl 随机指标Py {
    #[new]
    fn new() -> Self {
        unimplemented!("使用 首次计算 或 增量计算 创建")
    }

    #[getter]
    fn 时间戳(&self) -> String {
        self.inner.时间戳.to_string()
    }
    #[getter]
    fn 最高价(&self) -> f64 {
        self.inner.最高价
    }
    #[getter]
    fn 最低价(&self) -> f64 {
        self.inner.最低价
    }
    #[getter]
    fn 收盘价(&self) -> f64 {
        self.inner.收盘价
    }
    #[getter]
    fn N(&self) -> i64 {
        self.inner.N
    }
    #[getter]
    fn M1(&self) -> i64 {
        self.inner.M1
    }
    #[getter]
    fn M2(&self) -> i64 {
        self.inner.M2
    }
    #[getter]
    fn 超买阈值(&self) -> f64 {
        self.inner.超买阈值
    }
    #[getter]
    fn 超卖阈值(&self) -> f64 {
        self.inner.超卖阈值
    }
    #[getter]
    fn RSV(&self) -> Option<f64> {
        self.inner.RSV
    }
    #[getter]
    fn K(&self) -> Option<f64> {
        self.inner.K
    }
    #[getter]
    fn D(&self) -> Option<f64> {
        self.inner.D
    }
    #[getter]
    fn J(&self) -> Option<f64> {
        self.inner.J
    }
    #[getter]
    fn 历史最高价队列(&self) -> Vec<f64> {
        self.inner.历史最高价队列.clone()
    }
    #[getter]
    fn 历史最低价队列(&self) -> Vec<f64> {
        self.inner.历史最低价队列.clone()
    }
    #[getter]
    fn 前一个RSV(&self) -> Option<f64> {
        self.inner.前一个RSV
    }
    #[getter]
    fn 前一个K(&self) -> Option<f64> {
        self.inner.前一个K
    }
    #[getter]
    fn 前一个D(&self) -> Option<f64> {
        self.inner.前一个D
    }

    fn __str__(&self) -> String {
        format!(
            "随机指标(K={:?}, D={:?}, J={:?})",
            self.inner.K, self.inner.D, self.inner.J
        )
    }
    fn __repr__(&self) -> String {
        self.__str__()
    }

    #[classmethod]
    #[pyo3(signature = (初始最高价, 初始最低价, 初始收盘价, 初始时间, N = None, M1 = None, M2 = None, 超买阈值 = None, 超卖阈值 = None))]
    /// 首次计算KDJ（无历史数据时）
    fn 首次计算(
        _cls: &Bound<'_, PyType>,
        初始最高价: f64,
        初始最低价: f64,
        初始收盘价: f64,
        初始时间: i64,
        N: Option<i64>,
        M1: Option<i64>,
        M2: Option<i64>,
        超买阈值: Option<f64>,
        超卖阈值: Option<f64>,
    ) -> Self {
        Self {
            inner: chanlun::indicators::随机指标::首次计算(
                初始最高价,
                初始最低价,
                初始收盘价,
                初始时间,
                N.unwrap_or(9),
                M1.unwrap_or(3),
                M2.unwrap_or(3),
                超买阈值.unwrap_or(80.0),
                超卖阈值.unwrap_or(20.0),
            ),
        }
    }

    #[classmethod]
    #[pyo3(signature = (k线, _计算方式, RSV周期 = None, K值平滑周期 = None, D值平滑周期 = None, 超买阈值 = None, 超卖阈值 = None))]
    /// :param k线: 原始K线
    fn 首次计算_K线(
        _cls: &Bound<'_, PyType>,
        k线: &Bound<'_, PyAny>,
        _计算方式: &str,
        RSV周期: Option<i64>,
        K值平滑周期: Option<i64>,
        D值平滑周期: Option<i64>,
        超买阈值: Option<f64>,
        超卖阈值: Option<f64>,
    ) -> PyResult<Self> {
        let 收盘价: f64 = k线.getattr("收盘价")?.extract()?;
        let 最高价: f64 = k线.getattr("高")?.extract()?;
        let 最低价: f64 = k线.getattr("低")?.extract()?;
        Ok(Self {
            inner: chanlun::indicators::随机指标::首次计算(
                最高价,
                最低价,
                收盘价,
                获取时间戳(k线)?,
                RSV周期.unwrap_or(9),
                K值平滑周期.unwrap_or(3),
                D值平滑周期.unwrap_or(3),
                超买阈值.unwrap_or(80.0),
                超卖阈值.unwrap_or(20.0),
            ),
        })
    }

    #[classmethod]
    /// 基于前一个KDJ对象和当前三价，增量计算当前KDJ值
    fn 增量计算(
        _cls: &Bound<'_, PyType>,
        前一个KDJ: &Bound<'_, 随机指标Py>,
        当前最高价: f64,
        当前最低价: f64,
        当前收盘价: f64,
        当前时间: i64,
    ) -> Self {
        Self {
            inner: chanlun::indicators::随机指标::增量计算(
                &前一个KDJ.borrow().inner,
                当前最高价,
                当前最低价,
                当前收盘价,
                当前时间,
            ),
        }
    }

    #[classmethod]
    /// :param 前一个KDJ: 前一个KDJ指标对象
    fn 增量计算_K线(
        _cls: &Bound<'_, PyType>,
        前一个KDJ: &Bound<'_, 随机指标Py>,
        当前K线: &Bound<'_, PyAny>,
        _计算方式: &str,
    ) -> PyResult<Self> {
        let 收盘价: f64 = 当前K线.getattr("收盘价")?.extract()?;
        let 最高价: f64 = 当前K线.getattr("高")?.extract()?;
        let 最低价: f64 = 当前K线.getattr("低")?.extract()?;
        Ok(Self {
            inner: chanlun::indicators::随机指标::增量计算(
                &前一个KDJ.borrow().inner,
                最高价,
                最低价,
                收盘价,
                获取时间戳(当前K线)?,
            ),
        })
    }
}

// ========== 指标 (static namespace) ==========

/// 指标 — 静态工具类，提供指标计算的辅助方法。
///
/// 方法:
///   K线取值(k线, 指标计算方式) -> float (classmethod)
///     根据计算方式从K线提取数值。
///     计算方式: "收盘价" / "开盘价" / "高" / "低" / "均值" 等
#[pyclass(name = "指标", module = "chanlun._chanlun")]
pub struct 指标Py;

#[pymethods]
impl 指标Py {
    #[classmethod]
    /// 根据计算方式从K线中取值
    fn K线取值(
        _cls: &Bound<'_, PyType>,
        k线: &Bound<'_, PyAny>,
        指标计算方式: &str,
    ) -> PyResult<f64> {
        K线取值(k线, 指标计算方式)
    }
}

// ========== Helper functions ==========

pub(crate) fn K线取值(k线: &Bound<'_, PyAny>, 计算方式: &str) -> PyResult<f64> {
    let 开盘: f64 = k线.getattr("开盘价")?.extract()?;
    let 高: f64 = k线.getattr("高")?.extract()?;
    let 低: f64 = k线.getattr("低")?.extract()?;
    let 收盘: f64 = k线.getattr("收盘价")?.extract()?;
    Ok(chanlun::indicators::K线取值(
        开盘,
        高,
        低,
        收盘,
        计算方式,
    ))
}

fn 获取时间戳(k线: &Bound<'_, PyAny>) -> PyResult<i64> {
    let ts = k线.getattr("时间戳")?;
    if let Ok(i) = ts.extract::<i64>() {
        return Ok(i);
    }
    // Try float
    if let Ok(f) = ts.extract::<f64>() {
        return Ok(f as i64);
    }
    // Try calling .timestamp() if it's a datetime
    if let Ok(method) = ts.getattr("timestamp") {
        let result: f64 = method.call0()?.extract()?;
        return Ok(result as i64);
    }
    Err(pyo3::exceptions::PyTypeError::new_err("无法获取时间戳"))
}

pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<平滑异同移动平均线Py>()?;
    m.add_class::<相对强弱指数Py>()?;
    m.add_class::<随机指标Py>()?;
    m.add_class::<指标Py>()?;
    Ok(())
}
