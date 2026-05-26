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
use std::collections::HashMap;
use std::rc::Rc;

use crate::algorithm_py::中枢Py;
use crate::config_py::缠论配置Py;
use crate::kline_py::{缠论K线Py, K线Py};
use crate::types_py::{分型结构Py, 相对方向Py, 缺口Py};

// ========== 分型 ==========

#[pyclass(name = "分型", unsendable)]
#[derive(Clone)]
pub struct 分型Py {
    pub(crate) inner: Rc<chanlun::structure::fractal_obj::分型>,
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
            inner: Rc::new(chanlun::structure::fractal_obj::分型::new(
                左.map(|k| Rc::clone(&k.borrow().inner)),
                Rc::clone(&中.borrow().inner),
                右.map(|k| Rc::clone(&k.borrow().inner)),
            )),
        }
    }

    #[getter]
    fn 左(&self) -> Option<缠论K线Py> {
        self.inner.左.as_ref().map(|k| 缠论K线Py {
            inner: Rc::clone(k),
        })
    }

    #[getter]
    fn 中(&self) -> 缠论K线Py {
        缠论K线Py {
            inner: Rc::clone(&self.inner.中),
        }
    }

    #[getter]
    fn 右(&self) -> Option<缠论K线Py> {
        self.inner.右.as_ref().map(|k| 缠论K线Py {
            inner: Rc::clone(k),
        })
    }

    #[getter]
    fn 结构(&self) -> 分型结构Py {
        分型结构Py {
            inner: self.inner.结构,
        }
    }

    #[getter]
    fn 时间戳(&self) -> i64 {
        self.inner.时间戳
    }

    #[getter]
    fn 分型特征值(&self) -> f64 {
        self.inner.分型特征值
    }

    fn __str__(&self) -> String {
        format!("{}", self.inner)
    }

    fn __repr__(&self) -> String {
        self.__str__()
    }

    #[getter]
    fn 关系组(&self) -> Option<(相对方向Py, 相对方向Py, 相对方向Py)> {
        self.inner.关系组().map(|(a, b, c)| {
            (
                相对方向Py { inner: a },
                相对方向Py { inner: b },
                相对方向Py { inner: c },
            )
        })
    }

    #[getter]
    fn 强度(&self) -> String {
        self.inner.强度().to_string()
    }

    #[getter]
    fn 与MACD柱子分型匹配(&self) -> bool {
        self.inner.与MACD柱子分型匹配()
    }

    #[classmethod]
    #[pyo3(signature = (左, 右, 模式 = "中"))]
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
    fn 从缠K序列中获取分型(
        K线序列: Vec<Py<缠论K线Py>>,
        中: &Bound<'_, 缠论K线Py>,
        py: Python<'_>,
    ) -> Option<Self> {
        let ck_seq: Vec<Rc<chanlun::kline::chan_kline::缠论K线>> = K线序列
            .iter()
            .map(|k| Rc::clone(&k.bind(py).borrow().inner))
            .collect();
        chanlun::structure::fractal_obj::分型::从缠K序列中获取分型(
            &ck_seq,
            &中.borrow().inner,
        )
        .map(|inner| Self {
            inner: Rc::new(inner),
        })
    }

    #[staticmethod]
    fn 向序列中添加(
        分型序列: &Bound<'_, PyAny>, 当前分型: &Bound<'_, Self>
    ) -> PyResult<()> {
        let py = 分型序列.py();
        let inner = Rc::clone(&当前分型.borrow().inner);
        let wrapper = Py::new(
            py,
            Self {
                inner: Rc::clone(&inner),
            },
        )?;
        分型序列.call_method1("append", (wrapper,))?;
        Ok(())
    }
}

// ========== 虚线 ==========

#[pyclass(name = "虚线", unsendable)]
#[derive(Clone)]
pub struct 虚线Py {
    pub(crate) inner: Rc<chanlun::structure::dash_line::虚线>,
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
            inner: Rc::new(chanlun::structure::dash_line::虚线::new(
                序号,
                标识,
                Rc::clone(&文.borrow().inner),
                Rc::clone(&武.borrow().inner),
                级别,
                有效性,
            )),
        }
    }

    // ---- 基础 getters ----

    #[getter]
    fn 标识(&self) -> String {
        self.inner.标识.clone()
    }

    #[getter]
    fn 序号(&self) -> i64 {
        self.inner.序号
    }

    #[getter]
    fn 级别(&self) -> i64 {
        self.inner.级别
    }

    #[getter]
    fn 文(&self) -> 分型Py {
        分型Py {
            inner: Rc::clone(&self.inner.文),
        }
    }

    #[getter]
    fn 武(&self) -> 分型Py {
        分型Py {
            inner: Rc::clone(&self.inner.武),
        }
    }

    #[getter]
    fn 有效性(&self) -> bool {
        self.inner.有效性
    }

    #[getter]
    fn 模式(&self) -> String {
        self.inner.模式.clone()
    }

    #[getter]
    fn _特征序列_显示(&self) -> bool {
        self.inner._特征序列_显示
    }

    #[getter]
    fn 短路修正(&self) -> bool {
        self.inner.短路修正
    }

    #[getter]
    fn 确认K线(&self) -> Option<缠论K线Py> {
        self.inner.确认K线.as_ref().map(|k| 缠论K线Py {
            inner: Rc::clone(k),
        })
    }

    #[getter]
    fn 前一缺口(&self) -> Option<缺口Py> {
        self.inner.前一缺口.map(|q| 缺口Py { inner: q })
    }

    #[getter]
    fn 前一结束位置(&self) -> Option<Self> {
        self.inner.前一结束位置.as_ref().map(|d| Self {
            inner: Rc::clone(d),
        })
    }

    // ---- 序列 getters ----

    #[getter]
    fn 基础序列(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let list = pyo3::types::PyList::empty(py);
        for d in &self.inner.基础序列 {
            list.append(Py::new(
                py,
                Self {
                    inner: Rc::clone(d),
                },
            )?)?;
        }
        Ok(list.into())
    }

    #[getter]
    fn 实_中枢序列(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let list = pyo3::types::PyList::empty(py);
        for h in &self.inner.实_中枢序列 {
            list.append(中枢Py {
                inner: Rc::clone(h),
            })?;
        }
        Ok(list.into())
    }

    #[getter]
    fn 虚_中枢序列(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let list = pyo3::types::PyList::empty(py);
        for h in &self.inner.虚_中枢序列 {
            list.append(中枢Py {
                inner: Rc::clone(h),
            })?;
        }
        Ok(list.into())
    }

    #[getter]
    fn 合_中枢序列(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let list = pyo3::types::PyList::empty(py);
        for h in &self.inner.合_中枢序列 {
            list.append(中枢Py {
                inner: Rc::clone(h),
            })?;
        }
        Ok(list.into())
    }

    // ---- 计算属性 ----

    #[getter]
    fn 笔序列(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let list = pyo3::types::PyList::empty(py);
        for d in self.inner.笔序列() {
            list.append(Py::new(
                py,
                Self {
                    inner: Rc::clone(d),
                },
            )?)?;
        }
        Ok(list.into())
    }

    #[getter]
    fn 图表标题(&self) -> String {
        self.inner.图表标题()
    }

    #[getter]
    fn 方向(&self) -> 相对方向Py {
        相对方向Py {
            inner: self.inner.方向(),
        }
    }

    #[getter]
    fn 高(&self) -> f64 {
        self.inner.高()
    }

    #[getter]
    fn 低(&self) -> f64 {
        self.inner.低()
    }

    fn 之前是(&self, 之前: &Bound<'_, Self>) -> bool {
        self.inner.之前是(&之前.borrow().inner)
    }

    fn 之后是(&self, 之后: &Bound<'_, Self>) -> bool {
        self.inner.之后是(&之后.borrow().inner)
    }

    fn 获取普K序列(&self, 普K序列: Vec<Py<K线Py>>, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let rc_list: Vec<Rc<chanlun::kline::bar::K线>> = 普K序列
            .iter()
            .map(|k| Rc::new(k.bind(py).borrow().inner.clone()))
            .collect();
        let result = self.inner.获取普K序列(&rc_list);
        let list = pyo3::types::PyList::empty(py);
        for k in &result {
            list.append(K线Py {
                inner: (**k).clone(),
            })?;
        }
        Ok(list.into())
    }

    fn 获取缠K序列(
        &self, 缠K序列: Vec<Py<缠论K线Py>>, py: Python<'_>
    ) -> PyResult<Py<PyAny>> {
        let rc_list: Vec<Rc<chanlun::kline::chan_kline::缠论K线>> = 缠K序列
            .iter()
            .map(|k| Rc::clone(&k.bind(py).borrow().inner))
            .collect();
        let result = self.inner.获取缠K序列(&rc_list);
        let list = pyo3::types::PyList::empty(py);
        for k in &result {
            list.append(缠论K线Py {
                inner: Rc::clone(k),
            })?;
        }
        Ok(list.into())
    }

    #[getter]
    fn 获取数据文本(&self) -> String {
        self.inner.获取数据文本()
    }

    fn __str__(&self) -> String {
        format!("{}", self.inner)
    }

    fn __repr__(&self) -> String {
        self.__str__()
    }

    // ---- 静态工厂方法 ----

    #[classmethod]
    fn 创建笔(
        _cls: &Bound<'_, PyType>,
        文: &Bound<'_, 分型Py>,
        武: &Bound<'_, 分型Py>,
        有效性: bool,
    ) -> Self {
        Self {
            inner: Rc::new(chanlun::structure::dash_line::虚线::创建笔(
                Rc::clone(&文.borrow().inner),
                Rc::clone(&武.borrow().inner),
                有效性,
            )),
        }
    }

    #[classmethod]
    fn 创建线段(_cls: &Bound<'_, PyType>, 虚线序列: Vec<Py<Self>>, py: Python<'_>) -> Self {
        let rc_list: Vec<Rc<chanlun::structure::dash_line::虚线>> = 虚线序列
            .iter()
            .map(|d| Rc::clone(&d.bind(py).borrow().inner))
            .collect();
        Self {
            inner: Rc::new(chanlun::structure::dash_line::虚线::创建线段(
                &rc_list,
            )),
        }
    }

    // ---- 买卖点模式匹配 ----

    #[classmethod]
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
    fn 买卖点任意匹配(_cls: &Bound<'_, PyType>, 缠K: &Bound<'_, 缠论K线Py>) -> bool {
        chanlun::structure::dash_line::虚线::买卖点任意匹配(&缠K.borrow().inner)
    }

    #[classmethod]
    fn 买卖点全量匹配(_cls: &Bound<'_, PyType>, 缠K: &Bound<'_, 缠论K线Py>) -> bool {
        chanlun::structure::dash_line::虚线::买卖点全量匹配(&缠K.borrow().inner)
    }

    #[classmethod]
    fn 买卖点相对匹配(_cls: &Bound<'_, PyType>, 缠K: &Bound<'_, 缠论K线Py>) -> bool {
        chanlun::structure::dash_line::虚线::买卖点相对匹配(&缠K.borrow().inner)
    }

    // ---- MACD 相关 classmethods ----

    #[classmethod]
    fn 计算MACD柱子均值(
        _cls: &Bound<'_, PyType>,
        普K序列: Vec<Py<K线Py>>,
        实线: &Bound<'_, Self>,
        py: Python<'_>,
    ) -> f64 {
        let rc_list: Vec<Rc<chanlun::kline::bar::K线>> = 普K序列
            .iter()
            .map(|k| Rc::new(k.bind(py).borrow().inner.clone()))
            .collect();
        chanlun::structure::dash_line::虚线::计算MACD柱子均值(
            &rc_list,
            &实线.borrow().inner,
        )
    }

    #[classmethod]
    fn 武之全量MACD均值(
        _cls: &Bound<'_, PyType>,
        普K序列: Vec<Py<K线Py>>,
        实线: &Bound<'_, Self>,
        py: Python<'_>,
    ) -> bool {
        let rc_list: Vec<Rc<chanlun::kline::bar::K线>> = 普K序列
            .iter()
            .map(|k| Rc::new(k.bind(py).borrow().inner.clone()))
            .collect();
        chanlun::structure::dash_line::虚线::武之全量MACD均值(
            &rc_list,
            &实线.borrow().inner,
        )
    }

    #[classmethod]
    fn 武之MACD均值(
        _cls: &Bound<'_, PyType>,
        普K序列: Vec<Py<K线Py>>,
        实线: &Bound<'_, Self>,
        py: Python<'_>,
    ) -> bool {
        let rc_list: Vec<Rc<chanlun::kline::bar::K线>> = 普K序列
            .iter()
            .map(|k| Rc::new(k.bind(py).borrow().inner.clone()))
            .collect();
        chanlun::structure::dash_line::虚线::武之MACD均值(&rc_list, &实线.borrow().inner)
    }

    #[classmethod]
    fn 武之MACD极值(
        _cls: &Bound<'_, PyType>,
        普K序列: Vec<Py<K线Py>>,
        实线: &Bound<'_, Self>,
        py: Python<'_>,
    ) -> bool {
        let rc_list: Vec<Rc<chanlun::kline::bar::K线>> = 普K序列
            .iter()
            .map(|k| Rc::new(k.bind(py).borrow().inner.clone()))
            .collect();
        chanlun::structure::dash_line::虚线::武之MACD极值(&rc_list, &实线.borrow().inner)
    }

    #[classmethod]
    fn 计算MACD柱子均值_阴(
        _cls: &Bound<'_, PyType>,
        普K序列: Vec<Py<K线Py>>,
        实线: &Bound<'_, Self>,
        py: Python<'_>,
    ) -> Option<f64> {
        let rc_list: Vec<Rc<chanlun::kline::bar::K线>> = 普K序列
            .iter()
            .map(|k| Rc::new(k.bind(py).borrow().inner.clone()))
            .collect();
        chanlun::structure::dash_line::虚线::计算MACD柱子均值_阴(
            &rc_list,
            &实线.borrow().inner,
        )
    }

    #[classmethod]
    fn 计算MACD柱子均值_阳(
        _cls: &Bound<'_, PyType>,
        普K序列: Vec<Py<K线Py>>,
        实线: &Bound<'_, Self>,
        py: Python<'_>,
    ) -> Option<f64> {
        let rc_list: Vec<Rc<chanlun::kline::bar::K线>> = 普K序列
            .iter()
            .map(|k| Rc::new(k.bind(py).borrow().inner.clone()))
            .collect();
        chanlun::structure::dash_line::虚线::计算MACD柱子均值_阳(
            &rc_list,
            &实线.borrow().inner,
        )
    }

    #[classmethod]
    fn 武之MACD均值_阴(
        _cls: &Bound<'_, PyType>,
        普K序列: Vec<Py<K线Py>>,
        实线: &Bound<'_, Self>,
        py: Python<'_>,
    ) -> bool {
        let rc_list: Vec<Rc<chanlun::kline::bar::K线>> = 普K序列
            .iter()
            .map(|k| Rc::new(k.bind(py).borrow().inner.clone()))
            .collect();
        chanlun::structure::dash_line::虚线::武之MACD均值_阴(&rc_list, &实线.borrow().inner)
    }

    #[classmethod]
    fn 武之MACD均值_阳(
        _cls: &Bound<'_, PyType>,
        普K序列: Vec<Py<K线Py>>,
        实线: &Bound<'_, Self>,
        py: Python<'_>,
    ) -> bool {
        let rc_list: Vec<Rc<chanlun::kline::bar::K线>> = 普K序列
            .iter()
            .map(|k| Rc::new(k.bind(py).borrow().inner.clone()))
            .collect();
        chanlun::structure::dash_line::虚线::武之MACD均值_阳(&rc_list, &实线.borrow().inner)
    }

    #[classmethod]
    fn 计算K线序列MACD趋向背驰(
        _cls: &Bound<'_, PyType>,
        普K序列: Vec<Py<K线Py>>,
        方向: &Bound<'_, 相对方向Py>,
        py: Python<'_>,
    ) -> [bool; 3] {
        let rc_list: Vec<Rc<chanlun::kline::bar::K线>> = 普K序列
            .iter()
            .map(|k| Rc::new(k.bind(py).borrow().inner.clone()))
            .collect();
        chanlun::structure::dash_line::虚线::计算K线序列MACD趋向背驰(
            &rc_list,
            方向.borrow().inner,
        )
    }

    #[classmethod]
    fn 计算MACD柱子分段(
        _cls: &Bound<'_, PyType>,
        k线序列: Vec<Py<K线Py>>,
        py: Python<'_>,
    ) -> Vec<Vec<f64>> {
        let rc_list: Vec<Rc<chanlun::kline::bar::K线>> = k线序列
            .iter()
            .map(|k| Rc::new(k.bind(py).borrow().inner.clone()))
            .collect();
        chanlun::structure::dash_line::虚线::计算MACD柱子分段(&rc_list)
    }

    #[classmethod]
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
    fn 统计MACD行为(
        _cls: &Bound<'_, PyType>,
        普K序列: Vec<Py<K线Py>>,
        最大间隔: usize,
        最少交叉数: usize,
        py: Python<'_>,
    ) -> HashMap<String, String> {
        let rc_list: Vec<Rc<chanlun::kline::bar::K线>> = 普K序列
            .iter()
            .map(|k| Rc::new(k.bind(py).borrow().inner.clone()))
            .collect();
        chanlun::structure::dash_line::虚线::统计MACD行为(&rc_list, 最大间隔, 最少交叉数)
    }

    #[classmethod]
    fn 买卖意义(
        _cls: &Bound<'_, PyType>,
        实线: &Bound<'_, Self>,
        观察员: &Bound<'_, crate::business_py::观察者Py>,
    ) -> (bool, String) {
        let obs = 观察员.borrow();
        let obs_ref = obs.obs();
        chanlun::structure::dash_line::虚线::买卖意义(&实线.borrow().inner, &*obs_ref)
    }
}

// ========== 线段特征 ==========

#[pyclass(name = "线段特征", unsendable)]
#[derive(Clone)]
pub struct 线段特征Py {
    pub(crate) inner: Rc<chanlun::structure::segment_feat::线段特征>,
}

#[pymethods]
impl 线段特征Py {
    #[new]
    fn new(
        标识: String,
        基础序列: Vec<Py<虚线Py>>,
        线段方向: &Bound<'_, 相对方向Py>,
        py: Python<'_>,
    ) -> Self {
        let rc_list: Vec<Rc<chanlun::structure::dash_line::虚线>> = 基础序列
            .iter()
            .map(|d| Rc::clone(&d.bind(py).borrow().inner))
            .collect();
        Self {
            inner: Rc::new(chanlun::structure::segment_feat::线段特征::new(
                标识,
                rc_list,
                线段方向.borrow().inner,
            )),
        }
    }

    // ---- getters ----

    #[getter]
    fn 序号(&self) -> i64 {
        self.inner.序号
    }

    #[getter]
    fn 标识(&self) -> String {
        self.inner.标识.clone()
    }

    #[getter]
    fn 线段方向(&self) -> 相对方向Py {
        相对方向Py {
            inner: self.inner.线段方向,
        }
    }

    #[getter]
    fn 元素(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let list = pyo3::types::PyList::empty(py);
        for d in &self.inner.元素 {
            list.append(Py::new(
                py,
                虚线Py {
                    inner: Rc::clone(d),
                },
            )?)?;
        }
        Ok(list.into())
    }

    fn __str__(&self) -> String {
        format!("{}", self.inner)
    }

    fn __repr__(&self) -> String {
        self.__str__()
    }

    // ---- instance methods ----

    #[getter]
    fn 图表标题(&self) -> String {
        self.inner.图表标题()
    }

    #[getter]
    fn 文(&self) -> 分型Py {
        分型Py {
            inner: self.inner.文(),
        }
    }

    #[getter]
    fn 武(&self) -> 分型Py {
        分型Py {
            inner: self.inner.武(),
        }
    }

    #[getter]
    fn 方向(&self) -> 相对方向Py {
        相对方向Py {
            inner: self.inner.方向(),
        }
    }

    #[getter]
    fn 高(&self) -> f64 {
        self.inner.高()
    }

    #[getter]
    fn 低(&self) -> f64 {
        self.inner.低()
    }

    fn 添加(&mut self, 待添加虚线: &Bound<'_, 虚线Py>) -> PyResult<()> {
        let inner = Rc::make_mut(&mut self.inner);
        inner
            .添加(Rc::clone(&待添加虚线.borrow().inner))
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e))
    }

    fn 删除(&mut self, 待删除虚线: &Bound<'_, 虚线Py>) -> PyResult<()> {
        let inner = Rc::make_mut(&mut self.inner);
        inner
            .删除(&Rc::clone(&待删除虚线.borrow().inner))
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e))
    }

    // ---- classmethods ----

    #[classmethod]
    fn 新建(
        _cls: &Bound<'_, PyType>,
        虚线序列: Vec<Py<虚线Py>>,
        线段方向: &Bound<'_, 相对方向Py>,
        py: Python<'_>,
    ) -> Self {
        let rc_list: Vec<Rc<chanlun::structure::dash_line::虚线>> = 虚线序列
            .iter()
            .map(|d| Rc::clone(&d.bind(py).borrow().inner))
            .collect();
        Self {
            inner: Rc::new(chanlun::structure::segment_feat::线段特征::新建(
                rc_list,
                线段方向.borrow().inner,
            )),
        }
    }

    #[classmethod]
    fn 静态分析(
        _cls: &Bound<'_, PyType>,
        虚线序列: Vec<Py<虚线Py>>,
        线段方向: &Bound<'_, 相对方向Py>,
        四象: &str,
        是否忽视: bool,
        py: Python<'_>,
    ) -> Vec<Self> {
        let rc_list: Vec<Rc<chanlun::structure::dash_line::虚线>> = 虚线序列
            .iter()
            .map(|d| Rc::clone(&d.bind(py).borrow().inner))
            .collect();
        chanlun::structure::segment_feat::线段特征::静态分析(
            &rc_list,
            线段方向.borrow().inner,
            四象,
            是否忽视,
        )
        .into_iter()
        .map(|inner| Self { inner })
        .collect()
    }

    #[classmethod]
    fn 获取分型序列(
        _cls: &Bound<'_, PyType>,
        特征序列: Vec<Py<Self>>,
        py: Python<'_>,
    ) -> Vec<特征分型Py> {
        let rc_list: Vec<Rc<chanlun::structure::segment_feat::线段特征>> = 特征序列
            .iter()
            .map(|s| Rc::clone(&s.bind(py).borrow().inner))
            .collect();
        chanlun::structure::segment_feat::线段特征::获取分型序列(&rc_list)
            .into_iter()
            .map(|inner| 特征分型Py {
                inner: Rc::new(inner),
            })
            .collect()
    }
}

// ========== 特征分型 ==========

#[pyclass(name = "特征分型", unsendable)]
#[derive(Clone)]
pub struct 特征分型Py {
    pub(crate) inner: Rc<chanlun::structure::feat_fractal::特征分型>,
}

#[pymethods]
impl 特征分型Py {
    #[new]
    fn new(
        左: &Bound<'_, 线段特征Py>,
        中: &Bound<'_, 线段特征Py>,
        右: &Bound<'_, 线段特征Py>,
        结构: &Bound<'_, 分型结构Py>,
    ) -> Self {
        Self {
            inner: Rc::new(chanlun::structure::feat_fractal::特征分型::new(
                Rc::clone(&左.borrow().inner),
                Rc::clone(&中.borrow().inner),
                Rc::clone(&右.borrow().inner),
                结构.borrow().inner,
            )),
        }
    }

    #[getter]
    fn 左(&self) -> 线段特征Py {
        线段特征Py {
            inner: Rc::clone(&self.inner.左),
        }
    }

    #[getter]
    fn 中(&self) -> 线段特征Py {
        线段特征Py {
            inner: Rc::clone(&self.inner.中),
        }
    }

    #[getter]
    fn 右(&self) -> 线段特征Py {
        线段特征Py {
            inner: Rc::clone(&self.inner.右),
        }
    }

    #[getter]
    fn 结构(&self) -> 分型结构Py {
        分型结构Py {
            inner: self.inner.结构,
        }
    }

    fn __str__(&self) -> String {
        format!("{}", self.inner)
    }

    fn __repr__(&self) -> String {
        self.__str__()
    }
}

pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<分型Py>()?;
    m.add_class::<虚线Py>()?;
    m.add_class::<线段特征Py>()?;
    m.add_class::<特征分型Py>()?;
    Ok(())
}
