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
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use tracing::warn;

/// 分型模式 — True 时使用构造时缓存值（默认），False 时从 中 缠K 实时读取
pub static 分型模式: AtomicBool = AtomicBool::new(true);

/// 分型 — 由左中右三根缠论K线构成的顶/底分型。
///
/// 字段:
///   左: 左侧缠K（可能为 None，表示序列起始处分型左翼缺失）
///   中: 中间缠K（必须存在），分型的核心K线，其 `分型` 字段记录了该位置的分型结构
///   右: 右侧缠K（可能为 None，表示序列末尾处分型右翼缺失）
///   结构: 构造时从 `中.分型` 缓存的分型结构（上/下/顶/底/散）
///   时间戳: 构造时从 `中.时间戳` 缓存的时刻
///   分型特征值: 构造时从 `中.分型特征值` 缓存的值
///
/// "分型模式" 全局开关控制 时间戳/结构/分型特征值 的读取行为：
///   True（默认）— 返回构造时缓存值；False — 从 中 缠K 实时读取
#[derive(Debug, Clone)]
pub struct 分型 {
    /// 左侧缠K（序列起始处分型可能为 None）
    pub 左: Option<Arc<缠论K线>>,
    /// 中间缠K（必须存在）
    pub 中: Arc<缠论K线>,
    /// 右侧缠K（序列末尾处分型可能为 None）
    pub 右: Option<Arc<缠论K线>>,
    /// 构造时缓存的 分型结构（上/下/顶/底/散）
    pub 结构: 分型结构,
    /// 构造时缓存的 时间戳
    pub 时间戳: i64,
    /// 构造时缓存的 分型特征值（历史高低点极值）
    pub 分型特征值: f64,
}

impl 分型 {
    pub fn new(
        左: Option<Arc<缠论K线>>, 中: Arc<缠论K线>, 右: Option<Arc<缠论K线>>
    ) -> Self {
        if let (Some(左), Some(右)) = (&左, &右) {
            debug_assert!(
                左.时间戳.load(Ordering::Relaxed) < 中.时间戳.load(Ordering::Relaxed)
                    && 中.时间戳.load(Ordering::Relaxed) < 右.时间戳.load(Ordering::Relaxed),
                "分型时间戳断言失败: 左={}, 中={}, 右={}",
                左.时间戳.load(Ordering::Relaxed),
                中.时间戳.load(Ordering::Relaxed),
                右.时间戳.load(Ordering::Relaxed),
            );
        }
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

    /// 时间戳 — 根据 分型模式 决定返回缓存值（True）或实时值（False）
    pub fn 时间戳(&self) -> i64 {
        if 分型模式.load(Ordering::Relaxed) {
            self.时间戳
        } else {
            self.中.时间戳.load(Ordering::Relaxed)
        }
    }

    /// 结构 — 根据 分型模式 决定返回缓存值（True）或实时值（False）
    pub fn 结构(&self) -> 分型结构 {
        if 分型模式.load(Ordering::Relaxed) {
            self.结构
        } else {
            self.中.分型.read().unwrap().unwrap_or(分型结构::散) // FIXME 错误
        }
    }

    /// 分型特征值 — 根据 分型模式 决定返回缓存值（True）或实时值（False）
    pub fn 分型特征值(&self) -> f64 {
        if 分型模式.load(Ordering::Relaxed) {
            self.分型特征值
        } else {
            self.中.分型特征值.get()
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
        if self.结构() != 分型结构::底 && self.结构() != 分型结构::顶 {
            return "未知";
        }
        if self.右.is_none() || self.左.is_none() {
            return "未知";
        }

        if let Some(ref 关系组) = self.关系组() {
            if self.结构() == 分型结构::底 {
                if 关系组.2.是否向下() {
                    return "弱";
                } else if 关系组.2.是否向上() {
                    return "强";
                } else {
                    return "中";
                }
            } else if self.结构() == 分型结构::顶 {
                if 关系组.2.是否向上() {
                    return "弱";
                } else if 关系组.2.是否向下() {
                    return "强";
                } else {
                    return "中";
                }
            }
        }

        if let (Some(左), Some(右)) = (&self.左, &self.右) {
            if self.结构() == 分型结构::底 {
                if 右.标的K线.read().unwrap().收盘价 > 左.标的K线.read().unwrap().高 {
                    return "强";
                } else if 右.标的K线.read().unwrap().收盘价 > self.中.标的K线.read().unwrap().高
                {
                    return "中";
                } else {
                    return "弱";
                }
            } else if self.结构() == 分型结构::顶 {
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
        if let (Some(左), Some(右)) = (&self.左, &self.右) {
            if self.结构() == 分型结构::底 {
                let 左_k = 左.标的K线.read().unwrap();
                let 中_k = self.中.标的K线.read().unwrap();
                let 右_k = 右.标的K线.read().unwrap();
                let 左_m = 左_k.指标.read().unwrap();
                let 中_m = 中_k.指标.read().unwrap();
                let 右_m = 右_k.指标.read().unwrap();
                if let (Some(左macd), Some(中macd), Some(右macd)) =
                    (左_m.macd(), 中_m.macd(), 右_m.macd())
                {
                    return 左macd.MACD柱 > 中macd.MACD柱 && 中macd.MACD柱 < 右macd.MACD柱;
                }
            }
            if self.结构() == 分型结构::顶 {
                let 左_k = 左.标的K线.read().unwrap();
                let 中_k = self.中.标的K线.read().unwrap();
                let 右_k = 右.标的K线.read().unwrap();
                let 左_m = 左_k.指标.read().unwrap();
                let 中_m = 中_k.指标.read().unwrap();
                let 右_m = 右_k.指标.read().unwrap();
                if let (Some(左macd), Some(中macd), Some(右macd)) =
                    (左_m.macd(), 中_m.macd(), 右_m.macd())
                {
                    return 左macd.MACD柱 < 中macd.MACD柱 && 中macd.MACD柱 > 右macd.MACD柱;
                }
            }
        }
        false
    }

    /// 判断两个分型是否匹配
    pub fn 判断分型(左: &Arc<分型>, 右: &Arc<分型>, _模式: &str) -> bool {
        Arc::as_ptr(左) == Arc::as_ptr(右)
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
            if 当前分型.结构() != 分型结构::顶 && 当前分型.结构() != 分型结构::底
            {
                panic!("首次添加分型不为 顶底 {:?}", 当前分型);
            }
        } else {
            let 前一个 = &分型序列[分型序列.len() - 1];
            if 前一个.结构() == 当前分型.结构() {
                panic!("分型相同无法添加 {:?} {:?}", 前一个, 当前分型);
            }
            if 前一个.右.is_none() {
                warn!("分型.向序列中添加, 分型异常 {:?}", 前一个);
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
            self.结构(),
            self.时间戳(),
            crate::utils::format_f64_g(self.分型特征值()),
            if self.左.is_none() { "True" } else { "False" },
            if self.右.is_none() { "True" } else { "False" },
        )
    }
}
