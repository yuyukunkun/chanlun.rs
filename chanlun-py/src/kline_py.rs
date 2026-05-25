use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyType};
use std::collections::HashMap;

use crate::config_py::缠论配置Py;
use crate::indicators_py::{平滑异同移动平均线Py, 相对强弱指数Py, 随机指标Py};
use crate::types_py::相对方向Py;

// ========== K线 ==========

#[pyclass(name = "K线")]
#[derive(Clone)]
pub struct K线Py {
    pub(crate) inner: chanlun::kline::bar::K线,
}

#[pymethods]
impl K线Py {
    #[new]
    #[pyo3(signature = (标识 = "bar", 序号 = 0, 周期 = 60, 时间戳 = 0, 高 = 0.0, 低 = 0.0, 开盘价 = 0.0, 收盘价 = 0.0, 成交量 = 0.0))]
    fn new(
        标识: &str,
        序号: i64,
        周期: i64,
        时间戳: i64,
        高: f64,
        低: f64,
        开盘价: f64,
        收盘价: f64,
        成交量: f64,
    ) -> Self {
        Self {
            inner: chanlun::kline::bar::K线 {
                标识: 标识.to_string(),
                序号,
                周期,
                时间戳,
                高,
                低,
                开盘价,
                收盘价,
                成交量,
                macd: None,
                rsi: None,
                kdj: None,
            },
        }
    }

    #[getter]
    fn 标识(&self) -> String {
        self.inner.标识.clone()
    }
    #[setter]
    fn set_标识(&mut self, v: String) {
        self.inner.标识 = v;
    }

    #[getter]
    fn 序号(&self) -> i64 {
        self.inner.序号
    }
    #[setter]
    fn set_序号(&mut self, v: i64) {
        self.inner.序号 = v;
    }

    #[getter]
    fn 周期(&self) -> i64 {
        self.inner.周期
    }
    #[setter]
    fn set_周期(&mut self, v: i64) {
        self.inner.周期 = v;
    }

    #[getter]
    fn 时间戳(&self) -> i64 {
        self.inner.时间戳
    }
    #[setter]
    fn set_时间戳(&mut self, v: i64) {
        self.inner.时间戳 = v;
    }

    #[getter]
    fn 高(&self) -> f64 {
        self.inner.高
    }
    #[setter]
    fn set_高(&mut self, v: f64) {
        self.inner.高 = v;
    }

    #[getter]
    fn 低(&self) -> f64 {
        self.inner.低
    }
    #[setter]
    fn set_低(&mut self, v: f64) {
        self.inner.低 = v;
    }

    #[getter]
    fn 开盘价(&self) -> f64 {
        self.inner.开盘价
    }
    #[setter]
    fn set_开盘价(&mut self, v: f64) {
        self.inner.开盘价 = v;
    }

    #[getter]
    fn 收盘价(&self) -> f64 {
        self.inner.收盘价
    }
    #[setter]
    fn set_收盘价(&mut self, v: f64) {
        self.inner.收盘价 = v;
    }

    #[getter]
    fn 成交量(&self) -> f64 {
        self.inner.成交量
    }
    #[setter]
    fn set_成交量(&mut self, v: f64) {
        self.inner.成交量 = v;
    }

    #[getter]
    fn 方向(&self) -> 相对方向Py {
        相对方向Py {
            inner: self.inner.方向(),
        }
    }

    #[getter]
    fn MACD(&self) -> Option<平滑异同移动平均线Py> {
        self.inner
            .macd
            .as_ref()
            .map(|m| 平滑异同移动平均线Py { inner: m.clone() })
    }

    #[getter]
    fn RSI(&self) -> Option<相对强弱指数Py> {
        self.inner
            .rsi
            .as_ref()
            .map(|r| 相对强弱指数Py { inner: r.clone() })
    }

    #[getter]
    fn KDJ(&self) -> Option<随机指标Py> {
        self.inner
            .kdj
            .as_ref()
            .map(|k| 随机指标Py { inner: k.clone() })
    }

    fn __str__(&self) -> String {
        format!("{}", self.inner)
    }

    fn __repr__(&self) -> String {
        self.__str__()
    }

    fn __bytes__(&self, py: Python<'_>) -> Py<PyBytes> {
        PyBytes::new(py, &self.inner.to_bytes()).into()
    }

    #[classmethod]
    #[pyo3(signature = (标识, 时间戳, 开盘价, 最高价, 最低价, 收盘价, 成交量, 序号 = None, 周期 = None))]
    fn 创建普K(
        _cls: &Bound<'_, PyType>,
        标识: &str,
        时间戳: i64,
        开盘价: f64,
        最高价: f64,
        最低价: f64,
        收盘价: f64,
        成交量: f64,
        序号: Option<i64>,
        周期: Option<i64>,
    ) -> Self {
        Self {
            inner: chanlun::kline::bar::K线::创建普K(
                标识,
                时间戳,
                开盘价,
                最高价,
                最低价,
                收盘价,
                成交量,
                序号.unwrap_or(0),
                周期.unwrap_or(60),
            ),
        }
    }

    #[classmethod]
    fn 保存到DAT文件(
        _cls: &Bound<'_, PyType>,
        路径: &str,
        K线序列: Vec<Py<Self>>,
        py: Python<'_>,
    ) -> PyResult<()> {
        let refs: Vec<_> = K线序列.iter().map(|k| k.bind(py).borrow()).collect();
        let bars: Vec<&chanlun::kline::bar::K线> = refs.iter().map(|r| &r.inner).collect();
        chanlun::kline::bar::K线::保存到DAT文件(路径, &bars)
            .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))
    }

    #[classmethod]
    fn 读取大端字节数组(
        _cls: &Bound<'_, PyType>,
        字节组: &Bound<'_, PyBytes>,
        周期: i64,
        标识: &str,
    ) -> Option<Self> {
        chanlun::kline::bar::K线::读取大端字节数组(字节组.as_bytes(), 周期, 标识)
            .map(|inner| Self { inner })
    }

    #[classmethod]
    fn 获取MACD(
        _cls: &Bound<'_, PyType>,
        k线序列: Vec<Py<Self>>,
        始: &Bound<'_, Self>,
        终: &Bound<'_, Self>,
        py: Python<'_>,
    ) -> HashMap<String, f64> {
        let refs: Vec<_> = k线序列.iter().map(|k| k.bind(py).borrow()).collect();
        let bars: Vec<&chanlun::kline::bar::K线> = refs.iter().map(|r| &r.inner).collect();
        chanlun::kline::bar::K线::获取MACD(&bars, &始.borrow().inner, &终.borrow().inner)
    }

    #[staticmethod]
    fn 截取(
        序列: Vec<Py<Self>>,
        始: &Bound<'_, Self>,
        终: &Bound<'_, Self>,
    ) -> Option<Vec<Py<Self>>> {
        let start_ptr = 始.as_ptr();
        let end_ptr = 终.as_ptr();
        let start_idx = 序列.iter().position(|k| k.as_ptr() == start_ptr)?;
        let end_idx = 序列.iter().position(|k| k.as_ptr() == end_ptr)?;
        if start_idx <= end_idx {
            Some(
                序列
                    .into_iter()
                    .skip(start_idx)
                    .take(end_idx - start_idx + 1)
                    .collect(),
            )
        } else {
            None
        }
    }
}

// ========== 缠论K线 ==========

#[pyclass(name = "缠论K线", unsendable)]
#[derive(Clone)]
pub struct 缠论K线Py {
    pub(crate) inner: std::rc::Rc<chanlun::kline::chan_kline::缠论K线>,
}

#[pymethods]
impl 缠论K线Py {
    #[new]
    fn new() -> Self {
        unimplemented!("使用 创建缠K 创建")
    }

    #[getter]
    fn 序号(&self) -> i64 {
        self.inner.序号
    }

    #[getter]
    fn 时间戳(&self) -> i64 {
        self.inner.时间戳
    }

    #[getter]
    fn 高(&self) -> f64 {
        self.inner.高
    }

    #[getter]
    fn 低(&self) -> f64 {
        self.inner.低
    }

    #[getter]
    fn 方向(&self) -> 相对方向Py {
        相对方向Py {
            inner: self.inner.方向,
        }
    }

    #[getter]
    fn 分型(&self) -> Option<crate::types_py::分型结构Py> {
        self.inner
            .分型
            .map(|f| crate::types_py::分型结构Py { inner: f })
    }

    #[getter]
    fn 周期(&self) -> i64 {
        self.inner.周期
    }

    #[getter]
    fn 标识(&self) -> String {
        self.inner.标识.clone()
    }

    #[getter]
    fn 分型特征值(&self) -> f64 {
        self.inner.分型特征值
    }

    #[getter]
    fn 原始起始序号(&self) -> i64 {
        self.inner.原始起始序号
    }

    #[getter]
    fn 原始结束序号(&self) -> i64 {
        self.inner.原始结束序号
    }

    #[getter]
    fn 标的K线(&self) -> K线Py {
        K线Py {
            inner: (*self.inner.标的K线).clone(),
        }
    }

    fn __str__(&self) -> String {
        format!("{}", self.inner)
    }

    fn __repr__(&self) -> String {
        self.__str__()
    }

    fn 镜像(&self) -> Self {
        Self {
            inner: std::rc::Rc::new(self.inner.镜像()),
        }
    }

    #[getter]
    fn 与MACD柱子匹配(&self) -> bool {
        self.inner.与MACD柱子匹配()
    }

    #[getter]
    fn 与RSI匹配(&self) -> bool {
        self.inner.与RSI匹配()
    }

    #[getter]
    fn 与KDJ匹配(&self) -> bool {
        self.inner.与KDJ匹配()
    }

    #[classmethod]
    fn 时间戳对齐(
        _cls: &Bound<'_, PyType>,
        基线: Vec<Py<Self>>,
        k线: &Bound<'_, Self>,
        py: Python<'_>,
    ) -> i64 {
        let rc_list: Vec<_> = 基线
            .iter()
            .map(|k| std::rc::Rc::clone(&k.bind(py).borrow().inner))
            .collect();
        chanlun::kline::chan_kline::缠论K线::时间戳对齐(&rc_list, &k线.borrow().inner)
    }

    #[classmethod]
    fn 创建缠K(
        _cls: &Bound<'_, PyType>,
        时间戳: i64,
        高: f64,
        低: f64,
        方向: &Bound<'_, 相对方向Py>,
        结构: Option<&Bound<'_, crate::types_py::分型结构Py>>,
        原始序号: i64,
        普k: &Bound<'_, K线Py>,
        之前: Option<&Bound<'_, Self>>,
    ) -> Self {
        let prev_ref = 之前.map(|prev| prev.borrow());
        let prev_inner = prev_ref.as_ref().map(|r| r.inner.as_ref());
        let inner = chanlun::kline::chan_kline::缠论K线::创建缠K(
            时间戳,
            高,
            低,
            方向.borrow().inner,
            结构.map(|s| s.borrow().inner),
            原始序号,
            std::rc::Rc::new(普k.borrow().inner.clone()),
            prev_inner,
        );
        Self {
            inner: std::rc::Rc::new(inner),
        }
    }

    #[classmethod]
    fn 兼并(
        _cls: &Bound<'_, PyType>,
        之前缠K: Option<&Bound<'_, Self>>,
        当前缠K: &Bound<'_, Self>,
        当前普K: &Bound<'_, K线Py>,
        配置: &Bound<'_, 缠论配置Py>,
        py: Python<'_>,
    ) -> PyResult<(Option<Self>, Option<String>)> {
        let mut ck_inner = (*当前缠K.borrow().inner).clone();
        let config = 配置.borrow().to_rust_config(py)?;
        let prev_ref = 之前缠K.map(|prev| prev.borrow());
        let prev_inner = prev_ref.as_ref().map(|r| r.inner.as_ref());
        let (result, mode) = chanlun::kline::chan_kline::缠论K线::兼并(
            prev_inner,
            &mut ck_inner,
            &当前普K.borrow().inner,
            &config,
        );
        Ok((result.map(|rc| Self { inner: rc }), mode))
    }

    #[classmethod]
    fn 分析(
        _cls: &Bound<'_, PyType>,
        当前K线: &Bound<'_, K线Py>,
        缠K序列: Vec<Py<Self>>,
        普K序列: Vec<Py<K线Py>>,
        配置: &Bound<'_, 缠论配置Py>,
        py: Python<'_>,
    ) -> PyResult<(String, Option<Py<PyAny>>)> {
        let ck_inner = 当前K线.borrow().inner.clone();
        let config = 配置.borrow().to_rust_config(py)?;

        let mut ck_seq: Vec<_> = 缠K序列
            .iter()
            .map(|k| std::rc::Rc::clone(&k.bind(py).borrow().inner))
            .collect();
        let mut bar_seq: Vec<_> = 普K序列
            .iter()
            .map(|k| std::rc::Rc::new(k.bind(py).borrow().inner.clone()))
            .collect();

        let (status, _fractal) = chanlun::kline::chan_kline::缠论K线::分析(
            ck_inner,
            &mut ck_seq,
            &mut bar_seq,
            &config,
        );

        Ok((status, None))
    }

    #[staticmethod]
    fn 截取(
        序列: Vec<Py<Self>>,
        始: &Bound<'_, Self>,
        终: &Bound<'_, Self>,
    ) -> Option<Vec<Py<Self>>> {
        let start_ptr = 始.as_ptr();
        let end_ptr = 终.as_ptr();
        let start_idx = 序列.iter().position(|k| k.as_ptr() == start_ptr)?;
        let end_idx = 序列.iter().position(|k| k.as_ptr() == end_ptr)?;
        if start_idx <= end_idx {
            Some(
                序列
                    .into_iter()
                    .skip(start_idx)
                    .take(end_idx - start_idx + 1)
                    .collect(),
            )
        } else {
            None
        }
    }
}

pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<K线Py>()?;
    m.add_class::<缠论K线Py>()?;
    Ok(())
}
