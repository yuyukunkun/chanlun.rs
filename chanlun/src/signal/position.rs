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

//! 仓位配置 + 持仓状态机。
//!
//! 第三方代码声明：Position 概念参考 czsc（https://github.com/waditu/czsc，
//! Apache License 2.0），状态机逻辑已从 Python 迁移到 Rust。

use crate::signal::event::Event;
use crate::signal::operate::Operate;
use crate::signal::{信号字典, 缺键错误};

// ============================================================================
// 新类型定义
// ============================================================================

/// 操作记录 — 对应 Python operate 字典（`__create_operate` 产出）。
#[derive(Clone, Debug)]
pub struct 操作记录 {
    pub symbol: String,
    pub dt: i64, // Unix 秒
    pub bid: i64,
    pub price: f64,
    pub op: Operate,
    pub op_desc: String,
    pub pos: i32,
}

/// 持仓快照 — 对应 Python holds 元素（`self.holds.append`）。
#[derive(Clone, Debug)]
pub struct 持仓记录 {
    pub dt: i64,
    pub pos: i32,
    pub price: f64,
}

/// 开平配对 — `pairs` 属性的返回类型，对应 Python 的 pair 字典。
#[derive(Clone, Debug)]
pub struct 开平配对 {
    pub 标的代码: String,
    pub 策略标记: String,
    pub 交易方向: String,
    pub 开仓时间: i64,
    pub 平仓时间: i64,
    pub 开仓价格: f64,
    pub 平仓价格: f64,
    pub 持仓K线数: i64,
    pub 事件序列: String,
    pub 持仓天数: f64,
    pub 盈亏比例: f64, // 单位 BP (1BP = 0.0001)
}

/// 最近事件缓存 — 对应 Python last_event 字典。
#[derive(Clone, Debug, Default)]
pub struct 最近事件 {
    pub dt: Option<i64>,
    pub bid: Option<i64>,
    pub price: Option<f64>,
    pub op: Option<Operate>,
    pub op_desc: Option<String>,
}

// ============================================================================
// Position 结构体
// ============================================================================

#[derive(Clone, Debug)]
pub struct Position {
    // --- 配置字段（不变）---
    pub symbol: String,
    pub opens: Vec<Event>,
    pub exits: Vec<Event>,
    pub events: Vec<Event>,
    pub name: String,
    pub interval: i64,
    pub timeout: i64,
    pub stop_loss: i64,
    pub T0: bool,

    // --- 状态字段（新增）---
    /// 仓位：1 = 多头，-1 = 空头，0 = 空仓
    pub pos: i32,
    /// 本次 update 是否改变了仓位
    pub pos_changed: bool,
    /// 事件触发的操作列表（时间顺序）
    pub operates: Vec<操作记录>,
    /// 每个时间步的持仓快照
    pub holds: Vec<持仓记录>,
    /// 最近一次开仓事件缓存
    pub last_event: 最近事件,
    /// 最近一次开多时间（Unix 秒）
    pub last_lo_dt: Option<i64>,
    /// 最近一次开空时间（Unix 秒）
    pub last_so_dt: Option<i64>,
    /// 最近一次信号传入时间
    pub end_dt: Option<i64>,
}

impl Position {
    /// 构造。name 必填；每个 event.operate 必须 ∈ {开多,平多,开空,平空}。
    #[allow(clippy::too_many_arguments)]
    pub fn 新建(
        symbol: String,
        opens: Vec<Event>,
        exits: Vec<Event>,
        interval: i64,
        timeout: i64,
        stop_loss: i64,
        T0: bool,
        name: String,
    ) -> Result<Self, String> {
        if name.is_empty() {
            return Err("name 是必须的参数".to_string());
        }
        let mut events = opens.clone();
        events.extend(exits.clone());
        for e in &events {
            if !matches!(
                e.operate,
                Operate::开多 | Operate::平多 | Operate::开空 | Operate::平空
            ) {
                return Err(format!("非法 operate: {}", e.operate.value()));
            }
        }
        Ok(Self {
            symbol,
            opens,
            exits,
            events,
            name,
            interval,
            timeout,
            stop_loss,
            T0,
            // 状态字段初始化为默认值
            pos: 0,
            pos_changed: false,
            operates: Vec::new(),
            holds: Vec::new(),
            last_event: 最近事件::default(),
            last_lo_dt: None,
            last_so_dt: None,
            end_dt: None,
        })
    }

    pub fn unique_signals(&self) -> Vec<String> {
        let mut 集合 = std::collections::BTreeSet::new();
        for e in &self.events {
            for s in e.unique_signals() {
                集合.insert(s);
            }
        }
        集合.into_iter().collect()
    }

    // ========================================================================
    // 内部辅助 — push_operate
    // ========================================================================

    /// 创建操作记录并追加到 operates 列表。
    fn push_operate(&mut self, dt: i64, bid: i64, price: f64, op: Operate, op_desc: &str) {
        self.pos_changed = true;
        self.operates.push(操作记录 {
            symbol: self.symbol.clone(),
            dt,
            bid,
            price,
            op,
            op_desc: op_desc.to_string(),
            pos: self.pos,
        });
    }

    // ========================================================================
    // update — 核心状态机
    // ========================================================================

    /// 更新持仓状态。
    ///
    /// 参数：
    /// - `dt`: 信号时间（Unix 秒）
    /// - `price`: 收盘价
    /// - `bid`: K线序号
    /// - `signals`: 信号字典（key → 匹配值）
    ///
    /// 逻辑与 Python `Position.update(s)` 1:1 对应。
    pub fn update(
        &mut self,
        dt: i64,
        price: f64,
        bid: i64,
        signals: &信号字典,
    ) -> Result<(), 缺键错误> {
        // 1. 时间校验（信号时间不能倒退）
        if let Some(end) = self.end_dt
            && dt <= end
        {
            crate::warn!("请检查信号传入：最新信号时间{dt}在上次信号时间{end}之前");
            return Ok(());
        }

        self.pos_changed = false;

        // 2. 事件匹配 — 取第一个命中的事件
        let mut op = Operate::持币;
        let mut op_desc = String::new();
        for event in &self.events {
            let (m, f) = event.is_match(signals)?;
            if m {
                op = event.operate;
                op_desc = format!("{}@{}", event.name, f.as_deref().unwrap_or(""));
                break;
            }
        }

        // 3. 更新 end_dt
        self.end_dt = Some(dt);

        // 4. 开仓事件 → 更新 last_event 缓存
        if matches!(op, Operate::开多 | Operate::开空) {
            self.last_event = 最近事件 {
                dt: Some(dt),
                bid: Some(bid),
                price: Some(price),
                op: Some(op),
                op_desc: Some(op_desc.clone()),
            };
        }

        // 5. 开多处理
        if op == Operate::开多 {
            if self.pos != 1 && 间隔检查(self.last_lo_dt, dt, self.interval) {
                // 满足间隔要求 → 开多
                self.pos = 1;
                self.push_operate(dt, bid, price, Operate::开多, &op_desc);
                self.last_lo_dt = Some(dt);
            } else {
                // 不满足开多条件 → 仅平空
                if self.pos == -1 && 允许操作(self.T0, dt, self.last_so_dt) {
                    self.pos = 0;
                    self.push_operate(dt, bid, price, Operate::平空, &op_desc);
                }
            }
        }

        // 6. 开空处理
        if op == Operate::开空 {
            if self.pos != -1 && 间隔检查(self.last_so_dt, dt, self.interval) {
                // 满足间隔要求 → 开空
                self.pos = -1;
                self.push_operate(dt, bid, price, Operate::开空, &op_desc);
                self.last_so_dt = Some(dt);
            } else {
                // 不满足开空条件 → 仅平多
                if self.pos == 1 && 允许操作(self.T0, dt, self.last_lo_dt) {
                    self.pos = 0;
                    self.push_operate(dt, bid, price, Operate::平多, &op_desc);
                }
            }
        }

        // 7. 多头出场
        if self.pos == 1 && 允许操作(self.T0, dt, self.last_lo_dt) {
            // 安全断言：last_event 的时间不应早于开仓时间
            if let Some(le_dt) = self.last_event.dt
                && let Some(lo_dt) = self.last_lo_dt
            {
                debug_assert!(
                    le_dt >= lo_dt,
                    "last_event.dt({le_dt}) < last_lo_dt({lo_dt})"
                );
            }

            // 7a. 平多信号
            if op == Operate::平多 {
                self.pos = 0;
                self.push_operate(dt, bid, price, Operate::平多, &op_desc);
            }

            // 7b. 多头止损
            if let Some(last_price) = self.last_event.price
                && price / last_price - 1.0 < -(self.stop_loss as f64) / 10000.0
                && self.pos != 0
            {
                self.pos = 0;
                self.push_operate(
                    dt,
                    bid,
                    price,
                    Operate::平多,
                    &format!("平多@{}BP止损", self.stop_loss),
                );
            }

            // 7c. 多头超时
            if let Some(last_bid) = self.last_event.bid
                && bid - last_bid > self.timeout
                && self.pos != 0
            {
                self.pos = 0;
                self.push_operate(
                    dt,
                    bid,
                    price,
                    Operate::平多,
                    &format!("平多@{}K超时", self.timeout),
                );
            }
        }

        // 8. 空头出场
        if self.pos == -1 && 允许操作(self.T0, dt, self.last_so_dt) {
            if let Some(le_dt) = self.last_event.dt
                && let Some(so_dt) = self.last_so_dt
            {
                debug_assert!(
                    le_dt >= so_dt,
                    "last_event.dt({le_dt}) < last_so_dt({so_dt})"
                );
            }

            // 8a. 平空信号
            if op == Operate::平空 {
                self.pos = 0;
                self.push_operate(dt, bid, price, Operate::平空, &op_desc);
            }

            // 8b. 空头止损
            if let Some(last_price) = self.last_event.price
                && 1.0 - price / last_price < -(self.stop_loss as f64) / 10000.0
                && self.pos != 0
            {
                self.pos = 0;
                self.push_operate(
                    dt,
                    bid,
                    price,
                    Operate::平空,
                    &format!("平空@{}BP止损", self.stop_loss),
                );
            }

            // 8c. 空头超时
            if let Some(last_bid) = self.last_event.bid
                && bid - last_bid > self.timeout
                && self.pos != 0
            {
                self.pos = 0;
                self.push_operate(
                    dt,
                    bid,
                    price,
                    Operate::平空,
                    &format!("平空@{}K超时", self.timeout),
                );
            }
        }

        // 9. 记录持仓快照
        self.holds.push(持仓记录 {
            dt,
            pos: self.pos,
            price,
        });

        Ok(())
    }

    // ========================================================================
    // pairs — 开平配对
    // ========================================================================

    /// 从 operates 列表计算开平配对。
    ///
    /// 遍历配对相邻操作（op1, op2），其中 op1 是开仓（LO/SO），op2 是平仓。
    /// 盈亏比例单位为 BP (1BP = 0.0001)。
    pub fn pairs(&self) -> Vec<开平配对> {
        let mut result = Vec::new();
        for pair in self.operates.windows(2) {
            let op1 = &pair[0];
            let op2 = &pair[1];
            if !matches!(op1.op, Operate::开多 | Operate::开空) {
                continue;
            }
            let ykr = if op1.op == Operate::开多 {
                op2.price / op1.price - 1.0
            } else {
                1.0 - op2.price / op1.price
            };
            let 持仓天数 = (op2.dt - op1.dt) as f64 / (24.0 * 3600.0);
            result.push(开平配对 {
                标的代码: self.symbol.clone(),
                策略标记: self.name.clone(),
                交易方向: if op1.op == Operate::开多 {
                    "多头".to_string()
                } else {
                    "空头".to_string()
                },
                开仓时间: op1.dt,
                平仓时间: op2.dt,
                开仓价格: op1.price,
                平仓价格: op2.price,
                持仓K线数: op2.bid - op1.bid,
                事件序列: format!("{} -> {}", op1.op_desc, op2.op_desc),
                持仓天数: (持仓天数 * 100.0).round() / 100.0,
                盈亏比例: (ykr * 10000.0 * 100.0).round() / 100.0, // BP, 2 decimal places
            });
        }
        result
    }

    // ========================================================================
    // dump / load — 序列化
    // ========================================================================

    /// 序列化配置为 JSON Value。`with_data` 为 true 时附带 pairs 和 holds。
    ///
    /// 注意：Event/Factor/Signal 的序列化由 PyO3 层（EventPy.dump()）处理，
    /// 本方法仅序列化 Position 自身的字段。完整的 dict 由 PyO3 PositionPy::dump() 组装。
    pub fn dump_config(&self) -> serde_json::Value {
        serde_json::json!({
            "symbol": self.symbol,
            "name": self.name,
            "interval": self.interval,
            "timeout": self.timeout,
            "stop_loss": self.stop_loss,
            "T0": self.T0,
        })
    }

    /// 从 JSON Value 构造 Position（仅配置，不含 opens/exits/events）。
    /// opens/exits 需由调用方通过 Event 反序列化后传入。
    ///
    /// 状态字段初始化为默认值（等同于新仓）。
    pub fn load_config(raw: &serde_json::Value) -> Result<Self, String> {
        let symbol = raw["symbol"].as_str().unwrap_or("").to_string();
        let name = raw["name"].as_str().unwrap_or("").to_string();
        let interval = raw["interval"].as_i64().unwrap_or(0);
        let timeout = raw["timeout"].as_i64().unwrap_or(1000);
        let stop_loss = raw["stop_loss"].as_i64().unwrap_or(1000);
        let T0 = raw["T0"].as_bool().unwrap_or(false);
        Self::新建(
            symbol,
            vec![],
            vec![],
            interval,
            timeout,
            stop_loss,
            T0,
            name,
        )
    }
}

// ============================================================================
// 内部辅助函数
// ============================================================================

/// 判断两个 Unix 时间戳是否属于同一 UTC 日期。
fn 同一交易日(a: i64, b: i64) -> bool {
    const SECS_PER_DAY: i64 = 86400;
    a / SECS_PER_DAY == b / SECS_PER_DAY
}

/// 开仓间隔检查。
///
/// - `None` → 从未开仓，允许。
/// - `Some(last)` → interval == 0 不限制；或距上次超过 interval 秒。
fn 间隔检查(last_dt: Option<i64>, current_dt: i64, interval: i64) -> bool {
    match last_dt {
        None => true,
        Some(last) => interval == 0 || (current_dt - last) > interval,
    }
}

/// 是否允许对持仓进行操作。
///
/// `T0 == true` → 总是允许（日内回转）。
/// `T0 == false` → 仅当不在同一天时允许。
fn 允许操作(T0: bool, dt: i64, last_dt: Option<i64>) -> bool {
    if T0 {
        return true;
    }
    match last_dt {
        None => true,
        Some(last) => !同一交易日(dt, last),
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signal::factor::Factor;
    use crate::signal::signal::Signal;
    use crate::signal::匹配值;
    use std::collections::HashMap;

    // ---- 测试辅助 ----

    fn 开多事件() -> Event {
        let s = Signal::new("14400", "D1MO3", "中枢", "任意", "三买", "任意", 0);
        let f = Factor::新建(vec![s], vec![], vec![], "".into()).unwrap();
        Event::新建(Operate::开多, vec![f], vec![], vec![], vec![], "".into()).unwrap()
    }

    fn 平多事件() -> Event {
        let s = Signal::new("14400", "D1MO3", "中枢", "任意", "三卖", "任意", 0);
        let f = Factor::新建(vec![s], vec![], vec![], "".into()).unwrap();
        Event::新建(Operate::平多, vec![f], vec![], vec![], vec![], "".into()).unwrap()
    }

    fn 开空事件() -> Event {
        let s = Signal::new("14400", "D1MO3", "中枢", "任意", "三卖", "任意", 0);
        let f = Factor::新建(vec![s], vec![], vec![], "".into()).unwrap();
        Event::新建(Operate::开空, vec![f], vec![], vec![], vec![], "".into()).unwrap()
    }

    fn 平空事件() -> Event {
        let s = Signal::new("14400", "D1MO3", "中枢", "任意", "三买", "任意", 0);
        let f = Factor::新建(vec![s], vec![], vec![], "".into()).unwrap();
        Event::新建(Operate::平空, vec![f], vec![], vec![], vec![], "".into()).unwrap()
    }

    /// 构造一个"三买"信号字典（匹配开多/平空事件）
    fn 三买信号字典() -> HashMap<String, 匹配值> {
        let mut m = HashMap::new();
        m.insert(
            "14400_D1MO3_中枢".to_string(),
            匹配值::字符串("任意_三买_任意_0".to_string()),
        );
        m
    }

    /// 构造一个"三卖"信号字典（匹配平多/开空事件）
    fn 三卖信号字典() -> HashMap<String, 匹配值> {
        let mut m = HashMap::new();
        m.insert(
            "14400_D1MO3_中枢".to_string(),
            匹配值::字符串("任意_三卖_任意_0".to_string()),
        );
        m
    }

    /// 构造一个无匹配信号的字典（key 存在但 value 不匹配任何信号）
    fn 无匹配信号字典() -> HashMap<String, 匹配值> {
        let mut m = HashMap::new();
        m.insert(
            "14400_D1MO3_中枢".to_string(),
            匹配值::字符串("任意_无_任意_0".to_string()),
        );
        m
    }

    // ---- 构造函数测试 ----

    #[test]
    fn test_name_缺失_报错() {
        assert!(
            Position::新建(
                "btc".into(),
                vec![开多事件()],
                vec![],
                0,
                1000,
                1000,
                false,
                "".into()
            )
            .is_err()
        );
    }

    #[test]
    fn test_构造成功() {
        let p = Position::新建(
            "btc".into(),
            vec![开多事件()],
            vec![],
            0,
            1000,
            1000,
            false,
            "中枢".into(),
        )
        .unwrap();
        assert_eq!(p.name, "中枢");
        assert_eq!(p.events.len(), 1);
        assert_eq!(p.pos, 0);
        assert!(p.operates.is_empty());
        assert!(p.holds.is_empty());
    }

    #[test]
    fn test_unique_signals_去重() {
        let p = Position::新建(
            "btc".into(),
            vec![开多事件(), 开多事件()],
            vec![],
            0,
            1000,
            1000,
            false,
            "中枢".into(),
        )
        .unwrap();
        assert_eq!(p.unique_signals().len(), 1);
    }

    // ---- update 测试 ----

    #[test]
    fn test_update_空事件列表() {
        let mut p = Position::新建(
            "btc".into(),
            vec![],
            vec![],
            0,
            1000,
            1000,
            false,
            "空".into(),
        )
        .unwrap();
        p.update(1000, 50000.0, 1, &三买信号字典()).unwrap();
        assert_eq!(p.pos, 0); // op stays HO, pos unchanged
        assert_eq!(p.holds.len(), 1);
    }

    #[test]
    fn test_update_时间倒退_跳过() {
        let mut p = Position::新建(
            "btc".into(),
            vec![开多事件()],
            vec![],
            0,
            1000,
            1000,
            false,
            "测试".into(),
        )
        .unwrap();
        // First update at t=2000
        p.update(2000, 50000.0, 2, &三买信号字典()).unwrap();
        let operates_before = p.operates.len();
        let holds_before = p.holds.len();
        // Second update at earlier time → skipped
        p.update(1000, 50000.0, 1, &三买信号字典()).unwrap();
        assert_eq!(p.operates.len(), operates_before);
        assert_eq!(p.holds.len(), holds_before);
    }

    #[test]
    fn test_update_开多() {
        let mut p = Position::新建(
            "btc".into(),
            vec![开多事件()],
            vec![],
            0,
            1000,
            1000,
            false,
            "测试".into(),
        )
        .unwrap();
        p.update(1000, 50000.0, 1, &三买信号字典()).unwrap();
        assert_eq!(p.pos, 1);
        assert!(p.pos_changed);
        assert_eq!(p.operates.len(), 1);
        assert_eq!(p.operates[0].op, Operate::开多);
        assert_eq!(p.holds.len(), 1);
    }

    #[test]
    fn test_update_开多_间隔限制_interval内不重复开仓() {
        let mut p = Position::新建(
            "btc".into(),
            vec![开多事件()],
            vec![],
            3600, // interval = 1 hour
            1000,
            1000,
            false,
            "测试".into(),
        )
        .unwrap();
        // First: open long
        p.update(1000, 50000.0, 1, &三买信号字典()).unwrap();
        assert_eq!(p.pos, 1);
        assert_eq!(p.operates.len(), 1);
        // Second: within interval, already long → no new open
        p.update(2000, 51000.0, 2, &三买信号字典()).unwrap();
        assert_eq!(p.pos, 1);
        assert_eq!(p.operates.len(), 1); // no new operate
    }

    #[test]
    fn test_update_开多_间隔后允许() {
        let mut p = Position::新建(
            "btc".into(),
            vec![开多事件()],
            vec![],
            3600,
            1000,
            1000,
            false,
            "测试".into(),
        )
        .unwrap();
        p.update(1000, 50000.0, 1, &三买信号字典()).unwrap();
        // First close manually by calling with LE event
        // then re-open after interval
        // Actually, let's test: after interval expires from another position...
        // Reset position to 0, then try again after sufficient time
        p.pos = 0; // simulate close
        p.update(6000, 51000.0, 2, &三买信号字典()).unwrap();
        assert_eq!(p.pos, 1);
        assert_eq!(p.operates.len(), 2); // new open added
    }

    #[test]
    fn test_update_平多_空仓不执行() {
        let mut p = Position::新建(
            "btc".into(),
            vec![开多事件()],
            vec![平多事件()],
            0,
            1000,
            1000,
            false,
            "测试".into(),
        )
        .unwrap();
        // LE signal when pos=0 → nothing
        p.update(1000, 50000.0, 1, &三卖信号字典()).unwrap();
        assert_eq!(p.pos, 0);
        // 平多事件 matched first (三卖), op=平多, but pos=0, none of the exit blocks apply
        assert!(p.operates.is_empty());
    }

    #[test]
    fn test_update_开多后平多() {
        let mut p = Position::新建(
            "btc".into(),
            vec![开多事件()],
            vec![平多事件()],
            0,
            1000,
            1000,
            false,
            "测试".into(),
        )
        .unwrap();
        // Step 1: LO
        p.update(1000, 50000.0, 1, &三买信号字典()).unwrap();
        assert_eq!(p.pos, 1);
        assert_eq!(p.operates.len(), 1);
        // Step 2: LE (三卖 matches 平多 first)
        p.update(86600, 49000.0, 2, &三卖信号字典()).unwrap();
        // With T0=false and different day (t=1000 vs t=86600), 允许操作 = true
        assert_eq!(p.pos, 0, "pos should be 0 after exit");
        assert_eq!(p.operates.len(), 2);
        assert_eq!(p.operates[1].op, Operate::平多);
    }

    #[test]
    fn test_update_开空() {
        let mut p = Position::新建(
            "btc".into(),
            vec![开空事件()],
            vec![],
            0,
            1000,
            1000,
            false,
            "测试".into(),
        )
        .unwrap();
        p.update(1000, 50000.0, 1, &三卖信号字典()).unwrap();
        assert_eq!(p.pos, -1);
        assert!(p.pos_changed);
        assert_eq!(p.operates.len(), 1);
        assert_eq!(p.operates[0].op, Operate::开空);
    }

    #[test]
    fn test_update_止损_多头() {
        let mut p = Position::新建(
            "btc".into(),
            vec![开多事件()],
            vec![],
            0,
            1000,
            500, // stop_loss = 500 BP
            false,
            "测试".into(),
        )
        .unwrap();
        // Open long
        p.update(1000, 50000.0, 1, &三买信号字典()).unwrap();
        assert_eq!(p.pos, 1);
        // Price drops: 50000 → 47000 = -6% = -600 BP → triggers stop_loss at -500 BP
        // Different day to allow exit
        p.update(86600, 47000.0, 2, &无匹配信号字典()).unwrap();
        assert_eq!(p.pos, 0, "should be stopped out");
        assert_eq!(p.operates.len(), 2);
        assert_eq!(p.operates[1].op, Operate::平多);
        assert!(p.operates[1].op_desc.contains("止损"));
    }

    #[test]
    fn test_update_止损_空头() {
        let mut p = Position::新建(
            "btc".into(),
            vec![开空事件()],
            vec![],
            0,
            1000,
            500,
            false,
            "测试".into(),
        )
        .unwrap();
        // Open short
        p.update(1000, 50000.0, 1, &三卖信号字典()).unwrap();
        assert_eq!(p.pos, -1);
        // Price rises: 50000 → 53000 = +6% → for short: 1 - 53000/50000 = -0.06 = -600 BP → stop
        p.update(86600, 53000.0, 2, &无匹配信号字典()).unwrap();
        assert_eq!(p.pos, 0, "should be stopped out");
        assert_eq!(p.operates.len(), 2);
        assert_eq!(p.operates[1].op, Operate::平空);
        assert!(p.operates[1].op_desc.contains("止损"));
    }

    #[test]
    fn test_update_超时_多头() {
        let mut p = Position::新建(
            "btc".into(),
            vec![开多事件()],
            vec![],
            0,
            5, // timeout = 5 bars
            1000,
            false,
            "测试".into(),
        )
        .unwrap();
        p.update(1000, 50000.0, 1, &三买信号字典()).unwrap();
        assert_eq!(p.pos, 1);
        // bid jumps from 1 to 10, diff=9 > timeout=5 → exit
        p.update(86600, 50000.0, 10, &无匹配信号字典()).unwrap();
        assert_eq!(p.pos, 0, "should be timed out");
        assert!(p.operates.last().unwrap().op_desc.contains("超时"));
    }

    #[test]
    fn test_update_无匹配事件_仅追加holds() {
        let mut p = Position::新建(
            "btc".into(),
            vec![开多事件()],
            vec![],
            0,
            1000,
            1000,
            false,
            "测试".into(),
        )
        .unwrap();
        // 无匹配信号字典 → no event matches → op stays HO
        p.update(1000, 50000.0, 1, &无匹配信号字典()).unwrap();
        assert_eq!(p.pos, 0);
        assert!(p.operates.is_empty());
        assert_eq!(p.holds.len(), 1);
        assert_eq!(p.holds[0].pos, 0);
    }

    // ---- pairs 测试 ----

    #[test]
    fn test_pairs_空操作() {
        let p = Position::新建(
            "btc".into(),
            vec![开多事件()],
            vec![],
            0,
            1000,
            1000,
            false,
            "测试".into(),
        )
        .unwrap();
        assert!(p.pairs().is_empty());
    }

    #[test]
    fn test_pairs_单笔开平() {
        let mut p = Position::新建(
            "btc".into(),
            vec![开多事件()],
            vec![平多事件()],
            0,
            1000,
            1000,
            false,
            "测试".into(),
        )
        .unwrap();
        p.update(1000, 50000.0, 1, &三买信号字典()).unwrap(); // LO
        p.update(86600, 51000.0, 2, &三卖信号字典()).unwrap(); // LE
        let pairs = p.pairs();
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0].交易方向, "多头");
        assert_eq!(pairs[0].开仓价格, 50000.0);
        assert_eq!(pairs[0].平仓价格, 51000.0);
        // ykr = (51000/50000 - 1) * 10000 = 200 BP
        assert!(pairs[0].盈亏比例 > 0.0);
    }

    #[test]
    fn test_pairs_多头盈亏计算() {
        let mut p = Position::新建(
            "btc".into(),
            vec![开多事件()],
            vec![平多事件()],
            0,
            1000,
            1000,
            false,
            "测试".into(),
        )
        .unwrap();
        p.update(1000, 50000.0, 1, &三买信号字典()).unwrap(); // LO @ 50000
        p.update(86600, 48000.0, 2, &三卖信号字典()).unwrap(); // LE @ 48000
        let pairs = p.pairs();
        assert_eq!(pairs.len(), 1);
        // ykr = 48000/50000 - 1 = -0.04 → -400 BP
        assert!((pairs[0].盈亏比例 - (-400.0)).abs() < 0.1);
    }

    #[test]
    fn test_pairs_空头盈亏计算() {
        let mut p = Position::新建(
            "btc".into(),
            vec![开空事件()],
            vec![平空事件()],
            0,
            1000,
            1000,
            false,
            "测试".into(),
        )
        .unwrap();
        p.update(1000, 50000.0, 1, &三卖信号字典()).unwrap(); // SO @ 50000
        p.update(86600, 48000.0, 2, &三买信号字典()).unwrap(); // SE @ 48000
        let pairs = p.pairs();
        assert_eq!(pairs.len(), 1);
        // ykr = 1 - 48000/50000 = 0.04 → 400 BP (short profit)
        assert!((pairs[0].盈亏比例 - 400.0).abs() < 0.1);
    }

    // ---- 辅助函数测试 ----

    #[test]
    fn test_同一交易日_同一天() {
        // 2020-01-01 00:00:00 UTC = 1577836800
        // 2020-01-01 23:59:59 UTC = 1577923199
        assert!(同一交易日(1577836800, 1577923199));
    }

    #[test]
    fn test_同一交易日_不同天() {
        // 2020-01-01 23:59:59 → 2020-01-02 00:00:00
        assert!(!同一交易日(1577923199, 1577923200));
    }

    #[test]
    fn test_间隔检查_none允许() {
        assert!(间隔检查(None, 1000, 3600));
    }

    #[test]
    fn test_间隔检查_interval0不限制() {
        assert!(间隔检查(Some(1000), 2000, 0));
    }

    #[test]
    fn test_间隔检查_超过interval() {
        // last=1000, interval=500 → need t > 1500
        assert!(间隔检查(Some(1000), 2000, 500));
    }

    #[test]
    fn test_间隔检查_未超过interval() {
        assert!(!间隔检查(Some(1000), 1200, 500));
    }

    #[test]
    fn test_允许操作_T0模式() {
        // T0 → always allow, even same day
        assert!(允许操作(true, 1577836800, Some(1577836800)));
    }

    #[test]
    fn test_允许操作_非T0_同一天拒绝() {
        assert!(!允许操作(false, 1577836800, Some(1577836800)));
    }

    #[test]
    fn test_允许操作_非T0_不同天允许() {
        assert!(允许操作(false, 1577923200, Some(1577836800)));
    }
}
