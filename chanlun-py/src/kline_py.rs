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

use parking_lot::RwLock;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict, PyList, PyType};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use crate::config_py::缠论配置Py;
use crate::indicators_py::{
    平滑异同移动平均线Py, 指标容器Py, 相对强弱指数Py, 随机指标Py, 布林带Py,
};
use crate::structure_py::fractal_to_py;
use crate::types_py::相对方向Py;

// ========== K线 ==========

/// 原始 K 线 — OHLCV 数据，可内嵌 MACD/RSI/KDJ 指标。
///
/// 属性（大部分可读写）:
///   标识: str / 序号: int / 周期: int (秒) / 时间戳: int (Unix秒)
///   高: float / 低: float / 开盘价: float / 收盘价: float / 成交量: float
///   方向: 相对方向 (只读) — 根据开盘价vs收盘价判定
///   MACD: 平滑异同移动平均线|None (只读)
///   RSI: 相对强弱指数|None (只读)
///   KDJ: 随机指标|None (只读)
///
/// 类方法:
///   创建普K(标识, 时间戳, 开盘价, 最高价, 最低价, 收盘价, 成交量, 序号?, 周期?) -> K线
///      — 快捷构造普通K线
///   读取大端字节数组(bytes, 周期, 标识?) -> K线
///      — 从大端字节序二进制数据反序列化（兼容 .dat/.nb 文件格式）
///   保存到DAT文件(路径, K线序列) -> 写入 .dat 二进制文件
///   获取MACD(K线序列, 计算方式, 快线周期?, 慢线周期?, 信号周期?) -> list[平滑异同移动平均线]
///      — 对整个K线序列批量计算 MACD
///   截取(序列, 起点K线, 终点K线) -> list — 按时间戳截取K线区间
#[pyclass(name = "K线", module = "chanlun._chanlun")]
pub struct K线Py {
    pub(crate) inner: Arc<chanlun::kline::bar::K线>,
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
            inner: Arc::new(chanlun::kline::bar::K线 {
                标识: 标识.to_string(),
                序号,
                周期,
                时间戳,
                高,
                低,
                开盘价,
                收盘价,
                成交量,
                指标: RwLock::new(chanlun::indicators::指标容器::new()),
            }),
        }
    }

    #[getter]
    fn 标识(&self) -> String {
        self.inner.标识.clone()
    }

    #[getter]
    fn 序号(&self) -> i64 {
        self.inner.序号
    }

    #[getter]
    fn 周期(&self) -> i64 {
        self.inner.周期
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
    fn 开盘价(&self) -> f64 {
        self.inner.开盘价
    }

    #[getter]
    fn 收盘价(&self) -> f64 {
        self.inner.收盘价
    }

    #[getter]
    fn 成交量(&self) -> f64 {
        self.inner.成交量
    }

    #[getter]
    /// :return: 相对方向.向上（开盘<收盘）或 相对方向.向下（开盘>收盘）
    fn 方向(&self, py: Python<'_>) -> Py<相对方向Py> {
        crate::types_py::获取相对方向单例(py, self.inner.方向())
    }

    #[getter]
    fn macd(&self) -> Option<平滑异同移动平均线Py> {
        self.inner
            .指标
            .read()
            .macd_cloned()
            .map(|m| 平滑异同移动平均线Py { inner: m })
    }

    #[getter]
    fn rsi(&self) -> Option<相对强弱指数Py> {
        self.inner
            .指标
            .read()
            .rsi_cloned()
            .map(|r| 相对强弱指数Py { inner: r })
    }

    #[getter]
    fn kdj(&self) -> Option<随机指标Py> {
        self.inner
            .指标
            .read()
            .kdj_cloned()
            .map(|k| 随机指标Py { inner: k })
    }

    #[getter]
    fn boll(&self) -> Option<布林带Py> {
        self.inner
            .指标
            .read()
            .boll_cloned()
            .map(|b| 布林带Py { inner: b })
    }

    /// 读取均线值，如 `k.ma("SMA_5")` → `Optional[float]`
    fn ma(&self, key: &str) -> Option<f64> {
        self.inner.ma(key)
    }

    /// 指标容器 — 包含所有已注册指标（MACD/RSI/KDJ/BOLL/均线/单值）
    #[getter]
    fn 指标(&self) -> 指标容器Py {
        指标容器Py {
            inner: self.inner.指标.read().clone(),
        }
    }

    /// pandas 兼容 — 返回所有字段构成的字典
    #[getter]
    fn __dict__(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let dict = PyDict::new(py);
        dict.set_item("标识", self.标识())?;
        dict.set_item("序号", self.序号())?;
        dict.set_item("周期", self.周期())?;
        dict.set_item("时间戳", self.时间戳())?;
        dict.set_item("高", self.高())?;
        dict.set_item("低", self.低())?;
        dict.set_item("开盘价", self.开盘价())?;
        dict.set_item("收盘价", self.收盘价())?;
        dict.set_item("成交量", self.成交量())?;
        dict.set_item("方向", self.方向(py))?;
        if let Some(v) = self.macd() {
            dict.set_item("macd", v)?;
        }
        if let Some(v) = self.rsi() {
            dict.set_item("rsi", v)?;
        }
        if let Some(v) = self.kdj() {
            dict.set_item("kdj", v)?;
        }
        dict.set_item("指标", self.指标())?;
        Ok(dict.into())
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

    fn __eq__(&self, other: &Bound<'_, PyAny>) -> bool {
        if let Ok(other) = other.extract::<PyRef<'_, Self>>() {
            return Arc::as_ptr(&self.inner) == Arc::as_ptr(&other.inner);
        }
        false
    }

    fn __hash__(&self) -> u64 {
        Arc::as_ptr(&self.inner) as u64
    }

    #[classmethod]
    #[pyo3(signature = (标识, 时间戳, 开盘价, 最高价, 最低价, 收盘价, 成交量, 序号 = None, 周期 = None))]
    /// 快捷构造普通K线
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
            inner: Arc::new(chanlun::kline::bar::K线::创建普K(
                标识,
                时间戳,
                开盘价,
                最高价,
                最低价,
                收盘价,
                成交量,
                序号.unwrap_or(0),
                周期.unwrap_or(60),
            )),
        }
    }

    #[classmethod]
    /// 将K线序列保存为二进制DAT文件
    fn 保存到DAT文件(
        _cls: &Bound<'_, PyType>,
        路径: &str,
        K线序列: Vec<Py<Self>>,
        py: Python<'_>,
    ) -> PyResult<()> {
        let refs: Vec<_> = K线序列.iter().map(|k| k.bind(py).borrow()).collect();
        let bars: Vec<&chanlun::kline::bar::K线> = refs.iter().map(|r| r.inner.as_ref()).collect();
        chanlun::kline::bar::K线::保存到DAT文件(路径, &bars)
            .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))
    }

    #[classmethod]
    /// 从大端字节序二进制数据反序列化K线（兼容.dat/.nb文件格式）
    fn 读取大端字节数组(
        _cls: &Bound<'_, PyType>,
        字节组: &Bound<'_, PyBytes>,
        周期: i64,
        标识: &str,
    ) -> Option<Self> {
        chanlun::kline::bar::K线::读取大端字节数组(字节组.as_bytes(), 周期, 标识).map(|inner| {
            Self {
                inner: Arc::new(inner),
            }
        })
    }

    #[classmethod]
    /// 计算指定K线区间的MACD柱面积
    fn 获取MACD(
        _cls: &Bound<'_, PyType>,
        k线序列: Vec<Py<Self>>,
        始: &Bound<'_, Self>,
        终: &Bound<'_, Self>,
        py: Python<'_>,
    ) -> HashMap<String, f64> {
        let refs: Vec<_> = k线序列.iter().map(|k| k.bind(py).borrow()).collect();
        let bars: Vec<&chanlun::kline::bar::K线> = refs.iter().map(|r| r.inner.as_ref()).collect();
        chanlun::kline::bar::K线::获取MACD(&bars, &始.borrow().inner, &终.borrow().inner)
    }

    #[staticmethod]
    /// 按起止K线截取K线子序列
    fn 截取(
        序列: Vec<Py<Self>>,
        始: &Bound<'_, Self>,
        终: &Bound<'_, Self>,
        py: Python<'_>,
    ) -> PyResult<Vec<Py<Self>>> {
        let start_ptr = Arc::as_ptr(&始.borrow().inner);
        let end_ptr = Arc::as_ptr(&终.borrow().inner);
        let start_ts = 始.borrow().inner.时间戳;
        let end_ts = 终.borrow().inner.时间戳;
        let start_idx = 序列
            .iter()
            .position(|k| {
                Arc::as_ptr(&k.borrow(py).inner) == start_ptr
                    || k.borrow(py).inner.时间戳 == start_ts
            })
            .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("始 不在序列中"))?;
        let end_idx = 序列
            .iter()
            .position(|k| {
                Arc::as_ptr(&k.borrow(py).inner) == end_ptr || k.borrow(py).inner.时间戳 == end_ts
            })
            .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("终 不在序列中"))?;
        if start_idx > end_idx {
            return Err(pyo3::exceptions::PyValueError::new_err("始 排序在 终 之后"));
        }
        Ok(序列
            .into_iter()
            .skip(start_idx)
            .take(end_idx - start_idx + 1)
            .collect())
    }

    /// 根据当前K线和方向生成下一根K线（用于随机回测）
    #[pyo3(signature = (方向, 居中 = false))]
    fn 根据当前K线生成新K线(
        &self, 方向: &Bound<'_, PyAny>, 居中: bool
    ) -> PyResult<Self> {
        let dir: chanlun::types::相对方向 = if let Ok(d) = 方向.extract::<PyRef<'_, 相对方向Py>>()
        {
            d.inner
        } else if let Ok(i) = 方向.extract::<i64>() {
            match i {
                0 => chanlun::types::相对方向::向上,
                1 => chanlun::types::相对方向::向下,
                2 => chanlun::types::相对方向::向上缺口,
                3 => chanlun::types::相对方向::向下缺口,
                4 => chanlun::types::相对方向::衔接向上,
                5 => chanlun::types::相对方向::衔接向下,
                _ => {
                    return Err(pyo3::exceptions::PyValueError::new_err(format!(
                        "无效方向: {i}"
                    )));
                }
            }
        } else {
            return Err(pyo3::exceptions::PyTypeError::new_err(
                "方向 必须是 相对方向 或 int (0-5)",
            ));
        };
        let new_bar = self.inner.根据当前K线生成新K线(dir, 居中);
        Ok(Self {
            inner: Arc::new(new_bar),
        })
    }
}

// ========== 缠论K线 ==========

/// 缠论K线 — 经包含处理后的标准化K线，有方向（向上/向下）、分型结构标记。
///
/// 属性 (只读):
///   序号: int / 时间戳: int (Unix秒) / 高: float / 低: float / 周期: int / 标识: str
///   方向: 相对方向 — K线运行方向（向上/向下）
///   分型: 分型结构|None — 当前K线在分型中的角色
///
/// 类方法:
///   时间戳对齐(缠K序列, 参照K线或时间戳) -> int
///      — 在缠K序列中找到与参照最接近的时间戳
///   创建缠K(普K, 指示器) -> 缠论K线
///      — 从普通K线创建缠论K线
///   兼并(缠K序列, 方向, 配置, 需要特定强度?) -> list[缠论K线]
///      — 根据方向进行K线包含处理（合并）
///   分析(缠K序列, 配置, 可以逆序包含?, 忽视顺序包含?, 可以逆序包含新?) -> (str, 分型|None)
///      — 分析分型形成结果
///   截取(序列, 起点分型, 终点分型) -> list — 截取分型间的缠K子序列
#[pyclass(name = "缠论K线", module = "chanlun._chanlun", from_py_object)]
pub struct 缠论K线Py {
    pub(crate) inner: std::sync::Arc<chanlun::kline::chan_kline::缠论K线>,
}

impl 缠论K线Py {
    pub(crate) fn from_rc(inner: std::sync::Arc<chanlun::kline::chan_kline::缠论K线>) -> Self {
        Self { inner }
    }
}

pub(crate) fn bar_to_py(
    py: Python<'_>,
    inner: std::sync::Arc<chanlun::kline::bar::K线>,
) -> Py<K线Py> {
    let key = Arc::as_ptr(&inner) as usize;
    if let Some(cached) = crate::cache::bar_get(py, key) {
        return cached;
    }
    let obj = Py::new(py, K线Py { inner }).unwrap();
    crate::cache::bar_insert(py, key, &obj);
    obj
}

pub(crate) fn chan_kline_to_py(
    py: Python<'_>,
    inner: std::sync::Arc<chanlun::kline::chan_kline::缠论K线>,
) -> Py<缠论K线Py> {
    let key = Arc::as_ptr(&inner) as usize;
    if let Some(cached) = crate::cache::kline_get(py, key) {
        return cached;
    }
    let obj = Py::new(py, 缠论K线Py::from_rc(inner)).unwrap();
    crate::cache::kline_insert(py, key, &obj);
    obj
}

impl Clone for 缠论K线Py {
    fn clone(&self) -> Self {
        Self {
            inner: std::sync::Arc::clone(&self.inner),
        }
    }
}

#[pymethods]
impl 缠论K线Py {
    #[new]
    fn new() -> Self {
        unimplemented!("使用 创建缠K 创建")
    }

    #[getter]
    fn 序号(&self) -> i64 {
        self.inner.序号.load(Ordering::Relaxed)
    }

    #[getter]
    fn 时间戳(&self) -> i64 {
        self.inner.时间戳.load(Ordering::Relaxed)
    }

    #[getter]
    fn 高(&self) -> f64 {
        self.inner.高.get()
    }

    #[getter]
    fn 低(&self) -> f64 {
        self.inner.低.get()
    }

    #[getter]
    fn 方向(&self, py: Python<'_>) -> Py<相对方向Py> {
        crate::types_py::获取相对方向单例(py, *self.inner.方向.read())
    }

    #[getter]
    fn 分型(&self, py: Python<'_>) -> Option<Py<crate::types_py::分型结构Py>> {
        self.inner
            .分型
            .read()
            .map(|f| crate::types_py::获取分型结构单例(py, f))
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
        self.inner.分型特征值.get()
    }

    #[getter]
    fn 原始起始序号(&self) -> i64 {
        self.inner.原始起始序号
    }

    #[getter]
    fn 原始结束序号(&self) -> i64 {
        self.inner.原始结束序号.load(Ordering::Relaxed)
    }

    #[getter]
    fn 标的K线(&self, py: Python<'_>) -> Py<K线Py> {
        bar_to_py(py, self.inner.标的K线.read().clone())
    }

    /// pandas 兼容 — 返回所有字段构成的字典
    #[getter]
    fn __dict__(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let dict = PyDict::new(py);
        dict.set_item("序号", self.序号())?;
        dict.set_item("时间戳", self.时间戳())?;
        dict.set_item("高", self.高())?;
        dict.set_item("低", self.低())?;
        dict.set_item("方向", self.方向(py))?;
        dict.set_item("周期", self.周期())?;
        dict.set_item("标识", self.标识())?;
        dict.set_item("分型特征值", self.分型特征值())?;
        dict.set_item("原始起始序号", self.原始起始序号())?;
        dict.set_item("原始结束序号", self.原始结束序号())?;
        dict.set_item("与MACD柱子匹配", self.与MACD柱子匹配())?;
        dict.set_item("与RSI匹配", self.与RSI匹配())?;
        dict.set_item("与KDJ匹配", self.与KDJ匹配())?;

        if let Some(v) = self.分型(py) {
            dict.set_item("分型", v)?;
        }
        Ok(dict.into())
    }

    fn __str__(&self) -> String {
        format!("{}", self.inner)
    }

    fn __repr__(&self) -> String {
        self.__str__()
    }

    fn __eq__(&self, other: &Bound<'_, PyAny>) -> bool {
        if let Ok(other) = other.extract::<PyRef<'_, Self>>() {
            return Arc::as_ptr(&self.inner) == Arc::as_ptr(&other.inner);
        }
        false
    }

    fn __hash__(&self) -> u64 {
        Arc::as_ptr(&self.inner) as u64
    }

    #[getter]
    /// 创建当前缠K的浅拷贝副本
    fn 镜像(&self, py: Python<'_>) -> Self {
        let mirror = Self {
            inner: std::sync::Arc::new(self.inner.镜像()),
        };
        // 复制买卖点信息到镜像
        let src_key = Arc::as_ptr(&self.inner) as usize;
        let dst_key = Arc::as_ptr(&mirror.inner) as usize;
        let cached_src = crate::cache::bsp_get(py, src_key);
        if let Some(cached_src) = cached_src
            && let Ok(new_set) = pyo3::types::PySet::empty(py)
        {
            for item in cached_src.bind(py).iter() {
                let _ = new_set.add(item);
            }
            let py_set: Py<pyo3::types::PySet> = new_set.into();
            crate::cache::bsp_insert(py, dst_key, py_set);
        }
        mirror
    }

    #[getter]
    /// :return: 底分型时MACD柱<0，顶分型时MACD柱>0
    fn 与MACD柱子匹配(&self) -> bool {
        self.inner.与MACD柱子匹配()
    }

    #[getter]
    /// :return: 底分型时RSI < RSI_SMA，顶分型时RSI > RSI_SMA
    fn 与RSI匹配(&self) -> bool {
        self.inner.与RSI匹配()
    }

    #[getter]
    /// :return: 底分型时K<D，顶分型时K>D
    fn 与KDJ匹配(&self) -> bool {
        self.inner.与KDJ匹配()
    }

    #[getter]
    fn 买卖点信息(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let key = Arc::as_ptr(&self.inner) as usize;
        // 检查全局缓存
        let cached = crate::cache::bsp_get(py, key);
        if let Some(set) = cached {
            return Ok(set.into_any());
        }
        // 创建新的 PySet，从 Rust HashSet 同步已有内容
        let set = pyo3::types::PySet::empty(py)?;
        let bsp_info = self.inner.买卖点信息.read();
        for item in bsp_info.iter() {
            set.add(item.as_str())?;
        }
        drop(bsp_info);
        crate::cache::bsp_insert(py, key, set.into());
        Ok(crate::cache::bsp_get(py, key).unwrap().into_any())
    }

    #[classmethod]
    /// 在基线序列中找到与k线时间戳对齐的时间戳
    fn 时间戳对齐(
        _cls: &Bound<'_, PyType>,
        基线: Vec<Py<Self>>,
        k线: &Bound<'_, Self>,
        py: Python<'_>,
    ) -> i64 {
        let rc_list: Vec<_> = 基线
            .iter()
            .map(|k| std::sync::Arc::clone(&k.bind(py).borrow().inner))
            .collect();
        chanlun::kline::chan_kline::缠论K线::时间戳对齐(&rc_list, &k线.borrow().inner)
    }

    #[classmethod]
    #[pyo3(signature = (时间戳, 高, 低, 方向, 结构, 原始序号, 普k, 之前 = None))]
    /// 创建新的缠论K线
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
        py: Python<'_>,
    ) -> Py<Self> {
        let prev_ref = 之前.map(|prev| prev.borrow());
        let prev_inner = prev_ref.as_ref().map(|r| r.inner.as_ref());
        let inner = chanlun::kline::chan_kline::缠论K线::创建缠K(
            时间戳,
            高,
            低,
            方向.borrow().inner,
            结构.map(|s| s.borrow().inner),
            原始序号,
            普k.borrow().inner.clone(),
            prev_inner,
        );
        chan_kline_to_py(py, std::sync::Arc::new(inner))
    }

    #[classmethod]
    /// 分析K线，执行指标计算+包含处理+分型判定
    /// 缠K序列/普K序列 原地修改（与 chan.py 行为一致）
    /// :return: (状态, 分型|None)
    fn 分析(
        _cls: &Bound<'_, PyType>,
        当前K线: &Bound<'_, K线Py>,
        缠K序列: &Bound<'_, PyList>,
        普K序列: &Bound<'_, PyList>,
        配置: &Bound<'_, 缠论配置Py>,
        py: Python<'_>,
    ) -> PyResult<(String, Option<Py<PyAny>>)> {
        let ck_inner = (*当前K线.borrow().inner).clone();
        let config = 配置.borrow().to_rust_config(py)?;

        // 从 Python 列表提取
        let mut ck_seq = Vec::with_capacity(缠K序列.len());
        for item in 缠K序列.iter() {
            let ck: PyRef<'_, Self> = item.extract()?;
            ck_seq.push(std::sync::Arc::clone(&ck.inner));
        }
        let mut bar_seq = Vec::with_capacity(普K序列.len());
        for item in 普K序列.iter() {
            let bar: PyRef<'_, K线Py> = item.extract()?;
            bar_seq.push(bar.inner.clone());
        }

        let (status, fractal) = chanlun::kline::chan_kline::缠论K线::分析(
            ck_inner,
            &mut ck_seq,
            &mut bar_seq,
            &config,
        );

        // 写回 Python 列表（clear + extend）
        缠K序列.call_method0("clear")?;
        for k in ck_seq {
            缠K序列.call_method1("append", (chan_kline_to_py(py, k),))?;
        }
        普K序列.call_method0("clear")?;
        for k in bar_seq {
            普K序列.call_method1("append", (bar_to_py(py, k),))?;
        }

        Ok((status, fractal.map(|f| fractal_to_py(py, f).into_any())))
    }

    #[staticmethod]
    /// :param 序列: 缠K序列
    fn 截取(
        序列: Vec<Py<Self>>,
        始: &Bound<'_, Self>,
        终: &Bound<'_, Self>,
        py: Python<'_>,
    ) -> PyResult<Vec<Py<Self>>> {
        let start_ptr = Arc::as_ptr(&始.borrow().inner);
        let end_ptr = Arc::as_ptr(&终.borrow().inner);
        let start_ts = 始.borrow().inner.时间戳.load(Ordering::Relaxed);
        let end_ts = 终.borrow().inner.时间戳.load(Ordering::Relaxed);
        let start_idx = 序列
            .iter()
            .position(|k| {
                Arc::as_ptr(&k.borrow(py).inner) == start_ptr
                    || k.borrow(py).inner.时间戳.load(Ordering::Relaxed) == start_ts
            })
            .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("始 不在序列中"))?;
        let end_idx = 序列
            .iter()
            .position(|k| {
                Arc::as_ptr(&k.borrow(py).inner) == end_ptr
                    || k.borrow(py).inner.时间戳.load(Ordering::Relaxed) == end_ts
            })
            .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("终 不在序列中"))?;
        if start_idx > end_idx {
            return Err(pyo3::exceptions::PyValueError::new_err("始 排序在 终 之后"));
        }
        Ok(序列
            .into_iter()
            .skip(start_idx)
            .take(end_idx - start_idx + 1)
            .collect())
    }
}

pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<K线Py>()?;
    m.add_class::<缠论K线Py>()?;
    Ok(())
}
