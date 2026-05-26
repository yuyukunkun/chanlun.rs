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
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::algorithm_py::中枢Py;
use crate::config_py::缠论配置Py;
use crate::kline_py::{缠论K线Py, K线Py};
use crate::structure_py::{分型Py, 虚线Py};
use crate::types_py::买卖点类型Py;

// ========== 基础买卖点 ==========

#[pyclass(name = "基础买卖点", unsendable)]
#[derive(Clone)]
pub struct 基础买卖点Py {
    pub(crate) inner: chanlun::business::bsp::基础买卖点,
}

#[pymethods]
impl 基础买卖点Py {
    #[new]
    fn new(
        类型: &Bound<'_, 买卖点类型Py>,
        当前K线: &Bound<'_, K线Py>,
        买卖点分型: &Bound<'_, 分型Py>,
        备注: String,
        中枢破位值: f64,
    ) -> Self {
        Self {
            inner: chanlun::business::bsp::基础买卖点::new(
                类型.borrow().inner,
                Rc::new(当前K线.borrow().inner.clone()),
                Rc::clone(&买卖点分型.borrow().inner),
                备注,
                中枢破位值,
            ),
        }
    }

    #[getter]
    fn 备注(&self) -> String {
        self.inner.备注.clone()
    }

    #[getter]
    fn 类型(&self) -> 买卖点类型Py {
        买卖点类型Py {
            inner: self.inner.类型,
        }
    }

    #[getter]
    fn 买卖点分型(&self) -> 分型Py {
        分型Py {
            inner: Rc::clone(&self.inner.买卖点分型),
        }
    }

    #[getter]
    fn 买卖点K线(&self) -> 缠论K线Py {
        缠论K线Py {
            inner: Rc::clone(&self.inner.买卖点K线),
        }
    }

    #[getter]
    fn 当前K线(&self) -> K线Py {
        K线Py {
            inner: (*self.inner.当前K线).clone(),
        }
    }

    #[getter]
    fn 失效K线(&self) -> Option<K线Py> {
        self.inner.失效K线.as_ref().map(|k| K线Py {
            inner: (**k).clone(),
        })
    }

    #[getter]
    fn 终结K线(&self) -> Option<K线Py> {
        self.inner.终结K线.as_ref().map(|k| K线Py {
            inner: (**k).clone(),
        })
    }

    #[getter]
    fn 破位值(&self) -> f64 {
        self.inner.破位值
    }

    #[getter]
    fn 结构(&self) -> Option<crate::types_py::分型结构Py> {
        self.inner
            .结构
            .map(|f| crate::types_py::分型结构Py { inner: f })
    }

    fn 偏移(&self) -> i64 {
        self.inner.偏移()
    }

    fn 失效偏移(&self) -> i64 {
        self.inner.失效偏移()
    }

    #[getter]
    fn 有效性(&self) -> bool {
        self.inner.有效性()
    }

    #[getter]
    fn 与MACD柱子匹配(&self) -> bool {
        self.inner.与MACD柱子匹配()
    }

    #[getter]
    fn 与MACD柱子分型匹配(&self) -> bool {
        self.inner.与MACD柱子分型匹配()
    }

    fn __str__(&self) -> String {
        format!("{}", self.inner)
    }

    fn __repr__(&self) -> String {
        self.__str__()
    }
}

// ========== 买卖点 ==========

#[pyclass(name = "买卖点")]
pub struct 买卖点Py;

#[pymethods]
impl 买卖点Py {
    #[classmethod]
    fn 一卖点(
        _cls: &Bound<'_, PyType>,
        买卖点分型: &Bound<'_, 分型Py>,
        当前K线: &Bound<'_, K线Py>,
        标识: &str,
        备注: String,
        中枢破位值: f64,
    ) -> 基础买卖点Py {
        基础买卖点Py {
            inner: chanlun::business::bsp::买卖点::一卖点(
                Rc::clone(&买卖点分型.borrow().inner),
                Rc::new(当前K线.borrow().inner.clone()),
                标识,
                备注,
                中枢破位值,
            ),
        }
    }

    #[classmethod]
    fn 一买点(
        _cls: &Bound<'_, PyType>,
        买卖点分型: &Bound<'_, 分型Py>,
        当前K线: &Bound<'_, K线Py>,
        标识: &str,
        备注: String,
        中枢破位值: f64,
    ) -> 基础买卖点Py {
        基础买卖点Py {
            inner: chanlun::business::bsp::买卖点::一买点(
                Rc::clone(&买卖点分型.borrow().inner),
                Rc::new(当前K线.borrow().inner.clone()),
                标识,
                备注,
                中枢破位值,
            ),
        }
    }

    #[classmethod]
    fn 二卖点(
        _cls: &Bound<'_, PyType>,
        买卖点分型: &Bound<'_, 分型Py>,
        当前K线: &Bound<'_, K线Py>,
        标识: &str,
        备注: String,
        中枢破位值: f64,
    ) -> 基础买卖点Py {
        基础买卖点Py {
            inner: chanlun::business::bsp::买卖点::二卖点(
                Rc::clone(&买卖点分型.borrow().inner),
                Rc::new(当前K线.borrow().inner.clone()),
                标识,
                备注,
                中枢破位值,
            ),
        }
    }

    #[classmethod]
    fn 二买点(
        _cls: &Bound<'_, PyType>,
        买卖点分型: &Bound<'_, 分型Py>,
        当前K线: &Bound<'_, K线Py>,
        标识: &str,
        备注: String,
        中枢破位值: f64,
    ) -> 基础买卖点Py {
        基础买卖点Py {
            inner: chanlun::business::bsp::买卖点::二买点(
                Rc::clone(&买卖点分型.borrow().inner),
                Rc::new(当前K线.borrow().inner.clone()),
                标识,
                备注,
                中枢破位值,
            ),
        }
    }

    #[classmethod]
    fn 三卖点(
        _cls: &Bound<'_, PyType>,
        买卖点分型: &Bound<'_, 分型Py>,
        当前K线: &Bound<'_, K线Py>,
        标识: &str,
        备注: String,
        中枢破位值: f64,
    ) -> 基础买卖点Py {
        基础买卖点Py {
            inner: chanlun::business::bsp::买卖点::三卖点(
                Rc::clone(&买卖点分型.borrow().inner),
                Rc::new(当前K线.borrow().inner.clone()),
                标识,
                备注,
                中枢破位值,
            ),
        }
    }

    #[classmethod]
    fn 三买点(
        _cls: &Bound<'_, PyType>,
        买卖点分型: &Bound<'_, 分型Py>,
        当前K线: &Bound<'_, K线Py>,
        标识: &str,
        备注: String,
        中枢破位值: f64,
    ) -> 基础买卖点Py {
        基础买卖点Py {
            inner: chanlun::business::bsp::买卖点::三买点(
                Rc::clone(&买卖点分型.borrow().inner),
                Rc::new(当前K线.borrow().inner.clone()),
                标识,
                备注,
                中枢破位值,
            ),
        }
    }

    #[classmethod]
    fn 生成买卖点(
        _cls: &Bound<'_, PyType>,
        特征: &str,
        序号: &str,
        级别: &str,
        买卖点分型: &Bound<'_, 分型Py>,
        当前缠K: &Bound<'_, 缠论K线Py>,
    ) -> 基础买卖点Py {
        基础买卖点Py {
            inner: chanlun::business::bsp::买卖点::生成买卖点(
                特征,
                序号,
                级别,
                Rc::clone(&买卖点分型.borrow().inner),
                Rc::clone(&当前缠K.borrow().inner),
            ),
        }
    }
}

// ========== 观察者 ==========

#[pyclass(name = "观察者", subclass, unsendable)]
pub struct 观察者Py {
    pub(crate) inner: Option<Rc<RefCell<chanlun::business::observer::观察者>>>,
}

impl 观察者Py {
    pub(crate) fn obs(&self) -> std::cell::Ref<'_, chanlun::business::observer::观察者> {
        self.inner
            .as_ref()
            .expect("观察者 尚未初始化，请通过 __init__(符号, 周期, 配置) 构造")
            .borrow()
    }

    pub(crate) fn obs_mut(&self) -> std::cell::RefMut<'_, chanlun::business::observer::观察者> {
        self.inner
            .as_ref()
            .expect("观察者 尚未初始化，请通过 __init__(符号, 周期, 配置) 构造")
            .borrow_mut()
    }
}

#[pymethods]
impl 观察者Py {
    /// __new__ 只分配空壳，构造逻辑在 __init__
    #[new]
    #[pyo3(signature = (*args, **kwargs))]
    fn new(
        args: &Bound<'_, pyo3::types::PyTuple>,
        kwargs: Option<&Bound<'_, pyo3::types::PyDict>>,
    ) -> Self {
        let _ = (args, kwargs);
        Self { inner: None }
    }

    /// __init__ 执行真正的构造。子类可重写并调用 super().__init__(符号, 周期, 配置)
    #[pyo3(signature = (符号, 周期, 配置 = None))]
    fn __init__(
        &mut self,
        py: Python<'_>,
        符号: String,
        周期: i64,
        配置: Option<&Bound<'_, 缠论配置Py>>,
    ) -> PyResult<()> {
        let config = match 配置 {
            Some(cfg) => cfg.borrow().to_rust_config(py)?,
            None => chanlun::config::缠论配置::default(),
        };
        self.inner = Some(chanlun::business::observer::观察者::new(
            符号, 周期, config,
        ));
        Ok(())
    }

    #[getter]
    fn 观察员(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    #[getter]
    fn 标识(&self) -> String {
        self.obs().标识()
    }

    #[getter]
    fn 当前K线(&self) -> Option<K线Py> {
        self.obs().当前K线().map(|k| K线Py {
            inner: (**k).clone(),
        })
    }

    #[getter]
    fn 当前缠K(&self) -> Option<缠论K线Py> {
        self.obs().当前缠K().map(|k| 缠论K线Py {
            inner: Rc::clone(k),
        })
    }

    #[getter]
    fn 符号(&self) -> String {
        self.obs().符号.clone()
    }

    #[getter]
    fn 周期(&self) -> i64 {
        self.obs().周期
    }

    #[getter]
    fn 配置(&self) -> PyResult<缠论配置Py> {
        缠论配置Py::from_rust_config(&self.obs().配置)
    }

    fn 重置基础序列(&mut self) {
        self.obs_mut().重置基础序列();
    }

    fn 增加原始K线(&mut self, 普K: &Bound<'_, K线Py>) {
        self.obs_mut().增加原始K线(普K.borrow().inner.clone());
    }

    fn 静态重新分析(&mut self) {
        self.obs_mut().静态重新分析();
    }

    fn 测试_保存数据(&self, root: Option<&str>) {
        self.obs().测试_保存数据(root);
    }

    #[classmethod]
    fn 读取数据文件(
        _cls: &Bound<'_, PyType>,
        文件路径: &str,
        配置: Option<&Bound<'_, 缠论配置Py>>,
        py: Python<'_>,
    ) -> PyResult<Self> {
        let config = match 配置 {
            Some(cfg) => Some(cfg.borrow().to_rust_config(py)?),
            None => None,
        };
        chanlun::business::observer::观察者::读取数据文件(文件路径, config)
            .map(|inner| Self { inner: Some(inner) })
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e))
    }

    // ---- 序列 getters ----

    #[getter]
    fn 普通K线序列(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let list = pyo3::types::PyList::empty(py);
        for k in &self.obs().普通K线序列 {
            list.append(K线Py {
                inner: (**k).clone(),
            })?;
        }
        Ok(list.into())
    }

    #[getter]
    fn 缠论K线序列(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let list = pyo3::types::PyList::empty(py);
        for k in &self.obs().缠论K线序列 {
            list.append(缠论K线Py {
                inner: Rc::clone(k),
            })?;
        }
        Ok(list.into())
    }

    #[getter]
    fn 分型序列(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let list = pyo3::types::PyList::empty(py);
        for f in &self.obs().分型序列 {
            list.append(分型Py {
                inner: Rc::clone(f),
            })?;
        }
        Ok(list.into())
    }

    #[getter]
    fn 笔序列(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let list = pyo3::types::PyList::empty(py);
        for d in &self.obs().笔序列 {
            list.append(虚线Py {
                inner: Rc::clone(d),
            })?;
        }
        Ok(list.into())
    }

    #[getter]
    fn 笔_中枢序列(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let list = pyo3::types::PyList::empty(py);
        for h in &self.obs().笔_中枢序列 {
            list.append(中枢Py {
                inner: Rc::clone(h),
            })?;
        }
        Ok(list.into())
    }

    #[getter]
    fn 线段序列(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let list = pyo3::types::PyList::empty(py);
        for d in &self.obs().线段序列 {
            list.append(虚线Py {
                inner: Rc::clone(d),
            })?;
        }
        Ok(list.into())
    }

    #[getter]
    fn 中枢序列(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let list = pyo3::types::PyList::empty(py);
        for h in &self.obs().中枢序列 {
            list.append(中枢Py {
                inner: Rc::clone(h),
            })?;
        }
        Ok(list.into())
    }

    #[getter]
    fn 扩展线段序列(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let list = pyo3::types::PyList::empty(py);
        for d in &self.obs().扩展线段序列 {
            list.append(虚线Py {
                inner: Rc::clone(d),
            })?;
        }
        Ok(list.into())
    }

    #[getter]
    fn 扩展中枢序列(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let list = pyo3::types::PyList::empty(py);
        for h in &self.obs().扩展中枢序列 {
            list.append(中枢Py {
                inner: Rc::clone(h),
            })?;
        }
        Ok(list.into())
    }

    #[getter]
    fn 扩展线段序列_线段(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let list = pyo3::types::PyList::empty(py);
        for d in &self.obs().扩展线段序列_线段 {
            list.append(虚线Py {
                inner: Rc::clone(d),
            })?;
        }
        Ok(list.into())
    }

    #[getter]
    fn 扩展中枢序列_线段(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let list = pyo3::types::PyList::empty(py);
        for h in &self.obs().扩展中枢序列_线段 {
            list.append(中枢Py {
                inner: Rc::clone(h),
            })?;
        }
        Ok(list.into())
    }

    #[getter]
    fn 线段_线段序列(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let list = pyo3::types::PyList::empty(py);
        for d in &self.obs().线段_线段序列 {
            list.append(虚线Py {
                inner: Rc::clone(d),
            })?;
        }
        Ok(list.into())
    }

    #[getter]
    fn 线段_中枢序列(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let list = pyo3::types::PyList::empty(py);
        for h in &self.obs().线段_中枢序列 {
            list.append(中枢Py {
                inner: Rc::clone(h),
            })?;
        }
        Ok(list.into())
    }

    #[getter]
    fn 扩展线段序列_扩展线段(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let list = pyo3::types::PyList::empty(py);
        for d in &self.obs().扩展线段序列_扩展线段 {
            list.append(虚线Py {
                inner: Rc::clone(d),
            })?;
        }
        Ok(list.into())
    }

    #[getter]
    fn 扩展中枢序列_扩展线段(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let list = pyo3::types::PyList::empty(py);
        for h in &self.obs().扩展中枢序列_扩展线段 {
            list.append(中枢Py {
                inner: Rc::clone(h),
            })?;
        }
        Ok(list.into())
    }
}

// ========== K线合成器 ==========

#[pyclass(name = "K线合成器", unsendable)]
pub struct K线合成器Py {
    pub(crate) inner: chanlun::business::synthesizer::K线合成器,
}

#[pymethods]
impl K线合成器Py {
    #[new]
    fn new(标识: String, 周期组: Vec<i64>) -> Self {
        Self {
            inner: chanlun::business::synthesizer::K线合成器::new(标识, 周期组),
        }
    }

    fn 投喂K线(
        &mut self,
        普K: &Bound<'_, K线Py>,
        py: Python<'_>,
    ) -> PyResult<Vec<(i64, K线Py)>> {
        let results = self.inner.投喂K线(普K.borrow().inner.clone());
        Ok(results
            .into_iter()
            .map(|(周期, k)| (周期, K线Py { inner: k }))
            .collect())
    }

    fn 投喂(
        &mut self,
        时间戳: i64,
        开: f64,
        高: f64,
        低: f64,
        收: f64,
        量: f64,
    ) -> Vec<(i64, K线Py)> {
        let min_cycle = self.inner.周期组.iter().copied().min().unwrap_or(1);
        let k = chanlun::kline::bar::K线::创建普K(
            &self.inner.标识,
            时间戳,
            开,
            高,
            低,
            收,
            量,
            0,
            min_cycle,
        );
        let results = self.inner.投喂K线(k);
        results
            .into_iter()
            .map(|(周期, k2)| (周期, K线Py { inner: k2 }))
            .collect()
    }

    fn 获取当前K线(&self, 周期: i64) -> Option<K线Py> {
        self.inner
            .获取当前K线(周期)
            .map(|k| K线Py { inner: k.clone() })
    }

    #[getter]
    fn 标识(&self) -> String {
        self.inner.标识.clone()
    }

    #[getter]
    fn 周期组(&self) -> Vec<i64> {
        self.inner.周期组.clone()
    }
}

// ========== 立体分析器 ==========

#[pyclass(name = "立体分析器", unsendable)]
pub struct 立体分析器Py {
    pub(crate) inner: chanlun::business::multi_frame::立体分析器,
}

#[pymethods]
impl 立体分析器Py {
    #[new]
    #[pyo3(signature = (符号, 周期组, 配置 = None, 配置组 = None))]
    fn new(
        符号: String,
        周期组: Vec<i64>,
        配置: Option<&Bound<'_, 缠论配置Py>>,
        配置组: Option<&Bound<'_, PyAny>>,
        py: Python<'_>,
    ) -> PyResult<Self> {
        let config = match 配置 {
            Some(cfg) => Some(cfg.borrow().to_rust_config(py)?),
            None => None,
        };
        let cfg_map: Option<HashMap<i64, chanlun::config::缠论配置>> = match 配置组 {
            Some(dict_any) => {
                let dict = dict_any.downcast::<pyo3::types::PyDict>()?;
                let mut map = HashMap::new();
                for (key, value) in dict.iter() {
                    let period: i64 = key.extract()?;
                    let cfg: PyRef<'_, 缠论配置Py> = value.extract()?;
                    map.insert(period, cfg.to_rust_config(py)?);
                }
                Some(map)
            }
            None => None,
        };
        Ok(Self {
            inner: chanlun::business::multi_frame::立体分析器::new(
                符号, 周期组, config, cfg_map,
            ),
        })
    }

    fn 投喂K线(&mut self, 普K: &Bound<'_, K线Py>) {
        self.inner.投喂K线(普K.borrow().inner.clone());
    }

    fn 获取观察者(&self, 周期: i64) -> Option<观察者Py> {
        self.inner
            .获取观察者(周期)
            .map(|rc| 观察者Py { inner: Some(rc) })
    }

    fn 测试_保存数据(&self) {
        self.inner.测试_保存数据();
    }

    #[getter]
    fn 周期组(&self) -> Vec<i64> {
        self.inner.周期组.clone()
    }
}

pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<基础买卖点Py>()?;
    m.add_class::<买卖点Py>()?;
    m.add_class::<观察者Py>()?;
    m.add_class::<K线合成器Py>()?;
    m.add_class::<立体分析器Py>()?;
    Ok(())
}
