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

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::collections::HashMap;

use chanlun::signal::event::Event as 核心Event;
use chanlun::signal::factor::Factor as 核心Factor;
use chanlun::signal::operate::Operate as 核心Operate;
use chanlun::signal::position::Position as 核心Position;
use chanlun::signal::signal::Signal as 核心Signal;
use chanlun::signal::{信号字典, 匹配值};

/// Operate 枚举绑定。
#[pyclass(name = "Operate", module = "chanlun._chanlun", eq, eq_int)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OperatePy {
    HL,
    HS,
    HO,
    LO,
    LE,
    SO,
    SE,
}

impl OperatePy {
    pub(crate) fn 转核心(self) -> 核心Operate {
        match self {
            OperatePy::HL => 核心Operate::持多,
            OperatePy::HS => 核心Operate::持空,
            OperatePy::HO => 核心Operate::持币,
            OperatePy::LO => 核心Operate::开多,
            OperatePy::LE => 核心Operate::平多,
            OperatePy::SO => 核心Operate::开空,
            OperatePy::SE => 核心Operate::平空,
        }
    }
    pub(crate) fn 从核心(o: 核心Operate) -> Self {
        match o {
            核心Operate::持多 => OperatePy::HL,
            核心Operate::持空 => OperatePy::HS,
            核心Operate::持币 => OperatePy::HO,
            核心Operate::开多 => OperatePy::LO,
            核心Operate::平多 => OperatePy::LE,
            核心Operate::开空 => OperatePy::SO,
            核心Operate::平空 => OperatePy::SE,
        }
    }
}

#[pymethods]
impl OperatePy {
    #[getter]
    fn value(&self) -> &'static str {
        self.转核心().value()
    }
    fn __str__(&self) -> &'static str {
        self.转核心().value()
    }
    fn __repr__(&self) -> String {
        format!("Operate.{:?}", self)
    }
    /// 从中文值还原 Operate（供 Event.load 反序列化用，对应旧 Python `Operate("开多")`）。
    #[staticmethod]
    fn from_value(value: &str) -> PyResult<OperatePy> {
        match value {
            "持多" => Ok(OperatePy::HL),
            "持空" => Ok(OperatePy::HS),
            "持币" => Ok(OperatePy::HO),
            "开多" => Ok(OperatePy::LO),
            "平多" => Ok(OperatePy::LE),
            "开空" => Ok(OperatePy::SO),
            "平空" => Ok(OperatePy::SE),
            _ => Err(PyValueError::new_err(format!("未知 Operate 值: {value}"))),
        }
    }
}

/// 把 PyDict 转成核心层信号字典：str 值 → 字符串，其余 → 非字符串。
pub(crate) fn 字典转核心(s: &Bound<'_, PyDict>) -> PyResult<信号字典> {
    let mut out: 信号字典 = HashMap::new();
    for (k, v) in s.iter() {
        let key: String = k.extract()?;
        let 值 = match v.extract::<String>() {
            Ok(文本) if !文本.is_empty() => 匹配值::字符串(文本),
            _ => 匹配值::非字符串,
        };
        out.insert(key, 值);
    }
    Ok(out)
}

/// 反序列化辅助：从 dict 取字符串字段，缺省返回空串。
fn 取字符串(raw: &Bound<'_, PyDict>, key: &str) -> PyResult<String> {
    match raw.get_item(key)? {
        Some(v) => v.extract(),
        None => Ok(String::new()),
    }
}

/// 从七段字符串解析 Signal（格式: k1_k2_k3_v1_v2_v3_score）。
fn parse_signal_str(s: &str) -> PyResult<核心Signal> {
    let parts: Vec<&str> = s.split('_').collect();
    if parts.len() != 7 {
        return Err(PyValueError::new_err(format!(
            "Signal 格式无效：应为 k1_k2_k3_v1_v2_v3_score（7段），收到 {s}"
        )));
    }
    let score: i32 = parts[6]
        .parse()
        .map_err(|_| PyValueError::new_err(format!("无法解析 score: {}", parts[6])))?;
    Ok(核心Signal::new(
        parts[0], parts[1], parts[2], parts[3], parts[4], parts[5], score,
    ))
}

/// 反序列化辅助：从 dict 取信号串列表，逐个解析为核心 Signal。
fn 取信号列表(raw: &Bound<'_, PyDict>, key: &str) -> PyResult<Vec<核心Signal>> {
    let mut out = Vec::new();
    if let Some(item) = raw.get_item(key)? {
        let strs: Vec<String> = item.extract()?;
        for s in strs {
            out.push(parse_signal_str(&s)?);
        }
    }
    Ok(out)
}

/// 反序列化辅助：从 dict 取事件列表，逐个调用 Event.load。
fn 取事件列表(raw: &Bound<'_, PyDict>, key: &str) -> PyResult<Vec<核心Event>> {
    let mut out = Vec::new();
    if let Some(item) = raw.get_item(key)? {
        let dicts: Vec<Bound<'_, PyDict>> = item.extract()?;
        for d in &dicts {
            out.push(EventPy::load(d)?.inner);
        }
    }
    Ok(out)
}

/// Signal 绑定。
#[pyclass(name = "Signal", module = "chanlun._chanlun")]
#[derive(Clone)]
pub struct SignalPy {
    pub(crate) inner: 核心Signal,
}

#[pymethods]
impl SignalPy {
    #[new]
    #[pyo3(signature = (signal=String::new(), score=0, k1="任意".to_string(), k2="任意".to_string(), k3="任意".to_string(), v1="任意".to_string(), v2="任意".to_string(), v3="任意".to_string()))]
    fn new(
        signal: String,
        score: i32,
        k1: String,
        k2: String,
        k3: String,
        v1: String,
        v2: String,
        v3: String,
    ) -> PyResult<Self> {
        let inner = if signal.is_empty() {
            核心Signal::new(&k1, &k2, &k3, &v1, &v2, &v3, score)
        } else {
            parse_signal_str(&signal)?
        };
        Ok(Self { inner })
    }

    #[getter]
    fn signal(&self) -> String {
        self.inner.signal.clone()
    }
    #[getter]
    fn score(&self) -> i32 {
        self.inner.score
    }
    #[getter]
    fn k1(&self) -> String {
        self.inner.k1.clone()
    }
    #[getter]
    fn k2(&self) -> String {
        self.inner.k2.clone()
    }
    #[getter]
    fn k3(&self) -> String {
        self.inner.k3.clone()
    }
    #[getter]
    fn v1(&self) -> String {
        self.inner.v1.clone()
    }
    #[getter]
    fn v2(&self) -> String {
        self.inner.v2.clone()
    }
    #[getter]
    fn v3(&self) -> String {
        self.inner.v3.clone()
    }

    #[getter]
    fn key(&self) -> String {
        self.inner.key()
    }
    #[getter]
    fn value(&self) -> String {
        self.inner.value()
    }

    fn is_match(&self, s: &Bound<'_, PyDict>) -> PyResult<bool> {
        let 字典 = 字典转核心(s)?;
        self.inner
            .is_match(&字典)
            .map_err(|e| PyValueError::new_err(format!("{} 不在信号列表中", e.0)))
    }

    fn __repr__(&self) -> String {
        format!("Signal('{}')", self.inner.signal)
    }
}

/// Factor 绑定。signals_all 全满足 + signals_any 任一满足 + signals_not 全不满足。
#[pyclass(name = "Factor", module = "chanlun._chanlun")]
#[derive(Clone)]
pub struct FactorPy {
    pub(crate) inner: 核心Factor,
}

#[pymethods]
impl FactorPy {
    #[new]
    #[pyo3(signature = (signals_all, signals_any=Vec::new(), signals_not=Vec::new(), name=String::new()))]
    fn new(
        signals_all: Vec<SignalPy>,
        signals_any: Vec<SignalPy>,
        signals_not: Vec<SignalPy>,
        name: String,
    ) -> PyResult<Self> {
        let 取 = |v: Vec<SignalPy>| v.into_iter().map(|s| s.inner).collect::<Vec<_>>();
        let inner = 核心Factor::新建(取(signals_all), 取(signals_any), 取(signals_not), name)
            .map_err(PyValueError::new_err)?;
        Ok(Self { inner })
    }

    #[getter]
    fn name(&self) -> String {
        self.inner.name.clone()
    }

    #[getter]
    fn signals_all(&self) -> Vec<SignalPy> {
        self.inner
            .signals_all
            .iter()
            .cloned()
            .map(|inner| SignalPy { inner })
            .collect()
    }

    #[getter]
    fn signals_any(&self) -> Vec<SignalPy> {
        self.inner
            .signals_any
            .iter()
            .cloned()
            .map(|inner| SignalPy { inner })
            .collect()
    }

    #[getter]
    fn signals_not(&self) -> Vec<SignalPy> {
        self.inner
            .signals_not
            .iter()
            .cloned()
            .map(|inner| SignalPy { inner })
            .collect()
    }

    #[getter]
    fn unique_signals(&self) -> Vec<String> {
        self.inner.unique_signals()
    }

    fn is_match(&self, s: &Bound<'_, PyDict>) -> PyResult<bool> {
        let 字典 = 字典转核心(s)?;
        self.inner
            .is_match(&字典)
            .map_err(|e| PyValueError::new_err(format!("{} 不在信号列表中", e.0)))
    }

    fn __repr__(&self) -> String {
        format!("Factor('{}')", self.inner.name)
    }

    /// 序列化为 dict：{name, signals_all, signals_any, signals_not}（signals 存为信号串）。
    fn dump<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let d = PyDict::new(py);
        let 串 = |v: &[核心Signal]| v.iter().map(|s| s.signal.clone()).collect::<Vec<_>>();
        d.set_item("name", &self.inner.name)?;
        d.set_item("signals_all", 串(&self.inner.signals_all))?;
        d.set_item("signals_any", 串(&self.inner.signals_any))?;
        d.set_item("signals_not", 串(&self.inner.signals_not))?;
        Ok(d)
    }

    /// 从 dict 反序列化（对应旧 Python Factor.load）。
    #[staticmethod]
    fn load(raw: &Bound<'_, PyDict>) -> PyResult<FactorPy> {
        let inner = 核心Factor::新建(
            取信号列表(raw, "signals_all")?,
            取信号列表(raw, "signals_any")?,
            取信号列表(raw, "signals_not")?,
            取字符串(raw, "name")?,
        )
        .map_err(PyValueError::new_err)?;
        Ok(FactorPy { inner })
    }
}

/// Event 绑定。operate + 因子列表（任一因子满足则事件为真）。
#[pyclass(name = "Event", module = "chanlun._chanlun")]
#[derive(Clone)]
pub struct EventPy {
    pub(crate) inner: 核心Event,
}

#[pymethods]
impl EventPy {
    #[new]
    #[pyo3(signature = (operate, factors, signals_all=Vec::new(), signals_any=Vec::new(), signals_not=Vec::new(), name=String::new()))]
    fn new(
        operate: OperatePy,
        factors: Vec<FactorPy>,
        signals_all: Vec<SignalPy>,
        signals_any: Vec<SignalPy>,
        signals_not: Vec<SignalPy>,
        name: String,
    ) -> PyResult<Self> {
        let 取s = |v: Vec<SignalPy>| v.into_iter().map(|s| s.inner).collect::<Vec<_>>();
        let 取f = |v: Vec<FactorPy>| v.into_iter().map(|f| f.inner).collect::<Vec<_>>();
        let inner = 核心Event::新建(
            operate.转核心(),
            取f(factors),
            取s(signals_all),
            取s(signals_any),
            取s(signals_not),
            name,
        )
        .map_err(PyValueError::new_err)?;
        Ok(Self { inner })
    }

    #[getter]
    fn name(&self) -> String {
        self.inner.name.clone()
    }

    #[getter]
    fn sha256(&self) -> String {
        self.inner.sha256.clone()
    }

    #[getter]
    fn operate(&self) -> OperatePy {
        OperatePy::从核心(self.inner.operate)
    }

    #[getter]
    fn factors(&self) -> Vec<FactorPy> {
        self.inner
            .factors
            .iter()
            .cloned()
            .map(|inner| FactorPy { inner })
            .collect()
    }

    #[getter]
    fn unique_signals(&self) -> Vec<String> {
        self.inner.unique_signals()
    }

    fn is_match(&self, s: &Bound<'_, PyDict>) -> PyResult<(bool, Option<String>)> {
        let 字典 = 字典转核心(s)?;
        self.inner
            .is_match(&字典)
            .map_err(|e| PyValueError::new_err(format!("{} 不在信号列表中", e.0)))
    }

    fn __repr__(&self) -> String {
        format!("Event('{}')", self.inner.name)
    }

    /// 序列化为 dict：{name, operate, signals_all/any/not, factors}。
    fn dump<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let d = PyDict::new(py);
        let 串 = |v: &[核心Signal]| v.iter().map(|s| s.signal.clone()).collect::<Vec<_>>();
        d.set_item("name", &self.inner.name)?;
        d.set_item("operate", self.inner.operate.value())?;
        d.set_item("signals_all", 串(&self.inner.signals_all))?;
        d.set_item("signals_any", 串(&self.inner.signals_any))?;
        d.set_item("signals_not", 串(&self.inner.signals_not))?;
        let factors: Vec<Bound<'py, PyDict>> = self
            .inner
            .factors
            .iter()
            .map(|f| FactorPy { inner: f.clone() }.dump(py))
            .collect::<PyResult<_>>()?;
        d.set_item("factors", factors)?;
        Ok(d)
    }

    /// 从 dict 反序列化（对应旧 Python Event.load）。
    #[staticmethod]
    fn load(raw: &Bound<'_, PyDict>) -> PyResult<EventPy> {
        let operate = OperatePy::from_value(&取字符串(raw, "operate")?)?.转核心();
        let mut factors = Vec::new();
        if let Some(item) = raw.get_item("factors")? {
            let dicts: Vec<Bound<'_, PyDict>> = item.extract()?;
            for fd in dicts {
                factors.push(FactorPy::load(&fd)?.inner);
            }
        }
        let inner = 核心Event::新建(
            operate,
            factors,
            取信号列表(raw, "signals_all")?,
            取信号列表(raw, "signals_any")?,
            取信号列表(raw, "signals_not")?,
            取字符串(raw, "name")?,
        )
        .map_err(PyValueError::new_err)?;
        Ok(EventPy { inner })
    }
}

/// Position 绑定（可子类化）。Python 子类应实现 update() 状态机。
#[pyclass(name = "Position", module = "chanlun._chanlun", subclass)]
#[derive(Clone)]
pub struct PositionPy {
    pub(crate) inner: 核心Position,
}

/// 核心 Operate → PyO3 OperatePy 枚举变体映射。
fn 核心op转pyop(op: 核心Operate) -> OperatePy {
    match op {
        核心Operate::持多 => OperatePy::HL,
        核心Operate::持空 => OperatePy::HS,
        核心Operate::持币 => OperatePy::HO,
        核心Operate::开多 => OperatePy::LO,
        核心Operate::平多 => OperatePy::LE,
        核心Operate::开空 => OperatePy::SO,
        核心Operate::平空 => OperatePy::SE,
    }
}

/// 将 i64 Unix 时间戳转为 Python datetime（UTC）。
pub(crate) fn 时间戳转datetime(py: Python<'_>, ts: i64) -> PyResult<Py<PyAny>> {
    let datetime_mod = py.import("datetime")?;
    let tz = datetime_mod.getattr("timezone")?.getattr("utc")?;
    let dt = datetime_mod
        .getattr("datetime")?
        .call_method1("fromtimestamp", (ts as f64, tz))?;
    Ok(dt.into())
}

#[pymethods]
impl PositionPy {
    #[new]
    #[pyo3(signature = (symbol, opens, exits=Vec::new(), interval=0, timeout=1000, stop_loss=1000, T0=false, name=String::new()))]
    fn new(
        symbol: String,
        opens: Vec<EventPy>,
        exits: Vec<EventPy>,
        interval: i64,
        timeout: i64,
        stop_loss: i64,
        T0: bool,
        name: String,
    ) -> PyResult<Self> {
        let 取 = |v: Vec<EventPy>| v.into_iter().map(|e| e.inner).collect::<Vec<_>>();
        let inner = 核心Position::新建(
            symbol,
            取(opens),
            取(exits),
            interval,
            timeout,
            stop_loss,
            T0,
            name,
        )
        .map_err(PyValueError::new_err)?;
        Ok(Self { inner })
    }

    // --- 配置 getter（不变）---
    #[getter]
    fn symbol(&self) -> String {
        self.inner.symbol.clone()
    }
    #[getter]
    fn name(&self) -> String {
        self.inner.name.clone()
    }
    #[getter]
    fn opens(&self) -> Vec<EventPy> {
        self.inner
            .opens
            .iter()
            .cloned()
            .map(|inner| EventPy { inner })
            .collect()
    }
    #[getter]
    fn exits(&self) -> Vec<EventPy> {
        self.inner
            .exits
            .iter()
            .cloned()
            .map(|inner| EventPy { inner })
            .collect()
    }
    #[getter]
    fn events(&self) -> Vec<EventPy> {
        self.inner
            .events
            .iter()
            .cloned()
            .map(|inner| EventPy { inner })
            .collect()
    }
    #[getter]
    fn interval(&self) -> i64 {
        self.inner.interval
    }
    #[getter]
    fn timeout(&self) -> i64 {
        self.inner.timeout
    }
    #[getter]
    fn stop_loss(&self) -> i64 {
        self.inner.stop_loss
    }
    #[getter]
    fn T0(&self) -> bool {
        self.inner.T0
    }

    #[getter]
    fn unique_signals(&self) -> Vec<String> {
        self.inner.unique_signals()
    }

    // --- 状态 getter（新增）---
    #[getter]
    fn pos(&self) -> i32 {
        self.inner.pos
    }

    #[getter]
    fn pos_changed(&self) -> bool {
        self.inner.pos_changed
    }

    #[getter]
    fn operates<'py>(&self, py: Python<'py>) -> PyResult<Vec<Bound<'py, PyDict>>> {
        self.inner
            .operates
            .iter()
            .map(|r| {
                let d = PyDict::new(py);
                d.set_item("symbol", &r.symbol)?;
                d.set_item("dt", 时间戳转datetime(py, r.dt)?)?;
                d.set_item("bid", r.bid)?;
                d.set_item("price", r.price)?;
                d.set_item("op", 核心op转pyop(r.op))?;
                d.set_item("op_desc", &r.op_desc)?;
                d.set_item("pos", r.pos)?;
                Ok(d)
            })
            .collect()
    }

    #[getter]
    fn holds<'py>(&self, py: Python<'py>) -> PyResult<Vec<Bound<'py, PyDict>>> {
        self.inner
            .holds
            .iter()
            .map(|r| {
                let d = PyDict::new(py);
                d.set_item("dt", 时间戳转datetime(py, r.dt)?)?;
                d.set_item("pos", r.pos)?;
                d.set_item("price", r.price)?;
                Ok(d)
            })
            .collect()
    }

    #[getter]
    fn pairs<'py>(&self, py: Python<'py>) -> PyResult<Vec<Bound<'py, PyDict>>> {
        self.inner
            .pairs()
            .iter()
            .map(|r| {
                let d = PyDict::new(py);
                d.set_item("标的代码", &r.标的代码)?;
                d.set_item("策略标记", &r.策略标记)?;
                d.set_item("交易方向", &r.交易方向)?;
                d.set_item("开仓时间", 时间戳转datetime(py, r.开仓时间)?)?;
                d.set_item("平仓时间", 时间戳转datetime(py, r.平仓时间)?)?;
                d.set_item("开仓价格", r.开仓价格)?;
                d.set_item("平仓价格", r.平仓价格)?;
                d.set_item("持仓K线数", r.持仓K线数)?;
                d.set_item("事件序列", &r.事件序列)?;
                d.set_item("持仓天数", r.持仓天数)?;
                d.set_item("盈亏比例", r.盈亏比例)?;
                Ok(d)
            })
            .collect()
    }

    /// 更新持仓状态。接收一个信号字典（含 OHLCV 字段 + 信号键）。
    ///
    /// 信号字典必须包含：``dt``（datetime 或 Unix 时间戳）, ``close``（收盘价）。
    /// 可选：``id`` 或 ``bid``（K线序号）。
    #[pyo3(signature = (信号字典))]
    fn update(&mut self, 信号字典: &Bound<'_, PyDict>) -> PyResult<()> {
        // 1. 提取 dt（支持 datetime 对象和 int/float Unix 时间戳）
        let dt: i64 = match 信号字典.get_item("dt")? {
            Some(v) => {
                // 尝试 i64
                if let Ok(ts) = v.extract::<i64>() {
                    ts
                // 尝试 f64
                } else if let Ok(ts) = v.extract::<f64>() {
                    ts as i64
                // 尝试 datetime.timestamp()
                } else if let Ok(ts) = v.call_method0("timestamp") {
                    (ts.extract::<f64>()?) as i64
                } else {
                    return Err(PyValueError::new_err(
                        "无法从信号字典中提取 dt 字段（需要 datetime 或 Unix 时间戳）",
                    ));
                }
            }
            None => return Err(PyValueError::new_err("信号字典缺少 dt 字段")),
        };

        // 2. 提取 price
        let price: f64 = 信号字典
            .get_item("close")?
            .and_then(|v| v.extract::<f64>().ok())
            .ok_or_else(|| PyValueError::new_err("信号字典缺少 close 字段"))?;

        // 3. 提取 bid（可选）
        let bid: i64 = 信号字典
            .get_item("id")?
            .or_else(|| 信号字典.get_item("bid").ok().flatten())
            .and_then(|v| v.extract::<i64>().ok())
            .unwrap_or(0);

        // 4. 转换为信号字典（排除 OHLCV 键）
        let ohkcv_keys: std::collections::HashSet<&str> = [
            "symbol", "dt", "open", "high", "low", "close", "vol", "id", "bid",
        ]
        .iter()
        .copied()
        .collect();

        let mut signals: 信号字典 = HashMap::new();
        for (k, v) in 信号字典.iter() {
            let key: String = k.extract()?;
            if ohkcv_keys.contains(key.as_str()) {
                continue;
            }
            let 值 = match v.extract::<String>() {
                Ok(文本) if !文本.is_empty() => 匹配值::字符串(文本),
                _ => 匹配值::非字符串,
            };
            signals.insert(key, 值);
        }

        // 5. 调用核心状态机
        self.inner
            .update(dt, price, bid, &signals)
            .map_err(|e| PyValueError::new_err(format!("{} 不在信号列表中", e.0)))?;

        Ok(())
    }

    fn __repr__(&self) -> String {
        format!(
            "Position(name={}, symbol={}, timeout={}, stop_loss={}BP, T0={}, interval={}s, pos={})",
            self.inner.name,
            self.inner.symbol,
            self.inner.timeout,
            self.inner.stop_loss,
            self.inner.T0,
            self.inner.interval,
            self.inner.pos
        )
    }

    /// 序列化为 dict。
    /// `with_data=True` 时附带 state（pairs, holds）；`with_data=False` 时仅配置。
    #[pyo3(signature = (with_data=false))]
    fn dump<'py>(&self, py: Python<'py>, with_data: bool) -> PyResult<Bound<'py, PyDict>> {
        let d = PyDict::new(py);
        d.set_item("symbol", &self.inner.symbol)?;
        d.set_item("name", &self.inner.name)?;
        let 事件dump = |evts: &[核心Event]| -> PyResult<Vec<Bound<'py, PyDict>>> {
            evts.iter()
                .map(|e| EventPy { inner: e.clone() }.dump(py))
                .collect()
        };
        d.set_item("opens", 事件dump(&self.inner.opens)?)?;
        d.set_item("exits", 事件dump(&self.inner.exits)?)?;
        d.set_item("interval", self.inner.interval)?;
        d.set_item("timeout", self.inner.timeout)?;
        d.set_item("stop_loss", self.inner.stop_loss)?;
        d.set_item("T0", self.inner.T0)?;
        if with_data {
            d.set_item("pairs", self.pairs(py)?)?;
            d.set_item("holds", self.holds(py)?)?;
        }
        Ok(d)
    }

    /// 从 dict 反序列化（仅配置，状态字段初始化为默认值）。
    #[staticmethod]
    fn load(raw: &Bound<'_, PyDict>) -> PyResult<PositionPy> {
        let symbol = 取字符串(raw, "symbol")?;
        let name = 取字符串(raw, "name")?;
        let interval: i64 = raw
            .get_item("interval")?
            .and_then(|v| v.extract().ok())
            .unwrap_or(0);
        let timeout: i64 = raw
            .get_item("timeout")?
            .and_then(|v| v.extract().ok())
            .unwrap_or(1000);
        let stop_loss: i64 = raw
            .get_item("stop_loss")?
            .and_then(|v| v.extract().ok())
            .unwrap_or(1000);
        let T0: bool = raw
            .get_item("T0")?
            .and_then(|v| v.extract().ok())
            .unwrap_or(false);

        let opens = 取事件列表(raw, "opens")?;
        let exits = 取事件列表(raw, "exits")?;

        let inner =
            核心Position::新建(symbol, opens, exits, interval, timeout, stop_loss, T0, name)
                .map_err(PyValueError::new_err)?;
        Ok(Self { inner })
    }
}

pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<OperatePy>()?;
    m.add_class::<SignalPy>()?;
    m.add_class::<FactorPy>()?;
    m.add_class::<EventPy>()?;
    m.add_class::<PositionPy>()?;
    Ok(())
}
