use crate::indicators::{平滑异同移动平均线, 相对强弱指数, 随机指标};
use crate::types::相对方向;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Write;
use std::rc::Rc;

/// 原始K线 (OHLCV + 指标)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct K线 {
    pub 标识: String,
    pub 序号: i64,
    pub 周期: i64,
    pub 时间戳: i64,
    pub 高: f64,
    pub 低: f64,
    pub 开盘价: f64,
    pub 收盘价: f64,
    pub 成交量: f64,
    pub macd: Option<平滑异同移动平均线>,
    pub rsi: Option<相对强弱指数>,
    pub kdj: Option<随机指标>,
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
            macd: None,
            rsi: None,
            kdj: None,
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

    /// 创建普通K线
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
            macd: None,
            rsi: None,
            kdj: None,
        }
    }

    /// 保存K线序列到 DAT 文件
    pub fn 保存到DAT文件(路径: &str, K线序列: &[&Self]) -> std::io::Result<()> {
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
            if let Some(ref macd) = k.macd {
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

    /// 截取Rc<K线>序列中从始到终的片段
    pub fn 截取rc(序列: &[Rc<Self>], 始: &Rc<Self>, 终: &Rc<Self>) -> Vec<Rc<Self>> {
        let 始_ptr = Rc::as_ptr(始);
        let 终_ptr = Rc::as_ptr(终);
        let 始_idx = 序列.iter().position(|k| Rc::as_ptr(k) == 始_ptr);
        let 终_idx = 序列.iter().position(|k| Rc::as_ptr(k) == 终_ptr);
        match (始_idx, 终_idx) {
            (Some(s), Some(e)) => 序列[s..=e].to_vec(),
            _ => Vec::new(),
        }
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
}
