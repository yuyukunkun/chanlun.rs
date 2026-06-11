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

use chanlun::warn;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyType};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

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
#[pyclass(name = "缠论配置", module = "chanlun._chanlun")]
pub struct 缠论配置Py {
    fields: HashMap<String, Py<PyAny>>,
    缓存: parking_lot::Mutex<Option<chanlun::config::缠论配置>>,
    pub(crate) 版本: AtomicU64,
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
                if !fields.contains_key(&key) {
                    return Err(pyo3::exceptions::PyAttributeError::new_err(format!(
                        "缠论配置 没有字段: {key}"
                    )));
                }
                fields.insert(key, value.clone().unbind());
            }
        }

        // 全部通过 serde_json 往返验证类型，统一处理字符串数字/布尔强制转换
        let config = dict_to_rust_config(&fields)?;
        let fields = config_to_field_dict(&config)?;
        Ok(Self {
            fields,
            缓存: parking_lot::Mutex::new(Some(config)),
            版本: AtomicU64::new(1),
        })
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
            *self.缓存.lock() = None;
            self.版本.fetch_add(1, Ordering::Relaxed);
            // 通过 serde 往返验证类型
            match dict_to_rust_config(&self.fields) {
                Ok(config) => {
                    self.fields = config_to_field_dict(&config)?;
                    *self.缓存.lock() = Some(config);
                    Ok(())
                }
                Err(e) => Err(pyo3::exceptions::PyValueError::new_err(format!(
                    "配置转换失败: {e}"
                ))),
            }
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
        let valid = chanlun::config::缠论配置::model_fields();
        for (k, v) in &self.fields {
            if valid.contains(&k.as_str()) {
                dict.set_item(k, v.clone_ref(py))?;
            }
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
    #[pyo3(signature = (path = "缠论配置.json"))]
    fn 保存配置(&self, py: Python<'_>, path: &str) -> PyResult<()> {
        let json = self.to_json(py)?;
        std::fs::write(path, json).map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))
    }

    /// 从 JSON 文件加载配置（默认路径 "缠论配置.json"）。
    #[classmethod]
    #[pyo3(signature = (path = "缠论配置.json"))]
    fn 加载配置(_cls: &Bound<'_, PyType>, py: Python<'_>, path: &str) -> PyResult<Self> {
        let json_str = std::fs::read_to_string(path)
            .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))?;
        Self::from_json_str(py, &json_str)
    }

    #[classmethod]
    /// :param data: 字典数据
    fn from_dict(_cls: &Bound<'_, PyType>, data: &Bound<'_, PyDict>) -> PyResult<Self> {
        let default_config = chanlun::config::缠论配置::default();
        let mut fields = config_to_field_dict(&default_config)?;

        for (key, value) in data.iter() {
            let key: String = key.extract()?;
            if fields.contains_key(&key) {
                fields.insert(key, value.clone().unbind());
            }
        }

        let config = dict_to_rust_config(&fields)?;
        let fields = config_to_field_dict(&config)?;
        Ok(Self {
            fields,
            缓存: parking_lot::Mutex::new(Some(config)),
            版本: AtomicU64::new(1),
        })
    }

    #[classmethod]
    /// :param json_str: JSON字符串
    fn from_json(_cls: &Bound<'_, PyType>, py: Python<'_>, json_str: &str) -> PyResult<Self> {
        Self::from_json_str(py, json_str)
    }

    #[classmethod]
    /// 创建不推送任何图表的静默配置（用于纯计算场景）
    fn 不推送(_cls: &Bound<'_, PyType>) -> PyResult<Self> {
        let config = chanlun::config::缠论配置::default().不推送();
        let fields = config_to_field_dict(&config)?;
        Ok(Self {
            fields,
            缓存: parking_lot::Mutex::new(Some(config)),
            版本: AtomicU64::new(1),
        })
    }

    #[classmethod]
    /// 将形如 "1_open", "1_close", "2_open", "name" 的字典重组为嵌套结构
    fn 按序号重组字典(
        _cls: &Bound<'_, PyType>,
        默认配置: &Bound<'_, PyAny>,
        原始字典: &Bound<'_, PyDict>,
    ) -> PyResult<Py<PyDict>> {
        let py = 原始字典.py();
        let result = PyDict::new(py);
        if let Ok(default_dict) = 默认配置.cast::<PyDict>() {
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

    /// 创建当前配置的拷贝并可选择更新字段（对应 Python model_copy(update={...}, deep=True)）
    #[pyo3(signature = (update = None))]
    fn model_copy(&self, py: Python<'_>, update: Option<&Bound<'_, PyDict>>) -> PyResult<Self> {
        let current = self.to_dict(py)?;
        if let Some(updates) = update {
            for (key, value) in updates.iter() {
                current.bind(py).set_item(key, value)?;
            }
        }
        Self::from_dict(&py.get_type::<Self>(), current.bind(py))
    }

    /// 比较当前配置与另一个配置的差异（对应 Python 对比 → dict[字段名, 新值]）
    fn 对比(&self, py: Python<'_>, other: &Bound<'_, 缠论配置Py>) -> PyResult<Py<PyAny>> {
        let other_ref = other.borrow();
        let dict = PyDict::new(py);
        let valid = chanlun::config::缠论配置::model_fields();
        for key in valid {
            if let (Some(self_val), Some(other_val)) =
                (self.fields.get(*key), other_ref.fields.get(*key))
            {
                let a = self_val.clone_ref(py);
                let b = other_val.clone_ref(py);
                let eq = a.bind(py).eq(b.bind(py))?;
                if !eq {
                    dict.set_item(*key, b)?;
                }
            }
        }
        Ok(dict.into())
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

        let config = dict_to_rust_config(&fields)?;
        let fields = config_to_field_dict(&config)?;
        Ok(Self {
            fields,
            缓存: parking_lot::Mutex::new(Some(config)),
            版本: AtomicU64::new(1),
        })
    }

    pub(crate) fn to_rust_config(
        &self,
        _py: Python<'_>,
    ) -> PyResult<chanlun::config::缠论配置> {
        if let Some(ref cached) = *self.缓存.lock() {
            return Ok(cached.clone());
        }
        let config = dict_to_rust_config(&self.fields)?;
        *self.缓存.lock() = Some(config.clone());
        Ok(config)
    }

    pub(crate) fn from_rust_config(config: &chanlun::config::缠论配置) -> PyResult<Self> {
        config_to_field_dict(config).map(|fields| Self {
            fields,
            缓存: parking_lot::Mutex::new(Some(config.clone())),
            版本: AtomicU64::new(1),
        })
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
        let mut value: serde_json::Value = serde_json::from_str(&json_str)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("配置转换失败: {e}")))?;
        coerce_strings_to_numbers(&mut value);

        // 获取默认配置的 JSON 表示作为 schema
        let default_config = chanlun::config::缠论配置::default();
        let default_json = serde_json::to_value(&default_config)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("配置转换失败: {e}")))?;

        // 用默认值做基准，只合并类型匹配的字段
        let mut merged = default_json.clone();
        if let serde_json::Value::Object(ref input_map) = value
            && let serde_json::Value::Object(ref default_map) = default_json
        {
            for (key, input_val) in input_map {
                if let Some(default_val) = default_map.get(key) {
                    match validate_field(key, input_val, default_val) {
                        Ok(()) => {
                            merged[key] = input_val.clone();
                        }
                        Err(msg) => {
                            warn!("[配置警告] {key}: {msg}，已使用默认值 {default_val}");
                        }
                    }
                }
            }
        }

        serde_json::from_value(merged)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("配置转换失败: {e}")))
    })
}

/// 字符串字段的有效值集合
fn valid_string_values(field: &str) -> Option<&'static [&'static str]> {
    match field {
        "指标计算方式" => Some(&[
            "开",
            "高",
            "低",
            "收",
            "高低均值",
            "高低收均值",
            "开高低收均值",
        ]),
        "买卖点_指标模式" => Some(&["任意", "配置", "全量", "相对"]),
        "线段内部背驰_模式" => Some(&["任意", "配置", "全量", "相对"]),
        _ => None,
    }
}

/// 验证单个字段的值是否与默认值类型兼容
fn validate_field(
    key: &str,
    input: &serde_json::Value,
    default: &serde_json::Value,
) -> Result<(), String> {
    use serde_json::Value;

    // 输入为 null → 保留（对应 Optional/Infinity 字段）
    if input.is_null() {
        return Ok(());
    }

    // 字符串字段：检查有效值白名单
    if default.is_string() && input.is_string() {
        if let Some(valid) = valid_string_values(key) {
            let s = input.as_str().unwrap();
            if !valid.contains(&s) {
                return Err(format!("\"{s}\" 不在有效值 {valid:?} 内"));
            }
        }
        return Ok(());
    }

    // 类型匹配：直接通过
    match (default, input) {
        (Value::Bool(_), Value::Bool(_)) => return Ok(()),
        (Value::Number(_), Value::Number(_)) => return Ok(()),
        (Value::String(_), Value::String(_)) => return Ok(()),
        (Value::Array(_), Value::Array(_)) => return Ok(()),
        (Value::Object(_), Value::Object(_)) => return Ok(()),
        _ => {}
    }

    // 类型不匹配
    let type_name = match input {
        Value::Bool(_) => "布尔",
        Value::Number(_) => "数值",
        Value::String(_) => "字符串",
        Value::Array(_) => "数组",
        Value::Object(_) => "字典",
        Value::Null => "null",
    };
    let expected = match default {
        Value::Bool(_) => "布尔",
        Value::Number(_) => "数值",
        Value::String(_) => "字符串",
        Value::Array(_) => "数组",
        Value::Object(_) => "字典",
        Value::Null => "null",
    };
    Err(format!("类型不匹配（需要 {expected}，收到 {type_name}）"))
}

/// 递归遍历 JSON Value，将数字/布尔字符串转为对应类型。
fn coerce_strings_to_numbers(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            for (_, v) in map.iter_mut() {
                coerce_strings_to_numbers(v);
            }
        }
        serde_json::Value::Array(arr) => {
            for v in arr.iter_mut() {
                coerce_strings_to_numbers(v);
            }
        }
        serde_json::Value::String(s) => {
            // 先 clone 出独立副本，避免借用冲突
            let cloned = s.clone();
            if let Ok(n) = cloned.parse::<i64>() {
                *value = serde_json::Value::Number(serde_json::Number::from(n));
            } else if let Ok(n) = cloned.parse::<f64>() {
                if n.is_finite()
                    && let Some(num) = serde_json::Number::from_f64(n)
                {
                    *value = serde_json::Value::Number(num);
                }
            } else if cloned.eq_ignore_ascii_case("true") {
                *value = serde_json::Value::Bool(true);
            } else if cloned.eq_ignore_ascii_case("false") {
                *value = serde_json::Value::Bool(false);
            }
            // 非数字非布尔的原样保留，不做任何修改
        }
        _ => {}
    }
}

/// 将 Python 字符串数字/布尔值强制转为对应类型，非字符串保持不变。
/// 注：主代码路径现通过 serde 往返处理类型转换，此函数作为辅助保留。
#[allow(dead_code)]
fn coerce_py_value(value: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
    let py = value.py();

    let type_name: String = value.get_type().name()?.extract()?;
    if type_name != "str" {
        return Ok(value.clone().unbind());
    }

    let lower_obj = value.call_method0("lower")?;
    let lower: String = lower_obj.extract()?;

    if lower == "true" || lower == "false" {
        let b = lower == "true";
        let obj = pyo3::types::PyBool::new(py, b)
            .to_owned()
            .into_any()
            .unbind();
        return Ok(obj);
    }

    if let Ok(n) = lower.parse::<i64>() {
        return Ok(n.into_pyobject(py)?.into_any().unbind());
    }
    if let Ok(n) = lower.parse::<f64>()
        && n.is_finite()
    {
        return Ok(n.into_pyobject(py)?.into_any().unbind());
    }

    Ok(value.clone().unbind())
}

pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<缠论配置Py>()?;
    Ok(())
}
