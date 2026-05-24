use crate::kline::chan_kline::缠论K线;
use crate::types::分型结构;
use crate::types::相对方向;
use std::rc::Rc;

/// 分型 — 由三根缠K构成（可能缺左或右）
#[derive(Debug, Clone)]
pub struct 分型 {
    pub 左: Option<Rc<缠论K线>>,
    pub 中: Rc<缠论K线>,
    pub 右: Option<Rc<缠论K线>>,
    pub 结构: 分型结构,
    pub 时间戳: i64,
    pub 分型特征值: f64,
}

impl 分型 {
    pub fn new(左: Option<Rc<缠论K线>>, 中: Rc<缠论K线>, 右: Option<Rc<缠论K线>>) -> Self {
        let 结构 = 中.分型.unwrap_or(分型结构::散);
        let 时间戳 = 中.时间戳;
        let 分型特征值 = 中.分型特征值;
        Self {
            左,
            中,
            右,
            结构,
            时间戳,
            分型特征值,
        }
    }

    /// 左中右三组关系
    pub fn 关系组(&self) -> Option<(相对方向, 相对方向, 相对方向)> {
        let 左 = self.左.as_ref()?;
        let 右 = self.右.as_ref()?;
        Some((
            相对方向::分析(左.高, 左.低, self.中.高, self.中.低),
            相对方向::分析(self.中.高, self.中.低, 右.高, 右.低),
            相对方向::分析(左.高, 左.低, 右.高, 右.低),
        ))
    }

    /// 分型强度
    pub fn 强度(&self) -> f64 {
        match self.结构 {
            分型结构::顶 | 分型结构::上 => {
                if let Some(ref 左) = self.左 {
                    self.中.高 - 左.高
                } else {
                    0.0
                }
            }
            分型结构::底 | 分型结构::下 => {
                if let Some(ref 左) = self.左 {
                    左.低 - self.中.低
                } else {
                    0.0
                }
            }
            分型结构::散 => 0.0,
        }
    }

    /// MACD柱子分型匹配
    pub fn 与MACD柱子分型匹配(&self) -> bool {
        match self.结构 {
            分型结构::顶 | 分型结构::上 => {
                if let Some(ref 左) = self.左 {
                    左.与MACD柱子匹配()
                } else {
                    false
                }
            }
            分型结构::底 | 分型结构::下 => {
                if let Some(ref 右) = self.右 {
                    右.与MACD柱子匹配()
                } else {
                    false
                }
            }
            分型结构::散 => false,
        }
    }

    /// 判断两个分型是否匹配
    pub fn 判断分型(左: &分型, 右: &分型, 模式: &str) -> bool {
        match 模式 {
            "中" => 左.中.序号 == 右.中.序号,
            _ => false,
        }
    }

    /// 从缠K序列中获取以指定缠K为中元素的分型
    pub fn 从缠K序列中获取分型(K线序列: &[Rc<缠论K线>], 中: &Rc<缠论K线>) -> Option<Self> {
        let idx = K线序列.iter().position(|k| Rc::as_ptr(k) == Rc::as_ptr(中))?;
        let 左 = if idx > 0 {
            Some(Rc::clone(&K线序列[idx - 1]))
        } else {
            None
        };
        let 右 = if idx + 1 < K线序列.len() {
            Some(Rc::clone(&K线序列[idx + 1]))
        } else {
            None
        };
        Some(Self::new(左, Rc::clone(中), 右))
    }

    /// 向分型序列中添加新分型
    pub fn 向序列中添加(分型序列: &mut Vec<Rc<分型>>, 当前分型: Rc<分型>) {
        if let Some(前一个) = 分型序列.last() {
            if 前一个.时间戳 == 当前分型.时间戳 {
                // 同一时间戳: 比较强度，保留更强的
                match 当前分型.结构 {
                    分型结构::顶 | 分型结构::上 => {
                        if 当前分型.分型特征值 >= 前一个.分型特征值 {
                            分型序列.pop();
                            分型序列.push(当前分型);
                        }
                        return;
                    }
                    分型结构::底 | 分型结构::下 => {
                        if 当前分型.分型特征值 <= 前一个.分型特征值 {
                            分型序列.pop();
                            分型序列.push(当前分型);
                        }
                        return;
                    }
                    分型结构::散 => {}
                }
            }
            // 相同结构只保留更强的
            if 前一个.结构 == 当前分型.结构 {
                if (当前分型.结构 == 分型结构::顶 || 当前分型.结构 == 分型结构::上)
                    && 当前分型.分型特征值 >= 前一个.分型特征值
                {
                    分型序列.pop();
                    分型序列.push(当前分型);
                } else if (当前分型.结构 == 分型结构::底 || 当前分型.结构 == 分型结构::下)
                    && 当前分型.分型特征值 <= 前一个.分型特征值
                {
                    分型序列.pop();
                    分型序列.push(当前分型);
                }
                return;
            }
        }
        分型序列.push(当前分型);
    }
}

impl crate::types::fractal::有高低 for 分型 {
    fn 高(&self) -> f64 {
        self.中.高
    }
    fn 低(&self) -> f64 {
        self.中.低
    }
}

impl std::fmt::Display for 分型 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}<{}, {}, None: {}, None: {}>",
            self.中.分型.unwrap_or(crate::types::分型结构::散),
            self.时间戳,
            crate::utils::format_f64_g(self.分型特征值),
            if self.左.is_none() { "True" } else { "False" },
            if self.右.is_none() { "True" } else { "False" },
        )
    }
}
