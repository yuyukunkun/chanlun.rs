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
use std::sync::RwLock;

use crate::algorithm_py::hub_to_py;
use crate::kline_py::bar_to_py;
use crate::structure_py::{dashed_to_py, fractal_to_py};
use std::collections::HashMap;
use std::sync::Arc;

use crate::algorithm_py::中枢Py;
use crate::config_py::缠论配置Py;
use crate::kline_py::{缠论K线Py, K线Py};
use crate::structure_py::{分型Py, 虚线Py};
use crate::types_py::买卖点类型Py;

// ========== 基础买卖点 ==========

/// 基础买卖点 — 买卖点的数据结构，描述偏离买入/卖出位置的程度。
///
/// 属性:
///   偏移: int — 买卖点距当前K线的偏差
///   失效偏移: int — 买卖点失效的最大偏差
///   有效性: bool — 买卖点是否仍有效
///   与MACD柱子匹配: bool|None — 是否与MACD柱状图方向匹配
///   与MACD柱子分型匹配: bool|None — 是否与MACD柱分型匹配
#[pyclass(
    name = "基础买卖点",
    module = "chanlun._chanlun",
    subclass,
    from_py_object
)]
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
                当前K线.borrow().inner.clone(),
                Arc::clone(&买卖点分型.borrow().inner),
                备注,
                中枢破位值,
                当前K线.borrow().inner.序号,
            ),
        }
    }

    #[getter]
    fn 备注(&self) -> String {
        self.inner.备注.clone()
    }

    #[setter]
    #[pyo3(name = "备注")]
    fn 设置_备注(&mut self, value: String) {
        self.inner.备注 = value;
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
            inner: Arc::clone(&self.inner.买卖点分型),
        }
    }

    #[getter]
    fn 买卖点K线(&self, py: Python<'_>) -> Py<缠论K线Py> {
        crate::kline_py::chan_kline_to_py(py, Arc::clone(&self.inner.买卖点K线))
    }

    #[getter]
    /// 当前K线
    fn 当前K线(&self, py: Python<'_>) -> Py<K线Py> {
        bar_to_py(py, self.inner.当前K线.clone())
    }

    #[getter]
    fn 失效K线(&self, py: Python<'_>) -> Option<Py<K线Py>> {
        self.inner
            .失效K线
            .as_ref()
            .map(|k| bar_to_py(py, Arc::clone(k)))
    }

    #[getter]
    fn 终结K线(&self, py: Python<'_>) -> Option<Py<K线Py>> {
        self.inner
            .终结K线
            .as_ref()
            .map(|k| bar_to_py(py, Arc::clone(k)))
    }

    #[getter]
    /// 破位值
    fn 破位值(&self) -> f64 {
        self.inner.破位值
    }

    #[getter]
    fn 结构(&self) -> Option<crate::types_py::分型结构Py> {
        self.inner
            .结构
            .map(|f| crate::types_py::分型结构Py { inner: f })
    }

    #[getter]
    /// 偏移
    fn 偏移(&self) -> i64 {
        self.inner.偏移()
    }

    #[getter]
    /// 失效偏移
    fn 失效偏移(&self) -> i64 {
        self.inner.失效偏移()
    }

    #[getter]
    /// 有效性
    fn 有效性(&self) -> bool {
        self.inner.有效性()
    }

    #[getter]
    /// 与MACD柱子匹配
    fn 与MACD柱子匹配(&self) -> bool {
        self.inner.与MACD柱子匹配()
    }

    #[getter]
    /// 与MACD柱子分型匹配
    fn 与MACD柱子分型匹配(&self) -> bool {
        self.inner.与MACD柱子分型匹配()
    }

    /// pandas 兼容 — 返回关键标量字段构成的字典
    #[getter]
    fn __dict__(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let dict = PyDict::new(py);
        dict.set_item("备注", self.备注())?;
        dict.set_item("类型", self.类型())?;
        dict.set_item("破位值", self.破位值())?;
        dict.set_item("偏移", self.偏移())?;
        dict.set_item("失效偏移", self.失效偏移())?;
        dict.set_item("有效性", self.有效性())?;
        dict.set_item("与MACD柱子匹配", self.与MACD柱子匹配())?;
        dict.set_item("与MACD柱子分型匹配", self.与MACD柱子分型匹配())?;
        if let Some(v) = self.结构() {
            dict.set_item("结构", v)?;
        }
        Ok(dict.into())
    }

    fn __str__(&self) -> String {
        format!("{}", self.inner)
    }

    fn __repr__(&self) -> String {
        self.__str__()
    }
}

// ========== 买卖点 ==========

/// 买卖点 — 继承 基础买卖点，添加工厂类方法。
///
/// 类方法（均返回 买卖点 实例）:
///   一卖点(...) / 一买点(...) / 二卖点(...) / 二买点(...) / 三卖点(...) / 三买点(...)
///   生成买卖点(特征, 序号, 级别, 分型, 当前缠K) -> 买卖点
#[pyclass(name = "买卖点", module = "chanlun._chanlun", extends=基础买卖点Py)]
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
        py: Python<'_>,
    ) -> PyResult<Py<Self>> {
        let base = 基础买卖点Py {
            inner: chanlun::business::bsp::买卖点::一卖点(
                Arc::clone(&买卖点分型.borrow().inner),
                当前K线.borrow().inner.clone(),
                标识,
                备注,
                中枢破位值,
                当前K线.borrow().inner.序号,
            ),
        };
        let init = PyClassInitializer::from(base).add_subclass(买卖点Py);
        Ok(Bound::new(py, init)?.unbind())
    }

    #[classmethod]
    fn 一买点(
        _cls: &Bound<'_, PyType>,
        买卖点分型: &Bound<'_, 分型Py>,
        当前K线: &Bound<'_, K线Py>,
        标识: &str,
        备注: String,
        中枢破位值: f64,
        py: Python<'_>,
    ) -> PyResult<Py<Self>> {
        let base = 基础买卖点Py {
            inner: chanlun::business::bsp::买卖点::一买点(
                Arc::clone(&买卖点分型.borrow().inner),
                当前K线.borrow().inner.clone(),
                标识,
                备注,
                中枢破位值,
                当前K线.borrow().inner.序号,
            ),
        };
        let init = PyClassInitializer::from(base).add_subclass(买卖点Py);
        Ok(Bound::new(py, init)?.unbind())
    }

    #[classmethod]
    fn 二卖点(
        _cls: &Bound<'_, PyType>,
        买卖点分型: &Bound<'_, 分型Py>,
        当前K线: &Bound<'_, K线Py>,
        标识: &str,
        备注: String,
        中枢破位值: f64,
        py: Python<'_>,
    ) -> PyResult<Py<Self>> {
        let base = 基础买卖点Py {
            inner: chanlun::business::bsp::买卖点::二卖点(
                Arc::clone(&买卖点分型.borrow().inner),
                当前K线.borrow().inner.clone(),
                标识,
                备注,
                中枢破位值,
                当前K线.borrow().inner.序号,
            ),
        };
        let init = PyClassInitializer::from(base).add_subclass(买卖点Py);
        Ok(Bound::new(py, init)?.unbind())
    }

    #[classmethod]
    fn 二买点(
        _cls: &Bound<'_, PyType>,
        买卖点分型: &Bound<'_, 分型Py>,
        当前K线: &Bound<'_, K线Py>,
        标识: &str,
        备注: String,
        中枢破位值: f64,
        py: Python<'_>,
    ) -> PyResult<Py<Self>> {
        let base = 基础买卖点Py {
            inner: chanlun::business::bsp::买卖点::二买点(
                Arc::clone(&买卖点分型.borrow().inner),
                当前K线.borrow().inner.clone(),
                标识,
                备注,
                中枢破位值,
                当前K线.borrow().inner.序号,
            ),
        };
        let init = PyClassInitializer::from(base).add_subclass(买卖点Py);
        Ok(Bound::new(py, init)?.unbind())
    }

    #[classmethod]
    fn 三卖点(
        _cls: &Bound<'_, PyType>,
        买卖点分型: &Bound<'_, 分型Py>,
        当前K线: &Bound<'_, K线Py>,
        标识: &str,
        备注: String,
        中枢破位值: f64,
        py: Python<'_>,
    ) -> PyResult<Py<Self>> {
        let base = 基础买卖点Py {
            inner: chanlun::business::bsp::买卖点::三卖点(
                Arc::clone(&买卖点分型.borrow().inner),
                当前K线.borrow().inner.clone(),
                标识,
                备注,
                中枢破位值,
                当前K线.borrow().inner.序号,
            ),
        };
        let init = PyClassInitializer::from(base).add_subclass(买卖点Py);
        Ok(Bound::new(py, init)?.unbind())
    }

    #[classmethod]
    fn 三买点(
        _cls: &Bound<'_, PyType>,
        买卖点分型: &Bound<'_, 分型Py>,
        当前K线: &Bound<'_, K线Py>,
        标识: &str,
        备注: String,
        中枢破位值: f64,
        py: Python<'_>,
    ) -> PyResult<Py<Self>> {
        let base = 基础买卖点Py {
            inner: chanlun::business::bsp::买卖点::三买点(
                Arc::clone(&买卖点分型.borrow().inner),
                当前K线.borrow().inner.clone(),
                标识,
                备注,
                中枢破位值,
                当前K线.borrow().inner.序号,
            ),
        };
        let init = PyClassInitializer::from(base).add_subclass(买卖点Py);
        Ok(Bound::new(py, init)?.unbind())
    }

    #[classmethod]
    #[pyo3(signature = (特征, 序号, 级别, 买卖点分型, 当前缠K))]
    fn 生成买卖点(
        _cls: &Bound<'_, PyType>,
        特征: &str,
        序号: &str,
        级别: &str,
        买卖点分型: &Bound<'_, 分型Py>,
        当前缠K: &Bound<'_, 缠论K线Py>,
        py: Python<'_>,
    ) -> PyResult<Py<Self>> {
        let base = 基础买卖点Py {
            inner: chanlun::business::bsp::买卖点::生成买卖点(
                特征,
                序号,
                级别,
                Arc::clone(&买卖点分型.borrow().inner),
                Arc::clone(&当前缠K.borrow().inner),
            ),
        };
        let init = PyClassInitializer::from(base).add_subclass(买卖点Py);
        Ok(Bound::new(py, init)?.unbind())
    }
}

// ========== 观察者 ==========

/// 观察者 — 单周期分析器，接收 K 线流式输入后逐层计算所有序列。
///
/// 支持 Python 继承（__new__ + __init__ 构造函数）。每根 K 线喂入后
/// 自动增量计算: 缠论K线 → 分型 → 笔 → 线段 → 中枢 等全部层级。
///
/// 构造:
///   观察者(符号, 周期, 配置) — 创建指定周期和配置的分析器
///
/// 属性（只读）:
///   符号: str / 周期: int / 标识: str
///   当前K线: K线|None / 当前缠K: 缠论K线|None
///
/// 序列 getter（只读，返回 list 副本）:
///   普通K线序列 / 缠论K线序列 / 分型序列
///   笔序列 / 笔_中枢序列
///   线段序列 / 中枢序列
///   扩展线段序列 / 扩展中枢序列
///   扩展线段序列_线段 / 扩展中枢序列_线段
///   线段_线段序列 / 线段_中枢序列
///   扩展线段序列_扩展线段 / 扩展中枢序列_扩展线段
///
/// 核心方法:
///   增加原始K线(普K) — 喂入一根普通K线，触发全层级增量计算
///   重置基础序列() — 清空所有计算状态和序列
///   静态重新分析() — 对已有普通K线序列进行全量重新分析
///
/// 调试方法:
///   测试_保存数据(root?) — 将各层级序列保存到文件，用于与Python版对比
///   读取数据文件(文件路径, 配置) -> 观察者 (classmethod) — 从 .nb 文件加载数据
#[pyclass(name = "观察者", module = "chanlun._chanlun", subclass)]
pub struct 观察者Py {
    pub(crate) inner: Option<Arc<RwLock<chanlun::business::observer::观察者>>>,
}

impl 观察者Py {
    pub(crate) fn obs(
        &self,
    ) -> std::sync::RwLockReadGuard<'_, chanlun::business::observer::观察者> {
        self.inner
            .as_ref()
            .expect("观察者 尚未初始化，请通过 __init__(符号, 周期, 配置) 构造")
            .read()
            .unwrap_or_else(|e| e.into_inner())
    }

    pub(crate) fn obs_mut(
        &self,
    ) -> std::sync::RwLockWriteGuard<'_, chanlun::business::observer::观察者> {
        self.inner
            .as_ref()
            .expect("观察者 尚未初始化，请通过 __init__(符号, 周期, 配置) 构造")
            .write()
            .unwrap_or_else(|e| e.into_inner())
    }
}

#[pymethods]
impl 观察者Py {
    /// __new__ 从 *args/**kwargs 中提取 (符号, 周期, 配置)，完整初始化观察者。
    /// 由于形参可变（子类参数各异），不依赖固定签名，而是从 args/kwargs 中按位置和名称智能提取。
    #[new]
    #[pyo3(signature = (*args, **kwargs))]
    fn new(
        args: &Bound<'_, pyo3::types::PyTuple>,
        kwargs: Option<&Bound<'_, pyo3::types::PyDict>>,
    ) -> PyResult<Self> {
        let py = args.py();

        // 提取 符号（位置 0 或关键字）
        let 符号: String = if !args.is_empty() {
            args.get_item(0)?.extract()?
        } else {
            match kwargs.and_then(|kw| kw.get_item("符号").ok().flatten()) {
                Some(val) => val.extract()?,
                None => return Err(pyo3::exceptions::PyTypeError::new_err("缺少参数: 符号")),
            }
        };

        // 提取 周期（位置 1 或关键字）
        let 周期: i64 = if args.len() >= 2 {
            args.get_item(1)?.extract()?
        } else {
            match kwargs.and_then(|kw| kw.get_item("周期").ok().flatten()) {
                Some(val) => val.extract()?,
                None => return Err(pyo3::exceptions::PyTypeError::new_err("缺少参数: 周期")),
            }
        };

        // 提取 配置（关键字优先，然后扫描剩余位置参数）
        let mut config = None;
        if let Some(val) = kwargs.and_then(|kw| kw.get_item("配置").ok().flatten()) {
            let cfg: PyRef<'_, 缠论配置Py> = val.extract()?;
            config = Some(cfg.to_rust_config(py)?);
        }
        if config.is_none() {
            for i in 2..args.len() {
                if let Ok(cfg) = args.get_item(i)?.extract::<PyRef<'_, 缠论配置Py>>() {
                    config = Some(cfg.to_rust_config(py)?);
                    break;
                }
            }
        }
        let config = config.unwrap_or_else(chanlun::config::缠论配置::default);

        Ok(Self {
            inner: Some(chanlun::business::observer::观察者::new(
                符号, 周期, config,
            )),
        })
    }

    /// __init__ 不重复构造 — __new__ 已完成初始化。
    /// 接收 *args/**kwargs 是为了兼容 Python 构造函数给 __init__ 传入的任意形参。
    #[pyo3(signature = (*args, **kwargs))]
    fn __init__(
        &mut self,
        args: &Bound<'_, pyo3::types::PyTuple>,
        kwargs: Option<&Bound<'_, pyo3::types::PyDict>>,
    ) -> PyResult<()> {
        let _ = (args, kwargs);
        Ok(())
    }

    #[getter]
    /// 观察员（自引用）
    fn 观察员(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    #[getter]
    /// :return: "{符号}:{周期}"
    fn 标识(&self) -> String {
        self.obs().标识()
    }

    #[getter]
    /// :return: 最后一根原始K线
    fn 当前K线(&self, py: Python<'_>) -> Option<Py<K线Py>> {
        self.obs().当前K线().map(|k| bar_to_py(py, Arc::clone(k)))
    }

    #[getter]
    /// :return: 最后一根缠论K线
    fn 当前缠K(&self, py: Python<'_>) -> Option<Py<缠论K线Py>> {
        self.obs()
            .当前缠K()
            .map(|k| crate::kline_py::chan_kline_to_py(py, Arc::clone(k)))
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

    /// 清空所有分析序列，重置为初始状态
    fn 重置基础序列(&mut self) {
        self.obs_mut().重置基础序列();
    }

    /// 核心入口 — 投喂一根原始K线，增量更新所有层级
    fn 增加原始K线(&mut self, 普K: &Bound<'_, K线Py>) {
        self.obs_mut().增加原始K线((*普K.borrow().inner).clone());
    }

    /// 加载本地数据 — 从 .nb 文件加载K线数据（先重置，再通过 Python dispatch 逐根投喂，
    /// 确保子类重写的 增加原始K线 被正确调用）。
    fn 加载本地数据(slf: &Bound<'_, Self>, 文件路径: &str) -> PyResult<()> {
        let py = slf.py();

        // 重置基础序列
        slf.borrow_mut().obs_mut().重置基础序列();

        // 解析文件得到 K线 列表
        let bars = slf
            .borrow()
            .obs()
            .解析本地数据(文件路径)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e))?;

        // 通过 Python dispatch 逐根投喂，确保子类重写生效
        for k线 in bars {
            let k线_py = Py::new(
                py,
                K线Py {
                    inner: Arc::new(k线),
                },
            )?;
            slf.call_method1("增加原始K线", (k线_py,))?;
        }
        Ok(())
    }

    /// 静态重新分析（占位方法）
    fn 静态重新分析(&mut self) {
        self.obs_mut().静态重新分析();
    }

    #[pyo3(signature = (root = None))]
    /// 拆分各序列数据，单独存文件，文件名为对应变量名
    fn 测试_保存数据(&self, root: Option<&str>) {
        self.obs().测试_保存数据(root);
    }

    #[classmethod]
    #[pyo3(signature = (文件路径, 配置 = None))]
    /// :param 文件路径: 数据文件路径 格式如: btcusd-300-1631772074-1632222374.nb
    fn 读取数据文件(
        cls: &Bound<'_, PyType>,
        文件路径: &str,
        配置: Option<&Bound<'_, 缠论配置Py>>,
        py: Python<'_>,
    ) -> PyResult<Py<PyAny>> {
        let config = match 配置 {
            Some(cfg) => cfg.borrow().to_rust_config(py)?,
            None => chanlun::config::缠论配置::default(),
        };

        // 从文件名解析 符号/周期: "btcusd-300-1761327300-1776327900.nb"
        let path = std::path::Path::new(文件路径);
        let name = path
            .file_stem()
            .and_then(|n| n.to_str())
            .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("invalid filename"))?;
        let parts: Vec<&str> = name.split('-').collect();
        if parts.len() < 4 {
            return Err(pyo3::exceptions::PyValueError::new_err(format!(
                "invalid filename format: {}",
                name
            )));
        }
        let 符号 = parts[0].to_string();
        let 周期: i64 = parts[1]
            .parse()
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("parse period: {}", e)))?;

        // 通过 cls 构造实例（支持子类化）
        let cfg_py = 缠论配置Py::from_rust_config(&config)?;
        let cfg_obj = Py::new(py, cfg_py)?;
        let obj = cls.call1((符号.clone(), 周期, cfg_obj))?;

        // 读取文件并通过 Python 分发逐根投喂（支持子类重写 增加原始K线）
        let data = std::fs::read(文件路径)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("read file: {}", e)))?;
        let size: usize = 48;
        for i in 0..data.len() / size {
            let offset = i * size;
            if let Some(k线) =
                chanlun::kline::bar::K线::from_bytes(&data[offset..offset + size], 周期, &符号)
            {
                let k线_py = Py::new(
                    py,
                    K线Py {
                        inner: Arc::new(k线),
                    },
                )?;
                obj.call_method1("增加原始K线", (k线_py,))?;
            }
        }

        Ok(obj.unbind())
    }

    // ---- 序列 getters ----

    #[getter]
    fn 普通K线序列(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let list = pyo3::types::PyList::empty(py);
        for k in &self.obs().普通K线序列 {
            list.append(bar_to_py(py, Arc::clone(k)))?;
        }
        Ok(list.into())
    }

    #[getter]
    fn 基础缠K序列(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let list = pyo3::types::PyList::empty(py);
        for k in &self.obs().基础缠K序列 {
            list.append(crate::kline_py::chan_kline_to_py(py, Arc::clone(k)))?;
        }
        Ok(list.into())
    }

    #[setter]
    #[pyo3(name = "基础缠K序列")]
    fn 设置_基础缠K序列(&mut self, value: &Bound<'_, PyAny>) -> PyResult<()> {
        let list: &Bound<'_, pyo3::types::PyList> = value.cast()?;
        let mut vec = Vec::new();
        for item in list {
            let 缠k: PyRef<'_, 缠论K线Py> = item.extract()?;
            vec.push(Arc::clone(&缠k.inner));
        }
        self.obs_mut().基础缠K序列 = vec;
        Ok(())
    }

    #[getter]
    fn 缠论K线序列(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let list = pyo3::types::PyList::empty(py);
        for k in &self.obs().缠论K线序列 {
            list.append(crate::kline_py::chan_kline_to_py(py, Arc::clone(k)))?;
        }
        Ok(list.into())
    }

    #[getter]
    fn 分型序列(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let list = pyo3::types::PyList::empty(py);
        for f in &self.obs().分型序列 {
            list.append(fractal_to_py(py, Arc::clone(f)))?;
        }
        Ok(list.into())
    }

    #[getter]
    fn 笔序列(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let list = pyo3::types::PyList::empty(py);
        for d in &self.obs().笔序列 {
            list.append(dashed_to_py(py, Arc::clone(d)))?;
        }
        Ok(list.into())
    }

    #[getter]
    fn 笔_中枢序列(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let list = pyo3::types::PyList::empty(py);
        for h in &self.obs().笔_中枢序列 {
            list.append(hub_to_py(py, Arc::clone(h)))?;
        }
        Ok(list.into())
    }

    #[getter]
    fn 线段序列(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let list = pyo3::types::PyList::empty(py);
        for d in &self.obs().线段序列 {
            list.append(dashed_to_py(py, Arc::clone(d)))?;
        }
        Ok(list.into())
    }

    #[getter]
    fn 中枢序列(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let list = pyo3::types::PyList::empty(py);
        for h in &self.obs().中枢序列 {
            list.append(hub_to_py(py, Arc::clone(h)))?;
        }
        Ok(list.into())
    }

    #[getter]
    fn 扩展线段序列(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let list = pyo3::types::PyList::empty(py);
        for d in &self.obs().扩展线段序列 {
            list.append(dashed_to_py(py, Arc::clone(d)))?;
        }
        Ok(list.into())
    }

    #[getter]
    fn 扩展中枢序列(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let list = pyo3::types::PyList::empty(py);
        for h in &self.obs().扩展中枢序列 {
            list.append(hub_to_py(py, Arc::clone(h)))?;
        }
        Ok(list.into())
    }

    #[getter]
    fn 扩展线段序列_线段(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let list = pyo3::types::PyList::empty(py);
        for d in &self.obs().扩展线段序列_线段 {
            list.append(dashed_to_py(py, Arc::clone(d)))?;
        }
        Ok(list.into())
    }

    #[getter]
    fn 扩展中枢序列_线段(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let list = pyo3::types::PyList::empty(py);
        for h in &self.obs().扩展中枢序列_线段 {
            list.append(hub_to_py(py, Arc::clone(h)))?;
        }
        Ok(list.into())
    }

    #[getter]
    fn 线段_线段序列(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let list = pyo3::types::PyList::empty(py);
        for d in &self.obs().线段_线段序列 {
            list.append(dashed_to_py(py, Arc::clone(d)))?;
        }
        Ok(list.into())
    }

    #[getter]
    fn 线段_中枢序列(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let list = pyo3::types::PyList::empty(py);
        for h in &self.obs().线段_中枢序列 {
            list.append(hub_to_py(py, Arc::clone(h)))?;
        }
        Ok(list.into())
    }

    #[getter]
    fn 扩展线段序列_扩展线段(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let list = pyo3::types::PyList::empty(py);
        for d in &self.obs().扩展线段序列_扩展线段 {
            list.append(dashed_to_py(py, Arc::clone(d)))?;
        }
        Ok(list.into())
    }

    #[getter]
    fn 扩展中枢序列_扩展线段(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let list = pyo3::types::PyList::empty(py);
        for h in &self.obs().扩展中枢序列_扩展线段 {
            list.append(hub_to_py(py, Arc::clone(h)))?;
        }
        Ok(list.into())
    }
}

// ========== K线合成器 ==========

/// K线合成器 — 将小周期K线合成为大周期K线。
///
/// 构造:
///   K线合成器(标识, 周期组) — 周期组为升序排列的整数列表（如 [60, 300, 900]）
///
/// 方法:
///   投喂K线(普K) -> list[(周期, K线)] — 喂入普通K线，返回合成后的大周期K线
///   投喂(时间戳, 开盘价, 最高价, 最低价, 收盘价, 成交量) -> list[(周期, K线)]
///      — 快捷入口，免去构造K线对象
///   获取当前K线(周期) -> K线|None — 获取指定周期的当前合成结果
#[pyclass(name = "K线合成器", module = "chanlun._chanlun")]
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

    /// 统一入口 — 投喂最小周期K线，自动合成大周期并分发给各周期观察者
    fn 投喂K线(
        &mut self,
        普K: &Bound<'_, K线Py>,
        py: Python<'_>,
    ) -> PyResult<Vec<(i64, K线Py)>> {
        let results = self.inner.投喂K线((*普K.borrow().inner).clone());
        Ok(results
            .into_iter()
            .map(|(周期, k)| (周期, K线Py { inner: Arc::new(k) }))
            .collect())
    }

    /// 投喂原始tick数据
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
            .map(|(周期, k2)| {
                (
                    周期,
                    K线Py {
                        inner: Arc::new(k2),
                    },
                )
            })
            .collect()
    }

    /// 获取指定周期当前正在合成的K线
    fn 获取当前K线(&self, 周期: i64) -> Option<K线Py> {
        self.inner.获取当前K线(周期).map(|k| K线Py {
            inner: Arc::new(k.clone()),
        })
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

/// 立体分析器 — 多周期分析器，内部包含 K线合成器 + 每周期一个观察者。
///
/// 通过最小周期数据合成大周期，并将各周期分析结果对齐。
///
/// 构造:
///   立体分析器(符号, 周期组, 配置?, 配置组?) — 周期组为升序列表（如 [60, 300, 900]）
///     配置为默认配置，配置组为各周期独立配置（dict[周期, 缠论配置]）
///
/// 方法:
///   投喂K线(普K) — 喂入最小周期K线，自动合成大周期并分发给各周期观察者
///   测试_保存数据(root?) — 保存各周期的分析数据到文件
#[pyclass(name = "立体分析器", module = "chanlun._chanlun")]
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

    /// 统一入口 — 投喂最小周期K线，自动合成大周期并分发给各周期观察者
    fn 投喂K线(&mut self, 普K: &Bound<'_, K线Py>) {
        self.inner.投喂K线((*普K.borrow().inner).clone());
    }

    fn 获取观察者(&self, 周期: i64) -> Option<观察者Py> {
        self.inner
            .获取观察者(周期)
            .map(|rc| 观察者Py { inner: Some(rc) })
    }

    /// 拆分各序列数据，单独存文件，文件名为对应变量名
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
