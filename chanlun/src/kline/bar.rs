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

use crate::indicators::指标容器;
use crate::indicators::{布林带, 平滑异同移动平均线, 相对强弱指数, 随机指标};
use crate::info;
use crate::types::相对方向;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Write;
use std::sync::Arc;

mod rwlock_container_serde {
    use parking_lot::RwLock;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    /// Serde 序列化辅助（RwLock<指标容器> → 序列化器）
    pub fn serialize<S>(
        val: &RwLock<crate::indicators::指标容器>,
        ser: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        val.read().serialize(ser)
    }

    /// Serde 反序列化辅助（反序列化器 → RwLock<指标容器>）
    pub fn deserialize<'de, D>(de: D) -> Result<RwLock<crate::indicators::指标容器>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(RwLock::new(crate::indicators::指标容器::deserialize(
            de,
        )?))
    }
}

/// 原始K线 (OHLCV + 指标容器)
///
/// 所有指标统一通过 `指标容器` 访问。指标容器使用 RwLock 实现内部可变性，
/// 使 `计算并挂载` 能以 `&K线` 共享引用写入指标值。
#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct K线 {
    /// 品种标识（如 "btcusd"）
    pub 标识: String,
    /// K线序号（在序列中的位置）
    pub 序号: i64,
    /// 周期（秒），如 300=5分钟, 86400=日线
    pub 周期: i64,
    /// Unix 时间戳（秒）
    pub 时间戳: i64,
    /// 最高价
    pub 高: f64,
    /// 最低价
    pub 低: f64,
    /// 开盘价
    pub 开盘价: f64,
    /// 收盘价
    pub 收盘价: f64,
    /// 成交量
    pub 成交量: f64,
    /// 指标容器（MACD/RSI/KDJ/BOLL/均线等）
    #[serde(with = "rwlock_container_serde")]
    pub 指标: RwLock<指标容器>,
}

impl Default for K线 {
    fn default() -> Self {
        Self {
            标识: "bar".into(),
            序号: 0,
            周期: 60,
            时间戳: 0,
            高: 0.0,
            低: 0.0,
            开盘价: 0.0,
            收盘价: 0.0,
            成交量: 0.0,
            指标: RwLock::new(指标容器::new()),
        }
    }
}

impl Clone for K线 {
    fn clone(&self) -> Self {
        Self {
            标识: self.标识.clone(),
            序号: self.序号,
            周期: self.周期,
            时间戳: self.时间戳,
            高: self.高,
            低: self.低,
            开盘价: self.开盘价,
            收盘价: self.收盘价,
            成交量: self.成交量,
            指标: RwLock::new(self.指标.read().clone()),
        }
    }
}

impl K线 {
    /// 方向：阳（收盘 > 开盘）为向上，否则向下
    pub fn 方向(&self) -> 相对方向 {
        if self.开盘价 < self.收盘价 {
            相对方向::向上
        } else {
            相对方向::向下
        }
    }

    /// 序列化为大端字节序 48 字节
    /// 格式: >6d (时间戳, 开盘价, 高, 低, 收盘价, 成交量)
    /// TODO: 对齐 Python round(x, 8) 再序列化
    pub fn to_bytes(&self) -> [u8; 48] {
        let mut buf = [0u8; 48];
        {
            let mut writer = &mut buf[..];
            writer.write_f64::<BigEndian>(self.时间戳 as f64).unwrap();
            writer.write_f64::<BigEndian>(self.开盘价).unwrap();
            writer.write_f64::<BigEndian>(self.高).unwrap();
            writer.write_f64::<BigEndian>(self.低).unwrap();
            writer.write_f64::<BigEndian>(self.收盘价).unwrap();
            writer.write_f64::<BigEndian>(self.成交量).unwrap();
        }
        buf
    }

    /// 从大端字节序反序列化
    pub fn from_bytes(字节组: &[u8], 周期: i64, 标识: &str) -> Option<Self> {
        if 字节组.len() < 48 {
            return None;
        }
        let mut reader = &字节组[..48];
        let 时间戳 = reader.read_f64::<BigEndian>().ok()? as i64;
        let 开盘价 = reader.read_f64::<BigEndian>().ok()?;
        let 高 = reader.read_f64::<BigEndian>().ok()?;
        let 低 = reader.read_f64::<BigEndian>().ok()?;
        let 收盘价 = reader.read_f64::<BigEndian>().ok()?;
        let 成交量 = reader.read_f64::<BigEndian>().ok()?;

        Some(Self {
            时间戳,
            开盘价,
            高,
            低,
            收盘价,
            成交量,
            周期,
            标识: 标识.to_string(),
            序号: 0,
            ..Default::default()
        })
    }

    /// 读取 .nb 文件中的所有 K线
    pub fn 读取大端字节数组(字节组: &[u8], 周期: i64, 标识: &str) -> Option<Self> {
        Self::from_bytes(字节组, 周期, 标识)
    }

    /// 解析原始数据 — 只提取时间戳+OHLCV，不构造 K线
    pub fn 解析原始数据(字节组: &[u8]) -> Option<(i64, f64, f64, f64, f64, f64)> {
        if 字节组.len() < 48 {
            return None;
        }
        let mut reader = &字节组[..48];
        let 时间戳 = reader.read_f64::<BigEndian>().ok()? as i64;
        let 开 = reader.read_f64::<BigEndian>().ok()?;
        let 高 = reader.read_f64::<BigEndian>().ok()?;
        let 低 = reader.read_f64::<BigEndian>().ok()?;
        let 收 = reader.read_f64::<BigEndian>().ok()?;
        let 量 = reader.read_f64::<BigEndian>().ok()?;
        Some((时间戳, 开, 高, 低, 收, 量))
    }

    /// 创建普通K线
    #[allow(clippy::too_many_arguments)]
    pub fn 创建普K(
        标识: &str,
        时间戳: i64,
        开盘价: f64,
        最高价: f64,
        最低价: f64,
        收盘价: f64,
        成交量: f64,
        序号: i64,
        周期: i64,
    ) -> Self {
        Self {
            标识: 标识.to_string(),
            序号,
            周期,
            时间戳,
            高: 最高价,
            低: 最低价,
            开盘价,
            收盘价,
            成交量,
            指标: RwLock::new(指标容器::new()),
        }
    }

    /// 保存K线序列到 DAT 文件
    pub fn 保存到DAT文件(路径: &str, K线序列: &[&Self]) -> std::io::Result<()> {
        info!("保存到DAT文件: {}", 路径);
        let mut f = std::fs::File::create(路径)?;
        for k in K线序列 {
            f.write_all(&k.to_bytes())?;
        }
        Ok(())
    }

    /// 获取两K线之间的 MACD 柱面积
    pub fn 获取MACD(K线序列: &[&Self], 始: &Self, 终: &Self) -> HashMap<String, f64> {
        let 始_idx = K线序列
            .iter()
            .position(|k| std::ptr::eq(*k, 始))
            .expect("获取MACD: 始K线不在序列中");
        let 终_idx = K线序列
            .iter()
            .position(|k| std::ptr::eq(*k, 终))
            .expect("获取MACD: 终K线不在序列中");
        let 基序 = &K线序列[始_idx..=终_idx];

        let mut 阳 = 0.0f64;
        let mut 阴 = 0.0f64;
        for k in 基序 {
            if let Some(macd) = k.指标.read().macd() {
                let hist = macd.MACD柱;
                if hist >= 0.0 {
                    阳 += hist;
                } else {
                    阴 += hist;
                }
            }
        }
        let 合 = 阳 + 阴;
        let mut map = HashMap::new();
        map.insert("阳".into(), 阳);
        map.insert("阴".into(), 阴);
        map.insert("合".into(), 合);
        map.insert("总".into(), 阳 + 阴.abs());
        map
    }

    /// 截取K线序列中从始到终的片段
    pub fn 截取<'a>(序列: &'a [Self], 始: &'a Self, 终: &'a Self) -> Option<&'a [Self]> {
        let 始_idx = 序列.iter().position(|k| std::ptr::eq(k, 始))?;
        let 终_idx = 序列.iter().position(|k| std::ptr::eq(k, 终))?;
        Some(&序列[始_idx..=终_idx])
    }

    /// 结构化相等校验 — 比对各字段，浮点字段使用容差比较，返回 (是否相等, 差异描述)
    pub fn 相等(&self, other: &Self, 浮点容差: f64) -> (bool, String) {
        if self.标识 != other.标识 {
            return (
                false,
                format!("K线: [标识] 不等 A={},B={}", self.标识, other.标识),
            );
        }
        if self.序号 != other.序号 {
            return (
                false,
                format!("K线: [序号] 不等 A={},B={}", self.序号, other.序号),
            );
        }
        if self.周期 != other.周期 {
            return (
                false,
                format!("K线: [周期] 不等 A={},B={}", self.周期, other.周期),
            );
        }
        if self.时间戳 != other.时间戳 {
            return (
                false,
                format!("K线: [时间戳] 不等 A={},B={}", self.时间戳, other.时间戳),
            );
        }
        let 浮点字段 = [
            ("高", self.高, other.高),
            ("低", self.低, other.低),
            ("开盘价", self.开盘价, other.开盘价),
            ("收盘价", self.收盘价, other.收盘价),
            ("成交量", self.成交量, other.成交量),
        ];
        for (名, a, b) in &浮点字段 {
            if (a - b).abs() > 浮点容差 {
                return (
                    false,
                    format!("K线: [{名}] 浮点超限 容差={浮点容差:.2e} A={a:.10},B={b:.10}"),
                );
            }
        }
        (true, "K线: 全部字段一致".into())
    }

    /// 根据当前K线和方向生成下一根K线（与 chan.py 对齐）
    pub fn 根据当前K线生成新K线(&self, 方向: 相对方向, 居中: bool) -> Self {
        let 高低差 = self.高 - self.低;
        let 偏移 = if 居中 {
            高低差 * 0.5
        } else {
            let lo = (高低差 * 0.1279) as i64;
            let hi = (高低差 * 0.883) as i64;
            if hi > lo {
                fastrand::i64(lo..=hi) as f64
            } else {
                lo as f64
            }
        };
        let 缺口偏移 = if 居中 {
            高低差 * 1.5
        } else {
            let lo = (高低差 * 1.1279) as i64;
            let hi = (高低差 * 1.883) as i64;
            if hi > lo {
                fastrand::i64(lo..=hi) as f64
            } else {
                lo as f64
            }
        };
        let (高, 低) = match 方向 {
            相对方向::向上 => (self.高 + 偏移, self.低 + 偏移),
            相对方向::向下 => (self.高 - 偏移, self.低 - 偏移),
            相对方向::向上缺口 => (self.高 + 缺口偏移, self.低 + 缺口偏移),
            相对方向::向下缺口 => (self.高 - 缺口偏移, self.低 - 缺口偏移),
            相对方向::衔接向上 => {
                let off = 高低差;
                (self.高 + off, self.高)
            }
            相对方向::衔接向下 => {
                let off = 高低差;
                (self.低, self.低 - off)
            }
            _ => (self.高, self.低),
        };
        let 小数点 = [self.开盘价, self.高, self.低, self.收盘价]
            .iter()
            .map(|v| {
                let s = format!("{v}");
                s.split('.').nth(1).map(|d| d.len()).unwrap_or(0)
            })
            .max()
            .unwrap_or(2);
        let round = |v: f64| -> f64 {
            let scale = 10_f64.powi(小数点 as i32);
            (v * scale).round() / scale
        };
        let 开 = round(低 + (高 - 低) * fastrand::f64());
        let 收 = round(低 + (高 - 低) * fastrand::f64());
        Self::创建普K(
            &self.标识,
            self.时间戳 + self.周期,
            开,
            round(高),
            round(低),
            收,
            998.0 * fastrand::f64(),
            self.序号 + 1,
            self.周期,
        )
    }

    /// 截取Arc<K线>序列中从始到终的片段
    pub fn 截取rc(序列: &[Arc<Self>], 始: &Arc<Self>, 终: &Arc<Self>) -> Vec<Arc<Self>> {
        let 始_ptr = Arc::as_ptr(始);
        let 终_ptr = Arc::as_ptr(终);
        let 始_idx = 序列.iter().position(|k| Arc::as_ptr(k) == 始_ptr);
        let 终_idx = 序列.iter().position(|k| Arc::as_ptr(k) == 终_ptr);
        match (始_idx, 终_idx) {
            (Some(s), Some(e)) => 序列[s..=e].to_vec(),
            _ => Vec::new(),
        }
    }

    // ── 便捷指标访问（封装 RwLock<指标容器> boilerplate）──

    /// 读取 MACD 指标（已计算则返回克隆，否则 None）
    pub fn macd(&self) -> Option<平滑异同移动平均线> {
        self.指标.read().macd_cloned()
    }

    /// 读取 RSI 指标
    pub fn rsi(&self) -> Option<相对强弱指数> {
        self.指标.read().rsi_cloned()
    }

    /// 读取 KDJ 指标
    pub fn kdj(&self) -> Option<随机指标> {
        self.指标.read().kdj_cloned()
    }

    /// 读取 BOLL 指标
    pub fn boll(&self) -> Option<布林带> {
        self.指标.read().boll_cloned()
    }

    /// 读取均线值，如 `ma("SMA_5")` → `Option<f64>`
    pub fn ma(&self, key: &str) -> Option<f64> {
        self.指标.read().均线().and_then(|m| m.get(key).copied())
    }
}

impl std::fmt::Display for K线 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use crate::utils::format_f64_g;
        write!(
            f,
            "{}<{}, {}, {}, {}, {}, {}, {}, {}>",
            self.标识,
            self.序号,
            self.周期,
            self.方向(),
            self.时间戳,
            format_f64_g(self.开盘价),
            format_f64_g(self.高),
            format_f64_g(self.低),
            format_f64_g(self.收盘价)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_方向() {
        let 阳 = K线::创建普K("test", 1000, 100.0, 110.0, 95.0, 105.0, 1000.0, 0, 60);
        assert_eq!(阳.方向(), 相对方向::向上);

        let 阴 = K线::创建普K("test", 1000, 105.0, 110.0, 95.0, 100.0, 1000.0, 0, 60);
        assert_eq!(阴.方向(), 相对方向::向下);
    }

    #[test]
    fn test_serialization_roundtrip() {
        let k = K线::创建普K(
            "test", 1600000000, 100.5, 110.2, 95.3, 105.7, 5000.0, 42, 60,
        );
        let bytes = k.to_bytes();
        let restored = K线::from_bytes(&bytes, 60, "test").unwrap();

        assert_eq!(restored.时间戳, 1600000000);
        assert!((restored.开盘价 - 100.5).abs() < 0.01);
        assert!((restored.高 - 110.2).abs() < 0.01);
        assert!((restored.低 - 95.3).abs() < 0.01);
        assert!((restored.收盘价 - 105.7).abs() < 0.01);
        assert!((restored.成交量 - 5000.0).abs() < 0.01);
    }

    #[test]
    fn test_获取MACD_empty() {
        let k1 = K线::default();
        let k2 = K线::default();
        let seq = vec![&k1, &k2];
        let result = K线::获取MACD(&seq, &k1, &k2);
        assert_eq!(result.get("阳"), Some(&0.0));
        assert_eq!(result.get("阴"), Some(&0.0));
        assert_eq!(result.get("总"), Some(&0.0));
    }

    // ---- 根据当前K线生成新K线 ----

    #[test]
    fn test_生成K线_居中向上() {
        let bar = K线::创建普K(
            "test", 1000, 50000.0, 50200.0, 49800.0, 50100.0, 100.0, 0, 300,
        );
        let new = bar.根据当前K线生成新K线(相对方向::向上, true);
        // 居中: 偏移 = (50200-49800)*0.5 = 200
        assert!((new.高 - 50400.0).abs() < 1.0); // 50200 + 200
        assert!((new.低 - 50000.0).abs() < 1.0); // 49800 + 200
        assert_eq!(new.序号, 1);
        assert_eq!(new.时间戳, 1300);
    }

    #[test]
    fn test_生成K线_居中向下() {
        let bar = K线::创建普K(
            "test", 1000, 50000.0, 50200.0, 49800.0, 50100.0, 100.0, 0, 300,
        );
        let new = bar.根据当前K线生成新K线(相对方向::向下, true);
        assert!((new.高 - 50000.0).abs() < 1.0); // 50200 - 200
        assert!((new.低 - 49600.0).abs() < 1.0); // 49800 - 200
    }

    #[test]
    fn test_生成K线_居中向上缺口() {
        let bar = K线::创建普K(
            "test", 1000, 50000.0, 50200.0, 49800.0, 50100.0, 100.0, 0, 300,
        );
        let new = bar.根据当前K线生成新K线(相对方向::向上缺口, true);
        // 居中缺口: 偏移 = 400*1.5 = 600
        assert!((new.高 - 50800.0).abs() < 1.0); // 50200 + 600
        assert!((new.低 - 50400.0).abs() < 1.0); // 49800 + 600
    }

    #[test]
    fn test_生成K线_衔接向上() {
        let bar = K线::创建普K(
            "test", 1000, 50000.0, 50200.0, 49800.0, 50100.0, 100.0, 0, 300,
        );
        let new = bar.根据当前K线生成新K线(相对方向::衔接向上, true);
        let 高低差 = 50200.0 - 49800.0;
        assert!((new.高 - (50200.0 + 高低差)).abs() < 1.0);
        assert!((new.低 - 50200.0).abs() < 1.0); // 衔接向上: 低 = 原高
    }

    #[test]
    fn test_生成K线_衔接向下() {
        let bar = K线::创建普K(
            "test", 1000, 50000.0, 50200.0, 49800.0, 50100.0, 100.0, 0, 300,
        );
        let new = bar.根据当前K线生成新K线(相对方向::衔接向下, true);
        assert!((new.高 - 49800.0).abs() < 1.0); // 衔接向下: 高 = 原低
    }

    #[test]
    fn test_生成K线_非居中随机范围() {
        let bar = K线::创建普K(
            "test", 1000, 50000.0, 50200.0, 49800.0, 50100.0, 100.0, 0, 300,
        );
        // 非居中：偏移在 [高低差*0.1279, 高低差*0.883] 范围内随机
        for _ in 0..20 {
            let new = bar.根据当前K线生成新K线(相对方向::向上, false);
            assert!(new.高 > bar.高, "向上：新高应高于原高");
            assert!(new.低 > bar.低, "向上：新低应高于原低");
            let 偏移 = new.高 - bar.高;
            let 高低差 = bar.高 - bar.低;
            let lo = 高低差 * 0.1279;
            let hi = 高低差 * 0.883;
            assert!(
                偏移 >= lo && 偏移 <= hi + 1.0,
                "偏移 {偏移} 应在 [{lo}, {hi}] 范围内"
            );
        }
    }

    // ---- 从序列中机选 ----

    #[test]
    fn test_从序列中机选_可重复() {
        let dirs = vec![相对方向::向上, 相对方向::向下, 相对方向::向上缺口];
        let result = 相对方向::从序列中机选(5, &dirs, true);
        assert_eq!(result.len(), 5);
        for d in &result {
            assert!(dirs.contains(d));
        }
    }

    #[test]
    fn test_从序列中机选_不可重复() {
        let dirs = vec![相对方向::向上, 相对方向::向下, 相对方向::向上缺口];
        let result = 相对方向::从序列中机选(3, &dirs, false);
        assert_eq!(result.len(), 3);
        for (i, d) in result.iter().enumerate() {
            for prev in result[..i].iter() {
                assert_ne!(prev, d, "重复方向: {:?}", d);
            }
        }
    }

    #[test]
    #[should_panic(expected = "数量超过可选方向数")]
    fn test_从序列中机选_数量超限() {
        let dirs = vec![相对方向::向上, 相对方向::向下];
        相对方向::从序列中机选(3, &dirs, false);
    }

    #[test]
    fn test_从序列中机选_空序列() {
        let result = 相对方向::从序列中机选(0, &[], true);
        assert!(result.is_empty());
    }
}
