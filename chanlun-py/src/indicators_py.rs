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
use std::sync::Arc;

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

// ========== 布林带 ==========

/// 布林带（BOLL）— 基于移动平均和标准差的波动率通道。
///
/// 属性:
///   时间戳: int / 周期: int / 标准差倍数: float
///   上轨: float — 中轨 + 标准差倍数 * 标准差
///   中轨: float — 移动平均线
///   下轨: float — 中轨 - 标准差倍数 * 标准差
///
/// 方法（均为 classmethod，直接构造实例）:
///   首次计算(时间戳, 价格, 周期=20, 标准差倍数=2.0) -> 布林带
///   增量计算(前一个布林带, 时间戳, 价格) -> 布林带
#[pyclass(name = "布林带", module = "chanlun._chanlun", from_py_object)]
#[derive(Clone)]
pub struct 布林带Py {
    pub(crate) inner: chanlun::indicators::布林带,
}

#[pymethods]
impl 布林带Py {
    #[new]
    fn new() -> Self {
        unimplemented!("使用 首次计算 或 增量计算 创建")
    }

    #[getter]
    fn 时间戳(&self) -> i64 {
        self.inner.时间戳
    }

    #[getter]
    fn 周期(&self) -> usize {
        self.inner.周期
    }

    #[getter]
    fn 标准差倍数(&self) -> f64 {
        self.inner.标准差倍数
    }

    #[getter]
    fn 上轨(&self) -> f64 {
        self.inner.上轨
    }

    #[getter]
    fn 中轨(&self) -> f64 {
        self.inner.中轨
    }

    #[getter]
    fn 下轨(&self) -> f64 {
        self.inner.下轨
    }

    fn __str__(&self) -> String {
        format!(
            "布林带(上={:.2}, 中={:.2}, 下={:.2})",
            self.inner.上轨, self.inner.中轨, self.inner.下轨
        )
    }

    fn __repr__(&self) -> String {
        self.__str__()
    }

    #[classmethod]
    #[pyo3(signature = (k线, 计算方式, 周期 = 20, 标准差倍数 = 2.0))]
    fn 首次计算(
        _cls: &Bound<'_, PyType>,
        k线: &Bound<'_, PyAny>,
        计算方式: &str,
        周期: usize,
        标准差倍数: f64,
    ) -> PyResult<Self> {
        let 价格 = K线取值(k线, 计算方式)?;
        let 时间戳 = 获取时间戳(k线)?;
        Ok(Self {
            inner: chanlun::indicators::布林带::首次计算(时间戳, 价格, 周期, 标准差倍数),
        })
    }

    #[classmethod]
    fn 增量计算(
        _cls: &Bound<'_, PyType>,
        前一个布林带: &Bound<'_, 布林带Py>,
        当前K线: &Bound<'_, PyAny>,
        计算方式: &str,
    ) -> PyResult<Self> {
        let 价格 = K线取值(当前K线, 计算方式)?;
        let 时间戳 = 获取时间戳(当前K线)?;
        Ok(Self {
            inner: chanlun::indicators::布林带::增量计算(
                &前一个布林带.borrow().inner,
                时间戳,
                价格,
            ),
        })
    }
}

// ========== 指标容器 ==========

/// 指标容器 — 挂载在每根 K线上，基于注册表模式持有该时刻所有指标快照。
///
/// 与 Python `指标容器` 保持一致：
///   - 默认名称："macd"/"rsi"/"kdj"/"boll" → 对应指标对象
///   - 多参数变体：key 格式 "MACD_{快}_{慢}_{信号}" / "RSI_{周期}" 等
///   - 均线组：通过 "均线" 获取 dict[str, float]
///   - 单值指标：通过 "单值" 获取 dict[str, float]
#[pyclass(name = "指标容器", module = "chanlun._chanlun", skip_from_py_object)]
#[derive(Clone)]
pub struct 指标容器Py {
    pub(crate) inner: chanlun::indicators::指标容器,
}

/// 将 Rust 指标值 转换为 Python 对象
fn 指标值_to_py(value: &chanlun::indicators::指标值, py: Python<'_>) -> PyResult<Py<PyAny>> {
    use chanlun::indicators::指标值;
    match value {
        指标值::MACD(m) => {
            Ok(Py::new(py, 平滑异同移动平均线Py { inner: m.clone() })?.into_any())
        }
        指标值::RSI(r) => Ok(Py::new(py, 相对强弱指数Py { inner: r.clone() })?.into_any()),
        指标值::KDJ(k) => Ok(Py::new(py, 随机指标Py { inner: k.clone() })?.into_any()),
        指标值::BOLL(b) => Ok(Py::new(py, 布林带Py { inner: b.clone() })?.into_any()),
        指标值::均线(map) | 指标值::单值(map) => {
            let dict = pyo3::types::PyDict::new(py);
            for (k, v) in map {
                dict.set_item(k, *v)?;
            }
            Ok(dict.into())
        }
    }
}

#[pymethods]
impl 指标容器Py {
    #[new]
    fn new() -> Self {
        Self {
            inner: chanlun::indicators::指标容器::new(),
        }
    }

    /// 按名称获取指标值
    fn 获取(&self, 名称: &str, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        match self.inner.获取(名称) {
            Some(v) => 指标值_to_py(v, py).map(Some),
            None => Ok(None),
        }
    }

    /// 按名称设置指标值（仅支持 MACD/RSI/KDJ/BOLL 四种类型）
    #[pyo3(signature = (名称, 值))]
    fn 设置(&mut self, 名称: &str, 值: &Bound<'_, PyAny>) -> PyResult<()> {
        use chanlun::indicators::指标值;
        if let Ok(m) = 值.cast::<平滑异同移动平均线Py>() {
            self.inner
                .设置(名称, 指标值::MACD(m.borrow().inner.clone()));
            return Ok(());
        }
        if let Ok(r) = 值.cast::<相对强弱指数Py>() {
            self.inner.设置(名称, 指标值::RSI(r.borrow().inner.clone()));
            return Ok(());
        }
        if let Ok(k) = 值.cast::<随机指标Py>() {
            self.inner.设置(名称, 指标值::KDJ(k.borrow().inner.clone()));
            return Ok(());
        }
        if let Ok(b) = 值.cast::<布林带Py>() {
            self.inner
                .设置(名称, 指标值::BOLL(b.borrow().inner.clone()));
            return Ok(());
        }
        Err(pyo3::exceptions::PyTypeError::new_err(
            "不支持的类型，仅支持 MACD/RSI/KDJ/BOLL 指标",
        ))
    }

    /// 检查是否包含指定名称的指标
    fn 包含(&self, 名称: &str) -> bool {
        self.inner.包含(名称)
    }

    /// 返回所有已注册的指标名称
    fn keys(&self) -> Vec<String> {
        self.inner._数据.keys().cloned().collect()
    }

    fn __getitem__(&self, 名称: &str, py: Python<'_>) -> PyResult<Py<PyAny>> {
        if self.包含(名称) {
            match self.inner.获取(名称) {
                Some(v) => 指标值_to_py(v, py),
                None => Ok(py.None()),
            }
        } else {
            Err(pyo3::exceptions::PyKeyError::new_err(format!(
                "指标 '{}' 不存在",
                名称
            )))
        }
    }

    fn __getattr__(&self, 名称: &str, py: Python<'_>) -> PyResult<Py<PyAny>> {
        if 名称 == "_数据" {
            // 返回内部数据字典的 Python 表示
            let dict = pyo3::types::PyDict::new(py);
            for key in self.inner._数据.keys() {
                if let Some(v) = self.inner.获取(key) {
                    dict.set_item(key, 指标值_to_py(v, py)?)?;
                } else {
                    dict.set_item(key, py.None())?;
                }
            }
            return Ok(dict.into());
        }
        if self.包含(名称) {
            match self.inner.获取(名称) {
                Some(v) => 指标值_to_py(v, py),
                None => Ok(py.None()),
            }
        } else {
            Err(pyo3::exceptions::PyAttributeError::new_err(format!(
                "指标 '{}' 不存在于 指标容器 中",
                名称
            )))
        }
    }

    fn __contains__(&self, 名称: &str) -> bool {
        self.包含(名称)
    }

    fn __str__(&self) -> String {
        self.inner.to_string()
    }

    fn __repr__(&self) -> String {
        self.__str__()
    }
}

// ========== 指标 (static namespace) ==========

/// 指标 — 静态工具类，提供指标计算的辅助方法。
///
/// 方法:
///   :meth:`K线取值` — 根据计算方式从K线提取数值
///     (计算方式: "开"/"高"/"低"/"收"/"高低均值"/"高低收均值"/"开高低收均值")
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

// ========== 均线工具 ==========

/// 均线工具 — 增量 SMA/EMA 计算的静态方法容器。
///
/// 方法:
///   :meth:`增量SMA` — 基于前一根K线的 SMA 值，增量计算当前 SMA
///   :meth:`增量EMA` — 用前一根K线的 EMA 值递推计算当前 EMA
#[pyclass(name = "均线工具", module = "chanlun._chanlun")]
pub struct 均线工具Py;

#[pymethods]
impl 均线工具Py {
    /// 基于前一根K线的 SMA 值，增量计算当前 SMA
    #[staticmethod]
    #[pyo3(signature = (普K序列, period, 计算方式))]
    fn 增量SMA(
        普K序列: Vec<Py<crate::kline_py::K线Py>>,
        period: i64,
        计算方式: &str,
        py: Python<'_>,
    ) -> PyResult<f64> {
        if 普K序列.is_empty() {
            return Err(pyo3::exceptions::PyValueError::new_err("普K序列 不能为空"));
        }
        let n = 普K序列.len();
        // 提取所有K线值（一次性 borrow）
        let values: Vec<f64> = 普K序列
            .iter()
            .map(|k| {
                let inner = &k.bind(py).borrow().inner;
                chanlun::indicators::K线取值(
                    inner.开盘价,
                    inner.高,
                    inner.低,
                    inner.收盘价,
                    计算方式,
                )
            })
            .collect();

        if n <= period as usize {
            let start = n.saturating_sub(period as usize);
            let sum: f64 = values[start..].iter().sum();
            return Ok(sum / (n.max(1)) as f64);
        }

        let prev_key = format!("SMA_{}", period);
        // 尝试从前一根K线的均线缓存中读取
        let prev_cached = 普K序列[n - 2]
            .bind(py)
            .borrow()
            .inner
            .指标
            .read()
            .unwrap()
            .均线()
            .and_then(|m| m.get(&prev_key))
            .copied();
        if let Some(prev) = prev_cached {
            let 当前价 = values[n - 1];
            let oldest = values[n - period as usize - 1];
            return Ok(prev + (当前价 - oldest) / period as f64);
        }

        // 回退：完整计算最近 period 根K线
        let sum: f64 = values[n - period as usize..].iter().sum();
        Ok(sum / period as f64)
    }

    /// 用前一根K线的 EMA 值递推
    #[staticmethod]
    #[pyo3(signature = (普K序列, period, 计算方式, 前值 = None))]
    fn 增量EMA(
        普K序列: Vec<Py<crate::kline_py::K线Py>>,
        period: i64,
        计算方式: &str,
        前值: Option<f64>,
        py: Python<'_>,
    ) -> PyResult<f64> {
        if 普K序列.is_empty() {
            return Err(pyo3::exceptions::PyValueError::new_err("普K序列 不能为空"));
        }
        let last = 普K序列.last().unwrap().bind(py).borrow();
        let 当前价 = chanlun::indicators::K线取值(
            last.inner.开盘价,
            last.inner.高,
            last.inner.低,
            last.inner.收盘价,
            计算方式,
        );
        match 前值 {
            None => Ok(当前价),
            Some(prev) => {
                let k = 2.0 / (period as f64 + 1.0);
                Ok(当前价 * k + prev * (1.0 - k))
            }
        }
    }
}

// ========== 指标计算器 ==========

/// 指标计算器 — 在缠K合并之前，增量计算所有开启的指标并挂载到K线上。
///
/// 方法:
///   :meth:`计算并挂载` — 增量计算所有开启的指标，将结果写入 ``当前K线.指标``
#[pyclass(name = "指标计算器", module = "chanlun._chanlun")]
pub struct 指标计算器Py;

#[pymethods]
impl 指标计算器Py {
    /// 增量计算所有开启的指标，将结果写入 当前K线.指标
    #[staticmethod]
    fn 计算并挂载(
        _当前K线: &Bound<'_, crate::kline_py::K线Py>,
        全序列: Vec<Py<crate::kline_py::K线Py>>,
        配置: &Bound<'_, crate::config_py::缠论配置Py>,
        py: Python<'_>,
    ) -> PyResult<()> {
        let config = 配置.borrow().to_rust_config(py)?;
        let 全序列_rust: Vec<Arc<chanlun::kline::bar::K线>> = 全序列
            .iter()
            .map(|k| k.bind(py).borrow().inner.clone())
            .collect();
        chanlun::indicators::指标计算器::计算并挂载(&全序列_rust, &config);
        Ok(())
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
    m.add_class::<布林带Py>()?;
    m.add_class::<指标容器Py>()?;
    m.add_class::<指标Py>()?;
    m.add_class::<均线工具Py>()?;
    m.add_class::<指标计算器Py>()?;
    Ok(())
}
