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

/// 买卖点类型 —— 六类买卖点（一二三 + T1/T1P/T2/T2S/T3A/T3B）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum 买卖点类型 {
    #[serde(rename = "一买")]
    一买,
    #[serde(rename = "一卖")]
    一卖,
    #[serde(rename = "二买")]
    二买,
    #[serde(rename = "二卖")]
    二卖,
    #[serde(rename = "三买")]
    三买,
    #[serde(rename = "三卖")]
    三卖,
    #[serde(rename = "T1买")]
    T1买,
    #[serde(rename = "T1卖")]
    T1卖,
    #[serde(rename = "T1P买")]
    T1P买,
    #[serde(rename = "T1P卖")]
    T1P卖,
    #[serde(rename = "T2买")]
    T2买,
    #[serde(rename = "T2卖")]
    T2卖,
    #[serde(rename = "T2S买")]
    T2S买,
    #[serde(rename = "T2S卖")]
    T2S卖,
    #[serde(rename = "T3A买")]
    T3A买,
    #[serde(rename = "T3A卖")]
    T3A卖,
    #[serde(rename = "T3B买")]
    T3B买,
    #[serde(rename = "T3B卖")]
    T3B卖,
}

impl std::fmt::Display for 买卖点类型 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::一买 => write!(f, "一买"),
            Self::一卖 => write!(f, "一卖"),
            Self::二买 => write!(f, "二买"),
            Self::二卖 => write!(f, "二卖"),
            Self::三买 => write!(f, "三买"),
            Self::三卖 => write!(f, "三卖"),
            Self::T1买 => write!(f, "T1买"),
            Self::T1卖 => write!(f, "T1卖"),
            Self::T1P买 => write!(f, "T1P买"),
            Self::T1P卖 => write!(f, "T1P卖"),
            Self::T2买 => write!(f, "T2买"),
            Self::T2卖 => write!(f, "T2卖"),
            Self::T2S买 => write!(f, "T2S买"),
            Self::T2S卖 => write!(f, "T2S卖"),
            Self::T3A买 => write!(f, "T3A买"),
            Self::T3A卖 => write!(f, "T3A卖"),
            Self::T3B买 => write!(f, "T3B买"),
            Self::T3B卖 => write!(f, "T3B卖"),
        }
    }
}

impl 买卖点类型 {
    /// 判断是否为买点类型
    pub fn 是买点(&self) -> bool {
        matches!(
            self,
            Self::一买
                | Self::二买
                | Self::三买
                | Self::T1买
                | Self::T1P买
                | Self::T2买
                | Self::T2S买
                | Self::T3A买
                | Self::T3B买
        )
    }

    /// 判断是否为卖点类型
    pub fn 是卖点(&self) -> bool {
        matches!(
            self,
            Self::一卖
                | Self::二卖
                | Self::三卖
                | Self::T1卖
                | Self::T1P卖
                | Self::T2卖
                | Self::T2S卖
                | Self::T3A卖
                | Self::T3B卖
        )
    }
}
