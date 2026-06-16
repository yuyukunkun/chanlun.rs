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
    /// 增量计算所有开启的指标，将结果写入每一根 K 线。
    ///
    pub fn 计算并挂载(全序列: &[Arc<K线>], 配置: &缠论配置) {
        let n = 全序列.len();
        if n == 0 {
            return;
        }
        if !配置.计算指标 && 配置.均线参数列表.is_empty() {
            return;
        }

        // 找到第一个 MACD 缺失的 K 线索引，若全部已有则只处理最后一根
        let start = 全序列
            .iter()
            .position(|k| k.macd().is_none())
            .unwrap_or(n - 1);

        for i in start..n {
            let 当前K线 = &全序列[i];
            let 现有序列 = &全序列[..i];

            // 确保 prev guard 在写入当前K线前释放
            {
                let prev = if i > 0 {
                    Some(全序列[i - 1].指标.read())
                } else {
                    None
                };
                let prev_deref = prev.as_deref();

                if 配置.计算指标 {
                    Self::_计算MACD组(当前K线, prev_deref, 配置);
                    Self::_计算RSI组(当前K线, prev_deref, 配置);
                    Self::_计算KDJ组(当前K线, prev_deref, 配置);
                    Self::_计算BOLL组(当前K线, prev_deref, 配置);
                }
                Self::_更新均线(当前K线, 现有序列, 配置);
                // prev guard dropped here
            }
        }

        // 回填：若有新增指标参数但首K线未被本轮计算覆盖，仍需填充历史K线
        if n > 1 && start > 0 {
            Self::_回填新指标(全序列, 配置);
        }
    }

    fn _计算MACD组(当前K线: &K线, prev: Option<&指标容器>, 配置: &缠论配置) {
        for (key, 计算方式, 快, 慢, 信号) in 配置.MACD_参数列表.iter() {
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
            let val = if let Some(prev_val) = prev.and_then(|p| p.获取(key)) {
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
            当前K线.指标.write().设置(key, val.clone());
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
            let val = if let Some(prev_val) = prev.and_then(|p| p.获取(key)) {
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
            当前K线.指标.write().设置(key, val.clone());
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
        if existing_len < p {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::缠论配置;
    use crate::kline::bar::K线;
    use std::sync::Arc;

    /// 辅助：创建一根模拟 K 线
    fn 模拟K线(时间戳: i64, 开: f64, 高: f64, 低: f64, 收: f64, 量: f64) -> Arc<K线> {
        Arc::new(K线::创建普K("TEST", 时间戳, 开, 高, 低, 收, 量, 0, 300))
    }

    /// 辅助：生成连续上涨的 K 线序列（每根涨 ~1%）
    fn 生成上涨序列(n: usize, 起始时间: i64, 起始价: f64) -> Vec<Arc<K线>> {
        let mut seq = Vec::with_capacity(n);
        let mut price = 起始价;
        for i in 0..n {
            let 开 = price;
            let 收 = price * 1.005; // 上涨 0.5%
            let 高 = 收 * 1.002;
            let 低 = 开 * 0.998;
            let 量 = 1000.0 + i as f64 * 10.0;
            seq.push(模拟K线(起始时间 + i as i64 * 300, 开, 高, 低, 收, 量));
            price = 收;
        }
        seq
    }

    #[test]
    fn test_单根K线_首次计算_挂载成功() {
        let k线 = 模拟K线(1000, 100.0, 102.0, 98.0, 101.0, 500.0);
        let seq = vec![k线.clone()];
        let 配置 = 缠论配置::default();

        指标计算器::计算并挂载(&seq, &配置);

        // 单根 K 线首次计算：MACD DIF=0（EMA=SMA 初始近似），柱=0
        let m = k线.macd().expect("MACD 应已挂载");
        assert_eq!(m.DIF, Some(0.0), "首根K线 DIF 应为 0");
        assert_eq!(m.MACD柱, 0.0, "首根K线 MACD柱 应为 0");

        // RSI 首次计算后 RSI 为 None（需至少一个增量步才有值）
        // 但指标容器应已注册 RSI 槽位，boll_cloned() 返回的是字段默认值
        assert!(k线.rsi().is_some(), "RSI 结构体应已创建（即使 RSI 字段为 None）");
        assert!(k线.kdj().is_some(), "KDJ 结构体应已创建（即使 K/D 字段为 None）");
    }

    #[test]
    fn test_多根K线_增量计算_指标值递推() {
        let seq = 生成上涨序列(5, 1000, 100.0);
        let 配置 = 缠论配置::default();

        // 逐根计算（模拟流式投喂）
        for i in 0..seq.len() {
            指标计算器::计算并挂载(&seq[..=i], &配置);
        }

        // 第 5 根 K 线的 MACD DIF 应 > 0（持续上涨）
        let last = &seq[seq.len() - 1];
        let m = last.macd().expect("最后一根K线 MACD 应已挂载");
        assert!(m.DIF.unwrap() > 0.0, "上涨序列 DIF 应为正");

        // 所有 K 线均应有 MACD/RSI/KDJ
        for (i, k) in seq.iter().enumerate() {
            assert!(k.macd().is_some(), "K线[{i}] MACD 缺失");
            assert!(k.rsi().is_some(), "K线[{i}] RSI 缺失");
            assert!(k.kdj().is_some(), "K线[{i}] KDJ 缺失");
        }
    }

    #[test]
    fn test_指标未计算时_返回None() {
        let k线 = 模拟K线(1000, 100.0, 102.0, 98.0, 101.0, 500.0);
        // 未调用 计算并挂载 — 指标应为 None
        assert!(k线.macd().is_none(), "未计算时 MACD 应为 None");
        assert!(k线.rsi().is_none(), "未计算时 RSI 应为 None");
        assert!(k线.kdj().is_none(), "未计算时 KDJ 应为 None");
    }

    #[test]
    fn test_回填新指标_新增参数后历史K线也挂载() {
        let seq = 生成上涨序列(3, 1000, 100.0);
        let 配置 = 缠论配置::default();

        // 第一轮：只计算默认 macd 组
        指标计算器::计算并挂载(&seq[..=2], &配置);
        assert!(seq[2].macd().is_some());

        // 第二轮：新增一组 MACD 参数，模拟用户后期追加指标
        let mut 配置2 = 配置.clone();
        配置2.MACD_参数列表.push(("extra_macd".into(), "收".into(), 5, 10, 3));
        指标计算器::计算并挂载(&seq[..=2], &配置2);

        // 最后一根K线应同时有默认和 extra MACD
        let last = &seq[2];
        let guard = last.指标.read();
        assert!(guard.包含("macd"), "应有默认 macd");
        assert!(guard.包含("extra_macd"), "应有新指标 extra_macd");

        // 回填：第一根 K 线也应被回填 extra_macd
        assert!(seq[0].指标.read().包含("extra_macd"), "回填后首根K线应有 extra_macd");
    }

    #[test]
    fn test_多指标组_RSI_KDJ_BOLL_同时挂载() {
        let seq = 生成上涨序列(2, 1000, 100.0);
        let 配置 = 缠论配置::default();

        指标计算器::计算并挂载(&seq[..=1], &配置);

        let last = &seq[1];
        assert!(last.macd().is_some(), "MACD 应已挂载");
        assert!(last.rsi().is_some(), "RSI 应已挂载");
        assert!(last.kdj().is_some(), "KDJ 应已挂载");
        assert!(last.boll().is_some(), "BOLL 应已挂载");

        // 验证 RSI 值的范围
        let r = last.rsi().unwrap();
        if let Some(rsi_val) = r.RSI {
            assert!((0.0..=100.0).contains(&rsi_val), "RSI 应在 0~100 之间, 实际={rsi_val}");
        }

        // 验证 KDJ 值范围
        let k = last.kdj().unwrap();
        if let Some(k_val) = k.K {
            assert!((0.0..=100.0).contains(&k_val), "KDJ.K 应在 0~100 之间, 实际={k_val}");
        }

        // BOLL 上轨 >= 中轨 >= 下轨
        let b = last.boll().unwrap();
        assert!(b.上轨 >= b.中轨, "BOLL 上轨({})应 >= 中轨({})", b.上轨, b.中轨);
        assert!(b.中轨 >= b.下轨, "BOLL 中轨({})应 >= 下轨({})", b.中轨, b.下轨);
    }

    #[test]
    fn test_均线挂载() {
        let seq = 生成上涨序列(5, 1000, 100.0);
        let mut 配置 = 缠论配置::default();
        配置.均线参数列表 = vec![
            ("SMA_3".into(), "收".into(), "SMA".into(), 3),
        ];

        指标计算器::计算并挂载(&seq[..=4], &配置);

        let last = &seq[4];
        let ma_val = last.ma("SMA_3").expect("SMA_3 应已挂载");
        assert!(ma_val > 0.0, "SMA_3 应为正值");
    }

    #[test]
    fn test_观察者集成_确保指标已计算() {
        use crate::business::observer::观察者;

        let 观察员 = 观察者::new("TEST".into(), 300, 缠论配置::default());

        // 逐根投喂
        for i in 0..5 {
            let price = 100.0 * (1.0 + i as f64 * 0.01);
            观察员.write().投喂原始数据(
                1000 + i as i64 * 300, price, price * 1.02, price * 0.98, price * 1.01, 1000.0,
            );
        }

        // 确保指标已计算
        观察员.read().确保指标已计算();

        let obs = 观察员.read();
        let klines = &obs.普通K线序列;
        assert!(!klines.is_empty(), "应有K线");

        // 最后一根K线应有指标
        let last = &klines[klines.len() - 1];
        assert!(last.macd().is_some(), "观察者集成: MACD 应已挂载");
        assert!(last.rsi().is_some(), "观察者集成: RSI 应已挂载");
        assert!(last.kdj().is_some(), "观察者集成: KDJ 应已挂载");
    }

    /// 50 根 K 线后，各指标应有稳定、合理的数值（非初始默认值）。
    #[test]
    fn test_50根K线_指标值稳定合理() {
        // 模拟 50 根有涨有跌的 K 线
        let mut seq = Vec::with_capacity(50);
        let mut price = 100.0;
        let mut rng: u64 = 42;
        for i in 0..50 {
            // 简单 LCG 随机 ±2% 波动
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let change = ((rng as f64 / u64::MAX as f64) - 0.5) * 0.04; // -2% ~ +2%
            let 收 = price * (1.0 + change);
            let 开 = price;
            let 高 = 开.max(收) * (1.0 + (rng % 100) as f64 / 10000.0);
            let 低 = 开.min(收) * (1.0 - (rng % 100) as f64 / 10000.0);
            let 量 = 500.0 + (rng % 500) as f64;
            seq.push(模拟K线(1000 + i as i64 * 300, 开, 高, 低, 收, 量));
            price = 收;
        }

        let 配置 = 缠论配置::default();

        // 逐根增量计算（模拟流式管线）
        for i in 0..seq.len() {
            指标计算器::计算并挂载(&seq[..=i], &配置);
        }

        // ── 验证每根 K 线都有指标 ──
        for (i, k) in seq.iter().enumerate() {
            assert!(k.macd().is_some(), "K线[{i}] MACD 缺失");
            assert!(k.rsi().is_some(), "K线[{i}] RSI 缺失");
            assert!(k.kdj().is_some(), "K线[{i}] KDJ 缺失");
            assert!(k.boll().is_some(), "K线[{i}] BOLL 缺失");
        }

        // ── 第 50 根 K 线（最后一根）的详细校验 ──
        let last = &seq[49];

        // MACD
        let m = last.macd().unwrap();
        assert!(m.DIF.is_some(), "50根后 DIF 应有值");
        assert!(m.DEA.is_some(), "50根后 DEA 应有值");
        let dif = m.DIF.unwrap();
        let dea = m.DEA.unwrap();
        // DIF 和 DEA 不应同时为 0（50 根有波动数据 EMA 应已收敛）
        assert!(
            dif.abs() > 1e-9 || dea.abs() > 1e-9,
            "50根有波动数据 DIF/DEA 应非零, DIF={dif}, DEA={dea}"
        );
        // MACD 柱 = 2*(DIF-DEA)，数量级合理
        let bar = m.MACD柱;
        assert!(bar.is_finite(), "MACD柱 应为有限值");
        assert!(bar.abs() < 100.0, "MACD柱 不应过大, 实际={bar}");

        // RSI
        let r = last.rsi().unwrap();
        let rsi_val = r.RSI.expect("50根后 RSI 应有值");
        assert!((0.0..=100.0).contains(&rsi_val), "RSI 应在 0~100, 实际={rsi_val}");
        // 50 根随机数据 RSI 不应卡在极端值
        assert!(rsi_val > 0.1 && rsi_val < 99.9, "RSI 不应在极端值, 实际={rsi_val}");

        // KDJ
        let kdj = last.kdj().unwrap();
        let k_val = kdj.K.expect("50根后 KDJ.K 应有值");
        let d_val = kdj.D.expect("50根后 KDJ.D 应有值");
        let j_val = kdj.J.expect("50根后 KDJ.J 应有值");
        assert!((0.0..=100.0).contains(&k_val), "KDJ.K 应在 0~100, 实际={k_val}");
        assert!((0.0..=100.0).contains(&d_val), "KDJ.D 应在 0~100, 实际={d_val}");
        // J = 3K - 2D，可能略超 [0,100]
        assert!(j_val.is_finite(), "KDJ.J 应为有限值");

        // BOLL
        let b = last.boll().unwrap();
        assert!(b.上轨 > b.中轨 || b.中轨 > b.下轨,
            "50根波动数据 BOLL 带宽应 > 0, 上={:.4} 中={:.4} 下={:.4}",
            b.上轨, b.中轨, b.下轨);

        // ── 中间节点验证：第 25 根 K 线所有指标也应有值 ──
        let mid = &seq[24];
        let m25 = mid.macd().unwrap();
        assert!(m25.DIF.is_some(), "第25根 DIF 应有值");
        assert!(mid.rsi().unwrap().RSI.is_some(), "第25根 RSI 应有值");
        assert!(mid.kdj().unwrap().K.is_some(), "第25根 KDJ.K 应有值");

        // ── 印出第 50 根用于人工审查 ──
        println!(
            "=== 第 50 根 K线 指标状态 ===",
        );
        println!(
            "  MACD:  DIF={dif:.6}  DEA={dea:.6}  BAR={bar:.6}",
        );
        println!(
            "  RSI:   RSI={rsi_val:.4}",
        );
        println!(
            "  KDJ:   K={k_val:.4}  D={d_val:.4}  J={j_val:.4}",
        );
        println!(
            "  BOLL:  上={:.4}  中={:.4}  下={:.4}",
            b.上轨, b.中轨, b.下轨,
        );
    }
}
