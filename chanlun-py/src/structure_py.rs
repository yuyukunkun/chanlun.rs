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
use pyo3::types::{PyDict, PyType};
use std::sync::Arc;
use std::sync::atomic::Ordering;

use crate::algorithm_py::hub_to_py;
use crate::config_py::缠论配置Py;
use crate::kline_py::{K线Py, bar_to_py, 缠论K线Py};

// ---- 身份缓存 (弱引用：通过 refcnt 检测存活，仅缓存持有则视为过期) ----

// 缓存通过 crate::cache 模块管理（支持 thread_local / global 运行时切换）

pub(crate) fn fractal_to_py(
    py: Python<'_>,
    inner: Arc<chanlun::structure::fractal_obj::分型>,
) -> Py<分型Py> {
    let key = Arc::as_ptr(&inner) as usize;
    if let Some(cached) = crate::cache::fractal_get(py, key) {
        return cached;
    }
    let obj = Py::new(py, 分型Py { inner }).unwrap();
    crate::cache::fractal_insert(py, key, &obj);
    obj
}

pub(crate) fn dashed_to_py(
    py: Python<'_>,
    inner: Arc<chanlun::structure::dash_line::虚线>,
) -> Py<虚线Py> {
    let key = Arc::as_ptr(&inner) as usize;
    if let Some(cached) = crate::cache::dashed_get(py, key) {
        return cached;
    }
    let obj = Py::new(py, 虚线Py { inner }).unwrap();
    crate::cache::dashed_insert(py, key, &obj);
    obj
}

pub(crate) fn segfeat_to_py(
    py: Python<'_>,
    inner: Arc<chanlun::structure::segment_feat::线段特征>,
) -> Py<线段特征Py> {
    Py::new(py, 线段特征Py { inner }).unwrap()
}

use crate::types_py::{分型结构Py, 相对方向Py, 缺口Py};

// ========== 分型 ==========

/// 分型 — 由左中右三根缠论K线构成的顶/底分型。
///
/// 属性 (只读):
///   左: 缠论K线|None / 中: 缠论K线 / 右: 缠论K线|None
///   结构: 分型结构 — 上/下/顶/底/散
///   时间戳: int / 分型特征值: float / 强度: str
///   关系组: (相对方向, 相对方向, 相对方向)|None — 左中、中右、左右的相对方向
///   与MACD柱子分型匹配: bool|None — MACD 柱状图是否与分型结构一致
///
/// 类方法:
///   判断分型(缠K序列, 索引, 配置) -> 分型|None — 在缠K序列中指定位置尝试创建分型
///   从缠K序列中获取分型(缠K序列, 配置) -> list[分型] — 扫描全序列提取所有分型
///   向序列中添加(分型序列, 新分型) — 维护分型序列的顺序一致性
#[pyclass(name = "分型", module = "chanlun._chanlun", from_py_object)]
#[derive(Clone)]
pub struct 分型Py {
    pub(crate) inner: Arc<chanlun::structure::fractal_obj::分型>,
}

#[pymethods]
impl 分型Py {
    #[new]
    fn new(
        左: Option<&Bound<'_, 缠论K线Py>>,
        中: &Bound<'_, 缠论K线Py>,
        右: Option<&Bound<'_, 缠论K线Py>>,
    ) -> Self {
        Self {
            inner: Arc::new(chanlun::structure::fractal_obj::分型::new(
                左.map(|k| Arc::clone(&k.borrow().inner)),
                Arc::clone(&中.borrow().inner),
                右.map(|k| Arc::clone(&k.borrow().inner)),
            )),
        }
    }

    #[getter]
    fn 左(&self, py: Python<'_>) -> Option<Py<缠论K线Py>> {
        self.inner
            .左
            .as_ref()
            .map(|k| crate::kline_py::chan_kline_to_py(py, Arc::clone(k)))
    }

    #[getter]
    fn 中(&self, py: Python<'_>) -> Py<缠论K线Py> {
        crate::kline_py::chan_kline_to_py(py, Arc::clone(&self.inner.中))
    }

    #[getter]
    fn 右(&self, py: Python<'_>) -> Option<Py<缠论K线Py>> {
        self.inner
            .右
            .as_ref()
            .map(|k| crate::kline_py::chan_kline_to_py(py, Arc::clone(k)))
    }

    #[getter]
    fn 结构(&self, py: Python<'_>) -> Py<分型结构Py> {
        crate::types_py::获取分型结构单例(py, self.inner.结构())
    }

    #[getter]
    fn 时间戳(&self) -> i64 {
        self.inner.时间戳()
    }

    #[getter]
    fn 分型特征值(&self) -> f64 {
        self.inner.分型特征值()
    }

    #[getter]
    /// 构造时缓存的 _结构（不受 分型模式 影响）
    fn _结构(&self, py: Python<'_>) -> Py<分型结构Py> {
        crate::types_py::获取分型结构单例(py, self.inner.结构)
    }

    #[getter]
    /// 构造时缓存的 _时间戳（不受 分型模式 影响）
    fn _时间戳(&self) -> i64 {
        self.inner.时间戳
    }

    #[getter]
    /// 构造时缓存的 _分型特征值（不受 分型模式 影响）
    fn _分型特征值(&self) -> f64 {
        self.inner.分型特征值
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
    /// 左、中、右三对相对方向关系
    fn 关系组(
        &self,
        py: Python<'_>,
    ) -> Option<(Py<相对方向Py>, Py<相对方向Py>, Py<相对方向Py>)> {
        self.inner.关系组().map(|(a, b, c)| {
            (
                crate::types_py::获取相对方向单例(py, a),
                crate::types_py::获取相对方向单例(py, b),
                crate::types_py::获取相对方向单例(py, c),
            )
        })
    }

    #[getter]
    /// 分型强度（强/中/弱/未知）
    fn 强度(&self) -> String {
        self.inner.强度().to_string()
    }

    #[getter]
    /// :return: 底分型时左右MACD柱 > 中MACD柱，顶分型时左右MACD柱 < 中MACD柱
    fn 与MACD柱子分型匹配(&self) -> bool {
        self.inner.与MACD柱子分型匹配()
    }

    /// pandas 兼容 — 返回所有字段构成的字典
    #[getter]
    fn __dict__(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let dict = PyDict::new(py);
        dict.set_item("结构", self.结构(py))?;
        dict.set_item("时间戳", self.时间戳())?;
        dict.set_item("分型特征值", self.分型特征值())?;
        dict.set_item("强度", self.强度())?;
        dict.set_item("与MACD柱子分型匹配", self.与MACD柱子分型匹配())?;
        if let Some(v) = self.左(py) {
            dict.set_item("左", v)?;
        }
        dict.set_item("中", self.中(py))?;
        if let Some(v) = self.右(py) {
            dict.set_item("右", v)?;
        }
        if let Some(v) = self.关系组(py) {
            dict.set_item("关系组", v)?;
        }
        Ok(dict.into())
    }

    #[classmethod]
    #[pyo3(signature = (左, 右, 模式 = "中"))]
    /// 判断两个分型是否相同（identity比较）
    fn 判断分型(
        _cls: &Bound<'_, PyType>,
        左: &Bound<'_, Self>,
        右: &Bound<'_, Self>,
        模式: &str,
    ) -> bool {
        chanlun::structure::fractal_obj::分型::判断分型(
            &左.borrow().inner,
            &右.borrow().inner,
            模式,
        )
    }

    #[staticmethod]
    /// 从缠K序列中提取以指定缠K为中元素的分型
    fn 从缠K序列中获取分型(
        K线序列: Vec<Py<缠论K线Py>>,
        中: &Bound<'_, 缠论K线Py>,
        py: Python<'_>,
    ) -> Option<Self> {
        let ck_seq: Vec<Arc<chanlun::kline::chan_kline::缠论K线>> = K线序列
            .iter()
            .map(|k| Arc::clone(&k.bind(py).borrow().inner))
            .collect();
        chanlun::structure::fractal_obj::分型::从缠K序列中获取分型(
            &ck_seq,
            &中.borrow().inner,
        )
        .map(|inner| Self {
            inner: Arc::new(inner),
        })
    }

    #[staticmethod]
    /// 向分型序列尾部添加，自动校验顶底交替
    fn 向序列中添加(
        分型序列: &Bound<'_, PyAny>, 当前分型: &Bound<'_, Self>
    ) -> PyResult<()> {
        let py = 分型序列.py();
        let wrapper = fractal_to_py(py, Arc::clone(&当前分型.borrow().inner));
        分型序列.call_method1("append", (wrapper,))?;
        Ok(())
    }
}

// ========== 虚线 ==========

/// 虚线 — 笔/线段的通用数据结构，持有一组分型端点（文=起点分型, 武=终点分型）。
///
/// 属性 (只读):
///   标识: str — "笔" / "线段" / "扩展线段" 等
///   序号: int / 级别: int / 有效性: bool / 模式: str
///   文: 分型 — 起点分型
///   武: 分型 — 终点分型
///   方向: 相对方向 — 根据文/武高低判定
///   高: float (计算) / 低: float (计算)
///   确认K线: 缠论K线|None / 前一缺口: 缺口|None
///   前一结束位置: 虚线|None / 短路修正: bool
///   基础序列: list[虚线] — 构成该虚线的子级虚线序列
///   实_中枢序列: list[中枢] / 虚_中枢序列: list[中枢] / 合_中枢序列: list[中枢]
///   笔序列: list[虚线] (计算) — 递归获取所有笔
///
/// 方法:
///   之前是(其他虚线) -> bool — 判断当前虚线是否紧接在另一虚线之前
///   之后是(其他虚线) -> bool — 判断当前虚线是否紧接在另一虚线之后
///   获取普K序列(观察员) -> list[K线] — 截取该虚线的原始K线范围
///   获取缠K序列(观察员) -> list[缠论K线] — 截取该虚线的缠K范围
///   获取数据文本() -> str
///
/// 类方法（算法辅助）:
///   创建笔(序号, 标识, 文, 武, 级别) / 创建线段(...)
///   缠K买卖点模式(缠K) / 买卖点配置匹配 / 买卖点任意匹配 / 买卖点全量匹配 / 买卖点相对匹配
///   计算MACD柱子均值 / 武之全量MACD均值 / 武之MACD均值 / 武之MACD极值
///   计算K线序列MACD趋向背驰 / 买卖意义 / 计算MACD柱子分段
///   密集区域按间隔 / 统计MACD行为
#[pyclass(name = "虚线", module = "chanlun._chanlun", from_py_object)]
#[derive(Clone)]
pub struct 虚线Py {
    pub(crate) inner: Arc<chanlun::structure::dash_line::虚线>,
}

#[pymethods]
impl 虚线Py {
    #[new]
    #[pyo3(signature = (序号, 标识, 文, 武, 级别, 有效性 = true))]
    fn new(
        序号: i64,
        标识: String,
        文: &Bound<'_, 分型Py>,
        武: &Bound<'_, 分型Py>,
        级别: i64,
        有效性: bool,
    ) -> Self {
        Self {
            inner: Arc::new(chanlun::structure::dash_line::虚线::new(
                序号,
                标识,
                Arc::clone(&文.borrow().inner),
                Arc::clone(&武.borrow().inner),
                级别,
                有效性,
            )),
        }
    }

    // ---- 基础 getters ----

    #[getter]
    fn 标识(&self) -> String {
        self.inner.标识.read().clone()
    }

    #[getter]
    fn 序号(&self) -> i64 {
        self.inner.序号.load(Ordering::Relaxed)
    }

    #[getter]
    fn 级别(&self) -> i64 {
        self.inner.级别.load(Ordering::Relaxed)
    }

    #[getter]
    fn 文(&self, py: Python<'_>) -> Py<分型Py> {
        fractal_to_py(py, Arc::clone(&self.inner.文))
    }

    #[getter]
    fn 武(&self, py: Python<'_>) -> Py<分型Py> {
        fractal_to_py(py, Arc::clone(&*self.inner.武.read()))
    }

    #[getter]
    fn 有效性(&self) -> bool {
        self.inner.有效性.load(Ordering::Relaxed)
    }

    #[getter]
    fn 模式(&self) -> String {
        self.inner.模式.read().clone()
    }

    #[getter(_特征序列_显示)]
    fn get_特征序列_显示(&self) -> bool {
        self.inner._特征序列_显示.load(Ordering::Relaxed)
    }

    #[setter(_特征序列_显示)]
    fn set_特征序列_显示(&mut self, value: bool) {
        self.inner._特征序列_显示.store(value, Ordering::Relaxed);
    }

    #[getter]
    fn 特征序列(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let list = pyo3::types::PyList::empty(py);
        for item in self.inner.特征序列.read().iter() {
            match item {
                Some(feat) => list.append(segfeat_to_py(py, Arc::clone(feat)))?,
                None => {
                    list.append(py.None())?;
                }
            }
        }
        Ok(list.into())
    }

    #[getter]
    fn 短路修正(&self) -> bool {
        self.inner.短路修正.load(Ordering::Relaxed)
    }

    #[getter]
    fn 确认K线(&self, py: Python<'_>) -> Option<Py<缠论K线Py>> {
        self.inner
            .确认K线
            .read()
            .as_ref()
            .map(|k| crate::kline_py::chan_kline_to_py(py, Arc::clone(k)))
    }

    #[getter]
    fn 前一缺口(&self) -> Option<缺口Py> {
        self.inner.前一缺口.read().map(|q| 缺口Py { inner: q })
    }

    #[getter]
    fn 前一结束位置(&self, py: Python<'_>) -> Option<Py<虚线Py>> {
        self.inner
            .前一结束位置
            .read()
            .as_ref()
            .map(|d| dashed_to_py(py, Arc::clone(d)))
    }

    // ---- 序列 getters ----

    #[getter]
    fn 基础序列(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let list = pyo3::types::PyList::empty(py);
        for d in self.inner.基础序列.read().iter() {
            list.append(dashed_to_py(py, Arc::clone(d)))?;
        }
        Ok(list.into())
    }

    #[getter]
    fn 实_中枢序列(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let list = pyo3::types::PyList::empty(py);
        for h in self.inner.实_中枢序列.read().iter() {
            list.append(hub_to_py(py, Arc::clone(h)))?;
        }
        Ok(list.into())
    }

    #[getter]
    fn 虚_中枢序列(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let list = pyo3::types::PyList::empty(py);
        for h in self.inner.虚_中枢序列.read().iter() {
            list.append(hub_to_py(py, Arc::clone(h)))?;
        }
        Ok(list.into())
    }

    #[getter]
    fn 合_中枢序列(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let list = pyo3::types::PyList::empty(py);
        for h in self.inner.合_中枢序列.read().iter() {
            list.append(hub_to_py(py, Arc::clone(h)))?;
        }
        Ok(list.into())
    }

    // ---- 计算属性 ----

    #[getter]
    /// 笔序列
    fn 笔序列(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let list = pyo3::types::PyList::empty(py);
        for d in self.inner.基础序列.read().iter() {
            list.append(dashed_to_py(py, Arc::clone(d)))?;
        }
        Ok(list.into())
    }

    #[getter]
    /// :return: 图表显示标题
    fn 图表标题(&self) -> String {
        self.inner.图表标题()
    }

    #[getter]
    /// :return: 运行方向
    fn 方向(&self, py: Python<'_>) -> Py<相对方向Py> {
        crate::types_py::获取相对方向单例(py, self.inner.方向())
    }

    #[getter]
    /// 虚线区间的最高价（向上线段为终点分型最高价，向下线段为起点分型最高价）
    fn 高(&self) -> f64 {
        self.inner.高()
    }

    #[getter]
    /// 虚线区间的最低价（向下线段为终点分型最低价，向上线段为起点分型最低价）
    fn 低(&self) -> f64 {
        self.inner.低()
    }

    /// :param 之前: 前一条虚线
    fn 之前是(&self, 之前: &Bound<'_, Self>) -> bool {
        self.inner.之前是(&之前.borrow().inner)
    }

    /// :param 之后: 后一条虚线
    fn 之后是(&self, 之后: &Bound<'_, Self>) -> bool {
        self.inner.之后是(&之后.borrow().inner)
    }

    /// :param 观察员: 观察者实例
    fn 获取普K序列(
        &self,
        观察员: &Bound<'_, crate::business_py::观察者Py>,
    ) -> PyResult<Py<PyAny>> {
        let obs_ref = 观察员.borrow();
        let observer_inner = obs_ref.obs();
        let result = self.inner.获取普K序列(&observer_inner.普通K线序列);
        let py = 观察员.py();
        let list = pyo3::types::PyList::empty(py);
        for k in &result {
            list.append(bar_to_py(py, Arc::clone(k)))?;
        }
        Ok(list.into())
    }

    /// :param 观察员: 观察者实例
    fn 获取缠K序列(
        &self,
        观察员: &Bound<'_, crate::business_py::观察者Py>,
    ) -> PyResult<Py<PyAny>> {
        let obs_ref = 观察员.borrow();
        let py = 观察员.py();
        let observer_inner = obs_ref.obs();
        let result = self.inner.获取缠K序列(&observer_inner.缠论K线序列);
        let list = pyo3::types::PyList::empty(py);
        for k in &result {
            list.append(crate::kline_py::chan_kline_to_py(py, Arc::clone(k)))?;
        }
        Ok(list.into())
    }

    #[classmethod]
    /// 递归获取虚线的终点分型（笔直接返回武，线段递归到底层笔的武）
    fn 获取_武(_cls: &Bound<'_, PyType>, 实线: &Bound<'_, Self>) -> Py<分型Py> {
        let py = _cls.py();
        fractal_to_py(py, 实线.borrow().inner.获取_武())
    }

    /// 获取用于保存的数据文本
    fn 获取数据文本(&self) -> String {
        self.inner.获取数据文本()
    }

    /// pandas 兼容 — 返回关键标量字段构成的字典
    #[getter]
    fn __dict__(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let dict = PyDict::new(py);
        dict.set_item("标识", self.标识())?;
        dict.set_item("序号", self.序号())?;
        dict.set_item("级别", self.级别())?;
        dict.set_item("有效性", self.有效性())?;
        dict.set_item("模式", self.模式())?;
        dict.set_item("短路修正", self.短路修正())?;
        dict.set_item("图表标题", self.图表标题())?;
        dict.set_item("文_时间戳", self.文(py).bind(py).borrow().时间戳())?;
        dict.set_item("武_时间戳", self.武(py).bind(py).borrow().时间戳())?;
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

    // ---- 静态工厂方法 ----

    #[classmethod]
    #[pyo3(signature = (文, 武, 有效性 = true))]
    /// :param 文: 起点分型
    fn 创建笔(
        _cls: &Bound<'_, PyType>,
        文: &Bound<'_, 分型Py>,
        武: &Bound<'_, 分型Py>,
        有效性: bool,
    ) -> Self {
        Self {
            inner: Arc::new(chanlun::structure::dash_line::虚线::创建笔(
                Arc::clone(&文.borrow().inner),
                Arc::clone(&武.borrow().inner),
                有效性,
            )),
        }
    }

    #[classmethod]
    /// :param 虚线序列: 构成线段的虚线列表（笔）
    fn 创建线段(_cls: &Bound<'_, PyType>, 虚线序列: Vec<Py<Self>>, py: Python<'_>) -> Self {
        let rc_list: Vec<Arc<chanlun::structure::dash_line::虚线>> = 虚线序列
            .iter()
            .map(|d| Arc::clone(&d.bind(py).borrow().inner))
            .collect();
        Self {
            inner: Arc::new(chanlun::structure::dash_line::虚线::创建线段(
                &rc_list,
            )),
        }
    }

    // ---- 买卖点模式匹配 ----

    #[classmethod]
    /// :param 模式: "全量"/"任意"/"配置"/"相对"
    fn 缠K买卖点模式(
        _cls: &Bound<'_, PyType>,
        模式: &str,
        缠K: &Bound<'_, 缠论K线Py>,
        配置: &Bound<'_, 缠论配置Py>,
        py: Python<'_>,
    ) -> PyResult<bool> {
        let config = 配置.borrow().to_rust_config(py)?;
        Ok(chanlun::structure::dash_line::虚线::缠K买卖点模式(
            模式,
            &缠K.borrow().inner,
            &config,
        ))
    }

    #[classmethod]
    /// 根据配置中的指标开关检测缠K匹配情况（MACD/KDJ/RSI组合）
    fn 买卖点配置匹配(
        _cls: &Bound<'_, PyType>,
        缠K: &Bound<'_, 缠论K线Py>,
        配置: &Bound<'_, 缠论配置Py>,
        py: Python<'_>,
    ) -> PyResult<bool> {
        let config = 配置.borrow().to_rust_config(py)?;
        Ok(chanlun::structure::dash_line::虚线::买卖点配置匹配(
            &缠K.borrow().inner,
            &config,
        ))
    }

    #[classmethod]
    /// :param 缠K: 缠论K线
    fn 买卖点任意匹配(_cls: &Bound<'_, PyType>, 缠K: &Bound<'_, 缠论K线Py>) -> bool {
        chanlun::structure::dash_line::虚线::买卖点任意匹配(&缠K.borrow().inner)
    }

    #[classmethod]
    /// :param 缠K: 缠论K线
    fn 买卖点全量匹配(_cls: &Bound<'_, PyType>, 缠K: &Bound<'_, 缠论K线Py>) -> bool {
        chanlun::structure::dash_line::虚线::买卖点全量匹配(&缠K.borrow().inner)
    }

    #[classmethod]
    /// :param 缠K: 缠论K线
    fn 买卖点相对匹配(_cls: &Bound<'_, PyType>, 缠K: &Bound<'_, 缠论K线Py>) -> bool {
        chanlun::structure::dash_line::虚线::买卖点相对匹配(&缠K.borrow().inner)
    }

    // ---- MACD 相关 classmethods ----

    #[classmethod]
    /// :param 普K序列: 完整K线序列
    fn 计算MACD柱子均值(
        _cls: &Bound<'_, PyType>,
        普K序列: Vec<Py<K线Py>>,
        实线: &Bound<'_, Self>,
        py: Python<'_>,
    ) -> f64 {
        let rc_list: Vec<Arc<chanlun::kline::bar::K线>> = 普K序列
            .iter()
            .map(|k| k.bind(py).borrow().inner.clone())
            .collect();
        chanlun::structure::dash_line::虚线::计算MACD柱子均值(
            &rc_list,
            &实线.borrow().inner,
        )
    }

    #[classmethod]
    /// :param 普K序列: 完整K线序列
    fn 武之全量MACD均值(
        _cls: &Bound<'_, PyType>,
        普K序列: Vec<Py<K线Py>>,
        实线: &Bound<'_, Self>,
        py: Python<'_>,
    ) -> bool {
        let rc_list: Vec<Arc<chanlun::kline::bar::K线>> = 普K序列
            .iter()
            .map(|k| k.bind(py).borrow().inner.clone())
            .collect();
        chanlun::structure::dash_line::虚线::武之全量MACD均值(
            &rc_list,
            &实线.borrow().inner,
        )
    }

    #[classmethod]
    /// :param 普K序列: 完整K线序列
    fn 武之MACD均值(
        _cls: &Bound<'_, PyType>,
        普K序列: Vec<Py<K线Py>>,
        实线: &Bound<'_, Self>,
        py: Python<'_>,
    ) -> bool {
        let rc_list: Vec<Arc<chanlun::kline::bar::K线>> = 普K序列
            .iter()
            .map(|k| k.bind(py).borrow().inner.clone())
            .collect();
        chanlun::structure::dash_line::虚线::武之MACD均值(&rc_list, &实线.borrow().inner)
    }

    #[classmethod]
    /// :param 普K序列: 完整K线序列
    fn 武之MACD极值(
        _cls: &Bound<'_, PyType>,
        普K序列: Vec<Py<K线Py>>,
        实线: &Bound<'_, Self>,
        py: Python<'_>,
    ) -> bool {
        let rc_list: Vec<Arc<chanlun::kline::bar::K线>> = 普K序列
            .iter()
            .map(|k| k.bind(py).borrow().inner.clone())
            .collect();
        chanlun::structure::dash_line::虚线::武之MACD极值(&rc_list, &实线.borrow().inner)
    }

    #[classmethod]
    /// :param 普K序列: 完整K线序列
    fn 计算MACD柱子均值_阴(
        _cls: &Bound<'_, PyType>,
        普K序列: Vec<Py<K线Py>>,
        实线: &Bound<'_, Self>,
        py: Python<'_>,
    ) -> Option<f64> {
        let rc_list: Vec<Arc<chanlun::kline::bar::K线>> = 普K序列
            .iter()
            .map(|k| k.bind(py).borrow().inner.clone())
            .collect();
        chanlun::structure::dash_line::虚线::计算MACD柱子均值_阴(
            &rc_list,
            &实线.borrow().inner,
        )
    }

    #[classmethod]
    /// :param 普K序列: 完整K线序列
    fn 计算MACD柱子均值_阳(
        _cls: &Bound<'_, PyType>,
        普K序列: Vec<Py<K线Py>>,
        实线: &Bound<'_, Self>,
        py: Python<'_>,
    ) -> Option<f64> {
        let rc_list: Vec<Arc<chanlun::kline::bar::K线>> = 普K序列
            .iter()
            .map(|k| k.bind(py).borrow().inner.clone())
            .collect();
        chanlun::structure::dash_line::虚线::计算MACD柱子均值_阳(
            &rc_list,
            &实线.borrow().inner,
        )
    }

    #[classmethod]
    /// :param 普K序列: 完整K线序列
    fn 武之MACD均值_阴(
        _cls: &Bound<'_, PyType>,
        普K序列: Vec<Py<K线Py>>,
        实线: &Bound<'_, Self>,
        py: Python<'_>,
    ) -> bool {
        let rc_list: Vec<Arc<chanlun::kline::bar::K线>> = 普K序列
            .iter()
            .map(|k| k.bind(py).borrow().inner.clone())
            .collect();
        chanlun::structure::dash_line::虚线::武之MACD均值_阴(&rc_list, &实线.borrow().inner)
    }

    #[classmethod]
    /// :param 普K序列: 完整K线序列
    fn 武之MACD均值_阳(
        _cls: &Bound<'_, PyType>,
        普K序列: Vec<Py<K线Py>>,
        实线: &Bound<'_, Self>,
        py: Python<'_>,
    ) -> bool {
        let rc_list: Vec<Arc<chanlun::kline::bar::K线>> = 普K序列
            .iter()
            .map(|k| k.bind(py).borrow().inner.clone())
            .collect();
        chanlun::structure::dash_line::虚线::武之MACD均值_阳(&rc_list, &实线.borrow().inner)
    }

    #[classmethod]
    /// 计算K线序列的MACD柱/DIF/DEA趋向背驰（三元素判断）
    fn 计算K线序列MACD趋向背驰(
        _cls: &Bound<'_, PyType>,
        普K序列: Vec<Py<K线Py>>,
        方向: &Bound<'_, 相对方向Py>,
        py: Python<'_>,
    ) -> [bool; 3] {
        let rc_list: Vec<Arc<chanlun::kline::bar::K线>> = 普K序列
            .iter()
            .map(|k| k.bind(py).borrow().inner.clone())
            .collect();
        chanlun::structure::dash_line::虚线::计算K线序列MACD趋向背驰(
            &rc_list,
            方向.borrow().inner,
        )
    }

    #[classmethod]
    /// :param k线序列: K线序列
    fn 计算MACD柱子分段(
        _cls: &Bound<'_, PyType>,
        k线序列: Vec<Py<K线Py>>,
        py: Python<'_>,
    ) -> Vec<Vec<f64>> {
        let rc_list: Vec<Arc<chanlun::kline::bar::K线>> = k线序列
            .iter()
            .map(|k| k.bind(py).borrow().inner.clone())
            .collect();
        chanlun::structure::dash_line::虚线::计算MACD柱子分段(&rc_list)
    }

    #[classmethod]
    #[pyo3(signature = (交叉标记, 最大间隔 = 5usize, 最少交叉数 = 3usize))]
    /// 交叉标记: 长度为len(macd_list)的列表，0=无交叉, 1=金叉, -1=死叉
    fn 密集区域按间隔(
        _cls: &Bound<'_, PyType>,
        交叉标记: Vec<i32>,
        最大间隔: usize,
        最少交叉数: usize,
    ) -> Vec<(usize, usize, usize)> {
        chanlun::structure::dash_line::虚线::密集区域按间隔(
            &交叉标记,
            最大间隔,
            最少交叉数,
        )
    }

    #[classmethod]
    #[pyo3(signature = (普K序列, 最大间隔 = 8usize, 最少交叉数 = 3usize))]
    /// :param 普K序列: K线序列
    fn 统计MACD行为(
        _cls: &Bound<'_, PyType>,
        普K序列: Vec<Py<K线Py>>,
        最大间隔: usize,
        最少交叉数: usize,
        py: Python<'_>,
    ) -> PyResult<Py<PyAny>> {
        let rc_list: Vec<Arc<chanlun::kline::bar::K线>> = 普K序列
            .iter()
            .map(|k| k.bind(py).borrow().inner.clone())
            .collect();
        let result =
            chanlun::structure::dash_line::虚线::统计MACD行为(&rc_list, 最大间隔, 最少交叉数);
        let dict = pyo3::types::PyDict::new(py);
        dict.set_item("DIF上穿0", result.DIF上穿0)?;
        dict.set_item("DIF下穿0", result.DIF下穿0)?;
        dict.set_item("DEA上穿0", result.DEA上穿0)?;
        dict.set_item("DEA下穿0", result.DEA下穿0)?;
        dict.set_item("金叉次数", result.金叉次数)?;
        dict.set_item("死叉次数", result.死叉次数)?;
        let 密集区: Vec<Py<PyAny>> = result
            .密集交叉区域
            .iter()
            .map(|(a, b, c)| {
                let tup = pyo3::types::PyTuple::new(
                    py,
                    [
                        (*a).into_pyobject(py)?.into_any().unbind(),
                        (*b).into_pyobject(py)?.into_any().unbind(),
                        (*c).into_pyobject(py)?.into_any().unbind(),
                    ],
                )?;
                Ok(tup.into_any().unbind())
            })
            .collect::<PyResult<Vec<_>>>()?;
        dict.set_item("密集交叉区域", 密集区)?;
        Ok(dict.into())
    }

    #[classmethod]
    /// 静止是相对的，而运动是绝对的
    fn 买卖意义(
        _cls: &Bound<'_, PyType>,
        实线: &Bound<'_, Self>,
        观察员: &Bound<'_, crate::business_py::观察者Py>,
    ) -> (bool, String) {
        let obs = 观察员.borrow();
        let obs_ref = obs.obs();
        chanlun::structure::dash_line::虚线::买卖意义(&实线.borrow().inner, &obs_ref)
    }
}

// ========== 线段特征 ==========

/// 线段特征 — 特征序列元素构成的集合，支持 list-like 索引访问。
///
/// 实现 __getitem__ / __len__ / __iter__，可像 list 一样遍历。
///
/// 属性 (只读):
///   文: 分型 — 特征序列的起点分型
///   武: 分型 — 特征序列的终点分型
///   方向: 相对方向 / 高: float / 低: float
///   基本序列: list[虚线] (基础序列)
///
/// 类方法:
///   :meth:`静态分析` — 对虚线序列进行静态特征分析，返回 线段特征 列表
#[pyclass(name = "线段特征", module = "chanlun._chanlun", from_py_object)]
#[derive(Clone)]
pub struct 线段特征Py {
    pub(crate) inner: Arc<chanlun::structure::segment_feat::线段特征>,
}

#[pymethods]
impl 线段特征Py {
    #[getter]
    fn 序号(&self) -> i64 {
        self.inner.序号.load(Ordering::Relaxed)
    }

    #[setter]
    #[pyo3(name = "序号")]
    fn set_序号(&self, value: i64) {
        self.inner.序号.store(value, Ordering::Relaxed);
    }

    #[getter]
    fn 标识(&self) -> String {
        self.inner.标识.read().clone()
    }

    #[setter]
    fn set_标识(&self, value: String) {
        *self.inner.标识.write() = value;
    }

    #[getter]
    fn 线段方向(&self, py: Python<'_>) -> Py<相对方向Py> {
        crate::types_py::获取相对方向单例(py, self.inner.线段方向)
    }

    #[getter]
    fn 基础序列(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let list = pyo3::types::PyList::empty(py);
        for d in &self.inner.基础序列 {
            list.append(dashed_to_py(py, Arc::clone(d)))?;
        }
        Ok(list.into())
    }

    fn __str__(&self) -> String {
        format!("{}", self.inner)
    }

    fn __repr__(&self) -> String {
        self.__str__()
    }

    #[getter]
    /// :return: 图表标题
    fn 图表标题(&self) -> String {
        self.inner.图表标题()
    }

    #[getter]
    /// 起点分型（向上线段取高高中的最大者，向下线段取低低中的最小者）
    fn 文(&self, py: Python<'_>) -> Py<分型Py> {
        fractal_to_py(py, self.inner.文())
    }

    #[getter]
    /// 终点分型（向上线段取高高中的最大者，向下线段取低低中的最小者）
    fn 武(&self, py: Python<'_>) -> Py<分型Py> {
        fractal_to_py(py, self.inner.武())
    }

    #[getter]
    /// :return: 特征序列方向（线段方向的翻转）
    fn 方向(&self, py: Python<'_>) -> Py<相对方向Py> {
        crate::types_py::获取相对方向单例(py, self.inner.方向())
    }

    #[getter]
    /// :return: 文和武中分型特征值的较大者
    fn 高(&self) -> f64 {
        self.inner.高()
    }

    #[getter]
    /// :return: 文和武中分型特征值的较小者
    fn 低(&self) -> f64 {
        self.inner.低()
    }

    #[classmethod]
    #[pyo3(signature = (虚线序列, 线段方向, 四象, 是否忽视 = false))]
    fn 静态分析(
        _cls: &Bound<'_, PyType>,
        虚线序列: Vec<Py<虚线Py>>,
        线段方向: &Bound<'_, 相对方向Py>,
        四象: &str,
        是否忽视: bool,
        py: Python<'_>,
    ) -> PyResult<Py<PyAny>> {
        let rc_list: Vec<Arc<chanlun::structure::dash_line::虚线>> = 虚线序列
            .iter()
            .map(|d| Arc::clone(&d.bind(py).borrow().inner))
            .collect();
        let result = chanlun::structure::segment_feat::线段特征::静态分析(
            &rc_list,
            线段方向.borrow().inner,
            四象,
            是否忽视,
        );
        let list = pyo3::types::PyList::empty(py);
        for sf in result {
            list.append(segfeat_to_py(py, sf))?;
        }
        Ok(list.into())
    }
}

pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<分型Py>()?;
    m.add_class::<虚线Py>()?;
    m.add_class::<线段特征Py>()?;
    Ok(())
}
