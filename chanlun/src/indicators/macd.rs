use serde::{Deserialize, Serialize};

/// 平滑异同移动平均线 (MACD)
///
/// 使用 EMA 递推算法进行增量计算
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct 平滑异同移动平均线 {
    pub 时间戳: i64,
    pub 收盘价: f64,
    pub 快线周期: i64,
    pub 慢线周期: i64,
    pub 信号周期: i64,
    pub DIF: Option<f64>,
    pub DEA: Option<f64>,
    #[serde(rename = "MACD柱")]
    #[serde(default)]
    pub MACD柱: f64,
    pub 快线EMA: Option<f64>,
    pub 慢线EMA: Option<f64>,
    pub DEA_EMA: Option<f64>,
}

impl Default for 平滑异同移动平均线 {
    fn default() -> Self {
        Self {
            时间戳: 0,
            收盘价: 0.0,
            快线周期: 12,
            慢线周期: 26,
            信号周期: 9,
            DIF: None,
            DEA: None,
            MACD柱: 0.0,
            快线EMA: None,
            慢线EMA: None,
            DEA_EMA: None,
        }
    }
}

fn 平滑系数(周期: i64) -> f64 {
    2.0 / (周期 as f64 + 1.0)
}

impl 平滑异同移动平均线 {
    /// 首次计算 MACD 指标（无历史数据时使用）
    pub fn 首次计算(
        初始收盘价: f64,
        初始时间: i64,
        快线周期: i64,
        慢线周期: i64,
        信号周期: i64,
    ) -> Self {
        let 快线EMA = 初始收盘价;
        let 慢线EMA = 初始收盘价;
        let DIF = 快线EMA - 慢线EMA;
        let DEA_EMA = DIF;
        let MACD柱 = DIF - DEA_EMA;

        Self {
            时间戳: 初始时间,
            收盘价: 初始收盘价,
            快线周期,
            慢线周期,
            信号周期,
            DIF: Some(DIF),
            DEA: Some(DEA_EMA),
            MACD柱,
            快线EMA: Some(快线EMA),
            慢线EMA: Some(慢线EMA),
            DEA_EMA: Some(DEA_EMA),
        }
    }

    /// 基于前一个 MACD 指标增量计算当前 MACD
    pub fn 增量计算(前一个MACD: &Self, 当前收盘价: f64, 当前时间: i64) -> Self {
        // 快线 EMA
        let 快线EMA = match 前一个MACD.快线EMA {
            Some(prev) => {
                当前收盘价 * 平滑系数(前一个MACD.快线周期)
                    + prev * ((前一个MACD.快线周期 - 1) as f64 / (前一个MACD.快线周期 + 1) as f64)
            }
            None => 当前收盘价,
        };

        // 慢线 EMA
        let 慢线EMA = match 前一个MACD.慢线EMA {
            Some(prev) => {
                当前收盘价 * 平滑系数(前一个MACD.慢线周期)
                    + prev * ((前一个MACD.慢线周期 - 1) as f64 / (前一个MACD.慢线周期 + 1) as f64)
            }
            None => 当前收盘价,
        };

        // DIF
        let DIF = 快线EMA - 慢线EMA;

        // DEA_EMA
        let DEA_EMA = match 前一个MACD.DEA_EMA {
            Some(prev) => {
                DIF * 平滑系数(前一个MACD.信号周期)
                    + prev * ((前一个MACD.信号周期 - 1) as f64 / (前一个MACD.信号周期 + 1) as f64)
            }
            None => DIF,
        };

        // MACD 柱 (注意: Python 版没有 ×2)
        let MACD柱 = DIF - DEA_EMA;

        Self {
            时间戳: 当前时间,
            收盘价: 当前收盘价,
            快线周期: 前一个MACD.快线周期,
            慢线周期: 前一个MACD.慢线周期,
            信号周期: 前一个MACD.信号周期,
            DIF: Some(DIF),
            DEA: Some(DEA_EMA),
            MACD柱,
            快线EMA: Some(快线EMA),
            慢线EMA: Some(慢线EMA),
            DEA_EMA: Some(DEA_EMA),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_first_calc() {
        let macd = 平滑异同移动平均线::首次计算(100.0, 1000, 12, 26, 9);
        assert_eq!(macd.DIF, Some(0.0));
        assert_eq!(macd.MACD柱, 0.0);
        assert_eq!(macd.快线EMA, Some(100.0));
        assert_eq!(macd.慢线EMA, Some(100.0));
    }

    #[test]
    fn test_incremental_calc() {
        let first = 平滑异同移动平均线::首次计算(100.0, 1000, 12, 26, 9);

        // 价格上升
        let second = 平滑异同移动平均线::增量计算(&first, 102.0, 1001);
        assert!(second.DIF.unwrap() > 0.0);
        // 快线EMA 应该比慢线EMA 变化更快
        assert!(second.快线EMA.unwrap() > second.慢线EMA.unwrap());

        // 价格下降
        let third = 平滑异同移动平均线::增量计算(&second, 98.0, 1002);
        // DIF 应该变小
        assert!(third.DIF.unwrap() < second.DIF.unwrap());
    }
}
