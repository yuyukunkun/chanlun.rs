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

use crate::kline::bar::K线;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// 布林带（BOLL）— 基于移动平均和标准差的波动率通道
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct 布林带 {
    /// 数据时间戳
    pub 时间戳: i64,
    /// 计算周期
    pub 周期: usize,
    /// 标准差倍数（通常为 2.0）
    pub 标准差倍数: f64,
    /// 上轨（中轨 + 倍数 × 标准差）
    pub 上轨: f64,
    /// 中轨（移动平均线）
    pub 中轨: f64,
    /// 下轨（中轨 - 倍数 × 标准差）
    pub 下轨: f64,
    /// 内部历史队列（不序列化）
    #[serde(skip)]
    _历史队列: VecDeque<f64>,
    /// 内部均值缓存（不序列化）
    #[serde(skip)]
    _均值: f64,
    /// 内部方差和缓存（不序列化）
    #[serde(skip)]
    _方差和: f64,
}

impl Default for 布林带 {
    fn default() -> Self {
        Self {
            时间戳: 0,
            周期: 20,
            标准差倍数: 2.0,
            上轨: 0.0,
            中轨: 0.0,
            下轨: 0.0,
            _历史队列: VecDeque::new(),
            _均值: 0.0,
            _方差和: 0.0,
        }
    }
}

impl 布林带 {
    /// 首次计算 BOLL 指标 — 从 K线 取值后计算
    pub fn 首次计算_K线(
        k线: &K线, 计算方式: &str, 周期: usize, 标准差倍数: f64
    ) -> Self {
        let 价格 = crate::indicators::K线取值(k线.开盘价, k线.高, k线.低, k线.收盘价, 计算方式);
        Self::首次计算(k线.时间戳, 价格, 周期, 标准差倍数)
    }

    /// 增量计算 BOLL 指标 — 从 K线 取值后递推
    pub fn 增量计算_K线(prev: &布林带, 当前K线: &K线, 计算方式: &str) -> Self {
        let 价格 = crate::indicators::K线取值(
            当前K线.开盘价,
            当前K线.高,
            当前K线.低,
            当前K线.收盘价,
            计算方式,
        );
        Self::增量计算(prev, 当前K线.时间戳, 价格)
    }

    /// 首次计算 — 初始时上中下轨都等于当前价格
    pub fn 首次计算(时间戳: i64, 价格: f64, 周期: usize, 标准差倍数: f64) -> Self {
        Self {
            时间戳,
            周期,
            标准差倍数,
            上轨: 价格,
            中轨: 价格,
            下轨: 价格,
            _历史队列: VecDeque::from([价格]),
            _均值: 价格,
            _方差和: 0.0,
        }
    }

    /// 增量计算 — 基于前一个布林带状态递推计算新的布林带
    pub fn 增量计算(prev: &布林带, 时间戳: i64, 价格: f64) -> Self {
        let 周期 = prev.周期;
        let 标准差倍数 = prev.标准差倍数;

        let mut q = prev._历史队列.clone();
        q.push_back(价格);
        if q.len() > 周期 {
            q.pop_front();
        }

        let (_均值, _方差和) = if q.len() < 周期 {
            let mean = q.iter().sum::<f64>() / q.len() as f64;
            let var_sum = q.iter().map(|v| (v - mean).powi(2)).sum();
            (mean, var_sum)
        } else {
            let n = 周期 as f64;
            let old_val = if prev._历史队列.len() >= 周期 {
                prev._历史队列[0]
            } else {
                q[0]
            };
            let new_mean = prev._均值 + (价格 - old_val) / n;
            let new_var =
                prev._方差和 + (价格 - old_val) * (价格 - new_mean + old_val - prev._均值);
            (new_mean, new_var)
        };

        let std = (_方差和 / q.len() as f64).sqrt();
        Self {
            时间戳,
            周期,
            标准差倍数,
            中轨: _均值,
            上轨: _均值 + 标准差倍数 * std,
            下轨: _均值 - 标准差倍数 * std,
            _历史队列: q,
            _均值,
            _方差和,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boll_first() {
        let b = 布林带::首次计算(1000, 100.0, 20, 2.0);
        assert!((b.上轨 - 100.0).abs() < 0.01);
        assert!((b.中轨 - 100.0).abs() < 0.01);
        assert!((b.下轨 - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_boll_incremental() {
        let b1 = 布林带::首次计算(1000, 100.0, 5, 2.0);
        let b2 = 布林带::增量计算(&b1, 1001, 102.0);
        assert!(b2.上轨 >= b2.中轨);
        assert!(b2.下轨 <= b2.中轨);
    }
}
