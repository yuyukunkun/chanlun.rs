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

// ========== 买卖点类型 ==========

/// 买卖点类型 — 缠论的三类买卖点及扩展类型。
///
/// 类属性（可作为常量直接使用）:
///   一买, 一卖, 二买, 二卖, 三买, 三卖
///   T1买, T1卖, T1P买, T1P卖, T2买, T2卖,
///   T2S买, T2S卖, T3A买, T3A卖, T3B买, T3B卖
///
/// 属性:
///   是买点: bool — 是否为买入类型
///   是卖点: bool — 是否为卖出类型
#[pyclass(name = "买卖点类型")]
#[derive(Clone)]
pub struct 买卖点类型Py {
    pub inner: chanlun::types::买卖点类型,
}

#[pymethods]
impl 买卖点类型Py {
    fn __str__(&self) -> String {
        self.inner.to_string()
    }

    fn __repr__(&self) -> String {
        self.inner.to_string()
    }

    fn __eq__(&self, other: &Bound<'_, PyAny>) -> bool {
        if let Ok(s) = other.extract::<String>() {
            return self.inner.to_string() == s;
        }
        if let Ok(other) = other.extract::<PyRef<'_, Self>>() {
            return self.inner == other.inner;
        }
        false
    }

    fn __hash__(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        self.inner.to_string().hash(&mut h);
        h.finish()
    }

    #[getter]
    fn 是买点(&self) -> bool {
        self.inner.是买点()
    }

    #[getter]
    fn 是卖点(&self) -> bool {
        self.inner.是卖点()
    }
}

// ========== 相对方向 ==========

/// 相对方向 — 描述两个K线/分型之间相对位置关系的枚举。
///
/// 用于包含处理、分型判定、中枢重叠检测等。不代表绝对涨跌方向，
/// 而是描述前后两个价格区间的包含/位置关系。
///
/// 类属性（可作为常量直接使用）:
///   向上, 向下, 向上缺口, 向下缺口, 衔接向上, 衔接向下, 顺, 逆, 同
///
/// 属性:
///   是否向上: bool — 当前方向是否归类为"向上"（含衔接向上）
///   是否向下: bool — 当前方向是否归类为"向下"（含衔接向下）
///   是否包含: bool — 前后区间是否存在包含关系
///   是否缺口: bool — 前后区间之间是否存在缺口
///   是否衔接: bool — 前后区间是否恰好衔接
///
/// 方法:
///   翻转() -> 相对方向 — 返回方向的对立面（向上↔向下, 缺口↔反向缺口）
///   分析(前高, 前低, 后高, 后低) -> 相对方向 (classmethod) — 根据价格区间判断方向
#[pyclass(name = "相对方向")]
#[derive(Clone)]
pub struct 相对方向Py {
    pub inner: chanlun::types::相对方向,
}

#[pymethods]
impl 相对方向Py {
    fn __str__(&self) -> String {
        format!("{}", self.inner)
    }

    fn __repr__(&self) -> String {
        format!("{}", self.inner)
    }

    fn __eq__(&self, other: &Bound<'_, PyAny>) -> bool {
        if let Ok(other) = other.extract::<PyRef<'_, Self>>() {
            return self.inner == other.inner;
        }
        false
    }

    fn __hash__(&self) -> u64 {
        self.inner as u64
    }

    /// 返回方向的对立面（向上↔向下, 缺口↔反向缺口, 衔接↔反向衔接）。
    fn 翻转(&self) -> Self {
        Self {
            inner: self.inner.翻转(),
        }
    }

    fn 是否向上(&self) -> bool {
        self.inner.是否向上()
    }

    fn 是否向下(&self) -> bool {
        self.inner.是否向下()
    }

    fn 是否包含(&self) -> bool {
        self.inner.是否包含()
    }

    fn 是否缺口(&self) -> bool {
        self.inner.是否缺口()
    }

    fn 是否衔接(&self) -> bool {
        self.inner.是否衔接()
    }

    /// 根据前后价格区间的OHLC值分析方向关系。
    ///
    /// 参数:
    ///   前高: float — 前一个区间的高点
    ///   前低: float — 前一个区间的低点
    ///   后高: float — 后一个区间的高点
    ///   后低: float — 后一个区间的低点
    /// 返回:
    ///   相对方向 — 两个区间的相对位置关系
    #[classmethod]
    fn 分析(
        _cls: &Bound<'_, PyType>, 前高: f64, 前低: f64, 后高: f64, 后低: f64
    ) -> Self {
        Self {
            inner: chanlun::types::相对方向::分析(前高, 前低, 后高, 后低),
        }
    }
}

// ========== 分型结构 ==========

/// 分型结构 — 描述三根K线构成的顶底分型形态。
///
/// 类属性（可作为常量直接使用）:
///   上 — 向上结构（两根K线均向上）
///   下 — 向下结构（两根K线均向下）
///   顶 — 顶分型（先上后下）
///   底 — 底分型（先下后上）
///   散 — 松散结构（逆序包含）
///
/// 方法:
///   分析(左, 中, 右, 可以逆序包含?, 忽视顺序包含?) -> 分型结构|None (classmethod)
///      — 根据三根K线的高低价分析分型结构
#[pyclass(name = "分型结构")]
#[derive(Clone)]
pub struct 分型结构Py {
    pub inner: chanlun::types::分型结构,
}

#[pymethods]
impl 分型结构Py {
    fn __str__(&self) -> String {
        self.inner.to_string()
    }

    fn __repr__(&self) -> String {
        self.inner.to_string()
    }

    fn __eq__(&self, other: &Bound<'_, PyAny>) -> bool {
        if let Ok(other) = other.extract::<PyRef<'_, Self>>() {
            return self.inner == other.inner;
        }
        false
    }

    fn __hash__(&self) -> u64 {
        self.inner as u64
    }

    /// 根据左中右三根K线的高/低价分析分型结构。
    ///
    /// 参数:
    ///   左: object — 左侧K线，需有 高/低 属性
    ///   中: object — 中间K线，需有 高/低 属性
    ///   右: object — 右侧K线，需有 高/低 属性
    ///   可以逆序包含: bool — 是否允许逆序包含判定
    ///   忽视顺序包含: bool — 是否忽略顺序包含（K线合并后可能产生）
    /// 返回:
    ///   分型结构 或 None — 无法判定时返回 None
    #[classmethod]
    fn 分析(
        _cls: &Bound<'_, PyType>,
        左: &Bound<'_, PyAny>,
        中: &Bound<'_, PyAny>,
        右: &Bound<'_, PyAny>,
        可以逆序包含: Option<bool>,
        忽视顺序包含: Option<bool>,
    ) -> PyResult<Option<Self>> {
        let 可以逆序包含 = 可以逆序包含.unwrap_or(false);
        let 忽视顺序包含 = 忽视顺序包含.unwrap_or(false);

        let get_hl = |obj: &Bound<'_, PyAny>| -> PyResult<(f64, f64)> {
            Ok((
                obj.getattr("高")?.extract::<f64>()?,
                obj.getattr("低")?.extract::<f64>()?,
            ))
        };

        let (左高, 左低) = get_hl(左)?;
        let (中高, 中低) = get_hl(中)?;
        let (右高, 右低) = get_hl(右)?;

        let 左中关系 = chanlun::types::相对方向::分析(左高, 左低, 中高, 中低);
        let 中右关系 = chanlun::types::相对方向::分析(中高, 中低, 右高, 右低);

        let 向上类 = |d: chanlun::types::相对方向| d.是否向上();
        let 向下类 = |d: chanlun::types::相对方向| d.是否向下();

        let result = match (左中关系, 中右关系) {
            (d1, d2) if matches!(d1, chanlun::types::相对方向::顺) && !忽视顺序包含 => {
                panic!("顺序包含: {:?} {:?}", d1, d2);
            }
            (d1, d2) if matches!(d2, chanlun::types::相对方向::顺) && !忽视顺序包含 => {
                panic!("顺序包含: {:?} {:?}", d1, d2);
            }
            (a, b) if 向上类(a) && 向上类(b) => chanlun::types::分型结构::上,
            (a, b) if 向上类(a) && 向下类(b) => chanlun::types::分型结构::顶,
            (a, chanlun::types::相对方向::逆) if 向上类(a) && 可以逆序包含 => {
                chanlun::types::分型结构::上
            }
            (a, b) if 向下类(a) && 向上类(b) => chanlun::types::分型结构::底,
            (a, b) if 向下类(a) && 向下类(b) => chanlun::types::分型结构::下,
            (a, chanlun::types::相对方向::逆) if 向下类(a) && 可以逆序包含 => {
                chanlun::types::分型结构::下
            }
            (chanlun::types::相对方向::逆, a) if 向上类(a) && 可以逆序包含 => {
                chanlun::types::分型结构::底
            }
            (chanlun::types::相对方向::逆, a) if 向下类(a) && 可以逆序包含 => {
                chanlun::types::分型结构::顶
            }
            (chanlun::types::相对方向::逆, chanlun::types::相对方向::逆) if 可以逆序包含 => {
                chanlun::types::分型结构::散
            }
            _ => return Ok(None),
        };
        Ok(Some(Self { inner: result }))
    }
}

// ========== 缺口 ==========

/// 缺口 — 描述价格区间之间的缺口（未重叠部分）。
///
/// 属性:
///   高: float — 缺口上沿价格（可读写）
///   低: float — 缺口下沿价格（可读写）
///
/// 方法:
///   居中截取区间(起点, 终点, 比例=0.15) -> 缺口|None (classmethod)
///     — 在两个价格之间截取中间区域作为缺口
#[pyclass(name = "缺口")]
#[derive(Clone)]
pub struct 缺口Py {
    pub inner: chanlun::types::缺口,
}

#[pymethods]
impl 缺口Py {
    #[new]
    fn new(高: f64, 低: f64) -> PyResult<Self> {
        if 高 <= 低 {
            return Err(pyo3::exceptions::PyValueError::new_err(format!(
                "缺口高必须大于低: 高={高}, 低={低}"
            )));
        }
        Ok(Self {
            inner: chanlun::types::缺口::new(高, 低),
        })
    }

    fn __str__(&self) -> String {
        format!("{}", self.inner)
    }

    fn __repr__(&self) -> String {
        format!("{}", self.inner)
    }

    #[getter]
    #[pyo3(name = "高")]
    fn get_高(&self) -> f64 {
        self.inner.高
    }

    #[setter]
    #[pyo3(name = "高")]
    fn set_高(&mut self, value: f64) {
        self.inner.高 = value;
    }

    #[getter]
    #[pyo3(name = "低")]
    fn get_低(&self) -> f64 {
        self.inner.低
    }

    #[setter]
    #[pyo3(name = "低")]
    fn set_低(&mut self, value: f64) {
        self.inner.低 = value;
    }

    /// 在两个价格之间截取中间区域作为缺口。
    ///
    /// 参数:
    ///   起点: float — 起始价格
    ///   终点: float — 终止价格
    ///   比例: float — 截取区间占总区间的比例（默认 0.15）
    /// 返回:
    ///   缺口 或 None — 起点==终点时返回 None
    #[classmethod]
    fn 居中截取区间(
        _cls: &Bound<'_, PyType>,
        起点: f64,
        终点: f64,
        比例: Option<f64>,
    ) -> Option<Self> {
        let 比例 = 比例.unwrap_or(0.15);
        chanlun::types::缺口::居中截取区间(起点, 终点, 比例).map(|inner| Self { inner })
    }
}

// ========== Registration ==========

pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Register classes
    m.add_class::<买卖点类型Py>()?;
    m.add_class::<相对方向Py>()?;
    m.add_class::<分型结构Py>()?;
    m.add_class::<缺口Py>()?;

    // 买卖点类型 class attributes (singleton instances)
    let py = m.py();
    let bsp_class = m.getattr("买卖点类型")?;
    let bsp_class = bsp_class.downcast_into::<PyType>()?;

    let variants: &[(&str, chanlun::types::买卖点类型)] = &[
        ("一买", chanlun::types::买卖点类型::一买),
        ("一卖", chanlun::types::买卖点类型::一卖),
        ("二买", chanlun::types::买卖点类型::二买),
        ("二卖", chanlun::types::买卖点类型::二卖),
        ("三买", chanlun::types::买卖点类型::三买),
        ("三卖", chanlun::types::买卖点类型::三卖),
        ("T1买", chanlun::types::买卖点类型::T1买),
        ("T1卖", chanlun::types::买卖点类型::T1卖),
        ("T1P买", chanlun::types::买卖点类型::T1P买),
        ("T1P卖", chanlun::types::买卖点类型::T1P卖),
        ("T2买", chanlun::types::买卖点类型::T2买),
        ("T2卖", chanlun::types::买卖点类型::T2卖),
        ("T2S买", chanlun::types::买卖点类型::T2S买),
        ("T2S卖", chanlun::types::买卖点类型::T2S卖),
        ("T3A买", chanlun::types::买卖点类型::T3A买),
        ("T3A卖", chanlun::types::买卖点类型::T3A卖),
        ("T3B买", chanlun::types::买卖点类型::T3B买),
        ("T3B卖", chanlun::types::买卖点类型::T3B卖),
    ];

    for (name, value) in variants {
        let instance = Py::new(py, 买卖点类型Py { inner: *value })?;
        bsp_class.setattr(*name, instance)?;
    }

    // 相对方向 class attributes
    let dir_class = m.getattr("相对方向")?.downcast_into::<PyType>()?.clone();
    let dir_variants: &[(&str, chanlun::types::相对方向)] = &[
        ("向上", chanlun::types::相对方向::向上),
        ("向下", chanlun::types::相对方向::向下),
        ("向上缺口", chanlun::types::相对方向::向上缺口),
        ("向下缺口", chanlun::types::相对方向::向下缺口),
        ("衔接向上", chanlun::types::相对方向::衔接向上),
        ("衔接向下", chanlun::types::相对方向::衔接向下),
        ("顺", chanlun::types::相对方向::顺),
        ("逆", chanlun::types::相对方向::逆),
        ("同", chanlun::types::相对方向::同),
    ];

    for (name, value) in dir_variants {
        let instance = Py::new(py, 相对方向Py { inner: *value })?;
        dir_class.setattr(*name, instance)?;
    }

    // 分型结构 class attributes
    let frac_class = m.getattr("分型结构")?.downcast_into::<PyType>()?.clone();
    let frac_variants: &[(&str, chanlun::types::分型结构)] = &[
        ("上", chanlun::types::分型结构::上),
        ("下", chanlun::types::分型结构::下),
        ("顶", chanlun::types::分型结构::顶),
        ("底", chanlun::types::分型结构::底),
        ("散", chanlun::types::分型结构::散),
    ];

    for (name, value) in frac_variants {
        let instance = Py::new(py, 分型结构Py { inner: *value })?;
        frac_class.setattr(*name, instance)?;
    }

    // Register module-level functions
    m.add_function(wrap_pyfunction!(转化为时间戳, m)?)?;
    m.add_function(wrap_pyfunction!(转化为时间戳_数字, m)?)?;

    Ok(())
}

// ========== Module-level functions ==========

/// 将多种类型的时间表示转换为 Unix 时间戳（秒）。
///
/// 支持输入类型:
///   int — 直接返回
///   float — 取整数部分
///   str — ISO 8601 ("2024-10-15T16:45:00") 或 "YYYY-MM-DD HH:MM:SS" 或 "YYYY-MM-DD"
///   datetime — 调用其 .timestamp() 方法
#[pyfunction]
fn 转化为时间戳(ts: &Bound<'_, PyAny>) -> PyResult<i64> {
    // Try int first
    if let Ok(v) = ts.extract::<i64>() {
        return Ok(v);
    }
    // Try float
    if let Ok(v) = ts.extract::<f64>() {
        return Ok(v as i64);
    }
    // Try string (ISO 8601 or "YYYY-MM-DD HH:MM:SS")
    if let Ok(s) = ts.extract::<String>() {
        return parse_datetime_string(&s);
    }
    // Try datetime object
    if let Ok(dt) = ts.getattr("timestamp") {
        let ts_float: f64 = dt.call0()?.extract()?;
        return Ok(ts_float as i64);
    }
    Err(pyo3::exceptions::PyValueError::new_err(format!(
        "无法转化为时间戳: {:?}",
        ts
    )))
}

#[pyfunction]
fn 转化为时间戳_数字(ts: &Bound<'_, PyAny>) -> PyResult<i64> {
    转化为时间戳(ts)
}

fn parse_datetime_string(s: &str) -> PyResult<i64> {
    // Try ISO 8601: "2024-10-15T16:45:00"
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S") {
        return Ok(dt.and_utc().timestamp());
    }
    // Try "YYYY-MM-DD HH:MM:SS"
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
        return Ok(dt.and_utc().timestamp());
    }
    // Try "YYYY-MM-DD"
    if let Ok(d) = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        let dt = d
            .and_hms_opt(0, 0, 0)
            .ok_or_else(|| pyo3::exceptions::PyValueError::new_err(format!("无法解析日期: {s}")))?;
        return Ok(dt.and_utc().timestamp());
    }
    Err(pyo3::exceptions::PyValueError::new_err(format!(
        "无法解析时间字符串: {s}"
    )))
}
