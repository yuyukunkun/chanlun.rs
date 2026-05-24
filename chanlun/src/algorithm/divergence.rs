use crate::config::缠论配置;
use crate::kline::bar::K线;
use crate::structure::dash_line::虚线;
use std::rc::Rc;

/// 背驰分析 — 判断进入段和离开段之间是否存在背驰
pub struct 背驰分析;

impl 背驰分析 {
    /// MACD背驰 — 离开段MACD柱子面积小于进入段
    pub fn MACD背驰(进入段: &虚线, 离开段: &虚线, K线序列: &[Rc<K线>], 方式: &str) -> bool {
        // 获取进入段和离开段对应的K线
        let 进入段_始 = 进入段.文.中.原始起始序号 as usize;
        let 进入段_终 = 进入段.武.中.原始结束序号 as usize;
        let 离开段_始 = 离开段.文.中.原始起始序号 as usize;
        let 离开段_终 = 离开段.武.中.原始结束序号 as usize;

        if 进入段_终 >= K线序列.len() || 离开段_终 >= K线序列.len() {
            return false;
        }

        let 进入MACD = Self::柱子面积(&K线序列[进入段_始..=进入段_终], 方式);
        let 离开MACD = Self::柱子面积(&K线序列[离开段_始..=离开段_终], 方式);

        // 背驰条件: 离开段面积绝对值小于进入段
        let 进入总面积 = 进入MACD.阳 + 进入MACD.阴.abs();
        let 离开总面积 = 离开MACD.阳 + 离开MACD.阴.abs();

        if 进入总面积 < f64::EPSILON {
            return false;
        }
        离开总面积 < 进入总面积
    }

    /// 斜率背驰 — 离开段斜率小于进入段
    pub fn 斜率背驰(进入段: &虚线, 离开段: &虚线) -> bool {
        let 进入斜率 = (进入段.武.分型特征值 - 进入段.文.分型特征值).abs()
            / (进入段.武.时间戳 - 进入段.文.时间戳).abs() as f64;
        let 离开斜率 = (离开段.武.分型特征值 - 离开段.文.分型特征值).abs()
            / (离开段.武.时间戳 - 离开段.文.时间戳).abs() as f64;

        离开斜率 < 进入斜率
    }

    /// 测度背驰 — 价格测度的背驰判断
    pub fn 测度背驰(进入段: &虚线, 离开段: &虚线) -> bool {
        let 进入幅度 = (进入段.武.分型特征值 - 进入段.文.分型特征值).abs();
        let 离开幅度 = (离开段.武.分型特征值 - 离开段.文.分型特征值).abs();
        离开幅度 < 进入幅度
    }

    /// 全量背驰 — MACD + 斜率 + 测度 三者全满足
    pub fn 全量背驰(进入段: &虚线, 离开段: &虚线, 普K序列: &[Rc<K线>]) -> bool {
        Self::MACD背驰(进入段, 离开段, 普K序列, "总")
            && Self::斜率背驰(进入段, 离开段)
            && Self::测度背驰(进入段, 离开段)
    }

    /// 任意背驰 — 任一条件满足即可
    pub fn 任意背驰(进入段: &虚线, 离开段: &虚线, 普K序列: &[Rc<K线>]) -> bool {
        Self::MACD背驰(进入段, 离开段, 普K序列, "总")
            || Self::斜率背驰(进入段, 离开段)
            || Self::测度背驰(进入段, 离开段)
    }

    /// 配置背驰 — 根据配置选择判断方式
    pub fn 配置背驰(进入段: &虚线, 离开段: &虚线, 普K序列: &[Rc<K线>], 配置: &缠论配置) -> bool {
        let mut result = true;
        if 配置.线段内部背驰_MACD {
            result = result && Self::MACD背驰(进入段, 离开段, 普K序列, "总");
        }
        if 配置.线段内部背驰_斜率 {
            result = result && Self::斜率背驰(进入段, 离开段);
        }
        if 配置.线段内部背驰_测度 {
            result = result && Self::测度背驰(进入段, 离开段);
        }
        result
    }

    /// 任选背驰
    pub fn 任选背驰(进入段: &虚线, 离开段: &虚线, 普K序列: &[Rc<K线>]) -> bool {
        Self::任意背驰(进入段, 离开段, 普K序列)
    }

    /// 背驰模式 — 根据模式字符串选择判断方式
    pub fn 背驰模式(
        进入段: &虚线,
        离开段: &虚线,
        普K序列: &[Rc<K线>],
        配置: &缠论配置,
        模式: &str,
    ) -> bool {
        match 模式 {
            "全量" => Self::全量背驰(进入段, 离开段, 普K序列),
            "任意" => Self::任意背驰(进入段, 离开段, 普K序列),
            "配置" => Self::配置背驰(进入段, 离开段, 普K序列, 配置),
            "相对" => Self::测度背驰(进入段, 离开段),
            _ => Self::配置背驰(进入段, 离开段, 普K序列, 配置),
        }
    }

    // ---- 内部辅助 ----

    fn 柱子面积(K线序列: &[Rc<K线>], 方式: &str) -> MACD面积 {
        let mut 阳 = 0.0f64;
        let mut 阴 = 0.0f64;
        for k in K线序列 {
            if let Some(ref macd) = k.macd {
                let hist = macd.MACD柱;
                match 方式 {
                    "阳" => {
                        if hist > 0.0 {
                            阳 += hist
                        }
                    }
                    "阴" => {
                        if hist < 0.0 {
                            阴 += hist
                        }
                    }
                    "总" | _ => {
                        if hist >= 0.0 {
                            阳 += hist
                        } else {
                            阴 += hist
                        }
                    }
                }
            }
        }
        MACD面积 { 阳, 阴 }
    }
}

struct MACD面积 {
    阳: f64,
    阴: f64,
}
