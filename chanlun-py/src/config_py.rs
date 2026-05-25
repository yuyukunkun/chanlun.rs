use pyo3::prelude::*;
use pyo3::types::{PyDict, PyType};
use std::collections::HashMap;

/// 缠论配置 — Python binding
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

    fn __str__(&self) -> String {
        format!("缠论配置({} fields)", self.fields.len())
    }

    fn __repr__(&self) -> String {
        self.__str__()
    }

    fn to_dict(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let dict = PyDict::new(py);
        for (k, v) in &self.fields {
            dict.set_item(k, v.clone_ref(py))?;
        }
        Ok(dict.into())
    }

    fn to_json(&self, py: Python<'_>) -> PyResult<String> {
        let dict = self.to_dict(py)?;
        let json_mod = py.import("json")?;
        let dumps = json_mod.getattr("dumps")?;
        dumps.call1((dict,))?.extract()
    }

    fn 保存配置(&self, py: Python<'_>, path: Option<&str>) -> PyResult<()> {
        let path = path.unwrap_or("缠论配置.json");
        let json = self.to_json(py)?;
        std::fs::write(path, json).map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))
    }

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
