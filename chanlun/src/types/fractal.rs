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
use tracing::warn;

/// 分型结构 —— 三根K线构成的结构形态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum 分型结构 {
    #[serde(rename = "三连向上")]
    上,
    #[serde(rename = "三连向下")]
    下,
    #[serde(rename = "顶分型")]
    顶,
    #[serde(rename = "底分型")]
    底,
    #[serde(rename = "向右扩散")]
    散,
}

impl std::fmt::Display for 分型结构 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::上 => write!(f, "上"),
            Self::下 => write!(f, "下"),
            Self::顶 => write!(f, "顶"),
            Self::底 => write!(f, "底"),
            Self::散 => write!(f, "散"),
        }
    }
}

impl 分型结构 {
    /// 分析三根K线构成的分型结构（泛型版本）
    pub fn 分析(
        左: &impl 有高低,
        中: &impl 有高低,
        右: &impl 有高低,
        可以逆序包含: bool,
        忽视顺序包含: bool,
    ) -> Option<Self> {
        Self::分析_内部(
            左.高(),
            左.低(),
            中.高(),
            中.低(),
            右.高(),
            右.低(),
            可以逆序包含,
            忽视顺序包含,
        )
    }

    /// 分析三元素构成的分型结构（trait object 版本）
    pub fn 分析_对象(
        左: &dyn 有高低,
        中: &dyn 有高低,
        右: &dyn 有高低,
        可以逆序包含: bool,
        忽视顺序包含: bool,
    ) -> Option<Self> {
        Self::分析_内部(
            左.高(),
            左.低(),
            中.高(),
            中.低(),
            右.高(),
            右.低(),
            可以逆序包含,
            忽视顺序包含,
        )
    }

    #[allow(clippy::too_many_arguments)]
    /// 分析三K线的分型结构（内部实现，接收裸f64值）
    #[allow(clippy::too_many_arguments)]
    pub fn 分析_内部(
        左高: f64,
        左低: f64,
        中高: f64,
        中低: f64,
        右高: f64,
        右低: f64,
        可以逆序包含: bool,
        忽视顺序包含: bool,
    ) -> Option<Self> {
        use crate::types::direction::相对方向;

        let 左中关系 = 相对方向::分析(左高, 左低, 中高, 中低);
        let 中右关系 = 相对方向::分析(中高, 中低, 右高, 右低);

        let 向上类 = |d: 相对方向| d.是否向上();
        let 向下类 = |d: 相对方向| d.是否向下();

        match (左中关系, 中右关系) {
            (相对方向::顺, _) if !忽视顺序包含 => {
                panic!("顺序包含 左中相对方向");
            }
            (_, 相对方向::顺) if !忽视顺序包含 => {
                panic!("顺序包含 中右相对方向");
            }
            (a, b) if 向上类(a) && 向上类(b) => Some(Self::上),
            (a, b) if 向上类(a) && 向下类(b) => Some(Self::顶),
            (a, 相对方向::逆) if 向上类(a) && 可以逆序包含 => Some(Self::上),
            (a, b) if 向下类(a) && 向上类(b) => Some(Self::底),
            (a, b) if 向下类(a) && 向下类(b) => Some(Self::下),
            (a, 相对方向::逆) if 向下类(a) && 可以逆序包含 => Some(Self::下),
            (相对方向::逆, a) if 向上类(a) && 可以逆序包含 => Some(Self::底),
            (相对方向::逆, a) if 向下类(a) && 可以逆序包含 => Some(Self::顶),
            (相对方向::逆, 相对方向::逆) if 可以逆序包含 => Some(Self::散),
            _ => {
                warn!("未匹配的关系 {}, {}, {}", 可以逆序包含, 左中关系, 中右关系);
                None
            }
        }
    }
}

/// Trait for types that have 高 and 低 (used by 分型结构::分析)
pub trait 有高低 {
    fn 高(&self) -> f64;
    fn 低(&self) -> f64;
}
