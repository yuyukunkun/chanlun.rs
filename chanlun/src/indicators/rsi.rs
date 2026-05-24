use serde::{Deserialize, Serialize};

/// 相对强弱指数 (RSI)
///
/// 使用 Wilder 平滑（RMA）进行增量计算
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct 相对强弱指数 {
    pub 时间戳: i64,
    pub 收盘价: f64,
    pub 周期: i64,
    pub 超买阈值: f64,
    pub 超卖阈值: f64,
    pub RSI_SMA周期: Option<i64>,
    pub RSI: Option<f64>,
    pub 平均上涨: Option<f64>,
    pub 平均下跌: Option<f64>,
    pub 上涨幅度: f64,
    pub 下跌幅度: f64,
    pub 平滑系数: f64,
    pub RSI_SMA: Option<f64>,
    pub RSI历史队列: Vec<f64>,
}

impl Default for 相对强弱指数 {
    fn default() -> Self {
        Self {
            时间戳: 0,
            收盘价: 0.0,
            周期: 14,
            超买阈值: 70.0,
            超卖阈值: 30.0,
            RSI_SMA周期: None,
            RSI: None,
            平均上涨: None,
            平均下跌: None,
            上涨幅度: 0.0,
            下跌幅度: 0.0,
            平滑系数: 0.0,
            RSI_SMA: None,
            RSI历史队列: Vec::new(),
        }
    }
}

impl 相对强弱指数 {
    /// 首次计算 RSI（历史数据不足时）
    pub fn 首次计算(
        初始收盘价: f64,
        初始时间: i64,
        周期: i64,
        超买阈值: f64,
        超卖阈值: f64,
        RSI_SMA周期: Option<i64>,
    ) -> Self {
        Self {
            时间戳: 初始时间,
            收盘价: 初始收盘价,
            周期,
            超买阈值,
            超卖阈值,
            RSI_SMA周期,
            RSI: None,
            平均上涨: Some(0.0),
            平均下跌: Some(0.0),
            上涨幅度: 0.0,
            下跌幅度: 0.0,
            平滑系数: 1.0 / 周期 as f64,
            RSI_SMA: None,
            RSI历史队列: Vec::new(),
        }
    }

    /// 基于前一个 RSI 增量计算当前 RSI
    pub fn 增量计算(前一个RSI: &Self, 当前收盘价: f64, 当前时间: i64) -> Self {
        let 周期 = 前一个RSI.周期;
        let 超买阈值 = 前一个RSI.超买阈值;
        let 超卖阈值 = 前一个RSI.超卖阈值;
        let RSI_SMA周期 = 前一个RSI.RSI_SMA周期;
        let 平滑系数 = 1.0 / 周期 as f64;

        // 价格变化
        let 变化 = 当前收盘价 - 前一个RSI.收盘价;
        let 上涨 = 变化.max(0.0);
        let 下跌 = (-变化).max(0.0);

        // Wilder 平滑
        let (平均上涨, 平均下跌) = match (前一个RSI.平均上涨, 前一个RSI.平均下跌) {
            (Some(prev_up), Some(prev_down)) => {
                let avg_up = prev_up * (1.0 - 平滑系数) + 上涨 * 平滑系数;
                let avg_down = prev_down * (1.0 - 平滑系数) + 下跌 * 平滑系数;
                (avg_up, avg_down)
            }
            _ => (上涨, 下跌),
        };

        // RSI
        let RSI = if 平均下跌 == 0.0 {
            if 平均上涨 > 0.0 {
                100.0
            } else {
                50.0
            }
        } else {
            let RS = 平均上涨 / 平均下跌;
            100.0 - (100.0 / (1.0 + RS))
        };

        // RSI_SMA
        let (RSI_SMA, RSI历史队列) = match RSI_SMA周期 {
            Some(sma周期) if sma周期 > 0 => {
                let mut 队列 = 前一个RSI.RSI历史队列.clone();
                队列.push(RSI);
                if 队列.len() > sma周期 as usize {
                    队列.remove(0);
                }
                let sma = if 队列.is_empty() {
                    None
                } else {
                    Some(队列.iter().sum::<f64>() / 队列.len() as f64)
                };
                (sma, 队列)
            }
            _ => (None, Vec::new()),
        };

        Self {
            时间戳: 当前时间,
            收盘价: 当前收盘价,
            周期,
            超买阈值,
            超卖阈值,
            RSI_SMA周期,
            RSI: Some(RSI),
            平均上涨: Some(平均上涨),
            平均下跌: Some(平均下跌),
            上涨幅度: 上涨,
            下跌幅度: 下跌,
            平滑系数,
            RSI_SMA,
            RSI历史队列,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_first_calc() {
        let rsi = 相对强弱指数::首次计算(100.0, 1000, 14, 70.0, 30.0, None);
        assert_eq!(rsi.RSI, None);
        assert_eq!(rsi.平滑系数, 1.0 / 14.0);
    }

    #[test]
    fn test_incremental_calc() {
        let first = 相对强弱指数::首次计算(100.0, 1000, 14, 70.0, 30.0, None);
        let second = 相对强弱指数::增量计算(&first, 102.0, 1001);
        // 价格上涨 → RSI > 50
        assert!(second.RSI.unwrap() > 50.0);

        let third = 相对强弱指数::增量计算(&second, 98.0, 1002);
        // 价格低于之前 → RSI 下降
        assert!(third.RSI.unwrap() < second.RSI.unwrap());
    }

    #[test]
    fn test_rsi_sma() {
        let mut rsi = 相对强弱指数::首次计算(100.0, 1000, 14, 70.0, 30.0, Some(5));
        // 喂入多根K线来积累RSI历史队列
        let prices = [102.0, 103.0, 101.0, 104.0, 105.0, 103.0, 106.0];
        for (i, price) in prices.iter().enumerate() {
            rsi = 相对强弱指数::增量计算(&rsi, *price, 1001 + i as i64);
        }
        // SMA 应该已被计算（队列够长）
        assert!(rsi.RSI_SMA.is_some());
    }
}
