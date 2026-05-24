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
        if (前高 - 后高).abs() < f64::EPSILON && (前低 - 后低).abs() < f64::EPSILON {
            return Self::同;
        }
        if 前高 > 后高 && 前低 > 后低 {
            if (前低 - 后高).abs() < f64::EPSILON {
                return Self::衔接向下;
            }
            if 前低 > 后高 {
                return Self::向下缺口;
            }
            return Self::向下;
        }
        if 前高 < 后高 && 前低 < 后低 {
            if (前高 - 后低).abs() < f64::EPSILON {
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
        panic!("无法识别的方向: 前({},{}), 后({},{})", 前高, 前低, 后高, 后低);
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
