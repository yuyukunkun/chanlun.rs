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

use crate::business::observer::观察者;
use crate::config::缠论配置;
use crate::kline::bar::K线;
use crate::kline::chan_kline::缠论K线;
use crate::structure::dash_line::虚线;
use crate::structure::fractal_obj::分型;
use crate::types::{分型结构, 相对方向};
use crate::{error, warn};
use std::sync::Arc;
use std::sync::atomic::Ordering;

/// 笔 — 从分型生成笔的算法集合（静态方法命名空间）
pub struct 笔;

impl 笔 {
    /// 获取可成笔的缠K数量（考虑弱化模式）
    pub fn _获取缠K数量(
        缠K序列: &[Arc<缠论K线>],
        笔序列: &[Arc<虚线>],
        配置: &缠论配置,
    ) -> usize {
        let 实际数量 = 缠K序列.len();
        if 实际数量 >= 配置.笔内元素数量 as usize {
            return 实际数量;
        }

        if 配置.笔弱化 && 实际数量 >= 3 {
            let 实际高点 = Self::_实际高点(缠K序列, 配置.笔内相同终点取舍);
            let 实际低点 = Self::_实际低点(缠K序列, 配置.笔内相同终点取舍);

            if let (Some(高点), Some(低点)) = (&实际高点, &实际低点) {
                let 原始数量 = 1
                    + (低点.标的K线.read().序号 - 高点.标的K线.read().序号).unsigned_abs() as usize;
                if 原始数量 >= 配置.笔内元素数量 as usize {
                    return 配置.笔内元素数量 as usize;
                }
            }

            if !笔序列.is_empty() {
                // Try both high and low points (Python: 根据缠K找笔(笔序列, 实际高点) or 根据缠K找笔(笔序列, 实际低点))
                let 筆 = 实际高点
                    .as_ref()
                    .and_then(|h| Self::根据缠K找笔(笔序列, h, 1))
                    .or_else(|| {
                        实际低点
                            .as_ref()
                            .and_then(|l| Self::根据缠K找笔(笔序列, l, 1))
                    });

                if let Some(ref 筆) = 筆
                    && let (Some(高_k), Some(低_k)) = (&实际高点, &实际低点)
                {
                    let 原始数量 = 1
                        + (低_k.标的K线.read().序号 - 高_k.标的K线.read().序号).unsigned_abs()
                            as usize;
                    // 向上笔
                    if 筆.方向().是否向上()
                        && 低_k.低.get() < 筆.低()
                        && 原始数量 >= 配置.笔弱化_原始数量 as usize
                    {
                        return 配置.笔内元素数量 as usize;
                    }
                    // 向下笔
                    if 筆.方向().是否向下()
                        && 低_k.低.get() > 筆.高()
                        && 原始数量 >= 配置.笔弱化_原始数量 as usize
                    {
                        return 配置.笔内元素数量 as usize;
                    }
                }
            }
        }
        实际数量
    }

    /// 次高 — 排除最高值后的次高点
    pub fn _次高(缠K序列: &[Arc<缠论K线>], 取舍: bool) -> Option<Arc<缠论K线>> {
        let max_高 = 缠K序列
            .iter()
            .map(|k| k.高.get())
            .fold(f64::NEG_INFINITY, f64::max);
        // 排除最高值
        let filtered: Vec<&Arc<缠论K线>> =
            缠K序列.iter().filter(|k| k.高.get() != max_高).collect();
        // 筛选次高值
        let second_高 = filtered
            .iter()
            .map(|k| k.高.get())
            .fold(f64::NEG_INFINITY, f64::max);
        let mut candidates: Vec<&Arc<缠论K线>> = filtered
            .iter()
            .filter(|k| k.高.get() == second_高)
            .copied()
            .collect();
        // 按时间戳排序
        candidates.sort_by(|a, b| {
            a.时间戳
                .load(Ordering::Relaxed)
                .cmp(&b.时间戳.load(Ordering::Relaxed))
        });
        if 取舍 {
            Some(Arc::clone(candidates[candidates.len() - 1]))
        } else {
            Some(Arc::clone(candidates[0]))
        }
    }

    /// 次低 — 排除最低值后的次低点
    pub fn _次低(缠K序列: &[Arc<缠论K线>], 取舍: bool) -> Option<Arc<缠论K线>> {
        let min_低 = 缠K序列
            .iter()
            .map(|k| k.低.get())
            .fold(f64::INFINITY, f64::min);
        // 排除最低值
        let filtered: Vec<&Arc<缠论K线>> =
            缠K序列.iter().filter(|k| k.低.get() != min_低).collect();
        // 筛选次低值
        let second_低 = filtered
            .iter()
            .map(|k| k.低.get())
            .fold(f64::INFINITY, f64::min);
        let mut candidates: Vec<&Arc<缠论K线>> = filtered
            .iter()
            .filter(|k| k.低.get() == second_低)
            .copied()
            .collect();
        // 按时间戳排序
        candidates.sort_by(|a, b| {
            a.时间戳
                .load(Ordering::Relaxed)
                .cmp(&b.时间戳.load(Ordering::Relaxed))
        });
        if 取舍 {
            Some(Arc::clone(candidates[candidates.len() - 1]))
        } else {
            Some(Arc::clone(candidates[0]))
        }
    }

    /// 实际高点
    pub fn _实际高点(缠K序列: &[Arc<缠论K线>], 取舍: bool) -> Option<Arc<缠论K线>> {
        if 缠K序列.is_empty() {
            return None;
        }
        let max_高 = 缠K序列
            .iter()
            .map(|k| k.高.get())
            .fold(f64::NEG_INFINITY, f64::max);
        let mut candidates: Vec<&Arc<缠论K线>> =
            缠K序列.iter().filter(|k| k.高.get() == max_高).collect();
        if candidates.is_empty() {
            return Some(Arc::clone(&缠K序列[0]));
        }
        // 按时间戳排序
        candidates.sort_by(|a, b| {
            a.时间戳
                .load(Ordering::Relaxed)
                .cmp(&b.时间戳.load(Ordering::Relaxed))
        });
        if 取舍 {
            Some(Arc::clone(candidates[candidates.len() - 1]))
        } else {
            Some(Arc::clone(candidates[0]))
        }
    }

    /// 实际低点
    pub fn _实际低点(缠K序列: &[Arc<缠论K线>], 取舍: bool) -> Option<Arc<缠论K线>> {
        if 缠K序列.is_empty() {
            return None;
        }
        let min_低 = 缠K序列
            .iter()
            .map(|k| k.低.get())
            .fold(f64::INFINITY, f64::min);
        let mut candidates: Vec<&Arc<缠论K线>> =
            缠K序列.iter().filter(|k| k.低.get() == min_低).collect();
        if candidates.is_empty() {
            return Some(Arc::clone(&缠K序列[0]));
        }
        // 按时间戳排序
        candidates.sort_by(|a, b| {
            a.时间戳
                .load(Ordering::Relaxed)
                .cmp(&b.时间戳.load(Ordering::Relaxed))
        });
        if 取舍 {
            Some(Arc::clone(candidates[candidates.len() - 1]))
        } else {
            Some(Arc::clone(candidates[0]))
        }
    }

    /// 判断笔的相对关系是否合理
    pub fn _相对关系(筆: &虚线, 配置: &缠论配置) -> bool {
        let 文分型 = &筆.文;
        let 武分型 = 筆.武.read();

        let 相对关系 = if 配置.笔内起始分型包含整笔 {
            let 文中_rc = Arc::clone(&文分型.中);
            let 武中_rc = Arc::clone(&武分型.中);
            let 文_元素: [Option<&Arc<缠论K线>>; 3] =
                [文分型.左.as_ref(), Some(&文中_rc), 文分型.右.as_ref()];
            let 有效序列: Vec<&Arc<缠论K线>> = 文_元素.iter().filter_map(|x| *x).collect();
            let 文高 = 有效序列
                .iter()
                .map(|k| k.高.get())
                .fold(f64::NEG_INFINITY, f64::max);
            let 文低 = 有效序列
                .iter()
                .map(|k| k.低.get())
                .fold(f64::INFINITY, f64::min);

            let 武_右: Option<&Arc<缠论K线>> = if 配置.笔内起始分型包含整笔_包括右 {
                武分型.右.as_ref()
            } else {
                None
            };
            let 武_元素: [Option<&Arc<缠论K线>>; 3] = [武分型.左.as_ref(), Some(&武中_rc), 武_右];
            let 有效序列: Vec<&Arc<缠论K线>> = 武_元素.iter().filter_map(|x| *x).collect();
            let 武高 = 有效序列
                .iter()
                .map(|k| k.高.get())
                .fold(f64::NEG_INFINITY, f64::max);
            let 武低 = 有效序列
                .iter()
                .map(|k| k.低.get())
                .fold(f64::INFINITY, f64::min);

            crate::types::相对方向::分析(文高, 文低, 武高, 武低)
        } else {
            let 相对关系 = crate::types::相对方向::分析(
                文分型.中.高.get(),
                文分型.中.低.get(),
                武分型.中.高.get(),
                武分型.中.低.get(),
            );
            if 配置.笔内原始K线包含整笔 {
                let 文标的 = 文分型.中.标的K线.read();
                let 武标的 = 武分型.中.标的K线.read();
                if crate::types::相对方向::分析(文标的.高, 文标的.低, 武标的.高, 武标的.低)
                    .是否包含()
                {
                    return false;
                }
            }
            相对关系
        };

        if 筆.方向() == crate::types::相对方向::向下 {
            return 相对关系.是否向下();
        }
        相对关系.是否向上()
    }

    /// 以文会友 — 根据起点分型找笔
    pub fn 以文会友(笔序列: &[Arc<虚线>], 文: &Arc<分型>) -> Option<Arc<虚线>> {
        笔序列.iter().find(|b| Arc::ptr_eq(&b.文, 文)).cloned()
    }

    /// 以武会友 — 根据终点分型找笔
    pub fn 以武会友(笔序列: &[Arc<虚线>], 武: &Arc<分型>) -> Option<Arc<虚线>> {
        笔序列
            .iter()
            .rev()
            .find(|b| Arc::ptr_eq(&*b.武.read(), 武))
            .cloned()
    }

    /// 根据缠K找对应的笔
    pub fn 根据缠K找笔(
        笔序列: &[Arc<虚线>],
        缠K: &Arc<缠论K线>,
        偏移: i64,
    ) -> Option<Arc<虚线>> {
        // Python iterates in reverse: for 筆 in 笔序列[::-1]
        for b in 笔序列.iter().rev() {
            // Python: 筆.文.中.序号 - 偏移 <= 缠K.序号 <= 筆.武.中.序号
            if b.文.中.序号.load(Ordering::Relaxed) - 偏移 <= 缠K.序号.load(Ordering::Relaxed)
                && 缠K.序号.load(Ordering::Relaxed) <= b.武.read().中.序号.load(Ordering::Relaxed)
                && b.文.中.周期 == 缠K.周期
                && b.文.中.标识 == 缠K.标识
            {
                return Some(Arc::clone(b));
            }
        }
        None
    }

    /// 从分型序列中弹出最后一个分型和对应的笔
    fn _弹出旧笔(
        分型序列: &mut Vec<Arc<分型>>, 笔序列: &mut Vec<Arc<虚线>>, 行号: u32
    ) {
        let 旧分型 = 分型序列.pop();
        if let (Some(旧笔), Some(旧分型)) = (笔序列.pop(), 旧分型) {
            assert!(
                Arc::ptr_eq(&旧笔.武.read(), &旧分型),
                "最后一笔终点错误{}",
                行号
            );
            旧笔.有效性.store(false, Ordering::Relaxed);
        }
    }

    /// 核心笔分析 — 递归实现，逐句对照 chan.py 笔.分析 / 笔递归分析
    ///
    /// 返回: 递归层次数
    pub fn 分析(
        当前分型: Arc<分型>,
        分型序列: &mut Vec<Arc<分型>>,
        笔序列: &mut Vec<Arc<虚线>>,
        缠K序列: &[Arc<缠论K线>],
        _普K序列: &[Arc<K线>],
        递归层次: i64,
        配置: &缠论配置,
    ) -> i64 {
        // Python line 2315-2317: 递归深度限制
        if 递归层次 > 64 {
            warn!("笔.分析 递归深度超出 64 < {}", 递归层次);
        }

        // Python line 2319-2320: 非顶底分型跳过
        if !matches!(当前分型.结构, 分型结构::顶 | 分型结构::底) {
            return 递归层次;
        }

        // Python line 2322-2325: 第一个分型直接追加
        if 分型序列.is_empty() {
            if matches!(当前分型.结构, 分型结构::顶 | 分型结构::底) {
                分型序列.push(当前分型);
            }
            return 递归层次;
        }

        // Python line 2329-2335: 清理无效数据
        let 之前分型 = Arc::clone(分型序列.last().unwrap());
        if 之前分型.时间戳() == 当前分型.时间戳()
            || matches!(之前分型.结构, 分型结构::上 | 分型结构::下)
        {
            Self::_弹出旧笔(分型序列, 笔序列, line!());
            if 分型序列.is_empty() {
                if 当前分型.右.is_some() {
                    分型::向序列中添加(分型序列, 当前分型);
                }
                return 递归层次;
            }
        }

        // Python line 2337-2341: 时序检查
        let 之前分型 = Arc::clone(分型序列.last().unwrap());
        if 之前分型.时间戳() > 当前分型.时间戳()
            && 之前分型.中.序号.load(Ordering::Relaxed) - 当前分型.中.序号.load(Ordering::Relaxed)
                > 1
        {
            error!("时序错误-{}, {}, {}", 递归层次, 之前分型, 当前分型);
            return 递归层次;
        }

        // Python line 2343-2348: 笔弱化模式
        if 配置.笔弱化 && !笔序列.is_empty() {
            let 前一笔 = 笔序列.last().unwrap();
            let 前一笔缠K数 = 前一笔.武.read().中.序号.load(Ordering::Relaxed)
                - 前一笔.文.中.序号.load(Ordering::Relaxed)
                + 1;
            if 前一笔缠K数 == 3 {
                let 破位 = (前一笔.方向().是否向上()
                    && 前一笔.低() > 当前分型.分型特征值
                    && 当前分型.结构 == 分型结构::底)
                    || (前一笔.方向().是否向下()
                        && 前一笔.高() < 当前分型.分型特征值
                        && 当前分型.结构 == 分型结构::顶);
                if 破位 {
                    Self::_弹出旧笔(分型序列, 笔序列, line!());
                    return Self::分析(
                        当前分型,
                        分型序列,
                        笔序列,
                        缠K序列,
                        _普K序列,
                        递归层次 + 1,
                        配置,
                    );
                }
            }
        }

        // Python line 2350: 分型结构相反 → 可能成笔
        let 之前分型 = Arc::clone(分型序列.last().unwrap());
        if 之前分型.结构 != 当前分型.结构 {
            if let Some(基础序列) = 缠论K线::截取(缠K序列, &之前分型.中, &当前分型.中)
            {
                let 当前笔 = Arc::new(虚线::创建笔(
                    Arc::clone(&之前分型),
                    Arc::clone(&当前分型),
                    true,
                ));

                if Self::_获取缠K数量(&基础序列, 笔序列, 配置) >= 配置.笔内元素数量 as usize
                {
                    // Python line 2354-2357: 文官
                    let 文官 = match 之前分型.结构 {
                        分型结构::顶 => Self::_实际高点(&基础序列, false),
                        _ => Self::_实际低点(&基础序列, false),
                    };

                    // Python line 2359-2367: 文官调整
                    if let Some(ref 文官_k) = 文官
                        && !Arc::ptr_eq(文官_k, &之前分型.中)
                        && let Some(临时分型) =
                            分型::从缠K序列中获取分型(缠K序列, 文官_k)
                    {
                        assert!(
                            if 之前分型.结构 == 分型结构::顶 && 当前分型.结构 == 分型结构::底
                            {
                                临时分型.结构 == 分型结构::顶
                            } else {
                                临时分型.结构 == 分型结构::底
                            },
                            "文官分型结构不在预期: {}",
                            临时分型
                        );
                        let 递归层次 = Self::分析(
                            Arc::new(临时分型),
                            分型序列,
                            笔序列,
                            缠K序列,
                            _普K序列,
                            递归层次 + 1,
                            配置,
                        );
                        return Self::分析(
                            当前分型,
                            分型序列,
                            笔序列,
                            缠K序列,
                            _普K序列,
                            递归层次 + 1,
                            配置,
                        );
                    }

                    // Python line 2369-2375: 武将 and笔形成
                    let 武将 = match 当前分型.结构 {
                        分型结构::底 => Self::_实际低点(&基础序列, 配置.笔内相同终点取舍),
                        _ => Self::_实际高点(&基础序列, 配置.笔内相同终点取舍),
                    };

                    if Self::_相对关系(&当前笔, 配置)
                        && let Some(ref 武将_k) = 武将
                        && Arc::ptr_eq(武将_k, &当前分型.中)
                    {
                        // 直接添加（对照 Python _添加新笔：直接 append）
                        Self::_添加新笔(分型序列, 笔序列, 当前分型, 当前笔, line!());
                        return 递归层次;
                    }

                    // Python line 2378-2385: 笔次级成笔
                    if 配置.笔次级成笔 {
                        let 武将 = match 当前分型.结构 {
                            分型结构::底 => Self::_次低(&基础序列, 配置.笔内相同终点取舍),
                            _ => Self::_次高(&基础序列, 配置.笔内相同终点取舍),
                        };
                        if let Some(ref 武将_k) = 武将
                            && Arc::ptr_eq(武将_k, &当前分型.中)
                            && Self::_相对关系(&当前笔, 配置)
                        {
                            Self::_添加新笔(分型序列, 笔序列, 当前分型, 当前笔, line!());
                            return 递归层次;
                        }
                    }
                } else {
                    // Python line 2388-2390: 元素不足 → 右元素扩展
                    if let Some(ref 右) = 当前分型.右
                        && let Some(临时分型) = 分型::从缠K序列中获取分型(缠K序列, 右)
                    {
                        return Self::分析(
                            Arc::new(临时分型),
                            分型序列,
                            笔序列,
                            缠K序列,
                            _普K序列,
                            递归层次 + 1,
                            配置,
                        );
                    }
                }
            }
        } else {
            // Python line 2392-2419: 分型结构相同 → 更强则替换 + 修复错过笔
            let 分型特征值 = 当前分型.分型特征值;

            let 更强 = match 之前分型.结构 {
                分型结构::顶 => 之前分型.分型特征值 < 分型特征值,
                分型结构::底 => 之前分型.分型特征值 > 分型特征值,
                _ => false,
            };

            if 更强 {
                // 保存被弹出的之前分型（用于修复错过笔的范围计算）
                let 被替换分型 = Arc::clone(&之前分型);
                Self::_弹出旧笔(分型序列, 笔序列, line!());

                if let Some(k线序列) = 缠论K线::截取(缠K序列, &被替换分型.中, &当前分型.中)
                {
                    let 武将 = match 被替换分型.结构 {
                        分型结构::顶 => Self::_实际低点(&k线序列, 配置.笔内相同终点取舍),
                        _ => Self::_实际高点(&k线序列, 配置.笔内相同终点取舍),
                    };

                    if let Some(ref 武将_k) = 武将
                        && let Some(临时分型) =
                            分型::从缠K序列中获取分型(缠K序列, 武将_k)
                    {
                        let 临时分型_rc = Arc::new(临时分型);

                        if !分型序列.is_empty() {
                            let mut 递归层次 = Self::分析(
                                Arc::clone(&临时分型_rc),
                                分型序列,
                                笔序列,
                                缠K序列,
                                _普K序列,
                                递归层次 + 1,
                                配置,
                            );

                            // 修复错过的笔: 扫描武将之后的所有分型
                            if !分型序列.is_empty()
                                && Arc::as_ptr(分型序列.last().unwrap())
                                    == Arc::as_ptr(&临时分型_rc)
                                && let Some(武_idx) =
                                    缠K序列.iter().position(|k| Arc::ptr_eq(k, 武将_k))
                            {
                                for ck in &缠K序列[武_idx..] {
                                    if (*ck.分型.read() == Some(分型结构::底)
                                        || *ck.分型.read() == Some(分型结构::顶))
                                        && let Some(错过分型) =
                                            分型::从缠K序列中获取分型(缠K序列, ck)
                                    {
                                        let 错过分型_rc = Arc::new(错过分型);
                                        递归层次 = Self::分析(
                                            Arc::clone(&错过分型_rc),
                                            分型序列,
                                            笔序列,
                                            缠K序列,
                                            _普K序列,
                                            递归层次 + 1,
                                            配置,
                                        );
                                        if !分型序列.is_empty()
                                            && Arc::as_ptr(分型序列.last().unwrap())
                                                == Arc::as_ptr(&错过分型_rc)
                                        {
                                            warn!(
                                                "笔.分析 事后修复错过的笔:{}, 当前分型: {}",
                                                错过分型_rc, 当前分型
                                            );
                                        }
                                    }
                                }
                            }

                            return Self::分析(
                                当前分型,
                                分型序列,
                                笔序列,
                                缠K序列,
                                _普K序列,
                                递归层次 + 1,
                                配置,
                            );
                        } else {
                            分型::向序列中添加(分型序列, 当前分型);
                        }
                    }
                } else if 分型序列.is_empty() {
                    分型::向序列中添加(分型序列, 当前分型);
                } else {
                    return Self::分析(
                        当前分型,
                        分型序列,
                        笔序列,
                        缠K序列,
                        _普K序列,
                        递归层次 + 1,
                        配置,
                    );
                }
            }
        }

        递归层次
    }

    /// 添加新笔到序列（递归版本 — 直接追加，对应 Python _添加新笔）
    fn _添加新笔(
        分型序列: &mut Vec<Arc<分型>>,
        笔序列: &mut Vec<Arc<虚线>>,
        新分型: Arc<分型>,
        mut 新笔: Arc<虚线>,
        行号: u32,
    ) {
        // 首次添加分型检查（Python line 3537-3538）
        if 分型序列.is_empty() && !matches!(新分型.结构, 分型结构::顶 | 分型结构::底)
        {
            panic!("首次添加分型不为 顶底 {:?}", 新分型);
        }
        // 添加前分型检查（Python line 3539-3543）
        if let Some(前一) = 分型序列.last() {
            if 前一.结构() == 新分型.结构() {
                panic!("分型相同无法添加 {} {}", 前一, 新分型);
            }
            if 前一.右.is_none() {
                warn!("分型.向序列中添加, 分型异常 {}", 前一);
            }
        }

        分型序列.push(新分型);

        if let Some(前一笔) = 笔序列.last()
            && !前一笔.之后是(&新笔)
        {
            panic!("笔.向序列中添加 不连续 {} {}", 前一笔, 新笔);
        }

        if let Some(前一笔) = 笔序列.last() {
            let 新筆 = Arc::make_mut(&mut 新笔);
            新筆
                .序号
                .store(前一笔.序号.load(Ordering::Relaxed) + 1, Ordering::Relaxed);
            if 新筆.武.read().左.is_none() || 新筆.武.read().右.is_none() {
                新筆.有效性.store(false, Ordering::Relaxed);
            }
            if matches!(前一笔.武.read().结构(), 分型结构::上 | 分型结构::下) {
                error!("_添加新笔[{}] 出现无效分型 {}", 行号, 前一笔);
            }
        }
        笔序列.push(新笔);
    }

    /// 自检 — 验证笔的有效性（文为实际高/低点，武为实际低/高点）
    pub fn _自检(筆: &虚线, 观察员: &观察者) -> bool {
        let 笔序列 = &观察员.笔序列;
        let 配置 = &观察员.配置;
        let 基础序列 = 筆.获取缠K序列(&观察员.缠论K线序列);
        if Self::_获取缠K数量(&基础序列, 笔序列, 配置) >= 配置.笔内元素数量 as usize
        {
            if 筆.方向() == 相对方向::向下
                && let (Some(实际高), Some(实际低)) = (
                    Self::_实际高点(&基础序列, false),
                    Self::_实际低点(&基础序列, 配置.笔内相同终点取舍),
                )
                && Arc::ptr_eq(&筆.文.中, &实际高)
                && Arc::ptr_eq(&筆.武.read().中, &实际低)
            {
                return true;
            }
            if 筆.方向() == 相对方向::向上
                && let (Some(实际低), Some(实际高)) = (
                    Self::_实际低点(&基础序列, false),
                    Self::_实际高点(&基础序列, 配置.笔内相同终点取舍),
                )
                && Arc::ptr_eq(&筆.文.中, &实际低)
                && Arc::ptr_eq(&筆.武.read().中, &实际高)
            {
                return true;
            }
        }
        false
    }

    /// 获取所有停顿位置 — 在笔范围内找出所有能成笔的分型组合
    pub fn 获取所有停顿位置(筆: &虚线, 观察员: &观察者) -> Vec<虚线> {
        let 基础序列 = 筆.获取缠K序列(&观察员.缠论K线序列);
        let mut 笔序列 = Vec::with_capacity(基础序列.len() / 2);
        let 文 = Arc::clone(&筆.文);

        if 基础序列.len() < 5 {
            return 笔序列;
        }

        for i in 3..基础序列.len() - 1 {
            let k = &基础序列[i];

            let 匹配顶 = *k.分型.read() == Some(分型结构::顶) && 筆.方向() == 相对方向::向上;
            let 匹配底 = *k.分型.read() == Some(分型结构::底) && 筆.方向() == 相对方向::向下;
            if 匹配顶 || 匹配底 {
                let 左 = Arc::clone(&基础序列[i - 1]);
                let 中 = Arc::clone(k);
                let 右 = Arc::clone(&基础序列[i + 1]);
                let 武 = 分型::new(Some(左), 中, Some(右));
                let 当前笔 = 虚线::创建笔(Arc::clone(&文), Arc::new(武), true);
                当前笔
                    .序号
                    .store(筆.序号.load(Ordering::Relaxed), Ordering::Relaxed);
                if Self::_自检(&当前笔, 观察员) {
                    笔序列.push(当前笔);
                }
            }
        }

        笔序列
    }

    /// 是否背驰过 — 判断笔是否在停顿位置出现过MACD趋向背驰
    pub fn 是否背驰过(当前筆: &虚线, 观察员: &观察者) -> Vec<Arc<缠论K线>> {
        let 停顿位置 = Self::获取所有停顿位置(当前筆, 观察员);
        let mut 结果 = Vec::new();

        for 筆 in &停顿位置 {
            let k线范围 = K线::截取rc(
                &观察员.普通K线序列,
                &当前筆.文.中.标的K线.read().clone(),
                &当前筆.武.read().中.标的K线.read().clone(),
            );
            let 背驰信号 = 虚线::计算K线序列MACD趋向背驰(&k线范围, 筆.方向());
            if 背驰信号.iter().all(|&x| x) {
                结果.push(Arc::clone(&筆.武.read().中));
            }
        }

        结果
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kline::bar::K线;
    use crate::kline::chan_kline::缠论K线;
    use crate::structure::fractal_obj::分型;
    use crate::types::分型结构;
    use std::sync::atomic::Ordering;

    fn 辅助_创建普K(时间戳: i64, 高: f64, 低: f64) -> Arc<K线> {
        Arc::new(K线 {
            时间戳,
            高,
            低,
            开盘价: 低,
            收盘价: 高,
            ..Default::default()
        })
    }

    fn 辅助_创建缠K(
        时间戳: i64,
        高: f64,
        低: f64,
        方向: 相对方向,
        分型: Option<分型结构>,
    ) -> Arc<缠论K线> {
        let 普K = 辅助_创建普K(时间戳, 高, 低);
        Arc::new(缠论K线::创建缠K(
            时间戳, 高, 低, 方向, 分型, 0, 普K, None,
        ))
    }

    fn 辅助_创建分型(
        左: Arc<缠论K线>, 中: Arc<缠论K线>, 右: Arc<缠论K线>
    ) -> Arc<分型> {
        Arc::new(分型::new(Some(左), 中, Some(右)))
    }

    // ========== _次高 测试 ==========

    #[test]
    fn test_次高_基本查找() {
        let ks: Vec<Arc<缠论K线>> = vec![
            辅助_创建缠K(100, 100.0, 95.0, 相对方向::向上, None),
            辅助_创建缠K(200, 110.0, 98.0, 相对方向::向上, None),
            辅助_创建缠K(300, 105.0, 97.0, 相对方向::向上, None),
        ];
        // 最高=110(ts=200), 次高=105(ts=300)
        let r = 笔::_次高(&ks, false).unwrap();
        assert_eq!(r.高.get(), 105.0);
        assert_eq!(r.时间戳.load(Ordering::Relaxed), 300);
    }

    #[test]
    fn test_次高_取舍取最后() {
        let ks: Vec<Arc<缠论K线>> = vec![
            辅助_创建缠K(100, 100.0, 95.0, 相对方向::向上, None),
            辅助_创建缠K(200, 110.0, 98.0, 相对方向::向上, None),
            辅助_创建缠K(300, 105.0, 97.0, 相对方向::向上, None),
            辅助_创建缠K(400, 105.0, 96.0, 相对方向::向上, None),
        ];
        let r = 笔::_次高(&ks, true).unwrap(); // 取舍=true: 取最晚的 105
        assert_eq!(r.高.get(), 105.0);
        assert_eq!(r.时间戳.load(Ordering::Relaxed), 400);
    }

    // ========== _次低 测试 ==========

    #[test]
    fn test_次低_基本查找() {
        let ks: Vec<Arc<缠论K线>> = vec![
            辅助_创建缠K(100, 100.0, 95.0, 相对方向::向下, None),
            辅助_创建缠K(200, 100.0, 90.0, 相对方向::向下, None),
            辅助_创建缠K(300, 100.0, 93.0, 相对方向::向下, None),
        ];
        // 最低=90(ts=200), 次低=93(ts=300), 取舍=false: 取最早
        let r = 笔::_次低(&ks, false).unwrap();
        assert_eq!(r.低.get(), 93.0);
    }

    #[test]
    fn test_次低_取舍取最后() {
        let ks: Vec<Arc<缠论K线>> = vec![
            辅助_创建缠K(100, 100.0, 95.0, 相对方向::向下, None),
            辅助_创建缠K(200, 100.0, 90.0, 相对方向::向下, None),
            辅助_创建缠K(300, 100.0, 93.0, 相对方向::向下, None),
            辅助_创建缠K(400, 100.0, 93.0, 相对方向::向下, None),
        ];
        let r = 笔::_次低(&ks, true).unwrap();
        assert_eq!(r.低.get(), 93.0);
        assert_eq!(r.时间戳.load(Ordering::Relaxed), 400);
    }

    // ========== _实际高点 测试 ==========

    #[test]
    fn test_实际高点_单最高() {
        let ks: Vec<Arc<缠论K线>> = vec![
            辅助_创建缠K(100, 100.0, 95.0, 相对方向::向上, None),
            辅助_创建缠K(200, 110.0, 98.0, 相对方向::向上, None),
            辅助_创建缠K(300, 105.0, 97.0, 相对方向::向上, None),
        ];
        let r = 笔::_实际高点(&ks, false).unwrap();
        assert_eq!(r.高.get(), 110.0);
    }

    #[test]
    fn test_实际高点_多个相同最高_取舍false取最早() {
        let ks: Vec<Arc<缠论K线>> = vec![
            辅助_创建缠K(100, 110.0, 95.0, 相对方向::向上, None),
            辅助_创建缠K(200, 110.0, 98.0, 相对方向::向上, None),
            辅助_创建缠K(300, 105.0, 97.0, 相对方向::向上, None),
        ];
        let r = 笔::_实际高点(&ks, false).unwrap();
        assert_eq!(r.高.get(), 110.0);
        assert_eq!(r.时间戳.load(Ordering::Relaxed), 100);
    }

    #[test]
    fn test_实际高点_多个相同最高_取舍true取最晚() {
        let ks: Vec<Arc<缠论K线>> = vec![
            辅助_创建缠K(100, 110.0, 95.0, 相对方向::向上, None),
            辅助_创建缠K(200, 110.0, 98.0, 相对方向::向上, None),
            辅助_创建缠K(300, 105.0, 97.0, 相对方向::向上, None),
        ];
        let r = 笔::_实际高点(&ks, true).unwrap();
        assert_eq!(r.时间戳.load(Ordering::Relaxed), 200);
    }

    #[test]
    fn test_实际高点_空序列() {
        let ks: Vec<Arc<缠论K线>> = vec![];
        assert!(笔::_实际高点(&ks, false).is_none());
    }

    // ========== _实际低点 测试 ==========

    #[test]
    fn test_实际低点_基本() {
        let ks: Vec<Arc<缠论K线>> = vec![
            辅助_创建缠K(100, 100.0, 95.0, 相对方向::向下, None),
            辅助_创建缠K(200, 100.0, 90.0, 相对方向::向下, None),
            辅助_创建缠K(300, 100.0, 93.0, 相对方向::向下, None),
        ];
        let r = 笔::_实际低点(&ks, false).unwrap();
        assert_eq!(r.低.get(), 90.0);
    }

    #[test]
    fn test_实际低点_多个相同最低() {
        let ks: Vec<Arc<缠论K线>> = vec![
            辅助_创建缠K(100, 100.0, 90.0, 相对方向::向下, None),
            辅助_创建缠K(200, 100.0, 90.0, 相对方向::向下, None),
            辅助_创建缠K(300, 100.0, 95.0, 相对方向::向下, None),
        ];
        let r = 笔::_实际低点(&ks, false).unwrap();
        assert_eq!(r.低.get(), 90.0);
        assert_eq!(r.时间戳.load(Ordering::Relaxed), 100);
    }

    #[test]
    fn test_实际低点_空序列() {
        let ks: Vec<Arc<缠论K线>> = vec![];
        assert!(笔::_实际低点(&ks, false).is_none());
    }

    // ========== 以文会友 测试 ==========

    #[test]
    fn test_以文会友_找到匹配() {
        let 文 = 辅助_创建分型(
            辅助_创建缠K(90, 105.0, 100.0, 相对方向::向上, Some(分型结构::上)),
            辅助_创建缠K(100, 110.0, 100.0, 相对方向::向上, Some(分型结构::顶)),
            辅助_创建缠K(110, 105.0, 100.0, 相对方向::向下, Some(分型结构::下)),
        );
        let 武 = 辅助_创建分型(
            辅助_创建缠K(190, 95.0, 85.0, 相对方向::向下, Some(分型结构::下)),
            辅助_创建缠K(200, 95.0, 80.0, 相对方向::向下, Some(分型结构::底)),
            辅助_创建缠K(210, 100.0, 85.0, 相对方向::向上, Some(分型结构::上)),
        );
        let 筆 = 虚线::创建笔(文.clone(), 武, true);
        let 笔序列 = vec![Arc::new(筆)];

        let found = 笔::以文会友(&笔序列, &文);
        assert!(found.is_some());
    }

    #[test]
    fn test_以文会友_找不到匹配() {
        let 文 = 辅助_创建分型(
            辅助_创建缠K(90, 105.0, 100.0, 相对方向::向上, Some(分型结构::上)),
            辅助_创建缠K(100, 110.0, 100.0, 相对方向::向上, Some(分型结构::顶)),
            辅助_创建缠K(110, 105.0, 100.0, 相对方向::向下, Some(分型结构::下)),
        );
        let other_文 = 辅助_创建分型(
            辅助_创建缠K(190, 90.0, 80.0, 相对方向::向上, Some(分型结构::上)),
            辅助_创建缠K(200, 95.0, 80.0, 相对方向::向上, Some(分型结构::顶)),
            辅助_创建缠K(210, 90.0, 80.0, 相对方向::向下, Some(分型结构::下)),
        );
        let 武 = 辅助_创建分型(
            辅助_创建缠K(290, 85.0, 75.0, 相对方向::向下, Some(分型结构::下)),
            辅助_创建缠K(300, 85.0, 70.0, 相对方向::向下, Some(分型结构::底)),
            辅助_创建缠K(310, 90.0, 75.0, 相对方向::向上, Some(分型结构::上)),
        );
        let 筆 = 虚线::创建笔(other_文, 武, true);
        let 笔序列 = vec![Arc::new(筆)];

        let found = 笔::以文会友(&笔序列, &文);
        assert!(found.is_none());
    }

    // ========== 以武会友 测试 ==========

    #[test]
    fn test_以武会友_找到匹配() {
        let 文 = 辅助_创建分型(
            辅助_创建缠K(90, 105.0, 100.0, 相对方向::向上, Some(分型结构::上)),
            辅助_创建缠K(100, 110.0, 100.0, 相对方向::向上, Some(分型结构::顶)),
            辅助_创建缠K(110, 105.0, 100.0, 相对方向::向下, Some(分型结构::下)),
        );
        let 武 = 辅助_创建分型(
            辅助_创建缠K(190, 95.0, 85.0, 相对方向::向下, Some(分型结构::下)),
            辅助_创建缠K(200, 95.0, 80.0, 相对方向::向下, Some(分型结构::底)),
            辅助_创建缠K(210, 100.0, 85.0, 相对方向::向上, Some(分型结构::上)),
        );
        let 筆 = 虚线::创建笔(文, 武.clone(), true);
        let 笔序列 = vec![Arc::new(筆)];

        let found = 笔::以武会友(&笔序列, &武);
        assert!(found.is_some());
    }

    #[test]
    fn test_以武会友_找不到匹配() {
        let 文 = 辅助_创建分型(
            辅助_创建缠K(90, 105.0, 100.0, 相对方向::向上, Some(分型结构::上)),
            辅助_创建缠K(100, 110.0, 100.0, 相对方向::向上, Some(分型结构::顶)),
            辅助_创建缠K(110, 105.0, 100.0, 相对方向::向下, Some(分型结构::下)),
        );
        let 武 = 辅助_创建分型(
            辅助_创建缠K(190, 95.0, 85.0, 相对方向::向下, Some(分型结构::下)),
            辅助_创建缠K(200, 95.0, 80.0, 相对方向::向下, Some(分型结构::底)),
            辅助_创建缠K(210, 100.0, 85.0, 相对方向::向上, Some(分型结构::上)),
        );
        let other_武 = 辅助_创建分型(
            辅助_创建缠K(290, 85.0, 75.0, 相对方向::向下, Some(分型结构::下)),
            辅助_创建缠K(300, 85.0, 65.0, 相对方向::向下, Some(分型结构::底)),
            辅助_创建缠K(310, 90.0, 75.0, 相对方向::向上, Some(分型结构::上)),
        );
        let 筆 = 虚线::创建笔(文, 武, true);
        let 笔序列 = vec![Arc::new(筆)];

        let found = 笔::以武会友(&笔序列, &other_武);
        assert!(found.is_none());
    }
}
