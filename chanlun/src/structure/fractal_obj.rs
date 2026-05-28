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

use crate::kline::chan_kline::缠论K线;
use crate::types::分型结构;
use crate::types::相对方向;
use std::sync::atomic::Ordering;
use std::sync::Arc;

/// 分型 — 由三根缠K构成（可能缺左或右）
#[derive(Debug, Clone)]
pub struct 分型 {
    pub 左: Option<Arc<缠论K线>>,
    pub 中: Arc<缠论K线>,
    pub 右: Option<Arc<缠论K线>>,
    pub 结构: 分型结构,
    pub 时间戳: i64,
    pub 分型特征值: f64,
}

impl 分型 {
    pub fn new(
        左: Option<Arc<缠论K线>>, 中: Arc<缠论K线>, 右: Option<Arc<缠论K线>>
    ) -> Self {
        let 结构 = 中.分型.read().unwrap().unwrap_or(分型结构::散);
        let 时间戳 = 中.时间戳.load(Ordering::Relaxed);
        let 分型特征值 = 中.分型特征值.get();
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
            相对方向::分析(左.高.get(), 左.低.get(), self.中.高.get(), self.中.低.get()),
            相对方向::分析(self.中.高.get(), self.中.低.get(), 右.高.get(), 右.低.get()),
            相对方向::分析(左.高.get(), 左.低.get(), 右.高.get(), 右.低.get()),
        ))
    }

    /// 分型强度 — 返回 "强"/"中"/"弱"/"未知"
    pub fn 强度(&self) -> &str {
        if self.结构 != 分型结构::底 && self.结构 != 分型结构::顶 {
            return "未知";
        }
        if self.右.is_none() || self.左.is_none() {
            return "未知";
        }

        if let Some(ref 关系组) = self.关系组() {
            if self.结构 == 分型结构::底 {
                if 关系组.2.是否向下() {
                    return "弱";
                } else if 关系组.2.是否向上() {
                    return "强";
                } else {
                    return "中";
                }
            } else if self.结构 == 分型结构::顶 {
                if 关系组.2.是否向上() {
                    return "弱";
                } else if 关系组.2.是否向下() {
                    return "强";
                } else {
                    return "中";
                }
            }
        }

        if let (Some(ref 左), Some(ref 右)) = (&self.左, &self.右) {
            if self.结构 == 分型结构::底 {
                if 右.标的K线.read().unwrap().收盘价 > 左.标的K线.read().unwrap().高 {
                    return "强";
                } else if 右.标的K线.read().unwrap().收盘价 > self.中.标的K线.read().unwrap().高
                {
                    return "中";
                } else {
                    return "弱";
                }
            } else if self.结构 == 分型结构::顶 {
                if 右.标的K线.read().unwrap().收盘价 < 左.标的K线.read().unwrap().低 {
                    return "强";
                } else if 右.标的K线.read().unwrap().收盘价 < self.中.标的K线.read().unwrap().低
                {
                    return "中";
                } else {
                    return "弱";
                }
            }
        }
        "未知"
    }

    /// MACD柱子分型匹配 — 检查左中右MACD柱形成顶/底形态
    pub fn 与MACD柱子分型匹配(&self) -> bool {
        if let (Some(ref 左), Some(ref 右)) = (&self.左, &self.右) {
            if self.结构 == 分型结构::底 {
                let 左_k = 左.标的K线.read().unwrap();
                let 中_k = self.中.标的K线.read().unwrap();
                let 右_k = 右.标的K线.read().unwrap();
                if let (Some(ref 左macd), Some(ref 中macd), Some(ref 右macd)) =
                    (&左_k.macd, &中_k.macd, &右_k.macd)
                {
                    return 左macd.MACD柱 > 中macd.MACD柱 && 中macd.MACD柱 < 右macd.MACD柱;
                }
            }
            if self.结构 == 分型结构::顶 {
                let 左_k = 左.标的K线.read().unwrap();
                let 中_k = self.中.标的K线.read().unwrap();
                let 右_k = 右.标的K线.read().unwrap();
                if let (Some(ref 左macd), Some(ref 中macd), Some(ref 右macd)) =
                    (&左_k.macd, &中_k.macd, &右_k.macd)
                {
                    return 左macd.MACD柱 < 中macd.MACD柱 && 中macd.MACD柱 > 右macd.MACD柱;
                }
            }
        }
        false
    }

    /// 判断两个分型是否匹配
    pub fn 判断分型(左: &Arc<分型>, 右: &Arc<分型>, 模式: &str) -> bool {
        match 模式 {
            "中" => Arc::as_ptr(左) == Arc::as_ptr(右),
            _ => false,
        }
    }

    /// 从缠K序列中获取以指定缠K为中元素的分型
    pub fn 从缠K序列中获取分型(
        K线序列: &[Arc<缠论K线>],
        中: &Arc<缠论K线>,
    ) -> Option<Self> {
        let idx = K线序列
            .iter()
            .position(|k| Arc::as_ptr(k) == Arc::as_ptr(中))?;
        let 左 = if idx > 0 {
            Some(Arc::clone(&K线序列[idx - 1]))
        } else {
            None
        };
        let 右 = if idx + 1 < K线序列.len() {
            Some(Arc::clone(&K线序列[idx + 1]))
        } else {
            None
        };
        Some(Self::new(左, Arc::clone(中), 右))
    }

    /// 向分型序列中添加新分型
    pub fn 向序列中添加(分型序列: &mut Vec<Arc<分型>>, 当前分型: Arc<分型>) {
        if 分型序列.is_empty() {
            if 当前分型.结构 != 分型结构::顶 && 当前分型.结构 != 分型结构::底
            {
                panic!("首次添加分型不为 顶底 {:?}", 当前分型);
            }
        } else {
            let 前一个 = &分型序列[分型序列.len() - 1];
            if 前一个.结构 == 当前分型.结构 {
                panic!("分型相同无法添加 {:?} {:?}", 前一个, 当前分型);
            }
            if 前一个.右.is_none() {
                eprintln!("分型.向序列中添加, 分型异常 {:?}", 前一个);
            }
        }
        分型序列.push(当前分型);
    }
}

impl crate::types::fractal::有高低 for 分型 {
    fn 高(&self) -> f64 {
        self.中.高.get()
    }
    fn 低(&self) -> f64 {
        self.中.低.get()
    }
}

impl std::fmt::Display for 分型 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}<{}, {}, None: {}, None: {}>",
            self.中
                .分型
                .read()
                .unwrap()
                .unwrap_or(crate::types::分型结构::散),
            self.时间戳,
            crate::utils::format_f64_g(self.分型特征值),
            if self.左.is_none() { "True" } else { "False" },
            if self.右.is_none() { "True" } else { "False" },
        )
    }
}
