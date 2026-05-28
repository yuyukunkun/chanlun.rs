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

use crate::config::缠论配置;
use crate::kline::bar::K线;
use crate::structure::dash_line::虚线;
use crate::types::相对方向;
use std::sync::Arc;

/// 背驰分析 — 判断进入段和离开段之间是否存在背驰
pub struct 背驰分析;

impl 背驰分析 {
    /// MACD背驰 — MACD柱状线面积背驰
    /// 方式: "总"=阳+|阴|总面积, 其他=按进入段方向选阳或阴
    pub fn MACD背驰(
        进入段: &虚线, 离开段: &虚线, K线序列: &[Arc<K线>], 方式: &str
    ) -> bool {
        let 进入MACD = Self::_获取MACD面积(
            K线序列,
            &*进入段.文.中.标的K线.read().unwrap(),
            &*进入段.武.read().unwrap().中.标的K线.read().unwrap(),
        );
        let 离开MACD = Self::_获取MACD面积(
            K线序列,
            &*离开段.文.中.标的K线.read().unwrap(),
            &*离开段.武.read().unwrap().中.标的K线.read().unwrap(),
        );

        // 计算面积（绝对值求和）
        let 进入面积 = if 方式 == "总" {
            进入MACD.总().abs()
        } else if 进入段.方向() == 相对方向::向上 {
            进入MACD.阳.abs()
        } else {
            进入MACD.阴.abs()
        };
        let 离开面积 = if 方式 == "总" {
            离开MACD.总().abs()
        } else if 进入段.方向() == 相对方向::向上 {
            离开MACD.阳.abs()
        } else {
            离开MACD.阴.abs()
        };

        离开面积 < 进入面积
    }

    /// 斜率背驰 — 价格斜率背驰
    pub fn 斜率背驰(进入段: &虚线, 离开段: &虚线) -> bool {
        let dx = (进入段.武.read().unwrap().时间戳 - 进入段.文.时间戳) as f64;
        if dx == 0.0 {
            return false;
        }
        let dy = 进入段.武.read().unwrap().分型特征值 - 进入段.文.分型特征值;
        let 进入斜率 = dy / dx;

        let dx = (离开段.武.read().unwrap().时间戳 - 离开段.文.时间戳) as f64;
        if dx == 0.0 {
            return false;
        }
        let dy = 离开段.武.read().unwrap().分型特征值 - 离开段.文.分型特征值;
        let 离开斜率 = dy / dx;

        if 进入段.方向() == 相对方向::向上 {
            离开段.高() > 进入段.高() && 离开斜率.abs() < 进入斜率.abs()
        } else {
            离开段.低() < 进入段.低() && 离开斜率.abs() < 进入斜率.abs()
        }
    }

    /// 测度背驰 — 价格时间测度背驰
    pub fn 测度背驰(进入段: &虚线, 离开段: &虚线) -> bool {
        let dx = (进入段.武.read().unwrap().时间戳 - 进入段.文.时间戳) as f64;
        let dy = 进入段.武.read().unwrap().分型特征值 - 进入段.文.分型特征值;
        let 进入测度 = (dx * dx + dy * dy).sqrt();

        let dx = (离开段.武.read().unwrap().时间戳 - 离开段.文.时间戳) as f64;
        let dy = 离开段.武.read().unwrap().分型特征值 - 离开段.文.分型特征值;
        let 离开测度 = (dx * dx + dy * dy).sqrt();

        if 进入段.方向() == 相对方向::向上 {
            离开段.高() > 进入段.高() && 离开测度.abs() < 进入测度.abs()
        } else {
            离开段.低() < 进入段.低() && 离开测度.abs() < 进入测度.abs()
        }
    }

    /// 全量背驰 — MACD + 斜率 + 测度 三者全满足
    pub fn 全量背驰(进入段: &虚线, 离开段: &虚线, 普K序列: &[Arc<K线>]) -> bool {
        Self::MACD背驰(进入段, 离开段, 普K序列, "总")
            && Self::测度背驰(进入段, 离开段)
            && Self::斜率背驰(进入段, 离开段)
    }

    /// 任意背驰 — 任一条件满足即可
    pub fn 任意背驰(进入段: &虚线, 离开段: &虚线, 普K序列: &[Arc<K线>]) -> bool {
        Self::MACD背驰(进入段, 离开段, 普K序列, "总")
            || Self::测度背驰(进入段, 离开段)
            || Self::斜率背驰(进入段, 离开段)
    }

    /// 配置背驰 — 根据配置选择判断方式
    pub fn 配置背驰(
        进入段: &虚线,
        离开段: &虚线,
        普K序列: &[Arc<K线>],
        配置: &缠论配置,
    ) -> bool {
        match (
            配置.线段内部背驰_MACD,
            配置.线段内部背驰_测度,
            配置.线段内部背驰_斜率,
        ) {
            (true, true, true) => {
                Self::MACD背驰(进入段, 离开段, 普K序列, "总")
                    && Self::测度背驰(进入段, 离开段)
                    && Self::斜率背驰(进入段, 离开段)
            }
            (false, false, false) => false,

            (true, false, true) => {
                Self::MACD背驰(进入段, 离开段, 普K序列, "总") && Self::斜率背驰(进入段, 离开段)
            }
            (false, true, false) => Self::测度背驰(进入段, 离开段),

            (true, false, false) => Self::MACD背驰(进入段, 离开段, 普K序列, "总"),
            (false, true, true) => {
                Self::测度背驰(进入段, 离开段) && Self::斜率背驰(进入段, 离开段)
            }

            (false, false, true) => Self::斜率背驰(进入段, 离开段),
            (true, true, false) => {
                Self::MACD背驰(进入段, 离开段, 普K序列, "总") && Self::测度背驰(进入段, 离开段)
            }
        }
    }

    /// 任选背驰 — 至少两个条件满足（多数投票）
    pub fn 任选背驰(进入段: &虚线, 离开段: &虚线, 普K序列: &[Arc<K线>]) -> bool {
        let 混沌槽 = [
            Self::MACD背驰(进入段, 离开段, 普K序列, "总"),
            Self::测度背驰(进入段, 离开段),
            Self::斜率背驰(进入段, 离开段),
        ];
        混沌槽.iter().filter(|&&x| x).count() >= 2
    }

    /// 背驰模式 — 根据模式字符串选择判断方式
    pub fn 背驰模式(
        进入段: &虚线,
        离开段: &虚线,
        普K序列: &[Arc<K线>],
        配置: &缠论配置,
        模式: &str,
    ) -> bool {
        match 模式 {
            "全量" => Self::全量背驰(进入段, 离开段, 普K序列),
            "任意" => Self::任意背驰(进入段, 离开段, 普K序列),
            "配置" => Self::配置背驰(进入段, 离开段, 普K序列, 配置),
            "相对" => Self::任选背驰(进入段, 离开段, 普K序列),
            _ => false,
        }
    }

    // ---- 内部辅助 ----

    fn _获取MACD面积(K线序列: &[Arc<K线>], 始: &Arc<K线>, 终: &Arc<K线>) -> MACD面积 {
        let 始_idx = K线序列
            .iter()
            .position(|k| Arc::as_ptr(k) == Arc::as_ptr(始));
        let 终_idx = K线序列
            .iter()
            .position(|k| Arc::as_ptr(k) == Arc::as_ptr(终));

        let mut 阳 = 0.0f64;
        let mut 阴 = 0.0f64;

        if let (Some(始), Some(终)) = (始_idx, 终_idx) {
            let (始, 终) = if 始 <= 终 { (始, 终) } else { (终, 始) };
            for k in &K线序列[始..=终] {
                if let Some(ref macd) = k.macd {
                    let hist = macd.MACD柱;
                    if hist >= 0.0 {
                        阳 += hist;
                    } else {
                        阴 += hist;
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

impl MACD面积 {
    fn 总(&self) -> f64 {
        self.阳 + self.阴
    }
}
