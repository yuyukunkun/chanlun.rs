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
use std::collections::HashMap;

/// 缠论配置 — 控制所有分析阶段行为的参数集（共 60+ 字段，均有默认值）。
///
/// 构造:
///   缠论配置(**kwargs) — 创建配置，可选关键字参数覆盖默认值
///
/// 字段分组:
///   [基础]
///     标识: str — K线标识（默认 "bar"）
///
///   [缠K]
///     缠K合并替换: bool — K线合并时是否替换原值
///
///   [笔]
///     笔内元素数量: int — 笔的最小元素数（默认 5）
///     笔弱化: bool — 是否启用笔弱化模式
///     笔次成笔: bool — 线段内部次级笔是否成笔
///     笔内相同终点取舍: bool / 笔内起始分型包含整笔: bool /
///     笔内原始K线包含整笔: bool
///
///   [线段]
///     线段_特征序列忽视老阴老阳: bool / 线段_缺口后紧急修正: bool /
///     线段_非缺口下穿刺: bool / 线段_修正: bool / 线段内部中枢图显: bool /
///     扩展线段_当下分析: bool
///
///   [分析开关]
///     分析笔: bool / 分析线段: bool / 分析扩展线段: bool /
///     分析笔中枢: bool / 分析线段中枢: bool
///
///   [指标]
///     计算指标: bool / 指标计算方式: str
///     平滑异同移动平均线_快线周期: int / _慢线周期 / _信号周期
///     相对强弱指数_周期: int / _移动平均线周期 / _超买阈值 / _超卖阈值
///     随机指标_RSV周期: int / _K值平滑周期 / _D值平滑周期 / _超买阈值 / _超卖阈值
///
///   [推送/显示]
///     图表展示: bool / 推送K线: bool / 推送笔: bool / 推送线段: bool / 推送中枢: bool
///     图表展示_笔: bool / 图表展示_线段: bool / 图表展示_扩展线段: bool /
///     图表展示_中枢_笔: bool / 图表展示_中枢_线段: bool 等
///
///   [买卖点]
///     买卖点偏移: int / 买卖点激进识别: bool / 买卖点与MACD柱强相关: bool /
///     买卖点错过误差值: float / 买卖点_背离率: float /
///     买卖点_计算方式: str / 买卖点_中枢来源: str /
///     买卖点_指标模式: str / 买卖点_指标匹配_MACD: bool / _KDJ: bool / _RSI: bool /
///     买卖点_峰值条件: bool / 买卖点_依赖T1: bool /
///     买卖点_计算线段BSP1: bool / _处理BSP2: bool / _计算线段BSP3: bool
///
///   [背驰]
///     线段内部背驰_MACD: bool / 线段内部背驰_斜率: bool /
///     线段内部背驰_测度: bool / 线段内部背驰_模式: str
///
///   [其他]
///     手动终止: str — 手动终止时间（ISO 8601 字符串）
///     加载文件路径: str
///
/// 方法:
///   to_dict() -> dict — 导出为 Python 字典
///   to_json() -> str — 导出为 JSON 字符串
///   保存配置(path?) — 保存到 JSON 文件（默认 "缠论配置.json"）
///   加载配置(path?) -> 缠论配置 (classmethod) — 从 JSON 文件加载
///   from_dict(data) -> 缠论配置 (classmethod) — 从字典创建
///   from_json(json_str) -> 缠论配置 (classmethod) — 从 JSON 字符串创建
///   不推送() -> 缠论配置 (classmethod) — 创建关闭所有推送的配置副本
///   按序号重组字典(默认配置, 原始字典) -> dict (classmethod) — 按默认配置的键序重排字典
///   对比(other) -> dict — 返回与另一个配置的差异字段
#[pyclass(name = "缠论配置")]
pub struct 缠论配置Py {
    fields: HashMap<String, Py<PyAny>>,
}

#[pymethods]
impl 缠论配置Py {
    #[new]
    #[pyo3(signature = (**kwargs))]
    fn new(kwargs: Option<&Bound<'_, PyDict>>) -> PyResult<Self> {
        let default_config = chanlun::config::缠论配置::default();
        let mut fields = config_to_field_dict(&default_config)?;

        if let Some(kwargs) = kwargs {
            for (key, value) in kwargs.iter() {
                let key: String = key.extract()?;
                if fields.contains_key(&key) {
                    fields.insert(key, value.clone().unbind());
                } else {
                    return Err(pyo3::exceptions::PyAttributeError::new_err(format!(
                        "缠论配置 没有字段: {key}"
                    )));
                }
            }
        }

        Ok(Self { fields })
    }

    fn __getattr__(&self, name: &str, py: Python<'_>) -> PyResult<Py<PyAny>> {
        match self.fields.get(name) {
            Some(v) => Ok(v.clone_ref(py)),
            None => Err(pyo3::exceptions::PyAttributeError::new_err(format!(
                "缠论配置 没有字段: {name}"
            ))),
        }
    }

    fn __setattr__(&mut self, name: &str, value: &Bound<'_, PyAny>) -> PyResult<()> {
        if self.fields.contains_key(name) {
            self.fields.insert(name.to_string(), value.clone().unbind());
            Ok(())
        } else {
            Err(pyo3::exceptions::PyAttributeError::new_err(format!(
                "缠论配置 没有字段: {name}"
            )))
        }
    }

    fn __dir__(&self) -> Vec<String> {
        let mut names: Vec<String> = self.fields.keys().cloned().collect();
        names.sort();
        names
    }

    fn __str__(&self) -> String {
        format!("缠论配置({} fields)", self.fields.len())
    }

    fn __repr__(&self) -> String {
        self.__str__()
    }

    /// 将配置导出为 Python 字典。
    fn to_dict(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let dict = PyDict::new(py);
        for (k, v) in &self.fields {
            dict.set_item(k, v.clone_ref(py))?;
        }
        Ok(dict.into())
    }

    /// 将配置序列化为 JSON 字符串。
    fn to_json(&self, py: Python<'_>) -> PyResult<String> {
        let dict = self.to_dict(py)?;
        let json_mod = py.import("json")?;
        let dumps = json_mod.getattr("dumps")?;
        dumps.call1((dict,))?.extract()
    }

    /// 保存配置到 JSON 文件（默认路径 "缠论配置.json"）。
    fn 保存配置(&self, py: Python<'_>, path: Option<&str>) -> PyResult<()> {
        let path = path.unwrap_or("缠论配置.json");
        let json = self.to_json(py)?;
        std::fs::write(path, json).map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))
    }

    /// 从 JSON 文件加载配置（默认路径 "缠论配置.json"）。
    #[classmethod]
    fn 加载配置(
        _cls: &Bound<'_, PyType>,
        py: Python<'_>,
        path: Option<&str>,
    ) -> PyResult<Self> {
        let path = path.unwrap_or("缠论配置.json");
        let json_str = std::fs::read_to_string(path)
            .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))?;
        Self::from_json_str(py, &json_str)
    }

    #[classmethod]
    fn from_dict(_cls: &Bound<'_, PyType>, data: &Bound<'_, PyDict>) -> PyResult<Self> {
        let default_config = chanlun::config::缠论配置::default();
        let mut fields = config_to_field_dict(&default_config)?;

        for (key, value) in data.iter() {
            let key: String = key.extract()?;
            fields.insert(key, value.clone().unbind());
        }

        dict_to_rust_config(&fields)?;
        Ok(Self { fields })
    }

    #[classmethod]
    fn from_json(_cls: &Bound<'_, PyType>, py: Python<'_>, json_str: &str) -> PyResult<Self> {
        Self::from_json_str(py, json_str)
    }

    #[classmethod]
    fn 不推送(_cls: &Bound<'_, PyType>) -> PyResult<Self> {
        let config = chanlun::config::缠论配置::default().不推送();
        let fields = config_to_field_dict(&config)?;
        Ok(Self { fields })
    }

    #[classmethod]
    fn 按序号重组字典(
        _cls: &Bound<'_, PyType>,
        默认配置: &Bound<'_, PyAny>,
        原始字典: &Bound<'_, PyDict>,
    ) -> PyResult<Py<PyDict>> {
        let py = 原始字典.py();
        let result = PyDict::new(py);
        if let Ok(default_dict) = 默认配置.downcast::<PyDict>() {
            for (key, value) in default_dict.iter() {
                if 原始字典.contains(&key)? {
                    result.set_item(key.clone(), 原始字典.get_item(&key)?)?;
                } else {
                    result.set_item(key, value)?;
                }
            }
        }
        Ok(result.into())
    }

    fn 对比(
        &self,
        py: Python<'_>,
        other: &Bound<'_, 缠论配置Py>,
    ) -> PyResult<HashMap<String, (Py<PyAny>, Py<PyAny>)>> {
        let other_ref = other.borrow();
        let mut diff = HashMap::new();
        for (key, val) in &self.fields {
            if let Some(other_val) = other_ref.fields.get(key) {
                let a = val.clone_ref(py);
                let b = other_val.clone_ref(py);
                let eq = a.bind(py).eq(b.bind(py))?;
                if !eq {
                    diff.insert(key.clone(), (val.clone_ref(py), other_val.clone_ref(py)));
                }
            }
        }
        Ok(diff)
    }
}

impl 缠论配置Py {
    fn from_json_str(py: Python<'_>, json_str: &str) -> PyResult<Self> {
        let json_mod = py.import("json")?;
        let loads = json_mod.getattr("loads")?;
        let data: Bound<'_, PyDict> = loads.call1((json_str,))?.extract()?;

        let mut fields: HashMap<String, Py<PyAny>> = HashMap::new();
        for (key, value) in data.iter() {
            let key: String = key.extract()?;
            fields.insert(key, value.clone().unbind());
        }

        dict_to_rust_config(&fields)?;
        Ok(Self { fields })
    }

    pub(crate) fn to_rust_config(&self, py: Python<'_>) -> PyResult<chanlun::config::缠论配置> {
        dict_to_rust_config(&self.fields)
    }

    pub(crate) fn from_rust_config(config: &chanlun::config::缠论配置) -> PyResult<Self> {
        config_to_field_dict(config).map(|fields| Self { fields })
    }
}

fn config_to_field_dict(
    config: &chanlun::config::缠论配置,
) -> PyResult<HashMap<String, Py<PyAny>>> {
    Python::attach(|py| {
        let json_str = serde_json::to_string(config)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        let json_mod = py.import("json")?;
        let loads = json_mod.getattr("loads")?;
        let data: Bound<'_, PyDict> = loads.call1((json_str,))?.extract()?;

        let mut fields = HashMap::new();
        for (key, value) in data.iter() {
            let key: String = key.extract()?;
            fields.insert(key, value.clone().unbind());
        }
        Ok(fields)
    })
}

fn dict_to_rust_config(
    fields: &HashMap<String, Py<PyAny>>,
) -> PyResult<chanlun::config::缠论配置> {
    Python::attach(|py| {
        let json_mod = py.import("json")?;
        let dict = PyDict::new(py);
        for (k, v) in fields {
            dict.set_item(k, v.clone_ref(py))?;
        }
        let dumps = json_mod.getattr("dumps")?;
        let json_str: String = dumps.call1((dict,))?.extract()?;
        serde_json::from_str(&json_str)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("配置转换失败: {e}")))
    })
}

pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<缠论配置Py>()?;
    Ok(())
}
