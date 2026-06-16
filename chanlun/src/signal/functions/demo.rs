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

//! 示例信号函数 — 移植自 `chanlun-py/chanlun/signals/demo.py`。
//!
//! 第三方代码声明：信号函数模式参考 czsc（https://github.com/waditu/czsc，
//! Apache License 2.0），已适配为 Rust。

use std::collections::HashMap;

use serde_json::Value;

use chanlun_signal_macros::signal;

use crate::business::observer::观察者;
use crate::kline::bar::K线;
use crate::signal::Signal;
use crate::signal::params;

// =============================================================================
// bar — K线形态信号
// =============================================================================

/// 涨跌停检测信号。
///
/// `close == high && close >= prev_close` → 涨停
/// `close == low && close <= prev_close` → 跌停
#[signal(name = "bar_zdt_V230331", template = "{freq}_D{di}_涨跌停V230331")]
pub fn bar_zdt_V230331(obs: &观察者, params: &HashMap<String, Value>) -> Vec<Signal> {
    let di = params::get_int(params, "di", 1) as usize;
    let freq = params::get_string(params, "freq", "15分钟");
    let k1 = freq;
    let k2 = format!("D{di}");
    let k3 = "涨跌停V230331";

    let 普K序列 = &obs.普通K线序列;
    if 普K序列.len() < di + 2 {
        return vec![Signal::new_empty(&k1, &k2, k3)];
    }

    let 当前K线 = &普K序列[普K序列.len() - di];
    let 前K线 = &普K序列[普K序列.len() - di - 1];

    let v1 = if 当前K线.收盘价 == 当前K线.高 && 当前K线.收盘价 >= 前K线.收盘价
    {
        "涨停"
    } else if 当前K线.收盘价 == 当前K线.低 && 当前K线.收盘价 <= 前K线.收盘价 {
        "跌停"
    } else {
        "任意"
    };

    if v1 == "任意" {
        vec![Signal::new_empty(&k1, &k2, k3)]
    } else {
        vec![Signal::new(&k1, &k2, k3, v1, "任意", "任意", 0)]
    }
}

// =============================================================================
// tas — 技术指标信号
// =============================================================================

/// MACD 金叉死叉信号 — DIF 与 DEA 的交叉判断。
///
/// DIF 上穿 DEA → 金叉；DIF 下穿 DEA → 死叉。
#[signal(
    name = "macd_金叉_V260601",
    template = "{freq}_D{di}#MACD#{fast}#{slow}#{signal}_MACD交叉V260601"
)]
pub fn macd_金叉_V260601(obs: &观察者, params: &HashMap<String, Value>) -> Vec<Signal> {
    let fast = params::get_int(params, "fast", 13);
    let slow = params::get_int(params, "slow", 31);
    let signal_p = params::get_int(params, "signal", 11);
    let di = params::get_int(params, "di", 1) as usize;
    let freq = params::get_string(params, "freq", "15分钟");

    let k1 = freq;
    let k2 = format!("D{di}#MACD#{fast}#{slow}#{signal_p}");
    let k3 = "MACD交叉V260601";

    obs.确保指标已计算();

    let 普K序列 = &obs.普通K线序列;
    if 普K序列.len() < di + 2 {
        return vec![Signal::new_empty(&k1, &k2, k3)];
    }

    let 当前K线 = &普K序列[普K序列.len() - di];
    let 前K线 = &普K序列[普K序列.len() - di - 1];

    let cur_dif = match 当前K线.macd().as_ref().and_then(|m| m.DIF) {
        Some(v) => v,
        None => return vec![Signal::new_empty(&k1, &k2, k3)],
    };
    let cur_dea = match 当前K线.macd().as_ref().and_then(|m| m.DEA) {
        Some(v) => v,
        None => return vec![Signal::new_empty(&k1, &k2, k3)],
    };
    let prev_dif = match 前K线.macd().as_ref().and_then(|m| m.DIF) {
        Some(v) => v,
        None => return vec![Signal::new_empty(&k1, &k2, k3)],
    };
    let prev_dea = match 前K线.macd().as_ref().and_then(|m| m.DEA) {
        Some(v) => v,
        None => return vec![Signal::new_empty(&k1, &k2, k3)],
    };

    let v1 = if prev_dif <= prev_dea && cur_dif > cur_dea {
        "金叉"
    } else if prev_dif >= prev_dea && cur_dif < cur_dea {
        "死叉"
    } else {
        "任意"
    };

    if v1 == "任意" {
        vec![Signal::new_empty(&k1, &k2, k3)]
    } else {
        vec![Signal::new(&k1, &k2, k3, v1, "任意", "任意", 0)]
    }
}

/// MACD 方向信号 — DIF 在零轴上方为多头，下方为空头。
#[signal(
    name = "tas_macd_direct_V221106",
    template = "{freq}_D{di}#MACD#{fast}#{slow}#{signal}_MACD方向V221106"
)]
pub fn tas_macd_direct_V221106(obs: &观察者, params: &HashMap<String, Value>) -> Vec<Signal> {
    let fast = params::get_int(params, "fast", 13);
    let slow = params::get_int(params, "slow", 31);
    let signal_p = params::get_int(params, "signal", 11);
    let di = params::get_int(params, "di", 1) as usize;
    let freq = params::get_string(params, "freq", "15分钟");

    let k1 = freq;
    let k2 = format!("D{di}#MACD#{fast}#{slow}#{signal_p}");
    let k3 = "MACD方向V221106";

    obs.确保指标已计算();

    let 普K序列 = &obs.普通K线序列;
    if 普K序列.len() < di + 1 {
        return vec![Signal::new_empty(&k1, &k2, k3)];
    }

    let 当前K线 = &普K序列[普K序列.len() - di];
    let cur_dif = match 当前K线.macd().as_ref().and_then(|m| m.DIF) {
        Some(v) => v,
        None => return vec![Signal::new_empty(&k1, &k2, k3)],
    };

    let v1 = if cur_dif > 0.0 { "看多" } else { "看空" };

    let v2 = if 普K序列.len() >= di + 2 {
        let 前K线 = &普K序列[普K序列.len() - di - 1];
        match 前K线.macd().as_ref().and_then(|m| m.DIF) {
            Some(prev_dif) => {
                if cur_dif > prev_dif {
                    "向上"
                } else {
                    "向下"
                }
            }
            None => "任意",
        }
    } else {
        "任意"
    };

    vec![Signal::new(&k1, &k2, k3, v1, v2, "任意", 0)]
}

// =============================================================================
// 内部辅助 — 均线按需计算
// =============================================================================

/// 按需计算均线值（SMA / EMA）。
fn 计算均线(
    普K序列: &[std::sync::Arc<K线>],
    ma_type: &str,
    timeperiod: usize,
    offset: usize,
) -> Option<f64> {
    let n = 普K序列.len();
    let start = n.checked_sub(offset + timeperiod)?;
    let end = n.checked_sub(offset)?;
    if start >= end {
        return None;
    }
    let closes: Vec<f64> = 普K序列[start..end].iter().map(|k| k.收盘价).collect();
    if closes.is_empty() {
        return None;
    }
    match ma_type {
        "SMA" | "sma" => Some(closes.iter().sum::<f64>() / closes.len() as f64),
        "EMA" | "ema" => {
            let k = 2.0 / (timeperiod as f64 + 1.0);
            let mut ema = closes[0];
            for &price in &closes[1..] {
                ema = price * k + ema * (1.0 - k);
            }
            Some(ema)
        }
        _ => None,
    }
}

/// 从均线缓存或按需计算获取均线值。
fn 获取均线(
    普K序列: &[std::sync::Arc<K线>],
    k线: &K线,
    ma_type: &str,
    timeperiod: usize,
    offset: usize,
) -> Option<f64> {
    let ma_key = format!("{}_{}", ma_type.to_uppercase(), timeperiod);
    if let Some(ma_map) = k线.ma(&ma_key) {
        return Some(ma_map);
    }
    计算均线(普K序列, ma_type, timeperiod, offset)
}

/// 单均线多空和方向信号。
#[signal(
    name = "tas_ma_base_V230313",
    template = "{freq}_D{di}#{ma_type}#{timeperiod}MO{max_overlap}_BS辅助V230313"
)]
pub fn tas_ma_base_V230313(obs: &观察者, params: &HashMap<String, Value>) -> Vec<Signal> {
    let ma_type = params::get_string(params, "ma_type", "SMA").to_uppercase();
    let timeperiod = params::get_int(params, "timeperiod", 5) as usize;
    let di = params::get_int(params, "di", 1) as usize;
    let max_overlap = params::get_int(params, "max_overlap", 5);
    let freq = params::get_string(params, "freq", "15分钟");

    let k1 = freq;
    let k2 = format!("D{di}#{ma_type}#{timeperiod}MO{max_overlap}");
    let k3 = "BS辅助V230313";

    let 普K序列 = &obs.普通K线序列;
    if 普K序列.len() < di + 1 {
        return vec![Signal::new_empty(&k1, &k2, k3)];
    }

    let 当前K线 = &普K序列[普K序列.len() - di];
    let 当前均线 = match 获取均线(普K序列, 当前K线, &ma_type, timeperiod, di) {
        Some(v) => v,
        None => return vec![Signal::new_empty(&k1, &k2, k3)],
    };

    let v1 = if 当前K线.收盘价 > 当前均线 {
        "看多"
    } else {
        "看空"
    };

    let v2 = if 普K序列.len() >= di + 2 {
        let 前K线 = &普K序列[普K序列.len() - di - 1];
        match 获取均线(普K序列, 前K线, &ma_type, timeperiod, di + 1) {
            Some(前均线) => {
                if 当前均线 > 前均线 {
                    "向上"
                } else {
                    "向下"
                }
            }
            None => "任意",
        }
    } else {
        "任意"
    };

    vec![Signal::new(&k1, &k2, k3, v1, v2, "任意", 0)]
}

// =============================================================================
// cxt — 缠论形态信号
// =============================================================================

/// 停顿分型辅助信号 — 结合分型强度和 MACD 柱子匹配判断。
#[signal(
    name = "cxt_停顿分型_V230106",
    template = "{freq}_D{di}停顿分型_BE辅助V230106"
)]
pub fn cxt_停顿分型_V230106(obs: &观察者, params: &HashMap<String, Value>) -> Vec<Signal> {
    let di = params::get_int(params, "di", 0) as usize;
    let freq = params::get_string(params, "freq", "1分钟");

    let k1 = freq;
    let k2 = format!("D{di}停顿分型");
    let k3 = "BE辅助V230106";

    let 分型序列 = &obs.分型序列;
    if 分型序列.len() < di + 1 {
        return vec![Signal::new_empty(&k1, &k2, k3)];
    }

    let 当前分型 = &分型序列[分型序列.len() - (di + 1)];

    // 只对顶/底分型产出信号
    let 结构值 = 当前分型.结构.to_string();
    if 结构值 != "顶" && 结构值 != "底" {
        return vec![Signal::new_empty(&k1, &k2, k3)];
    }

    let v1 = if 结构值 == "顶" {
        "看空"
    } else {
        "看多"
    };
    let v2 = 当前分型.强度();

    // 仅强/中分型产出有效信号
    if v2 != "强" && v2 != "中" {
        return vec![Signal::new_empty(&k1, &k2, k3)];
    }

    vec![Signal::new(&k1, &k2, k3, v1, v2, "任意", 0)]
}

/// 笔结束辅助信号 — 统计最后笔之后的新高/新低分型次数。
#[signal(
    name = "cxt_bi_end_V230222",
    template = "{freq}_D1MO{max_overlap}_BE辅助V230222"
)]
pub fn cxt_bi_end_V230222(obs: &观察者, params: &HashMap<String, Value>) -> Vec<Signal> {
    let max_overlap = params::get_int(params, "max_overlap", 3);
    let freq = params::get_string(params, "freq", "日线");

    let k1 = freq;
    let k2 = format!("D1MO{max_overlap}");
    let k3 = "BE辅助V230222";

    let 分型序列 = &obs.分型序列;
    let 笔序列 = &obs.笔序列;

    if 分型序列.len() < 2 || 笔序列.is_empty() {
        return vec![Signal::new_empty(&k1, &k2, k3)];
    }

    let 最后笔 = &笔序列[笔序列.len() - 1];
    let 当前分型 = &分型序列[分型序列.len() - 1];

    // 找到最后笔的武（终点分型）在分型序列中的位置
    let 笔武分型: std::sync::Arc<crate::structure::fractal_obj::分型> =
        { 最后笔.武.read().clone() };

    let 笔终点时间戳 = 笔武分型.时间戳;
    let 笔终点结构 = 笔武分型.结构;

    let 笔终点索引 = match 分型序列
        .iter()
        .position(|f| f.时间戳 == 笔终点时间戳 && f.结构 == 笔终点结构)
    {
        Some(idx) => idx,
        None => return vec![Signal::new_empty(&k1, &k2, k3)],
    };

    // 取笔终点之后的分型
    if 笔终点索引 + 1 >= 分型序列.len() {
        return vec![Signal::new_empty(&k1, &k2, k3)];
    }
    let 未成笔分型 = &分型序列[笔终点索引 + 1..];

    let 当前结构 = 当前分型.结构;
    let 当前分型特征值 = 当前分型.分型特征值;

    if 当前结构.to_string() == "顶" {
        let mut 笔终点顶高 = 笔武分型.分型特征值;
        let mut 计数 = 0i32;
        for f in 未成笔分型 {
            if f.结构 == 当前结构 && f.分型特征值 > 笔终点顶高 {
                计数 += 1;
                笔终点顶高 = f.分型特征值;
            }
        }
        if 计数 > 0 && 当前分型特征值 >= 笔终点顶高 {
            vec![Signal::new(
                &k1,
                &k2,
                k3,
                "新高",
                &format!("第{计数}次"),
                "任意",
                0,
            )]
        } else {
            vec![Signal::new_empty(&k1, &k2, k3)]
        }
    } else if 当前结构.to_string() == "底" {
        let mut 笔终点底低 = 笔武分型.分型特征值;
        let mut 计数 = 0i32;
        for f in 未成笔分型 {
            if f.结构 == 当前结构 && f.分型特征值 < 笔终点底低 {
                计数 += 1;
                笔终点底低 = f.分型特征值;
            }
        }
        if 计数 > 0 && 当前分型特征值 <= 笔终点底低 {
            vec![Signal::new(
                &k1,
                &k2,
                k3,
                "新低",
                &format!("第{计数}次"),
                "任意",
                0,
            )]
        } else {
            vec![Signal::new_empty(&k1, &k2, k3)]
        }
    } else {
        vec![Signal::new_empty(&k1, &k2, k3)]
    }
}
