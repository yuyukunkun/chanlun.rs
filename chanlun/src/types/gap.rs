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

/// 缺口 —— 两个价格区间之间的空隙
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct 缺口 {
    /// 缺口上沿
    pub 高: f64,
    /// 缺口下沿
    pub 低: f64,
}

impl 缺口 {
    /// 创建缺口，高必须大于低
    pub fn new(高: f64, 低: f64) -> Self {
        assert!(高 > 低, "缺口高必须大于低: 高={高}, 低={低}");
        Self { 高, 低 }
    }

    /// 在 [起点, 终点] 区间内居中截取一个子区间
    pub fn 居中截取区间(起点: f64, 终点: f64, 比例: f64) -> Option<Self> {
        let (起点, 终点) = if 起点 <= 终点 {
            (起点, 终点)
        } else {
            (终点, 起点)
        };

        let 总长 = 终点 - 起点;
        let 偏移 = 总长 * 比例;
        let 中心 = (起点 + 终点) / 2.0;

        let 下界 = 中心 - 偏移;
        let 上界 = 中心 + 偏移;

        if 下界 > 终点 || 上界 < 起点 {
            return None;
        }

        let 下界 = 下界.max(起点);
        let 上界 = 上界.min(终点);

        Some(Self::new(上界.max(下界), 上界.min(下界)))
    }

    /// 结构化相等校验 — 浮点容差比较高/低，返回 (是否相等, 差异描述)
    pub fn 相等(&self, other: &Self, 浮点容差: f64) -> (bool, String) {
        if (self.高 - other.高).abs() > 浮点容差 {
            return (
                false,
                format!(
                    "缺口: [高] 浮点超限 容差={浮点容差:.2e} A={:.10},B={:.10}",
                    self.高, other.高
                ),
            );
        }
        if (self.低 - other.低).abs() > 浮点容差 {
            return (
                false,
                format!(
                    "缺口: [低] 浮点超限 容差={浮点容差:.2e} A={:.10},B={:.10}",
                    self.低, other.低
                ),
            );
        }
        (true, "缺口: 高低价格一致".into())
    }
}

impl std::fmt::Display for 缺口 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "缺口区间<{} <=> {}>",
            crate::utils::format_f64_g(self.低),
            crate::utils::format_f64_g(self.高)
        )
    }
}
