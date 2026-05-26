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

use serde::{Deserialize, Serialize};

/// 相对方向 —— K线之间的相对位置关系
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum 相对方向 {
    #[serde(rename = "交叠向上")]
    向上,
    #[serde(rename = "交叠向下")]
    向下,
    #[serde(rename = "向上缺口")]
    向上缺口,
    #[serde(rename = "向下缺口")]
    向下缺口,
    #[serde(rename = "衔接向上")]
    衔接向上,
    #[serde(rename = "衔接向下")]
    衔接向下,
    #[serde(rename = "顺序包含")]
    顺,
    #[serde(rename = "逆序包含")]
    逆,
    #[serde(rename = "相同包含")]
    同,
}

impl 相对方向 {
    pub fn 翻转(&self) -> Self {
        match self {
            Self::向上 => Self::向下,
            Self::向下 => Self::向上,
            Self::向上缺口 => Self::向下缺口,
            Self::向下缺口 => Self::向上缺口,
            Self::衔接向上 => Self::衔接向下,
            Self::衔接向下 => Self::衔接向上,
            Self::顺 => Self::逆,
            Self::逆 => Self::顺,
            other => *other,
        }
    }

    pub fn 是否向上(&self) -> bool {
        matches!(self, Self::向上 | Self::向上缺口 | Self::衔接向上)
    }

    pub fn 是否向下(&self) -> bool {
        matches!(self, Self::向下 | Self::向下缺口 | Self::衔接向下)
    }

    pub fn 是否包含(&self) -> bool {
        matches!(self, Self::顺 | Self::逆 | Self::同)
    }

    pub fn 是否缺口(&self) -> bool {
        matches!(self, Self::向下缺口 | Self::向上缺口)
    }

    pub fn 是否衔接(&self) -> bool {
        matches!(self, Self::衔接向下 | Self::衔接向上)
    }

    /// 分析两个K线之间的相对方向
    pub fn 分析(前高: f64, 前低: f64, 后高: f64, 后低: f64) -> Self {
        // NaN 值无法判断方向，视为"同"避免 panic
        if 前高.is_nan() || 前低.is_nan() || 后高.is_nan() || 后低.is_nan() {
            return Self::同;
        }
        if 前高 == 后高 && 前低 == 后低 {
            return Self::同;
        }
        if 前高 > 后高 && 前低 > 后低 {
            if 前低 == 后高 {
                return Self::衔接向下;
            }
            if 前低 > 后高 {
                return Self::向下缺口;
            }
            return Self::向下;
        }
        if 前高 < 后高 && 前低 < 后低 {
            if 前高 == 后低 {
                return Self::衔接向上;
            }
            if 前高 < 后低 {
                return Self::向上缺口;
            }
            return Self::向上;
        }
        if 前高 >= 后高 && 前低 <= 后低 {
            return Self::顺;
        }
        if 前高 <= 后高 && 前低 >= 后低 {
            return Self::逆;
        }
        panic!(
            "无法识别的方向: 前({},{}), 后({},{})",
            前高, 前低, 后高, 后低
        );
    }
}

impl std::fmt::Display for 相对方向 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::向上 => write!(f, "相对方向.向上"),
            Self::向下 => write!(f, "相对方向.向下"),
            Self::向上缺口 => write!(f, "相对方向.向上缺口"),
            Self::向下缺口 => write!(f, "相对方向.向下缺口"),
            Self::衔接向上 => write!(f, "相对方向.衔接向上"),
            Self::衔接向下 => write!(f, "相对方向.衔接向下"),
            Self::顺 => write!(f, "相对方向.顺"),
            Self::逆 => write!(f, "相对方向.逆"),
            Self::同 => write!(f, "相对方向.同"),
        }
    }
}
