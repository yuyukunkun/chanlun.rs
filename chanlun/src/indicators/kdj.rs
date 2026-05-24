use serde::{Deserialize, Serialize};

/// 随机指标 (KDJ)
///
/// 使用滑动窗口 + 逐值平滑进行增量计算
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct 随机指标 {
    pub 时间戳: i64,
    pub 最高价: f64,
    pub 最低价: f64,
    pub 收盘价: f64,
    pub N: i64,
    pub M1: i64,
    pub M2: i64,
    pub 超买阈值: f64,
    pub 超卖阈值: f64,
    pub RSV: Option<f64>,
    pub K: Option<f64>,
    pub D: Option<f64>,
    pub J: Option<f64>,
    pub 历史最高价队列: Vec<f64>,
    pub 历史最低价队列: Vec<f64>,
    pub 前一个RSV: Option<f64>,
    pub 前一个K: Option<f64>,
    pub 前一个D: Option<f64>,
}

impl Default for 随机指标 {
    fn default() -> Self {
        Self {
            时间戳: 0,
            最高价: 0.0,
            最低价: 0.0,
            收盘价: 0.0,
            N: 9,
            M1: 3,
            M2: 3,
            超买阈值: 80.0,
            超卖阈值: 20.0,
            RSV: None,
            K: None,
            D: None,
            J: None,
            历史最高价队列: Vec::new(),
            历史最低价队列: Vec::new(),
            前一个RSV: None,
            前一个K: None,
            前一个D: None,
        }
    }
}

impl 随机指标 {
    /// 首次计算 KDJ（无历史数据时）
    pub fn 首次计算(
        初始最高价: f64,
        初始最低价: f64,
        初始收盘价: f64,
        初始时间: i64,
        N: i64,
        M1: i64,
        M2: i64,
        超买阈值: f64,
        超卖阈值: f64,
    ) -> Self {
        Self {
            时间戳: 初始时间,
            最高价: 初始最高价,
            最低价: 初始最低价,
            收盘价: 初始收盘价,
            N,
            M1,
            M2,
            超买阈值,
            超卖阈值,
            RSV: None,
            K: None,
            D: None,
            J: None,
            历史最高价队列: vec![初始最高价],
            历史最低价队列: vec![初始最低价],
            前一个RSV: None,
            前一个K: None,
            前一个D: None,
        }
    }

    /// 基于前一个 KDJ 增量计算当前 KDJ
    pub fn 增量计算(
        前一个KDJ: &Self,
        当前最高价: f64,
        当前最低价: f64,
        当前收盘价: f64,
        当前时间: i64,
    ) -> Self {
        let N = 前一个KDJ.N;
        let M1 = 前一个KDJ.M1;
        let M2 = 前一个KDJ.M2;
        let 超买阈值 = 前一个KDJ.超买阈值;
        let 超卖阈值 = 前一个KDJ.超卖阈值;

        // 更新历史最高价队列
        let mut 历史最高价 = 前一个KDJ.历史最高价队列.clone();
        历史最高价.push(当前最高价);
        if 历史最高价.len() > N as usize {
            历史最高价.remove(0);
        }

        // 更新历史最低价队列
        let mut 历史最低价 = 前一个KDJ.历史最低价队列.clone();
        历史最低价.push(当前最低价);
        if 历史最低价.len() > N as usize {
            历史最低价.remove(0);
        }

        // RSV
        let RSV = if 历史最高价.len() == N as usize && 历史最低价.len() == N as usize {
            let highest = 历史最高价.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let lowest = 历史最低价.iter().cloned().fold(f64::INFINITY, f64::min);
            if (highest - lowest).abs() > f64::EPSILON {
                Some((当前收盘价 - lowest) / (highest - lowest) * 100.0)
            } else {
                Some(50.0)
            }
        } else {
            None
        };

        // K值
        let K = match RSV {
            Some(rsv) => match 前一个KDJ.K {
                None => Some(rsv),
                Some(prev_k) => Some((prev_k * (M1 - 1) as f64 + rsv) / M1 as f64),
            },
            None => 前一个KDJ.K,
        };

        // D值
        let D = match K {
            Some(k) => match 前一个KDJ.D {
                None => Some(k),
                Some(prev_d) => Some((prev_d * (M2 - 1) as f64 + k) / M2 as f64),
            },
            None => 前一个KDJ.D,
        };

        // J值
        let J = match (K, D) {
            (Some(k), Some(d)) => Some(3.0 * k - 2.0 * d),
            _ => None,
        };

        Self {
            时间戳: 当前时间,
            最高价: 当前最高价,
            最低价: 当前最低价,
            收盘价: 当前收盘价,
            N,
            M1,
            M2,
            超买阈值,
            超卖阈值,
            RSV,
            K,
            D,
            J,
            历史最高价队列: 历史最高价,
            历史最低价队列: 历史最低价,
            前一个RSV: RSV,
            前一个K: K,
            前一个D: D,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_first_calc() {
        let kdj = 随机指标::首次计算(110.0, 90.0, 100.0, 1000, 9, 3, 3, 80.0, 20.0);
        assert_eq!(kdj.历史最高价队列, vec![110.0]);
        assert_eq!(kdj.历史最低价队列, vec![90.0]);
        assert_eq!(kdj.K, None);
    }

    #[test]
    fn test_incremental_after_n_bars() {
        let mut kdj = 随机指标::首次计算(110.0, 90.0, 100.0, 1000, 5, 3, 3, 80.0, 20.0);

        // 喂入足够数据填充窗口
        let data = [
            (112.0, 91.0, 105.0),
            (115.0, 93.0, 110.0),
            (113.0, 95.0, 108.0),
            (116.0, 98.0, 112.0),
            (118.0, 100.0, 115.0),
        ];
        for (i, (高, 低, 收)) in data.iter().enumerate() {
            kdj = 随机指标::增量计算(&kdj, *高, *低, *收, 1001 + i as i64);
        }

        // 窗口填满后 KDJ 应有值
        assert!(kdj.K.is_some());
        assert!(kdj.D.is_some());
        assert!(kdj.J.is_some());
    }
}
