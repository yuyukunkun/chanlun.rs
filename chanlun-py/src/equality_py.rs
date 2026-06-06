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

use std::num::NonZeroUsize;
use std::sync::Mutex;

use lru::LruCache;
use pyo3::prelude::*;

/// 缓存辅助宏：在调用点创建静态 LruCache，先查后存
macro_rules! with_cache {
    ($cache:ident, $size:literal, $key_expr:expr, $compute:expr) => {{
        use std::sync::LazyLock;
        static $cache: LazyLock<Mutex<LruCache<(usize, usize, i64), (bool, String)>>> =
            LazyLock::new(|| Mutex::new(LruCache::new(NonZeroUsize::new($size).unwrap())));
        let key = $key_expr;
        if let Some(cached) = $cache.lock().unwrap().get(&key) {
            return Ok(cached.clone());
        }
        let result: PyResult<(bool, String)> = $compute;
        if let Ok(ref r) = result {
            $cache.lock().unwrap().put(key, r.clone());
        }
        result
    }};
}

/// 从 Python 值中提取时间戳（兼容 i64 和 datetime 两种类型）
fn 提取时间戳(val: &Bound<'_, PyAny>) -> PyResult<i64> {
    if let Ok(ts) = val.extract::<i64>() {
        return Ok(ts);
    }
    let ts_f: f64 = val.call_method0("timestamp")?.extract()?;
    Ok(ts_f as i64)
}

/// 从对象获取属性，依次尝试多个候选名
fn 获取属性任意<'a>(
    obj: &'a Bound<'_, PyAny>,
    候选名: &[&str],
) -> PyResult<Option<Bound<'a, PyAny>>> {
    for name in 候选名 {
        if obj.hasattr(name)? {
            return Ok(Some(obj.getattr(name)?));
        }
    }
    Ok(None)
}

/// 比较两个 Python 值是否为 float（容差比较）
fn 尝试浮点比较(
    valA: &Bound<'_, PyAny>,
    valB: &Bound<'_, PyAny>,
    容差: f64,
) -> Option<PyResult<(bool, String)>> {
    if let (Ok(a), Ok(b)) = (valA.extract::<f64>(), valB.extract::<f64>()) {
        if (a - b).abs() > 容差 {
            return Some(Ok((
                false,
                format!("浮点超限 容差={:.2e} A={:.10},B={:.10}", 容差, a, b),
            )));
        }
        return Some(Ok((true, String::new())));
    }
    None
}

/// 尝试从对象获取 `标识` 字段，失败返回空字符串
fn 尝试获取标识(obj: &Bound<'_, PyAny>) -> String {
    if let Ok(val) = obj.getattr("标识")
        && let Ok(py_str) = val.str()
    {
        return py_str.extract::<String>().unwrap_or_default();
    }
    String::new()
}

/// None 检查辅助：双方为 None 返回 true，单方为 None 返回 false+消息
fn 检查空值一致(
    valA: &Bound<'_, PyAny>,
    valB: &Bound<'_, PyAny>,
    字段: &str,
    标签: &str,
) -> Option<(bool, String)> {
    let a_none = valA.is_none();
    let b_none = valB.is_none();
    if a_none && b_none {
        return Some((true, String::new()));
    }
    if a_none || b_none {
        return Some((
            false,
            format!("{标签}: [{字段}] 空值不一致 A=None={a_none},B=None={b_none}"),
        ));
    }
    None
}

// ========== K线相等 ==========

#[pyfunction]
#[pyo3(signature = (A, B, 浮点容差 = 1e-9))]
fn K线相等(
    A: &Bound<'_, PyAny>,
    B: &Bound<'_, PyAny>,
    浮点容差: f64,
) -> PyResult<(bool, String)> {
    with_cache!(
        C_KLINE,
        128,
        (
            A.as_ptr() as usize,
            B.as_ptr() as usize,
            浮点容差.to_bits() as i64
        ),
        {
            // 快速路径
            if let (Ok(a), Ok(b)) = (
                A.cast::<crate::kline_py::K线Py>(),
                B.cast::<crate::kline_py::K线Py>(),
            ) {
                return Ok(a.borrow().inner.相等(&b.borrow().inner, 浮点容差));
            }
            // 回退路径
            let 标签 = "K线校验";
            let 比对字段 = [
                "标识",
                "序号",
                "周期",
                "时间戳",
                "高",
                "低",
                "开盘价",
                "收盘价",
                "成交量",
            ];
            for &字段 in &比对字段 {
                let (a有, b有) = (A.hasattr(字段)?, B.hasattr(字段)?);
                if a有 && !b有 {
                    return Ok((false, format!("{标签}: [{字段}] A存在属性 B缺失属性")));
                }
                if !a有 && b有 {
                    return Ok((false, format!("{标签}: [{字段}] B存在属性 A缺失属性")));
                }
                if !a有 && !b有 {
                    continue;
                }
                let valA = A.getattr(字段)?;
                let valB = B.getattr(字段)?;
                if let Some(r) = 尝试浮点比较(&valA, &valB, 浮点容差) {
                    let (ok, m) = r?;
                    if !ok {
                        return Ok((false, format!("{标签}: [{字段}]{}", m)));
                    }
                } else if 字段 == "时间戳" {
                    let a = 提取时间戳(&valA).unwrap_or(0);
                    let b = 提取时间戳(&valB).unwrap_or(0);
                    if a != b {
                        return Ok((false, format!("{标签}: [{字段}] 数值不等 A={a},B={b}")));
                    }
                } else {
                    let eq: bool = valA.eq(&valB)?;
                    if !eq {
                        let ra = valA.repr()?.extract::<String>().unwrap_or_default();
                        let rb = valB.repr()?.extract::<String>().unwrap_or_default();
                        return Ok((false, format!("{标签}: [{字段}] 数值不等 A={ra},B={rb}")));
                    }
                }
            }
            Ok((true, format!("{标签}: 全部字段一致")))
        }
    )
}

// ========== 缠论K线相等 ==========

#[pyfunction]
#[pyo3(signature = (A, B, 浮点容差 = 1e-9))]
fn 缠论K线相等(
    A: &Bound<'_, PyAny>,
    B: &Bound<'_, PyAny>,
    浮点容差: f64,
) -> PyResult<(bool, String)> {
    with_cache!(
        C_CHAN_K,
        4096,
        (
            A.as_ptr() as usize,
            B.as_ptr() as usize,
            浮点容差.to_bits() as i64
        ),
        {
            if let (Ok(a), Ok(b)) = (
                A.cast::<crate::kline_py::缠论K线Py>(),
                B.cast::<crate::kline_py::缠论K线Py>(),
            ) {
                return Ok(a.borrow().inner.相等(&b.borrow().inner, 浮点容差));
            }
            let 标签 = "缠论K线校验";
            let 比对字段 = [
                "序号",
                "时间戳",
                "高",
                "低",
                "方向",
                "分型",
                "周期",
                "标识",
                "分型特征值",
                "原始起始序号",
                "原始结束序号",
                "标的K线",
                "买卖点信息",
            ];
            for &字段 in &比对字段 {
                let (a有, b有) = (A.hasattr(字段)?, B.hasattr(字段)?);
                if a有 && !b有 {
                    return Ok((false, format!("{标签}: [{字段}] A存在 B缺失属性")));
                }
                if !a有 && b有 {
                    return Ok((false, format!("{标签}: [{字段}] B存在 A缺失属性")));
                }
                if !a有 && !b有 {
                    continue;
                }
                let valA = A.getattr(字段)?;
                let valB = B.getattr(字段)?;

                if let Some(r) = 尝试浮点比较(&valA, &valB, 浮点容差) {
                    let (ok, m) = r?;
                    if !ok {
                        return Ok((false, format!("{标签}: [{字段}]{m}")));
                    }
                } else if 字段 == "标的K线" {
                    if let Some(r) = 检查空值一致(&valA, &valB, 字段, 标签) {
                        if !r.0 {
                            return Ok((false, r.1));
                        } else {
                            continue;
                        }
                    }
                    let (eq, msg) = K线相等(&valA, &valB, 浮点容差)?;
                    if !eq {
                        return Ok((false, format!("{标签}: 标的K线子项异常 >> {msg}")));
                    }
                } else if 字段 == "时间戳" {
                    let a = 提取时间戳(&valA).unwrap_or(0);
                    let b = 提取时间戳(&valB).unwrap_or(0);
                    if a != b {
                        return Ok((false, format!("{标签}: [{字段}] 数值不等 A={a},B={b}")));
                    }
                } else if 字段 == "方向" || 字段 == "分型" {
                    let sa = valA.str()?.extract::<String>().unwrap_or_default();
                    let sb = valB.str()?.extract::<String>().unwrap_or_default();
                    if sa != sb {
                        return Ok((false, format!("{标签}: [{字段}] 数值不等 A={sa},B={sb}")));
                    }
                } else if 字段 == "买卖点信息" {
                    let py = A.py();
                    let set_a = py.import("builtins")?.getattr("set")?.call1((&valA,))?;
                    let set_b = py.import("builtins")?.getattr("set")?.call1((&valB,))?;
                    let eq: bool = set_a.eq(set_b)?;
                    if !eq {
                        let ra = valA.repr()?.extract::<String>().unwrap_or_default();
                        let rb = valB.repr()?.extract::<String>().unwrap_or_default();
                        return Ok((false, format!("{标签}: [{字段}] 数值不等 A={ra},B={rb}")));
                    }
                } else {
                    let eq: bool = valA.eq(&valB)?;
                    if !eq {
                        let ra = valA.repr()?.extract::<String>().unwrap_or_default();
                        let rb = valB.repr()?.extract::<String>().unwrap_or_default();
                        return Ok((false, format!("{标签}: [{字段}] 数值不等 A={ra},B={rb}")));
                    }
                }
            }
            Ok((true, format!("{标签}: 全部字段嵌套校验一致")))
        }
    )
}

// ========== 分型相等 ==========

#[pyfunction]
#[pyo3(signature = (A, B, 浮点容差 = 1e-9))]
fn 分型相等(
    A: &Bound<'_, PyAny>,
    B: &Bound<'_, PyAny>,
    浮点容差: f64,
) -> PyResult<(bool, String)> {
    with_cache!(
        C_FRACTAL,
        4096,
        (
            A.as_ptr() as usize,
            B.as_ptr() as usize,
            浮点容差.to_bits() as i64
        ),
        {
            if let (Ok(a), Ok(b)) = (
                A.cast::<crate::structure_py::分型Py>(),
                B.cast::<crate::structure_py::分型Py>(),
            ) {
                return Ok(a.borrow().inner.相等(&b.borrow().inner, 浮点容差));
            }
            let 标签 = "分型校验";
            // Python 分型内部用 _结构/_时间戳/_分型特征值 作为 slot 名，Rust 用 结构/时间戳/分型特征值 作为 getter
            for &字段 in &["左", "中", "右"] {
                let valA = A.getattr(字段)?;
                let valB = B.getattr(字段)?;
                if let Some(r) = 检查空值一致(&valA, &valB, 字段, 标签) {
                    if !r.0 {
                        return Ok((false, r.1));
                    } else {
                        continue;
                    }
                }
                let (eq, msg) = 缠论K线相等(&valA, &valB, 浮点容差)?;
                if !eq {
                    return Ok((false, format!("{标签}: [{字段}]缠论K线子项异常 >> {msg}")));
                }
            }
            for &(字段, 字段别名) in &[
                ("_结构", "结构"),
                ("_时间戳", "时间戳"),
                ("_分型特征值", "分型特征值"),
            ] {
                // 先尝试 Python 侧的下划线名，再尝试 Rust 侧的无下划线名
                let valA = 获取属性任意(A, &[字段, 字段别名])?;
                let valB = 获取属性任意(B, &[字段, 字段别名])?;
                let (a有, b有) = (valA.is_some(), valB.is_some());
                if a有 && !b有 {
                    return Ok((false, format!("{标签}: [{字段}] A存在属性 B缺失属性")));
                }
                if !a有 && b有 {
                    return Ok((false, format!("{标签}: [{字段}] B存在属性 A缺失属性")));
                }
                if !a有 && !b有 {
                    continue;
                }
                let valA = valA.unwrap();
                let valB = valB.unwrap();

                if let Some(r) = 尝试浮点比较(&valA, &valB, 浮点容差) {
                    let (ok, m) = r?;
                    if !ok {
                        return Ok((false, format!("{标签}: [{字段}]{m}")));
                    }
                } else if 字段 == "_时间戳" {
                    let a = 提取时间戳(&valA).unwrap_or(0);
                    let b = 提取时间戳(&valB).unwrap_or(0);
                    if a != b {
                        return Ok((false, format!("{标签}: [{字段}] 数值不等 A={a},B={b}")));
                    }
                } else if 字段 == "_结构" {
                    let sa = valA.str()?.extract::<String>().unwrap_or_default();
                    let sb = valB.str()?.extract::<String>().unwrap_or_default();
                    if sa != sb {
                        return Ok((false, format!("{标签}: [{字段}] 数值不等 A={sa},B={sb}")));
                    }
                } else {
                    let eq: bool = valA.eq(&valB)?;
                    if !eq {
                        let ra = valA.repr()?.extract::<String>().unwrap_or_default();
                        let rb = valB.repr()?.extract::<String>().unwrap_or_default();
                        return Ok((false, format!("{标签}: [{字段}] 数值不等 A={ra},B={rb}")));
                    }
                }
            }
            Ok((true, format!("{标签}: 自有字段+三根缠论K线全部校验一致")))
        }
    )
}

// ========== 缺口相等 ==========

#[pyfunction]
#[pyo3(signature = (A, B, 浮点容差 = 1e-9))]
fn 缺口相等(
    A: &Bound<'_, PyAny>,
    B: &Bound<'_, PyAny>,
    浮点容差: f64,
) -> PyResult<(bool, String)> {
    with_cache!(
        C_GAP,
        4096,
        (
            A.as_ptr() as usize,
            B.as_ptr() as usize,
            浮点容差.to_bits() as i64
        ),
        {
            if let (Ok(a), Ok(b)) = (
                A.cast::<crate::types_py::缺口Py>(),
                B.cast::<crate::types_py::缺口Py>(),
            ) {
                return Ok(a.borrow().inner.相等(&b.borrow().inner, 浮点容差));
            }
            let 标签 = "缺口校验";
            for &字段 in &["高", "低"] {
                let (a有, b有) = (A.hasattr(字段)?, B.hasattr(字段)?);
                if a有 && !b有 {
                    return Ok((false, format!("{标签}: [{字段}] A存在 B缺失属性")));
                }
                if !a有 && b有 {
                    return Ok((false, format!("{标签}: [{字段}] B存在 A缺失属性")));
                }
                if !a有 && !b有 {
                    continue;
                }
                let valA = A.getattr(字段)?;
                let valB = B.getattr(字段)?;
                if let Some(r) = 尝试浮点比较(&valA, &valB, 浮点容差) {
                    let (ok, m) = r?;
                    if !ok {
                        return Ok((false, format!("{标签}: [{字段}]{m}")));
                    }
                } else {
                    let eq: bool = valA.eq(&valB)?;
                    if !eq {
                        let ra = valA.repr()?.extract::<String>().unwrap_or_default();
                        let rb = valB.repr()?.extract::<String>().unwrap_or_default();
                        return Ok((false, format!("{标签}: [{字段}] 数值不等 A={ra},B={rb}")));
                    }
                }
            }
            Ok((true, format!("{标签}: 上下沿价格校验完全一致")))
        }
    )
}

// ========== 线段特征相等 ==========

#[pyfunction]
#[pyo3(signature = (A, B, 浮点容差 = 1e-9))]
fn 线段特征相等(
    A: &Bound<'_, PyAny>,
    B: &Bound<'_, PyAny>,
    浮点容差: f64,
) -> PyResult<(bool, String)> {
    with_cache!(
        C_SEG_FEAT,
        4096,
        (
            A.as_ptr() as usize,
            B.as_ptr() as usize,
            浮点容差.to_bits() as i64
        ),
        {
            if let (Ok(a), Ok(b)) = (
                A.cast::<crate::structure_py::线段特征Py>(),
                B.cast::<crate::structure_py::线段特征Py>(),
            ) {
                return Ok(a.borrow().inner.相等(&b.borrow().inner, 浮点容差));
            }
            let 标签 = "线段特征校验";
            for &字段 in &["序号", "标识", "线段方向", "基础序列"] {
                let (a有, b有) = (A.hasattr(字段)?, B.hasattr(字段)?);
                if a有 && !b有 {
                    return Ok((false, format!("{标签}: [{字段}] A存在 B缺失属性")));
                }
                if !a有 && b有 {
                    return Ok((false, format!("{标签}: [{字段}] B存在 A缺失属性")));
                }
                if !a有 && !b有 {
                    continue;
                }
                let valA = A.getattr(字段)?;
                let valB = B.getattr(字段)?;

                if 字段 == "基础序列" {
                    let len_a: usize = valA.len()?;
                    let len_b: usize = valB.len()?;
                    if len_a != len_b {
                        return Ok((
                            false,
                            format!("{标签}: [基础序列] 列表长度不一致 A={len_a},B={len_b}"),
                        ));
                    }
                    for idx in 0..len_a {
                        let itemA = valA.get_item(idx)?;
                        let itemB = valB.get_item(idx)?;
                        let (eq, msg) = 虚线相等(&itemA, &itemB, 浮点容差)?;
                        if !eq {
                            return Ok((
                                false,
                                format!("{标签}: 基础序列[{idx}]子虚线异常 >> {msg}"),
                            ));
                        }
                    }
                } else if 字段 == "线段方向" {
                    let sa = valA.str()?.extract::<String>().unwrap_or_default();
                    let sb = valB.str()?.extract::<String>().unwrap_or_default();
                    if sa != sb {
                        return Ok((false, format!("{标签}: [{字段}] 数值不等 A={sa},B={sb}")));
                    }
                } else {
                    let eq: bool = valA.eq(&valB)?;
                    if !eq {
                        let ra = valA.repr()?.extract::<String>().unwrap_or_default();
                        let rb = valB.repr()?.extract::<String>().unwrap_or_default();
                        return Ok((false, format!("{标签}: [{字段}] 数值不等 A={ra},B={rb}")));
                    }
                }
            }
            Ok((true, format!("{标签}: 字段与内部虚线序列全部一致")))
        }
    )
}

// ========== 中枢相等 ==========

#[pyfunction]
#[pyo3(signature = (A, B, 浮点容差 = 1e-9))]
fn 中枢相等(
    A: &Bound<'_, PyAny>,
    B: &Bound<'_, PyAny>,
    浮点容差: f64,
) -> PyResult<(bool, String)> {
    if let (Ok(a), Ok(b)) = (
        A.cast::<crate::algorithm_py::中枢Py>(),
        B.cast::<crate::algorithm_py::中枢Py>(),
    ) {
        return Ok(a.borrow().inner.相等(&b.borrow().inner, 浮点容差));
    }
    let a标识 = 尝试获取标识(A);
    let b标识 = 尝试获取标识(B);
    let 标签 = format!("中枢校验[A标识={a标识},B标识={b标识}]");
    for &字段 in &[
        "序号",
        "标识",
        "级别",
        "基础序列",
        "第三买卖线",
        "本级_第三买卖线",
    ] {
        let (a有, b有) = (A.hasattr(字段)?, B.hasattr(字段)?);
        if a有 && !b有 {
            return Ok((false, format!("{标签}: [{字段}] A存在 B缺失属性")));
        }
        if !a有 && b有 {
            return Ok((false, format!("{标签}: [{字段}] B存在 A缺失属性")));
        }
        if !a有 && !b有 {
            continue;
        }
        let valA = A.getattr(字段)?;
        let valB = B.getattr(字段)?;

        if 字段 == "基础序列" {
            let len_a: usize = valA.len()?;
            let len_b: usize = valB.len()?;
            if len_a != len_b {
                return Ok((
                    false,
                    format!("{标签}: [基础序列] 长度不一致 A={len_a},B={len_b}"),
                ));
            }
            for idx in 0..len_a {
                let itemA = valA.get_item(idx)?;
                let itemB = valB.get_item(idx)?;
                let (eq, msg) = 虚线相等(&itemA, &itemB, 浮点容差)?;
                if !eq {
                    return Ok((false, format!("{标签}: 基础序列[{idx}]虚线异常 >> {msg}")));
                }
            }
        } else if 字段 == "第三买卖线" || 字段 == "本级_第三买卖线" {
            if let Some(r) = 检查空值一致(&valA, &valB, 字段, &标签) {
                if !r.0 {
                    return Ok((false, r.1));
                } else {
                    continue;
                }
            }
            let (eq, msg) = 虚线相等(&valA, &valB, 浮点容差)?;
            if !eq {
                return Ok((false, format!("{标签}: [{字段}]子虚线异常 >> {msg}")));
            }
        } else {
            let eq: bool = valA.eq(&valB)?;
            if !eq {
                let ra = valA.repr()?.extract::<String>().unwrap_or_default();
                let rb = valB.repr()?.extract::<String>().unwrap_or_default();
                return Ok((false, format!("{标签}: [{字段}] 数值不等 A={ra},B={rb}")));
            }
        }
    }
    Ok((true, format!("{标签}: 基础序列+第三买卖线全部校验一致")))
}

// ========== 虚线相等 ==========

#[pyfunction]
#[pyo3(signature = (A, B, 浮点容差 = 1e-9))]
fn 虚线相等(
    A: &Bound<'_, PyAny>,
    B: &Bound<'_, PyAny>,
    浮点容差: f64,
) -> PyResult<(bool, String)> {
    if let (Ok(a), Ok(b)) = (
        A.cast::<crate::structure_py::虚线Py>(),
        B.cast::<crate::structure_py::虚线Py>(),
    ) {
        return Ok(a.borrow().inner.相等(&b.borrow().inner, 浮点容差));
    }
    let a标识 = 尝试获取标识(A);
    let b标识 = 尝试获取标识(B);
    let 标签 = format!("虚线校验[A标识={a标识},B标识={b标识}]");
    let 比对字段 = [
        "标识",
        "序号",
        "级别",
        "文",
        "武",
        "有效性",
        "基础序列",
        "特征序列",
        "实_中枢序列",
        "虚_中枢序列",
        "合_中枢序列",
        "确认K线",
        "模式",
        "_特征序列_显示",
        "前一缺口",
        "前一结束位置",
        "短路修正",
    ];
    for &字段 in &比对字段 {
        let (a有, b有) = (A.hasattr(字段)?, B.hasattr(字段)?);
        if a有 && !b有 {
            return Ok((false, format!("{标签}: [{字段}] A存在属性 B缺失属性")));
        }
        if !a有 && b有 {
            return Ok((false, format!("{标签}: [{字段}] B存在属性 A缺失属性")));
        }
        if !a有 && !b有 {
            continue;
        }
        let valA = A.getattr(字段)?;
        let valB = B.getattr(字段)?;

        // 文/武：分型
        if 字段 == "文" || 字段 == "武" {
            if let Some(r) = 检查空值一致(&valA, &valB, 字段, &标签) {
                if !r.0 {
                    return Ok((false, r.1));
                } else {
                    continue;
                }
            }
            let (eq, msg) = 分型相等(&valA, &valB, 浮点容差)?;
            if !eq {
                return Ok((false, format!("{标签}: [{字段}]子分型异常 >> {msg}")));
            }
        }
        // 前一缺口
        else if 字段 == "前一缺口" {
            if let Some(r) = 检查空值一致(&valA, &valB, 字段, &标签) {
                if !r.0 {
                    return Ok((false, r.1));
                } else {
                    continue;
                }
            }
            let (eq, msg) = 缺口相等(&valA, &valB, 浮点容差)?;
            if !eq {
                return Ok((false, format!("{标签}: [前一缺口]子缺口异常 >> {msg}")));
            }
        }
        // 前一结束位置
        else if 字段 == "前一结束位置" {
            if let Some(r) = 检查空值一致(&valA, &valB, 字段, &标签) {
                if !r.0 {
                    return Ok((false, r.1));
                } else {
                    continue;
                }
            }
            let (eq, msg) = 虚线相等(&valA, &valB, 浮点容差)?;
            if !eq {
                return Ok((false, format!("{标签}: [前一结束位置]异常 >> {msg}")));
            }
        }
        // 确认K线
        else if 字段 == "确认K线" {
            if let Some(r) = 检查空值一致(&valA, &valB, 字段, &标签) {
                if !r.0 {
                    return Ok((false, r.1));
                } else {
                    continue;
                }
            }
            let (eq, msg) = 缠论K线相等(&valA, &valB, 浮点容差)?;
            if !eq {
                return Ok((false, format!("{标签}: [确认K线]子缠论K线异常 >> {msg}")));
            }
        }
        // 各类列表
        else if 字段 == "基础序列"
            || 字段 == "实_中枢序列"
            || 字段 == "虚_中枢序列"
            || 字段 == "合_中枢序列"
            || 字段 == "特征序列"
        {
            let len_a: usize = valA.len()?;
            let len_b: usize = valB.len()?;
            if len_a != len_b {
                return Ok((
                    false,
                    format!("{标签}: [{字段}]列表长度不一致 A={len_a},B={len_b}"),
                ));
            }
            for idx in 0..len_a {
                let itemA = valA.get_item(idx)?;
                let itemB = valB.get_item(idx)?;
                if let Some(r) =
                    检查空值一致(&itemA, &itemB, &format!("{字段}[{idx}]"), &标签)
                {
                    if !r.0 {
                        return Ok((false, r.1));
                    } else {
                        continue;
                    }
                }
                let (eq, msg) = if 字段 == "基础序列" {
                    虚线相等(&itemA, &itemB, 浮点容差)?
                } else if 字段.contains("中枢") {
                    中枢相等(&itemA, &itemB, 浮点容差)?
                } else {
                    线段特征相等(&itemA, &itemB, 浮点容差)?
                };
                if !eq {
                    return Ok((false, format!("{标签}: [{字段}][{idx}]子项异常 >> {msg}")));
                }
            }
        }
        // 普通字段
        else {
            let eq: bool = valA.eq(&valB)?;
            if !eq {
                let ra = valA.repr()?.extract::<String>().unwrap_or_default();
                let rb = valB.repr()?.extract::<String>().unwrap_or_default();
                return Ok((false, format!("{标签}: [{字段}]数值不等 A={ra},B={rb}")));
            }
        }
    }
    Ok((true, format!("{标签}: 全字段所有嵌套子结构校验一致")))
}

pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(K线相等, m)?)?;
    m.add_function(wrap_pyfunction!(缠论K线相等, m)?)?;
    m.add_function(wrap_pyfunction!(分型相等, m)?)?;
    m.add_function(wrap_pyfunction!(缺口相等, m)?)?;
    m.add_function(wrap_pyfunction!(线段特征相等, m)?)?;
    m.add_function(wrap_pyfunction!(中枢相等, m)?)?;
    m.add_function(wrap_pyfunction!(虚线相等, m)?)?;
    Ok(())
}
