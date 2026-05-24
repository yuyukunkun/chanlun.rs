use crate::business::observer::观察者;
use crate::config::缠论配置;
use crate::kline::bar::K线;
use crate::kline::chan_kline::缠论K线;
use crate::structure::dash_line::虚线;
use crate::structure::fractal_obj::分型;
use crate::types::{分型结构, 相对方向};
use std::rc::Rc;

/// 笔 — 从分型生成笔的算法集合（静态方法命名空间）
pub struct 笔;

impl 笔 {
    /// 获取可成笔的缠K数量（考虑弱化模式）
    pub fn 获取缠K数量(缠K序列: &[Rc<缠论K线>], 笔序列: &[Rc<虚线>], 配置: &缠论配置) -> usize {
        let 实际数量 = 缠K序列.len();
        if 实际数量 >= 配置.笔内元素数量 as usize {
            return 实际数量;
        }

        if 配置.笔弱化 && 实际数量 >= 3 {
            let 实际高点 = Self::实际高点(缠K序列, 配置.笔内相同终点取舍);
            let 实际低点 = Self::实际低点(缠K序列, 配置.笔内相同终点取舍);

            if let (Some(ref 高点), Some(ref 低点)) = (&实际高点, &实际低点) {
                let 原始数量 = 1 + (低点.标的K线.序号 - 高点.标的K线.序号).unsigned_abs() as usize;
                if 原始数量 >= 配置.笔内元素数量 as usize {
                    return 配置.笔内元素数量 as usize;
                }
            }

            if !笔序列.is_empty() {
                // Try both high and low points (Python: 根据缠K找笔(笔序列, 实际高点) or 根据缠K找笔(笔序列, 实际低点))
                let 筆 = 实际高点.as_ref()
                    .and_then(|h| Self::根据缠K找笔(笔序列, h, 1))
                    .or_else(|| 实际低点.as_ref().and_then(|l| Self::根据缠K找笔(笔序列, l, 1)));

                if let Some(ref 筆) = 筆 {
                    if let (Some(ref 高_k), Some(ref 低_k)) = (&实际高点, &实际低点) {
                        let 原始数量 = 1 + (低_k.标的K线.序号 - 高_k.标的K线.序号).unsigned_abs() as usize;
                        // 向上笔
                        if 筆.方向().是否向上() && 低_k.低 < 筆.低() {
                            if 原始数量 >= 配置.笔弱化_原始数量 as usize {
                                return 配置.笔内元素数量 as usize;
                            }
                        }
                        // 向下笔
                        if 筆.方向().是否向下() && 低_k.低 > 筆.高() {
                            if 原始数量 >= 配置.笔弱化_原始数量 as usize {
                                return 配置.笔内元素数量 as usize;
                            }
                        }
                    }
                }
            }
        }
        实际数量
    }

    /// 次高 — 排除最高值后的次高点
    pub fn 次高(缠K序列: &[Rc<缠论K线>], 取舍: bool) -> Option<Rc<缠论K线>> {
        if 缠K序列.len() < 2 {
            return 缠K序列.first().cloned();
        }
        let max_高 = 缠K序列.iter().map(|k| k.高).fold(f64::NEG_INFINITY, f64::max);
        // 排除最高值
        let filtered: Vec<&Rc<缠论K线>> = 缠K序列.iter().filter(|k| k.高 != max_高).collect();
        if filtered.is_empty() {
            return 缠K序列.first().cloned();
        }
        // 筛选次高值
        let second_高 = filtered.iter().map(|k| k.高).fold(f64::NEG_INFINITY, f64::max);
        let mut candidates: Vec<&Rc<缠论K线>> = filtered.iter().filter(|k| k.高 == second_高).copied().collect();
        // 按时间戳排序
        candidates.sort_by(|a, b| a.时间戳.cmp(&b.时间戳));
        if 取舍 {
            Some(Rc::clone(candidates[candidates.len() - 1]))
        } else {
            Some(Rc::clone(candidates[0]))
        }
    }

    /// 次低 — 排除最低值后的次低点
    pub fn 次低(缠K序列: &[Rc<缠论K线>], 取舍: bool) -> Option<Rc<缠论K线>> {
        if 缠K序列.len() < 2 {
            return 缠K序列.first().cloned();
        }
        let min_低 = 缠K序列.iter().map(|k| k.低).fold(f64::INFINITY, f64::min);
        // 排除最低值
        let filtered: Vec<&Rc<缠论K线>> = 缠K序列.iter().filter(|k| k.低 != min_低).collect();
        if filtered.is_empty() {
            return 缠K序列.first().cloned();
        }
        // 筛选次低值
        let second_低 = filtered.iter().map(|k| k.低).fold(f64::INFINITY, f64::min);
        let mut candidates: Vec<&Rc<缠论K线>> = filtered.iter().filter(|k| k.低 == second_低).copied().collect();
        // 按时间戳排序
        candidates.sort_by(|a, b| a.时间戳.cmp(&b.时间戳));
        if 取舍 {
            Some(Rc::clone(candidates[candidates.len() - 1]))
        } else {
            Some(Rc::clone(candidates[0]))
        }
    }

    /// 实际高点
    pub fn 实际高点(缠K序列: &[Rc<缠论K线>], 取舍: bool) -> Option<Rc<缠论K线>> {
        if 缠K序列.is_empty() {
            return None;
        }
        let max_高 = 缠K序列.iter().map(|k| k.高).fold(f64::NEG_INFINITY, f64::max);
        let mut candidates: Vec<&Rc<缠论K线>> = 缠K序列.iter().filter(|k| k.高 == max_高).collect();
        if candidates.is_empty() {
            return Some(Rc::clone(&缠K序列[0]));
        }
        // 按时间戳排序
        candidates.sort_by(|a, b| a.时间戳.cmp(&b.时间戳));
        if 取舍 {
            Some(Rc::clone(candidates[candidates.len() - 1]))
        } else {
            Some(Rc::clone(candidates[0]))
        }
    }

    /// 实际低点
    pub fn 实际低点(缠K序列: &[Rc<缠论K线>], 取舍: bool) -> Option<Rc<缠论K线>> {
        if 缠K序列.is_empty() {
            return None;
        }
        let min_低 = 缠K序列.iter().map(|k| k.低).fold(f64::INFINITY, f64::min);
        let mut candidates: Vec<&Rc<缠论K线>> = 缠K序列.iter().filter(|k| k.低 == min_低).collect();
        if candidates.is_empty() {
            return Some(Rc::clone(&缠K序列[0]));
        }
        // 按时间戳排序
        candidates.sort_by(|a, b| a.时间戳.cmp(&b.时间戳));
        if 取舍 {
            Some(Rc::clone(candidates[candidates.len() - 1]))
        } else {
            Some(Rc::clone(candidates[0]))
        }
    }

    /// 判断笔的相对关系是否合理
    pub fn 相对关系(筆: &虚线, _配置: &缠论配置) -> bool {
        let 文 = &筆.文;
        let 武 = &筆.武;

        // 向上笔：文(底)低 → 武(顶)高
        if 筆.方向().是否向上() {
            return 武.分型特征值 > 文.分型特征值;
        }
        // 向下笔：文(顶)高 → 武(底)低
        武.分型特征值 < 文.分型特征值
    }

    /// 以文会友 — 根据起点分型找笔
    pub fn 以文会友(笔序列: &[Rc<虚线>], 文: &Rc<分型>) -> Option<Rc<虚线>> {
        笔序列.iter().find(|b| Rc::as_ptr(&b.文) == Rc::as_ptr(文)).cloned()
    }

    /// 以武会友 — 根据终点分型找笔
    pub fn 以武会友(笔序列: &[Rc<虚线>], 武: &Rc<分型>) -> Option<Rc<虚线>> {
        笔序列.iter().find(|b| Rc::as_ptr(&b.武) == Rc::as_ptr(武)).cloned()
    }

    /// 根据缠K找对应的笔
    pub fn 根据缠K找笔(笔序列: &[Rc<虚线>], 缠K: &Rc<缠论K线>, 偏移: i64) -> Option<Rc<虚线>> {
        // Python iterates in reverse: for 筆 in 笔序列[::-1]
        for b in 笔序列.iter().rev() {
            // Python: 筆.文.中.序号 - 偏移 <= 缠K.序号 <= 筆.武.中.序号
            if b.文.中.序号 - 偏移 <= 缠K.序号 && 缠K.序号 <= b.武.中.序号 {
                return Some(Rc::clone(b));
            }
        }
        None
    }

    /// 从分型序列中弹出最后一个分型和对应的笔
    fn 弹出旧笔(分型序列: &mut Vec<Rc<分型>>, 笔序列: &mut Vec<Rc<虚线>>) {
        分型序列.pop();
        if !笔序列.is_empty() {
            // Python sets旧笔.有效性 = False; with Rc we just drop the笔
            笔序列.pop();
        }
    }

    /// 核心笔分析 — 使用显式栈模拟递归
    ///
    /// 返回: 递归层次数
    pub fn 分析(
        初始分型: Rc<分型>,
        分型序列: &mut Vec<Rc<分型>>,
        笔序列: &mut Vec<Rc<虚线>>,
        缠K序列: &[Rc<缠论K线>],
        _普K序列: &[Rc<K线>],
        配置: &缠论配置,
    ) -> i64 {
        enum 栈项 {
            分型(Rc<分型>, i64),
            /// 修复错过笔哨兵: 若临时分型被接受为最后一个元素，则扫描武将之后的所有分型
            修复错过笔 {
                临时分型: Rc<分型>,
                武将缠K: Rc<缠论K线>,
                层次: i64,
            },
        }

        let mut 栈: Vec<栈项> = Vec::new();
        栈.push(栈项::分型(初始分型, 0));

        while let Some(项) = 栈.pop() {
            let (当前分型, 递归层次) = match 项 {
                栈项::分型(fx, lvl) => (fx, lvl),
                栈项::修复错过笔 {
                    临时分型,
                    武将缠K,
                    层次,
                } => {
                    // Python line 2406: only scan if临时分型 was accepted as last element
                    if !分型序列.is_empty() {
                        if let Some(last_fx) = 分型序列.last() {
                            if Rc::as_ptr(last_fx) == Rc::as_ptr(&临时分型) {
                                if let Some(武_idx) = 缠K序列
                                    .iter()
                                    .position(|k| Rc::as_ptr(k) == Rc::as_ptr(&武将缠K))
                                {
                                    let mut 错过: Vec<Rc<分型>> = Vec::new();
                                    for ck in &缠K序列[武_idx..] {
                                        if ck.分型 == Some(分型结构::底)
                                            || ck.分型 == Some(分型结构::顶)
                                        {
                                            if let Some(fx) =
                                                分型::从缠K序列中获取分型(缠K序列, ck)
                                            {
                                                错过.push(Rc::new(fx));
                                            }
                                        }
                                    }
                                    // Push in reverse so first in slice is processed first
                                    for m in 错过.into_iter().rev() {
                                        栈.push(栈项::分型(m, 层次 + 1));
                                    }
                                }
                            }
                        }
                    }
                    // 当前分型 is already next on the stack (pushed before this哨兵)
                    continue;
                }
            };

            if 递归层次 > 256 {
                continue;
            }

            if !matches!(当前分型.结构, 分型结构::顶 | 分型结构::底) {
                continue;
            }

            // Python line 2322-2325: 第一个分型直接追加
            if 分型序列.is_empty() {
                分型序列.push(当前分型);
                continue;
            }

            let 之前分型 = Rc::clone(分型序列.last().unwrap());

            // Python line 2330-2335: 清理无效数据
            if 之前分型.时间戳 == 当前分型.时间戳
                || matches!(之前分型.结构, 分型结构::上 | 分型结构::下)
            {
                Self::弹出旧笔(分型序列, 笔序列);
                if 分型序列.is_empty() {
                    if 当前分型.右.is_some() {
                        分型序列.push(当前分型);
                    }
                    continue;
                }
            }

            let 之前分型 = Rc::clone(分型序列.last().unwrap());

            // Python line 2338: 时序检查 — skip out-of-order fractals
            if 之前分型.时间戳 > 当前分型.时间戳 && 之前分型.中.序号 - 当前分型.中.序号 > 1 {
                continue;
            }

            // Python line 2343-2348: 笔弱化模式
            if 配置.笔弱化 && !笔序列.is_empty() {
                let 前一笔 = 笔序列.last().unwrap();
                let 前一笔缠K数 = 前一笔.武.中.序号 - 前一笔.文.中.序号 + 1;
                if 前一笔缠K数 == 3 {
                    let 破位 = (前一笔.方向().是否向上()
                        && 前一笔.低() > 当前分型.分型特征值
                        && 当前分型.结构 == 分型结构::底)
                        || (前一笔.方向().是否向下()
                            && 前一笔.高() < 当前分型.分型特征值
                            && 当前分型.结构 == 分型结构::顶);
                    if 破位 {
                        Self::弹出旧笔(分型序列, 笔序列);
                        栈.push(栈项::分型(当前分型, 递归层次 + 1));
                        continue;
                    }
                }
            }

            // Re-read之前分型 again after笔弱化 pop
            let 之前分型 = Rc::clone(分型序列.last().unwrap());

            // Python line 2350: 分型结构相反 → 可能成笔
            if 之前分型.结构 != 当前分型.结构 {
                let 文_idx = 缠K序列.iter().position(|k| Rc::as_ptr(k) == Rc::as_ptr(&之前分型.中));
                let 武_idx = 缠K序列.iter().position(|k| Rc::as_ptr(k) == Rc::as_ptr(&当前分型.中));

                if let (Some(文_idx), Some(武_idx)) = (文_idx, 武_idx) {
                    let 基础序列 = &缠K序列[文_idx..=武_idx];
                    let 缠K数量 = Self::获取缠K数量(基础序列, 笔序列, 配置);

                    if 缠K数量 >= 配置.笔内元素数量 as usize {
                        // Python line 2354-2357: 文官 always uses false (not 笔内相同终点取舍)
                        let 文官 = match 之前分型.结构 {
                            分型结构::顶 => Self::实际高点(基础序列, false),
                            _ => Self::实际低点(基础序列, false),
                        };

                        // Python line 2359-2367: 文官 != 之前分型.中 → adjust
                        if let Some(ref 文官_k) = 文官 {
                            if Rc::as_ptr(文官_k) != Rc::as_ptr(&之前分型.中) {
                                if let Some(临时分型) =
                                    分型::从缠K序列中获取分型(缠K序列, 文官_k)
                                {
                                    栈.push(栈项::分型(当前分型, 递归层次));
                                    栈.push(栈项::分型(Rc::new(临时分型), 递归层次 + 1));
                                    continue;
                                }
                            }
                        }

                        // Python line 2369-2372: 武将
                        let 武将 = match 当前分型.结构 {
                            分型结构::底 => Self::实际低点(基础序列, 配置.笔内相同终点取舍),
                            _ => Self::实际高点(基础序列, 配置.笔内相同终点取舍),
                        };

                        let 新笔 = Rc::new(虚线::创建笔(
                            Rc::clone(&之前分型),
                            Rc::clone(&当前分型),
                            true,
                        ));

                        // Python line 2374-2376: 相对关系 and武将 matches
                        if Self::相对关系(&新笔, 配置) {
                            if let Some(ref 武将_k) = 武将 {
                                if Rc::as_ptr(武将_k) == Rc::as_ptr(&当前分型.中) {
                                    Self::_添加新笔递归(分型序列, 笔序列, 当前分型, 新笔);
                                    continue;
                                }
                            }
                        }

                        // Python line 2378-2385: 笔次级成笔
                        if 配置.笔次级成笔 {
                            let 武将 = match 当前分型.结构 {
                                分型结构::底 => Self::次低(基础序列, 配置.笔内相同终点取舍),
                                _ => Self::次高(基础序列, 配置.笔内相同终点取舍),
                            };
                            if let Some(ref 武将_k) = 武将 {
                                if Rc::as_ptr(武将_k) == Rc::as_ptr(&当前分型.中)
                                    && Self::相对关系(&新笔, 配置)
                                {
                                    Self::_添加新笔递归(分型序列, 笔序列, 当前分型, 新笔);
                                    continue;
                                }
                            }
                        }
                    } else {
                        // Python line 2388-2390: 元素不足 → 右元素扩展
                        if let Some(ref 右) = 当前分型.右 {
                            if let Some(临时分型) = 分型::从缠K序列中获取分型(缠K序列, 右) {
                                栈.push(栈项::分型(Rc::new(临时分型), 递归层次 + 1));
                                continue;
                            }
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
                    let 被替换分型 = Rc::clone(&之前分型);
                    Self::弹出旧笔(分型序列, 笔序列);

                    if let Some(k线序列) =
                        缠论K线::截取(缠K序列, &被替换分型.中, &当前分型.中)
                    {
                        let 武将 = match 被替换分型.结构 {
                            分型结构::顶 => {
                                Self::实际低点(&k线序列, 配置.笔内相同终点取舍)
                            }
                            _ => Self::实际高点(&k线序列, 配置.笔内相同终点取舍),
                        };

                        if let Some(ref 武将_k) = 武将 {
                            if let Some(临时分型) =
                                分型::从缠K序列中获取分型(缠K序列, 武将_k)
                            {
                                let 临时分型_rc = Rc::new(临时分型);

                                if !分型序列.is_empty() {
                                    // Push in reverse processing order (LIFO):
                                    栈.push(栈项::分型(Rc::clone(&当前分型), 递归层次 + 2));
                                    栈.push(栈项::修复错过笔 {
                                        临时分型: Rc::clone(&临时分型_rc),
                                        武将缠K: Rc::clone(武将_k),
                                        层次: 递归层次 + 1,
                                    });
                                    栈.push(栈项::分型(临时分型_rc, 递归层次 + 1));
                                    continue;
                                } else {
                                    分型序列.push(当前分型);
                                    continue;
                                }
                            }
                        }
                    }

                    if 分型序列.is_empty() {
                        分型序列.push(当前分型);
                    } else {
                        栈.push(栈项::分型(当前分型, 递归层次 + 1));
                    }
                }
            }
        }

        栈.len() as i64
    }

    /// 核心笔分析 — 递归实现，逐句对照 chan.py 笔.分析 / 笔递归分析
    ///
    /// 返回: 递归层次数
    pub fn 分析递归(
        当前分型: Rc<分型>,
        分型序列: &mut Vec<Rc<分型>>,
        笔序列: &mut Vec<Rc<虚线>>,
        缠K序列: &[Rc<缠论K线>],
        _普K序列: &[Rc<K线>],
        递归层次: i64,
        配置: &缠论配置,
    ) -> i64 {
        // Python line 2315-2317: 递归深度限制
        if 递归层次 > 64 {
            println!("笔.分析 递归深度超出 64 < {}", 递归层次);
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
        let 之前分型 = Rc::clone(分型序列.last().unwrap());
        if 之前分型.时间戳 == 当前分型.时间戳
            || matches!(之前分型.结构, 分型结构::上 | 分型结构::下)
        {
            Self::弹出旧笔(分型序列, 笔序列);
            if 分型序列.is_empty() {
                if 当前分型.右.is_some() {
                    分型::向序列中添加(分型序列, 当前分型);
                }
                return 递归层次;
            }
        }

        // Python line 2337-2341: 时序检查
        let 之前分型 = Rc::clone(分型序列.last().unwrap());
        if 之前分型.时间戳 > 当前分型.时间戳 && 之前分型.中.序号 - 当前分型.中.序号 > 1 {
            println!("时序错误-{}, {}, {}", 递归层次, 之前分型, 当前分型);
            return 递归层次;
        }

        // Python line 2343-2348: 笔弱化模式
        if 配置.笔弱化 && !笔序列.is_empty() {
            let 前一笔 = 笔序列.last().unwrap();
            let 前一笔缠K数 = 前一笔.武.中.序号 - 前一笔.文.中.序号 + 1;
            if 前一笔缠K数 == 3 {
                let 破位 = (前一笔.方向().是否向上()
                    && 前一笔.低() > 当前分型.分型特征值
                    && 当前分型.结构 == 分型结构::底)
                    || (前一笔.方向().是否向下()
                        && 前一笔.高() < 当前分型.分型特征值
                        && 当前分型.结构 == 分型结构::顶);
                if 破位 {
                    Self::弹出旧笔(分型序列, 笔序列);
                    return Self::分析递归(
                        当前分型, 分型序列, 笔序列, 缠K序列, _普K序列, 递归层次 + 1, 配置,
                    );
                }
            }
        }

        // Python line 2350: 分型结构相反 → 可能成笔
        let 之前分型 = Rc::clone(分型序列.last().unwrap());
        if 之前分型.结构 != 当前分型.结构 {
            if let Some(基础序列) = 缠论K线::截取(缠K序列, &之前分型.中, &当前分型.中) {
                let 当前笔 = Rc::new(虚线::创建笔(
                    Rc::clone(&之前分型),
                    Rc::clone(&当前分型),
                    true,
                ));

                if Self::获取缠K数量(&基础序列, 笔序列, 配置) >= 配置.笔内元素数量 as usize
                {
                    // Python line 2354-2357: 文官
                    let 文官 = match 之前分型.结构 {
                        分型结构::顶 => Self::实际高点(&基础序列, false),
                        _ => Self::实际低点(&基础序列, false),
                    };

                    // Python line 2359-2367: 文官调整
                    if let Some(ref 文官_k) = 文官 {
                        if Rc::as_ptr(文官_k) != Rc::as_ptr(&之前分型.中) {
                            if let Some(临时分型) =
                                分型::从缠K序列中获取分型(缠K序列, 文官_k)
                            {
                                let 递归层次 = Self::分析递归(
                                    Rc::new(临时分型),
                                    分型序列,
                                    笔序列,
                                    缠K序列,
                                    _普K序列,
                                    递归层次 + 1,
                                    配置,
                                );
                                return Self::分析递归(
                                    当前分型, 分型序列, 笔序列, 缠K序列, _普K序列,
                                    递归层次 + 1,
                                    配置,
                                );
                            }
                        }
                    }

                    // Python line 2369-2375: 武将 and笔形成
                    let 武将 = match 当前分型.结构 {
                        分型结构::底 => {
                            Self::实际低点(&基础序列, 配置.笔内相同终点取舍)
                        }
                        _ => Self::实际高点(&基础序列, 配置.笔内相同终点取舍),
                    };

                    if Self::相对关系(&当前笔, 配置) {
                        if let Some(ref 武将_k) = 武将 {
                            if Rc::as_ptr(武将_k) == Rc::as_ptr(&当前分型.中) {
                                // 直接添加（对照 Python _添加新笔：直接 append）
                                Self::_添加新笔递归(分型序列, 笔序列, 当前分型, 当前笔);
                                return 递归层次;
                            }
                        }
                    }

                    // Python line 2378-2385: 笔次级成笔
                    if 配置.笔次级成笔 {
                        let 武将 = match 当前分型.结构 {
                            分型结构::底 => {
                                Self::次低(&基础序列, 配置.笔内相同终点取舍)
                            }
                            _ => Self::次高(&基础序列, 配置.笔内相同终点取舍),
                        };
                        if let Some(ref 武将_k) = 武将 {
                            if Rc::as_ptr(武将_k) == Rc::as_ptr(&当前分型.中)
                                && Self::相对关系(&当前笔, 配置)
                            {
                                Self::_添加新笔递归(分型序列, 笔序列, 当前分型, 当前笔);
                                return 递归层次;
                            }
                        }
                    }
                } else {
                    // Python line 2388-2390: 元素不足 → 右元素扩展
                    if let Some(ref 右) = 当前分型.右 {
                        if let Some(临时分型) =
                            分型::从缠K序列中获取分型(缠K序列, 右)
                        {
                            return Self::分析递归(
                                Rc::new(临时分型),
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
                let 被替换分型 = Rc::clone(&之前分型);
                Self::弹出旧笔(分型序列, 笔序列);

                if let Some(k线序列) =
                    缠论K线::截取(缠K序列, &被替换分型.中, &当前分型.中)
                {
                    let 武将 = match 被替换分型.结构 {
                        分型结构::顶 => {
                            Self::实际低点(&k线序列, 配置.笔内相同终点取舍)
                        }
                        _ => Self::实际高点(&k线序列, 配置.笔内相同终点取舍),
                    };

                    if let Some(ref 武将_k) = 武将 {
                        if let Some(临时分型) =
                            分型::从缠K序列中获取分型(缠K序列, 武将_k)
                        {
                            let 临时分型_rc = Rc::new(临时分型);

                            if !分型序列.is_empty() {
                                let mut 递归层次 = Self::分析递归(
                                    Rc::clone(&临时分型_rc),
                                    分型序列,
                                    笔序列,
                                    缠K序列,
                                    _普K序列,
                                    递归层次 + 1,
                                    配置,
                                );

                                // 修复错过的笔: 扫描武将之后的所有分型
                                if !分型序列.is_empty()
                                    && Rc::as_ptr(分型序列.last().unwrap())
                                        == Rc::as_ptr(&临时分型_rc)
                                {
                                    if let Some(武_idx) = 缠K序列
                                        .iter()
                                        .position(|k| Rc::as_ptr(k) == Rc::as_ptr(武将_k))
                                    {
                                        for ck in &缠K序列[武_idx..] {
                                            if ck.分型 == Some(分型结构::底)
                                                || ck.分型 == Some(分型结构::顶)
                                            {
                                                if let Some(错过分型) =
                                                    分型::从缠K序列中获取分型(缠K序列, ck)
                                                {
                                                    let 错过分型_rc =
                                                        Rc::new(错过分型);
                                                    递归层次 = Self::分析递归(
                                                        Rc::clone(&错过分型_rc),
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
                                    }
                                }

                                return Self::分析递归(
                                    当前分型, 分型序列, 笔序列, 缠K序列, _普K序列,
                                    递归层次 + 1,
                                    配置,
                                );
                            } else {
                                分型::向序列中添加(分型序列, 当前分型);
                            }
                        }
                    }
                } else if 分型序列.is_empty() {
                    分型::向序列中添加(分型序列, 当前分型);
                } else {
                    return Self::分析递归(
                        当前分型, 分型序列, 笔序列, 缠K序列, _普K序列, 递归层次 + 1, 配置,
                    );
                }
            }
        }

        递归层次
    }

    /// 添加新笔到序列（递归版本 — 直接追加，对应 Python _添加新笔）
    fn _添加新笔递归(
        分型序列: &mut Vec<Rc<分型>>,
        笔序列: &mut Vec<Rc<虚线>>,
        新分型: Rc<分型>,
        新笔: Rc<虚线>,
    ) {
        分型序列.push(新分型);
        let 序号 = if 笔序列.is_empty() {
            0
        } else {
            笔序列.last().unwrap().序号 + 1
        };
        let mut 虚线笔 = (*新笔).clone();
        虚线笔.序号 = 序号;
        if 虚线笔.武.左.is_none() && 虚线笔.武.右.is_none() {
            虚线笔.有效性 = false;
        }
        笔序列.push(Rc::new(虚线笔));
    }

    /// 自检 — 验证笔的有效性（文为实际高/低点，武为实际低/高点）
    pub fn 自检(筆: &虚线, 观察员: &观察者) -> bool {
        let 笔序列 = &观察员.笔序列;
        let 配置 = &观察员.配置;
        let 基础序列 = 筆.获取缠K序列(&观察员.缠论K线序列);
        if Self::获取缠K数量(&基础序列, 笔序列, 配置) >= 配置.笔内元素数量 as usize {
            if 筆.方向() == 相对方向::向下 {
                if let (Some(实际高), Some(实际低)) = (
                    Self::实际高点(&基础序列, false),
                    Self::实际低点(&基础序列, 配置.笔内相同终点取舍),
                ) {
                    if Rc::ptr_eq(&筆.文.中, &实际高) && Rc::ptr_eq(&筆.武.中, &实际低) {
                        return true;
                    }
                }
            }
            if 筆.方向() == 相对方向::向上 {
                if let (Some(实际低), Some(实际高)) = (
                    Self::实际低点(&基础序列, false),
                    Self::实际高点(&基础序列, 配置.笔内相同终点取舍),
                ) {
                    if Rc::ptr_eq(&筆.文.中, &实际低) && Rc::ptr_eq(&筆.武.中, &实际高) {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// 获取所有停顿位置 — 在笔范围内找出所有能成笔的分型组合
    pub fn 获取所有停顿位置(筆: &虚线, 观察员: &观察者) -> Vec<虚线> {
        let mut 笔序列 = Vec::new();
        let 文 = Rc::clone(&筆.文);
        let 基础序列 = 筆.获取缠K序列(&观察员.缠论K线序列);

        if 基础序列.len() < 5 {
            return 笔序列;
        }

        for i in 3..基础序列.len() - 1 {
            let k = &基础序列[i];

            if k.分型 == Some(分型结构::顶) && 筆.方向() == 相对方向::向上 {
                let 左 = Rc::clone(&基础序列[i - 1]);
                let 中 = Rc::clone(k);
                let 右 = Rc::clone(&基础序列[i + 1]);
                let 武 = 分型::new(Some(左), 中, Some(右));
                let mut 当前笔 = 虚线::创建笔(Rc::clone(&文), Rc::new(武), true);
                当前笔.序号 = 筆.序号;
                if Self::自检(&当前笔, 观察员) {
                    笔序列.push(当前笔);
                }
            } else if k.分型 == Some(分型结构::底) && 筆.方向() == 相对方向::向下 {
                let 左 = Rc::clone(&基础序列[i - 1]);
                let 中 = Rc::clone(k);
                let 右 = Rc::clone(&基础序列[i + 1]);
                let 武 = 分型::new(Some(左), 中, Some(右));
                let mut 当前笔 = 虚线::创建笔(Rc::clone(&文), Rc::new(武), true);
                当前笔.序号 = 筆.序号;
                if Self::自检(&当前笔, 观察员) {
                    笔序列.push(当前笔);
                }
            }
        }

        笔序列
    }

    /// 是否背驰过 — 判断笔是否在停顿位置出现过MACD趋向背驰
    pub fn 是否背驰过(当前筆: &虚线, 观察员: &观察者) -> Vec<Rc<分型>> {
        let 停顿位置 = Self::获取所有停顿位置(当前筆, 观察员);
        let mut 结果 = Vec::new();

        for 筆 in &停顿位置 {
            let k线范围 = K线::截取rc(
                &观察员.普通K线序列,
                &当前筆.文.中.标的K线,
                &当前筆.武.中.标的K线,
            );
            let 背驰信号 = 虚线::计算K线序列MACD趋向背驰(&k线范围, 筆.方向());
            if 背驰信号.iter().all(|&x| x) {
                结果.push(Rc::clone(&筆.武));
            }
        }

        结果
    }
}
