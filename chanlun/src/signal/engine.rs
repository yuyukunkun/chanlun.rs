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

//! 信号计算引擎 — 通过 `SIGNAL_REGISTRY` 按名查找信号函数并执行。
//!
//! 第三方代码声明：引擎架构参考 czsc 的 `信号计算器`
//!（https://github.com/waditu/czsc，Apache License 2.0），已适配为 Rust。
//!
//! # 示例
//!
//! ```ignore
//! use chanlun::signal::engine::{SignalEngine, SignalConfig, call_signal};
//!
//! let engine = SignalEngine::new(vec![SignalConfig {
//!     signal_name: "youwukuncheng_中枢第三买卖点_V230602".into(),
//!     freq: 86400,
//!     params: params_map,
//! }]);
//! engine.自动挂载指标(&analyzer);
//! let results = engine.更新(&analyzer);
//! ```

use crate::business::multi_frame::立体分析器;
use crate::business::observer::观察者;
use crate::signal::Signal;
use crate::signal::registry;
use serde_json::Value;
use std::collections::{HashMap, HashSet};

/// 单一信号配置项 — 对应 Python 信号配置列表中的一条。
#[derive(Debug, Clone)]
pub struct SignalConfig {
    /// 注册表中的信号名，如 `"youwukuncheng_中枢第三买卖点_V230602"`
    pub signal_name: String,
    /// 本配置作用的周期（秒）
    pub freq: i64,
    /// 信号参数（含 `freq`，统一为字符串以便 Rust 信号函数读取）
    pub params: HashMap<String, Value>,
}

/// 完整更新结果：信号字典 + 基础周期行情数据。
#[derive(Debug, Clone)]
pub struct 完整更新结果 {
    /// 信号 key → value 映射
    pub signals: HashMap<String, String>,
    /// 基础周期最后一根 K 线的 OHLCV 数据（若无 K 线则为 None）
    pub market: Option<MarketData>,
}

/// 基础周期行情数据 — 对应 Python `信号计算器.行情`。
#[derive(Debug, Clone)]
pub struct MarketData {
    pub symbol: String,
    pub dt: i64, // Unix 秒（K线时间戳）
    pub id: i64, // K线序号
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub vol: f64,
}

/// 信号计算引擎 — 持有配置列表，按 `&立体分析器` 执行。
///
/// 引擎不持有分析器引用——每次调用时传入，避免借用冲突。
pub struct SignalEngine {
    configs: Vec<SignalConfig>,
}

impl SignalEngine {
    /// 创建引擎。配置中的信号名延迟到 `更新()` 时校验。
    pub fn new(configs: Vec<SignalConfig>) -> Self {
        Self { configs }
    }

    /// 返回当前配置数量
    pub fn len(&self) -> usize {
        self.configs.len()
    }

    /// 配置是否为空
    pub fn is_empty(&self) -> bool {
        self.configs.is_empty()
    }

    /// 扫描信号名中的 MACD / 均线关键字，向各周期 observer 的配置中
    /// 追加缺失的指标参数，然后调用 `确保指标已计算()`（幂等）。
    ///
    /// 与 Python `_自动挂载指标()` 逻辑一致。
    pub fn 自动挂载指标(&self, analyzer: &立体分析器) {
        // 第一遍：按周期收集需要的参数
        let mut macd_by_freq: HashMap<i64, Vec<(String, i64, i64, i64)>> = HashMap::new();
        let mut ma_by_freq: HashMap<i64, Vec<(String, String, i64)>> = HashMap::new();

        for cfg in &self.configs {
            let name_lower = cfg.signal_name.to_lowercase();

            // MACD 信号检测
            if name_lower.contains("macd")
                || name_lower.contains("中枢")
                || name_lower.contains("背驰")
                || name_lower.contains("金叉")
            {
                let fast = cfg
                    .params
                    .get("fast")
                    .and_then(|v| v.as_i64())
                    .or_else(|| cfg.params.get("快线周期").and_then(|v| v.as_i64()))
                    .unwrap_or(13);
                let slow = cfg
                    .params
                    .get("slow")
                    .and_then(|v| v.as_i64())
                    .or_else(|| cfg.params.get("慢线周期").and_then(|v| v.as_i64()))
                    .unwrap_or(31);
                let signal = cfg
                    .params
                    .get("signal")
                    .and_then(|v| v.as_i64())
                    .or_else(|| cfg.params.get("信号周期").and_then(|v| v.as_i64()))
                    .unwrap_or(11);

                let key = format!("macd_{fast}_{slow}_{signal}");
                macd_by_freq
                    .entry(cfg.freq)
                    .or_default()
                    .push((key, fast, slow, signal));
            }

            // 均线信号检测
            if name_lower.contains("ma_")
                || name_lower.contains("tas_ma")
                || name_lower.contains("均线")
            {
                let ma_type = cfg
                    .params
                    .get("ma_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("SMA")
                    .to_uppercase();
                let period = cfg
                    .params
                    .get("timeperiod")
                    .and_then(|v| v.as_i64())
                    .or_else(|| cfg.params.get("周期").and_then(|v| v.as_i64()))
                    .unwrap_or(5);

                let key = format!("{ma_type}_{period}");
                ma_by_freq
                    .entry(cfg.freq)
                    .or_default()
                    .push((key, ma_type, period));
            }
        }

        // 第二遍：写入 observer 配置（先收集已有 key，再 drop 后写入）
        for (freq, entries) in &macd_by_freq {
            if let Some(obs_arc) = analyzer.获取观察者(*freq) {
                let needs_push: Vec<(String, String, i64, i64, i64)> = {
                    let obs = obs_arc.read();
                    let existing: HashSet<String> =
                        obs.配置.MACD_参数列表.iter().map(|t| t.0.clone()).collect();
                    entries
                        .iter()
                        .filter(|(key, _, _, _)| !existing.contains(key))
                        .map(|(key, fast, slow, signal)| {
                            (key.clone(), "收".to_string(), *fast, *slow, *signal)
                        })
                        .collect()
                };
                if !needs_push.is_empty() {
                    let mut obs = obs_arc.write();
                    for tuple in needs_push {
                        obs.配置.MACD_参数列表.push(tuple);
                    }
                    obs.配置.计算指标 = true;
                }
            }
        }

        for (freq, entries) in &ma_by_freq {
            if let Some(obs_arc) = analyzer.获取观察者(*freq) {
                let needs_push: Vec<(String, String, String, i64)> = {
                    let obs = obs_arc.read();
                    let existing: HashSet<String> =
                        obs.配置.均线参数列表.iter().map(|t| t.0.clone()).collect();
                    entries
                        .iter()
                        .filter(|(key, _, _)| !existing.contains(key))
                        .map(|(key, ma_type, period)| {
                            (key.clone(), "收".to_string(), ma_type.clone(), *period)
                        })
                        .collect()
                };
                if !needs_push.is_empty() {
                    let mut obs = obs_arc.write();
                    for tuple in needs_push {
                        obs.配置.均线参数列表.push(tuple);
                    }
                    obs.配置.计算指标 = true;
                }
            }
        }

        // 第三遍：确保所有周期观察者的指标已计算（幂等）
        for freq in &analyzer.周期组 {
            if let Some(obs_arc) = analyzer.获取观察者(*freq) {
                obs_arc.read().确保指标已计算();
            }
        }
    }

    /// 遍历所有配置，执行信号函数，收集非空结果。
    ///
    /// 返回 `{信号key: 信号value}` 字典（已过滤 `"任意_任意_任意_0"`）。
    /// 缺失的 observer 或未注册信号名会通过 tracing::warn! 记录并跳过。
    pub fn 更新(&self, analyzer: &立体分析器) -> HashMap<String, String> {
        let mut results: HashMap<String, String> = HashMap::new();

        for cfg in &self.configs {
            let obs_arc = match analyzer.获取观察者(cfg.freq) {
                Some(o) => o,
                None => {
                    tracing::warn!("信号引擎: 未找到周期 {} 的观察者", cfg.freq);
                    continue;
                }
            };

            let meta = match registry::get_signal(&cfg.signal_name) {
                Some(m) => m,
                None => {
                    tracing::warn!("信号引擎: 信号未注册: {}", cfg.signal_name);
                    continue;
                }
            };

            let signals = {
                let obs_guard = obs_arc.read();
                (meta.func)(&obs_guard, &cfg.params)
            };

            for sig in signals {
                if sig.value() != "任意_任意_任意_0" {
                    results.insert(sig.key(), sig.value());
                }
            }
        }

        results
    }

    /// 运行信号计算并附带基础周期行情。
    ///
    /// `base_freq` 使用分析器的第一个周期（最小周期）。
    /// 返回的 `完整更新结果` 可直接组合为 Python `信号字典` 格式。
    pub fn 更新_完整(&self, analyzer: &立体分析器) -> 完整更新结果 {
        let signals = self.更新(analyzer);

        let base_freq = analyzer.周期组.first().copied().unwrap_or(0);
        let market = analyzer.单体分析器.get(&base_freq).and_then(|obs| {
            let obs_guard = obs.read();
            obs_guard.普通K线序列.last().map(|k| MarketData {
                symbol: obs_guard.符号.clone(),
                dt: k.时间戳,
                id: k.序号,
                open: k.开盘价,
                high: k.高,
                low: k.低,
                close: k.收盘价,
                vol: k.成交量,
            })
        });

        完整更新结果 { signals, market }
    }
}

/// 按名查找并调用单个信号函数。
///
/// 适用于已有 `&观察者` 的场景（测试、单周期分析），无需构造完整的 `SignalEngine`。
pub fn call_signal(
    name: &str,
    obs: &观察者,
    params: &HashMap<String, Value>,
) -> Result<Vec<Signal>, String> {
    let meta = registry::get_signal(name).ok_or_else(|| format!("信号未注册: {name}"))?;
    Ok((meta.func)(obs, params))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::缠论配置;

    /// 通过 call_signal 调用 youwukuncheng 信号，验证产出格式。
    #[test]
    fn test_call_signal_youwukuncheng() {
        let nb_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../templates/btcusd-86400-1608854400-1781568000.nb"
        );

        let 观察员 = 观察者::new("btcusd".into(), 86400, 缠论配置::default());
        观察员
            .write()
            .读取数据文件(nb_path, 缠论配置::default().不推送())
            .expect("读取数据文件失败");

        let obs = 观察员.read();
        // 信号函数内部会调用 确保指标已计算，但为稳妥先调用一次
        obs.确保指标已计算();

        let mut params: HashMap<String, Value> = HashMap::new();
        params.insert("freq".into(), Value::String("日线".into()));
        params.insert(
            "max_overlap".into(),
            Value::Number(serde_json::Number::from(3)),
        );
        params.insert("本级完整性".into(), Value::String("实".into()));
        params.insert("同级完整性".into(), Value::String("合".into()));

        let signals = call_signal("youwukuncheng_中枢第三买卖点_V230602", &obs, &params)
            .expect("call_signal 应成功");

        assert!(!signals.is_empty(), "至少应返回一个信号（可能是空）");
        for s in &signals {
            assert!(s.k3.ends_with("V230602"), "k3 应以 V230602 结尾: {}", s.k3);
            assert!((0..=100).contains(&s.score), "score 超范围: {}", s.score);
        }

        // 验证非空信号
        let non_empty: Vec<_> = signals
            .iter()
            .filter(|s| s.value() != "任意_任意_任意_0")
            .collect();
        println!(
            "call_signal: {} signals, {} non-empty",
            signals.len(),
            non_empty.len()
        );
        for s in &non_empty {
            println!("  k3={} v1={} v2={} score={}", s.k3, s.v1, s.v2, s.score);
        }
    }

    /// 空配置返回空结果
    #[test]
    fn test_engine_空配置_返回空() {
        use crate::business::multi_frame::立体分析器;

        // 立体分析器 至少需要 2 个周期（周期组[0]=输入周期，周期组[1]=显示周期）
        let analyzer = 立体分析器::new("test".into(), vec![300, 900], None, None);
        let engine = SignalEngine::new(vec![]);
        let results = engine.更新(&analyzer);
        assert!(results.is_empty());
    }
}
