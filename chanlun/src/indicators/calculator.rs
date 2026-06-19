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

use super::container::{指标值, 指标容器};
use super::{布林带, 平滑异同移动平均线, 相对强弱指数, 随机指标};
use crate::config::缠论配置;
use crate::kline::bar::K线;
use std::sync::Arc;

/// 指标计算器 — 在缠K合并之前，增量计算所有开启的指标并挂载到K线上
pub struct 指标计算器;

impl 指标计算器 {
    /// 增量计算所有开启的指标，将结果写入 当前K线.指标
    ///
    /// `全序列` 包含当前K线（在末尾）；prev 取自 全序列[..-1].last()
    /// 通过 RwLock 内部可变性，以 `&K线` 共享引用写入指标值
    pub fn 计算并挂载(全序列: &[Arc<K线>], 配置: &缠论配置) {
        let n = 全序列.len();
        let 当前K线 = &全序列[n - 1];
        let 现有序列 = if n > 1 { &全序列[..n - 1] } else { &[] };

        // 作用域化 prev_guard：在 _回填新指标 之前释放，避免读锁与回填写锁冲突
        let has_prev;
        {
            let prev_guard = if n > 1 {
                Some(全序列[n - 2].指标.read())
            } else {
                None
            };
            let prev = prev_guard.as_deref();
            if 配置.计算指标 {
                Self::_计算MACD组(当前K线, prev, 配置);
                Self::_计算RSI组(当前K线, prev, 配置);
                Self::_计算KDJ组(当前K线, prev, 配置);
                Self::_计算BOLL组(当前K线, prev, 配置);
            }
            Self::_更新均线(当前K线, 现有序列, 配置); // 当前K线在 全序列[n-1], 现有序列不含它
            has_prev = n > 1;
        } // prev_guard dropped here

        if has_prev {
            Self::_回填新指标(全序列, 配置);
        }
    }

    fn _计算MACD组(当前K线: &K线, prev: Option<&指标容器>, 配置: &缠论配置) {
        for (i, (key, 计算方式, 快, 慢, 信号)) in 配置.MACD_参数列表.iter().enumerate()
        {
            let val = if let Some(prev_val) = prev.and_then(|p| p.获取(key)) {
                if let 指标值::MACD(prev_macd) = prev_val {
                    指标值::MACD(平滑异同移动平均线::增量计算(
                        prev_macd,
                        super::K线取值(
                            当前K线.开盘价,
                            当前K线.高,
                            当前K线.低,
                            当前K线.收盘价,
                            计算方式,
                        ),
                        当前K线.时间戳,
                    ))
                } else {
                    continue;
                }
            } else {
                指标值::MACD(平滑异同移动平均线::首次计算(
                    super::K线取值(
                        当前K线.开盘价,
                        当前K线.高,
                        当前K线.低,
                        当前K线.收盘价,
                        计算方式,
                    ),
                    当前K线.时间戳,
                    *快,
                    *慢,
                    *信号,
                ))
            };
            当前K线.指标.write().设置(key, val.clone());
        }
    }

    fn _计算RSI组(当前K线: &K线, prev: Option<&指标容器>, 配置: &缠论配置) {
        for (key, 计算方式, 周期, ma周期, 超买, 超卖) in 配置.RSI_周期列表.iter()
        {
            let val = if let Some(prev_val) = prev.and_then(|p| p.获取(&key)) {
                if let 指标值::RSI(prev_rsi) = prev_val {
                    指标值::RSI(相对强弱指数::增量计算(
                        prev_rsi,
                        super::K线取值(
                            当前K线.开盘价,
                            当前K线.高,
                            当前K线.低,
                            当前K线.收盘价,
                            计算方式,
                        ),
                        当前K线.时间戳,
                    ))
                } else {
                    continue;
                }
            } else {
                指标值::RSI(相对强弱指数::首次计算(
                    super::K线取值(
                        当前K线.开盘价,
                        当前K线.高,
                        当前K线.低,
                        当前K线.收盘价,
                        计算方式,
                    ),
                    当前K线.时间戳,
                    *周期,
                    *超买,
                    *超卖,
                    Some(*ma周期),
                ))
            };
            当前K线.指标.write().设置(&key, val.clone());
        }
    }

    fn _计算KDJ组(当前K线: &K线, prev: Option<&指标容器>, 配置: &缠论配置) {
        for (key, _fm, rsv, k平滑, d平滑, 超买, 超卖) in 配置.KDJ_参数列表.iter() {
            let val = if let Some(prev_val) = prev.and_then(|p| p.获取(key)) {
                if let 指标值::KDJ(prev_kdj) = prev_val {
                    指标值::KDJ(随机指标::增量计算(
                        prev_kdj,
                        当前K线.高,
                        当前K线.低,
                        当前K线.收盘价,
                        当前K线.时间戳,
                    ))
                } else {
                    continue;
                }
            } else {
                指标值::KDJ(随机指标::首次计算(
                    当前K线.高,
                    当前K线.低,
                    当前K线.收盘价,
                    当前K线.时间戳,
                    *rsv,
                    *k平滑,
                    *d平滑,
                    *超买,
                    *超卖,
                ))
            };
            当前K线.指标.write().设置(key, val.clone());
        }
    }

    fn _计算BOLL组(当前K线: &K线, prev: Option<&指标容器>, 配置: &缠论配置) {
        for (key, 计算方式, 周期, 标准差倍数) in 配置.BOLL_参数列表.iter() {
            let val = if let Some(prev_val) = prev.and_then(|p| p.获取(&key)) {
                if let 指标值::BOLL(prev_boll) = prev_val {
                    指标值::BOLL(布林带::增量计算(
                        prev_boll,
                        当前K线.时间戳,
                        super::K线取值(
                            当前K线.开盘价,
                            当前K线.高,
                            当前K线.低,
                            当前K线.收盘价,
                            计算方式,
                        ),
                    ))
                } else {
                    continue;
                }
            } else {
                指标值::BOLL(布林带::首次计算(
                    当前K线.时间戳,
                    super::K线取值(
                        当前K线.开盘价,
                        当前K线.高,
                        当前K线.低,
                        当前K线.收盘价,
                        计算方式,
                    ),
                    *周期 as usize,
                    *标准差倍数,
                ))
            };
            当前K线.指标.write().设置(&key, val.clone());
        }
    }

    fn _更新均线(当前K线: &K线, 现有序列: &[Arc<K线>], 配置: &缠论配置) {
        if 配置.均线参数列表.is_empty() {
            return;
        }
        for (key, 计算方式, ma_type, period) in &配置.均线参数列表 {
            let 值 = match ma_type.as_str() {
                "SMA" => Self::_增量SMA(当前K线, 现有序列, 计算方式, *period, key),
                "EMA" => Self::_增量EMA(当前K线, 现有序列, 计算方式, *period, key),
                _ => continue,
            };
            if let Some(均线_map) = 当前K线.指标.write().均线_mut() {
                均线_map.insert(key.clone(), 值);
            }
        }
    }

    fn _增量SMA(
        当前K线: &K线,
        现有序列: &[Arc<K线>],
        计算方式: &str,
        period: i64,
        prev_key: &str,
    ) -> f64 {
        let 当前价 = super::K线取值(
            当前K线.开盘价,
            当前K线.高,
            当前K线.低,
            当前K线.收盘价,
            计算方式,
        );
        let existing_len = 现有序列.len();
        let p = period as usize;
        if existing_len + 1 <= p {
            let sum: f64 = 现有序列
                .iter()
                .map(|k| super::K线取值(k.开盘价, k.高, k.低, k.收盘价, 计算方式))
                .sum::<f64>()
                + 当前价;
            return sum / ((existing_len + 1) as f64).max(1.0);
        }
        if let Some(prev_sma) = 现有序列
            .last()
            .and_then(|k| k.指标.read().均线().and_then(|m| m.get(prev_key)).copied())
        {
            let oldest = super::K线取值(
                现有序列[existing_len - p].开盘价,
                现有序列[existing_len - p].高,
                现有序列[existing_len - p].低,
                现有序列[existing_len - p].收盘价,
                计算方式,
            );
            return prev_sma + (当前价 - oldest) / period as f64;
        }
        let sum: f64 = 现有序列[existing_len.saturating_sub(p.saturating_sub(1))..]
            .iter()
            .map(|k| super::K线取值(k.开盘价, k.高, k.低, k.收盘价, 计算方式))
            .sum::<f64>()
            + 当前价;
        sum / ((existing_len + 1) as f64).min(p as f64)
    }

    fn _增量EMA(
        当前K线: &K线,
        现有序列: &[Arc<K线>],
        计算方式: &str,
        period: i64,
        prev_key: &str,
    ) -> f64 {
        let 当前价 = super::K线取值(
            当前K线.开盘价,
            当前K线.高,
            当前K线.低,
            当前K线.收盘价,
            计算方式,
        );
        let 前值 = 现有序列
            .last()
            .and_then(|k| k.指标.read().均线().and_then(|m| m.get(prev_key)).copied());
        match 前值 {
            None => 当前价,
            Some(prev) => {
                let k = 2.0 / (period as f64 + 1.0);
                当前价 * k + prev * (1.0 - k)
            }
        }
    }

    /// 运行中新增指标参数时，回填所有历史K线
    fn _回填新指标(全序列: &[Arc<K线>], 配置: &缠论配置) {
        let (新MACD, 新RSI, 新KDJ, 新BOLL) = {
            let 首K_guard = 全序列[0].指标.read();
            let 尾K_guard = 全序列[全序列.len() - 1].指标.read();
            let 新MACD: Vec<_> = 配置
                .MACD_参数列表
                .iter()
                .filter(|(key, ..)| 尾K_guard.包含(key) && !首K_guard.包含(key))
                .cloned()
                .collect();
            let 新RSI: Vec<_> = 配置
                .RSI_周期列表
                .iter()
                .filter(|(key, ..)| 尾K_guard.包含(key) && !首K_guard.包含(key))
                .cloned()
                .collect();
            let 新KDJ: Vec<_> = 配置
                .KDJ_参数列表
                .iter()
                .filter(|(key, ..)| 尾K_guard.包含(key) && !首K_guard.包含(key))
                .cloned()
                .collect();
            let 新BOLL: Vec<_> = 配置
                .BOLL_参数列表
                .iter()
                .filter(|(key, ..)| 尾K_guard.包含(key) && !首K_guard.包含(key))
                .cloned()
                .collect();
            (新MACD, 新RSI, 新KDJ, 新BOLL)
        };

        if 新MACD.is_empty() && 新RSI.is_empty() && 新KDJ.is_empty() && 新BOLL.is_empty() {
            return;
        }

        for i in 0..全序列.len() {
            let k线 = &全序列[i];
            let prev_guard = if i > 0 {
                Some(全序列[i - 1].指标.read())
            } else {
                None
            };

            for (key, 计算方式, 快, 慢, 信号) in &新MACD {
                let val = match prev_guard.as_ref().and_then(|p| p.获取(key)) {
                    Some(指标值::MACD(prev_macd)) => 指标值::MACD(
                        平滑异同移动平均线::增量计算_K线(prev_macd, k线, 计算方式),
                    ),
                    _ => 指标值::MACD(平滑异同移动平均线::首次计算_K线(
                        k线,
                        计算方式,
                        *快,
                        *慢,
                        *信号,
                    )),
                };
                k线.指标.write().设置(key, val);
            }

            for (key, 计算方式, 周期, ma周期, 超买, 超卖) in &新RSI {
                let val = match prev_guard.as_ref().and_then(|p| p.获取(key)) {
                    Some(指标值::RSI(prev_rsi)) => 指标值::RSI(
                        相对强弱指数::增量计算_K线(prev_rsi, k线, 计算方式),
                    ),
                    _ => 指标值::RSI(相对强弱指数::首次计算_K线(
                        k线,
                        计算方式,
                        *周期,
                        *超买,
                        *超卖,
                        Some(*ma周期),
                    )),
                };
                k线.指标.write().设置(key, val);
            }

            for (key, _fm, rsv, k平滑, d平滑, 超买, 超卖) in &新KDJ {
                let val = match prev_guard.as_ref().and_then(|p| p.获取(key)) {
                    Some(指标值::KDJ(prev_kdj)) => {
                        指标值::KDJ(随机指标::增量计算_K线(prev_kdj, k线))
                    }
                    _ => 指标值::KDJ(随机指标::首次计算_K线(
                        k线, *rsv, *k平滑, *d平滑, *超买, *超卖,
                    )),
                };
                k线.指标.write().设置(key, val);
            }

            for (key, 计算方式, 周期, 标准差倍数) in &新BOLL {
                let val = match prev_guard.as_ref().and_then(|p| p.获取(key)) {
                    Some(指标值::BOLL(prev_boll)) => {
                        指标值::BOLL(布林带::增量计算_K线(prev_boll, k线, 计算方式))
                    }
                    _ => 指标值::BOLL(布林带::首次计算_K线(
                        k线,
                        计算方式,
                        *周期 as usize,
                        *标准差倍数,
                    )),
                };
                k线.指标.write().设置(key, val);
            }
        }
    }
}
