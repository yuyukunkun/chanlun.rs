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

use crate::algorithm::bi::笔;
use crate::algorithm::hub::中枢;
use crate::business::observer::观察者;
use crate::config::缠论配置;
use crate::kline::bar::K线;
use crate::kline::chan_kline::缠论K线;
use crate::structure::dash_line::虚线;
use crate::structure::fractal_obj::分型;
use crate::structure::segment_feat::线段特征;
use crate::types::{分型结构, 相对方向, 缺口};
use crate::warn;
use std::sync::Arc;
use std::sync::atomic::Ordering;

/// 线段 — 从笔生成线段的算法集合（静态方法命名空间）
pub struct 线段;

type 分割结果 = (
    Vec<Arc<虚线>>,
    Vec<Arc<虚线>>,
    Vec<Arc<虚线>>,
    Option<Arc<虚线>>,
);
type 中枢序列组 = (Vec<Arc<中枢>>, Vec<Arc<中枢>>, Vec<Arc<中枢>>);

impl 线段 {
    // ================================================================
    // 基础操作
    // ================================================================

    /// 从序号计算索引位置（O(1) — 与 Python _索引 对齐）
    ///
    /// 序列元素序号连续递增，因此可用 `项.序号 - 序列[0].序号` 直接得到位置。
    /// 当笔序列中元素被 pop 再 push 替换时（_弹出旧笔 → _添加新笔），
    /// 新旧元素序号相同但 Arc 不同，此时 Arc::ptr_eq 会漏判，而序号算术不受影响。
    pub fn _索引(序列: &[Arc<虚线>], 项: &虚线) -> usize {
        let 项序号 = 项.序号.load(Ordering::Relaxed);
        let 首序号 = 序列[0].序号.load(Ordering::Relaxed);
        (项序号 - 首序号) as usize
    }

    /// 解引用 Arc<虚线> → &虚线。内部 RwLock/Atomic 字段支持通过不可变引用修改。
    fn 取段(段_rc: &Arc<虚线>) -> &虚线 {
        段_rc
    }

    /// 向线段的基础序列中添加一笔
    pub fn _添加虚线(段_rc: &Arc<虚线>, 筆: Arc<虚线>) {
        let 段 = Self::取段(段_rc);
        let 基础序列 = 段.基础序列.read();
        if !基础序列.is_empty() {
            let 最后 = 基础序列.last().unwrap();
            if !分型::判断分型(&最后.武.read(), &筆.文, "中") {
                panic!("线段.添加虚线 不连续 {} {}", 最后, 筆);
            }
            if *最后.标识.read() != *筆.标识.read() {
                panic!(
                    "线段.添加虚线 标识不符 {} {}",
                    *最后.标识.read(),
                    筆.标识.read()
                );
            }
        }
        drop(基础序列);
        段.基础序列.write().push(筆);
    }

    /// 更新线段的终点分型
    pub fn _武斗(段_rc: &Arc<虚线>, 武: &Arc<分型>, 行号: u32) {
        let 段 = Self::取段(段_rc);
        if Arc::ptr_eq(&*段.武.read(), 武) {
            return;
        }
        let 段_武 = 段.武.read();
        if 段_武.分型特征值 == 武.分型特征值 && 段_武.时间戳() != 武.时间戳()
        {
            warn!(
                "{}.武斗[{}], 发现特征值相等但时间戳不同, {}, {}",
                段.标识.read(),
                行号,
                段_武,
                武
            );
        }
        drop(段_武);
        if 段.文.结构 == 武.结构 {
            panic!("文武结构相同 {} {} {}", 行号, 段.文, 武);
        }
        if let Some(右) = &武.右
            && let Some(分析结构) = 分型结构::分析(
                武.左.as_ref().unwrap().as_ref(),
                武.中.as_ref(),
                右.as_ref(),
                false,
                false,
            )
            && 分析结构 != 武.结构
        {
            panic!("武斗[{}], 分型结构不一致 {} != {}", 行号, 分析结构, 武.结构);
        }
        if 段.方向() == 相对方向::向上 {
            if 武.分型特征值 < 段.文.分型特征值 {
                panic!(
                    "向上线段, 结束点小于起点 {} {} {}",
                    段.标识.read(),
                    段.文,
                    武
                );
            }
            let 段_武 = 段.武.read();
            if 段_武.分型特征值 > 武.分型特征值 && 段.模式.read().as_str() == "文武"
            {
                warn!(
                    "{}.武斗[{}] 出现回退 从 {} ==>>> {}",
                    段.标识.read(),
                    行号,
                    段_武,
                    武
                );
            }
        } else {
            if 武.分型特征值 > 段.文.分型特征值 {
                panic!(
                    "向下线段, 结束点大于起点 {} {} {}",
                    段.标识.read(),
                    段.文,
                    武
                );
            }
            let 段_武 = 段.武.read();
            if 段_武.分型特征值 < 武.分型特征值 && 段.模式.read().as_str() == "文武"
            {
                warn!(
                    "{}.武斗[{}] 出现回退 从 {} ==>>> {}",
                    段.标识.read(),
                    行号,
                    段_武,
                    武
                );
            }
        }
        *段.武.write() = Arc::clone(武);
    }

    /// 武终 — 线段终结时设置终点
    pub fn _武终(段_rc: &Arc<虚线>, 行号: u32) {
        let 武 = {
            let 段 = Self::取段(段_rc);
            if 段.模式.read().as_str() != "文武" {
                Some(Arc::clone(&*段.基础序列.read().last().unwrap().武.read()))
            } else {
                None
            }
        };
        if let Some(武) = 武 {
            Self::_武斗(段_rc, &武, 行号);
        }
    }

    /// 验证序列 — 截断无效尾部
    pub fn _验证序列(段_rc: &Arc<虚线>, 序列: &[Arc<虚线>]) {
        let 段 = Self::取段(段_rc);
        let guard = 段.基础序列.read();
        let 首序号 = 序列[0].序号.load(Ordering::Relaxed);
        let mut 基础序列: Vec<Arc<虚线>> = Vec::with_capacity(guard.len());
        for 元素 in guard.iter() {
            let idx = (元素.序号.load(Ordering::Relaxed) - 首序号) as usize;
            if idx >= 序列.len() || !Arc::ptr_eq(&序列[idx], 元素) {
                break;
            }
            if !基础序列.is_empty() && !基础序列.last().unwrap().之后是(元素) {
                warn!("线段._验证序列 数据不连续");
                break;
            }
            基础序列.push(Arc::clone(元素));
        }
        if 基础序列.len().is_multiple_of(2) {
            基础序列.pop();
        }
        drop(guard);
        *段.基础序列.write() = 基础序列;
    }

    /// 序列重置 — 截取到序列中的有效部分
    pub fn 序列重置(段_rc: &Arc<虚线>, 序列: &[Arc<虚线>]) {
        let 段 = Self::取段(段_rc);
        let guard = 段.基础序列.read();
        let 首序号 = 序列[0].序号.load(Ordering::Relaxed);
        let mut 基础序列: Vec<Arc<虚线>> = Vec::with_capacity(guard.len());
        for 元素 in guard.iter() {
            let idx = (元素.序号.load(Ordering::Relaxed) - 首序号) as usize;
            if idx >= 序列.len() || !Arc::ptr_eq(&序列[idx], 元素) {
                break;
            }
            if !基础序列.is_empty() && !基础序列.last().unwrap().之后是(元素) {
                break;
            }
            基础序列.push(Arc::clone(元素));
        }
        drop(guard);
        *段.基础序列.write() = 基础序列;
        段.特征序列.write()[2] = None;
    }

    // ================================================================
    // 基础判断
    // ================================================================

    /// 基础判断 — 连续三笔且重叠才能构成线段
    pub fn _基础判断(
        左: &虚线, 中: &虚线, 右: &虚线, 关系序列: &[相对方向]
    ) -> bool {
        if !左.之后是(中) {
            return false;
        }
        if !中.之后是(右) {
            return false;
        }

        if !相对方向::分析(左.高(), 左.低(), 中.高(), 中.低()).是否包含() {
            return false;
        }
        if !相对方向::分析(中.高(), 中.低(), 右.高(), 右.低()).是否包含() {
            return false;
        }

        let 关系 = 相对方向::分析(左.高(), 左.低(), 右.高(), 右.低());
        if !关系序列.contains(&关系) {
            return false;
        }

        if 左.方向() == 相对方向::向下 && !关系.是否向下() {
            return false;
        }
        if 左.方向() == 相对方向::向上 && !关系.是否向上() {
            return false;
        }
        true
    }

    // ================================================================
    // 四象 / 缺口 / 特征序列
    // ================================================================

    /// 四象 — 线段的四种状态分类
    ///
    /// 老阳: 向下线段第一二特征序列有缺口, 后一向上线段
    /// 老阴: 向上线段第一二特征序列有缺口, 后一向下线段
    /// 小阳: 向上线段
    /// 少阴: 向下线段
    pub fn 四象(段: &虚线) -> String {
        if 段.前一缺口.read().is_some() {
            if 段.方向() == 相对方向::向上 {
                "老阳".into()
            } else {
                "老阴".into()
            }
        } else if 段.方向() == 相对方向::向上 {
            "小阳".into()
        } else {
            "少阴".into()
        }
    }

    /// 获取缺口 — 从特征序列第一二元素之间检测缺口
    pub fn 获取缺口(段: &虚线) -> Option<缺口> {
        if 段.模式.read().as_str() != "文武" {
            return None;
        }
        let 特序 = 段.特征序列.read();
        let 左 = 特序[0].as_ref()?;
        let 中 = 特序[1].as_ref()?;
        let 相对关系 = 相对方向::分析(左.高(), 左.低(), 中.高(), 中.低());
        if 相对关系.是否缺口() {
            let 高 = 左.文().分型特征值.max(中.文().分型特征值);
            let 低 = 左.文().分型特征值.min(中.文().分型特征值);
            Some(缺口 { 高, 低 })
        } else {
            None
        }
    }

    /// 特征分型终结 — 检查特征序列是否形成正常分型终结
    pub fn 特征分型终结(段: &虚线) -> bool {
        let 特征序列 =
            线段特征::静态分析(&段.基础序列.read(), 段.方向(), &Self::四象(段), false);
        if 特征序列.len() >= 3 {
            let idx = 特征序列.len();
            if let Some(结构) = 分型结构::分析(
                &*特征序列[idx - 3],
                &*特征序列[idx - 2],
                &*特征序列[idx - 1],
                true,
                true,
            ) {
                if 段.方向() == 相对方向::向上 {
                    return 结构 == 分型结构::顶;
                } else {
                    return 结构 == 分型结构::底;
                }
            }
        }
        false
    }

    /// 特征序列状态 — 返回三个特征序列元素是否为 Some
    pub fn 特征序列状态(段: &虚线) -> (bool, bool, bool) {
        let 特序 = 段.特征序列.read();
        (特序[0].is_some(), 特序[1].is_some(), 特序[2].is_some())
    }

    /// 设置特征序列
    pub fn _设置特征序列(
        段_rc: &Arc<虚线>, 序列: Vec<Option<Arc<线段特征>>>, 行号: u32
    ) {
        let 段 = Self::取段(段_rc);
        if 段.模式.read().as_str() != "文武" {
            return;
        }

        for f in 序列.iter().flatten() {
            if f.方向() == 段.方向() {
                panic!("特征序列方向不匹配[{}]", 行号);
            }
        }

        let 左 = 序列[0].clone();
        let 中 = 序列[1].clone();
        let 右 = 序列[2].clone();
        *段.特征序列.write() = vec![左, 中, 右];

        if let Some(ref 右特征) = 段.特征序列.read()[2] {
            let guard = 段.基础序列.read();
            let mut 基础序列: Vec<Arc<虚线>> = Vec::with_capacity(guard.len());
            let 右尾 = 右特征.基础序列.last().expect("特征序列元素不应为空");
            if !guard.iter().any(|x| Arc::ptr_eq(x, 右尾)) {
                panic!("右特征最后一个元素不在基础序列中");
            }
            for 元素 in guard.iter() {
                基础序列.push(Arc::clone(元素));
                if Arc::ptr_eq(元素, 右尾) {
                    break;
                }
            }
            drop(guard);

            if 基础序列.len() >= 6 && 基础序列.len().is_multiple_of(2) {
                *段.基础序列.write() = 基础序列;
            } else {
                panic!("设置特征序列: 基础序列长度不足或非偶数");
            }
        }
    }

    /// 刷新特征序列
    pub fn _刷新特征序列(段_rc: &Arc<虚线>, 配置: &缠论配置) {
        // Compute new feature sequence, then delegate to 设置特征序列 for truncation
        let 序列: Vec<Option<Arc<线段特征>>> = {
            let 段 = &**段_rc;
            if 段.模式.read().as_str() != "文武" {
                return;
            }
            // 仅在需要时克隆（避免全量克隆后立即截断的双重分配）
            let 基础序列 = {
                let 完整 = 段.基础序列.read();
                if let Some(ref 前结束) = *段.前一结束位置.read() {
                    let idx = 线段::_索引(&完整, 前结束);
                    if idx > 0 && idx < 完整.len() {
                        完整[idx - 1..].to_vec()
                    } else {
                        完整.clone()
                    }
                } else {
                    完整.clone()
                }
            };

            let 四象 = Self::四象(段);
            let 特征序列 = 线段特征::静态分析(
                &基础序列,
                段.方向(),
                &四象,
                配置.线段_特征序列忽视老阴老阳,
            );

            if 特征序列.len() >= 3 {
                let 分型序列 = 线段特征::获取分型序列(&特征序列);
                let 最后分型 = &分型序列[分型序列.len() - 1];
                if (段.方向() == 相对方向::向上 && 最后分型.结构 == 分型结构::顶)
                    || (段.方向() == 相对方向::向下 && 最后分型.结构 == 分型结构::底)
                {
                    vec![
                        Some(Arc::clone(&最后分型.左)),
                        Some(Arc::clone(&最后分型.中)),
                        Some(Arc::clone(&最后分型.右)),
                    ]
                } else {
                    vec![
                        Some(Arc::clone(&特征序列[特征序列.len() - 2])),
                        Some(Arc::clone(&特征序列[特征序列.len() - 1])),
                        None,
                    ]
                }
            } else {
                let mut 填充: Vec<Option<Arc<线段特征>>> = 特征序列.into_iter().map(Some).collect();
                填充.resize(3, None);
                填充
            }
        }; // 段 borrow ends here

        Self::_设置特征序列(段_rc, 序列, line!());
    }

    /// 查找贯穿伤 — 基础序列中穿透文分型特征值的笔
    pub fn 查找贯穿伤(段: &虚线) -> Option<Arc<虚线>> {
        for 贯穿伤 in 段.基础序列.read().iter().skip(3) {
            if 段.方向().是否向上() {
                if 贯穿伤.武.read().分型特征值 < 段.文.分型特征值 {
                    return Some(Arc::clone(贯穿伤));
                }
            } else {
                if 贯穿伤.武.read().分型特征值 > 段.文.分型特征值 {
                    return Some(Arc::clone(贯穿伤));
                }
            }
        }
        None
    }

    // ================================================================
    // 分割序列
    // ================================================================

    /// 分割序列 — 将线段的基础序列分为前、后、第三买卖线、贯穿伤
    pub fn 分割序列(段: &虚线, 所属中枢: Option<&中枢>) -> 分割结果 {
        if 段.模式.read().as_str() != "文武" {
            return (段.基础序列.read().clone(), Vec::new(), Vec::new(), None);
        }

        let len = 段.基础序列.read().len();
        let mut 前 = Vec::with_capacity(len);
        let mut 后 = Vec::with_capacity(len / 2);
        let mut 第三买卖线: Vec<Arc<虚线>> = Vec::new();
        let mut 贯穿伤: Option<Arc<虚线>> = None;

        for 筆 in 段.基础序列.read().iter() {
            if 前.is_empty() {
                前.push(Arc::clone(筆));
                continue;
            }
            if !Arc::ptr_eq(&*前.last().unwrap().武.read(), &*段.武.read()) && 后.is_empty() {
                前.push(Arc::clone(筆));
            }

            if !后.is_empty() {
                后.push(Arc::clone(筆));
            }
            if Arc::ptr_eq(&筆.文, &*段.武.read()) {
                后.push(Arc::clone(筆));
            }
        }

        let mut 状态 = None;

        if let Some(中枢) = 所属中枢 {
            *中枢.本级_第三买卖线.write() = None;
            let 尾部 = if let Some(后笔) = 后.last() {
                后笔.武.read().clone()
            } else {
                段.武.read().clone()
            };

            if 中枢.高() >= 尾部.分型特征值 && 尾部.分型特征值 >= 中枢.低() {
                状态 = Some("中枢之中");
            } else if 中枢.高() < 尾部.分型特征值 {
                状态 = Some("中枢之上");
            } else if 中枢.低() > 尾部.分型特征值 {
                状态 = Some("中枢之下");
            }
        }

        if 状态 == Some("中枢之上") {
            let 中枢高 = 所属中枢.as_ref().unwrap().高();
            let 中枢低 = 所属中枢.as_ref().unwrap().低();
            for 筆 in 段.基础序列.read().iter().rev() {
                if 筆.方向() == 相对方向::向下 {
                    let 关系 = 相对方向::分析(中枢高, 中枢低, 筆.高(), 筆.低());
                    if 关系 == 相对方向::向上缺口 {
                        第三买卖线.push(Arc::clone(筆));
                    } else {
                        break;
                    }
                }
            }
        }

        if 状态 == Some("中枢之下") {
            for 筆 in 段.基础序列.read().iter().rev() {
                if 筆.方向() == 相对方向::向上 {
                    let 关系 = 相对方向::分析(
                        所属中枢.as_ref().unwrap().高(),
                        所属中枢.as_ref().unwrap().低(),
                        筆.高(),
                        筆.低(),
                    );
                    if 关系 == 相对方向::向下缺口 {
                        第三买卖线.push(Arc::clone(筆));
                    } else {
                        break;
                    }
                }
            }
        }

        if !第三买卖线.is_empty() {
            第三买卖线.reverse();
            if let Some(中枢) = 所属中枢 {
                *中枢.本级_第三买卖线.write() = Some(Arc::clone(&第三买卖线[0]));
            }
        }

        if !后.is_empty() {
            if 段.方向().是否向上() {
                if 后[0].武.read().分型特征值 < 段.文.分型特征值 {
                    贯穿伤 = Some(Arc::clone(&后[0]));
                }
            } else {
                if 后[0].武.read().分型特征值 > 段.文.分型特征值 {
                    贯穿伤 = Some(Arc::clone(&后[0]));
                }
            }
        }

        (前, 后, 第三买卖线, 贯穿伤)
    }

    /// 刷新 — 完整刷新线段的特征序列和内部中枢
    pub fn _刷新(段_rc: &Arc<虚线>, 配置: &缠论配置) {
        let 段 = Self::取段(段_rc);
        if 段.模式.read().as_str() != "文武" {
            return;
        }
        if 段.基础序列.read().is_empty() {
            warn!("    线段.刷新 基础序列为空");
            return;
        }

        Self::_刷新特征序列(段_rc, 配置);

        // After 刷新特征序列, work with the updated segment
        let (武斗_武文, 特征后一笔_opt) = {
            let 段2 = Self::取段(段_rc);
            let 特序_ref = 段2.特征序列.read();
            let 有效特征序列: Vec<&Arc<线段特征>> =
                特序_ref.iter().filter_map(|x| x.as_ref()).collect();

            if 有效特征序列.len() == 3 {
                (Some(Arc::clone(&有效特征序列[1].文())), None)
            } else if !有效特征序列.is_empty() {
                let 最近特征 = 有效特征序列[有效特征序列.len() - 1];

                let 特征后一笔 = if 最近特征
                    .基础序列
                    .last()
                    .map(|x| 段2.基础序列.read().iter().any(|b| Arc::ptr_eq(b, x)))
                    == Some(true)
                {
                    Some(Arc::clone(最近特征.基础序列.last().unwrap()))
                } else {
                    笔::以武会友(
                        &段2.基础序列.read(),
                        &最近特征.基础序列.last().unwrap().武.read(),
                    )
                };

                if 特征后一笔.is_none() {
                    warn!(
                        "    线段.刷新 特征后一笔 = None, {}, 有效特征: {}",
                        段2,
                        有效特征序列.len()
                    );
                }
                (None, 特征后一笔)
            } else {
                panic!("线段.刷新 有效特征序列为空！");
            }
        };

        if let Some(武文) = 武斗_武文 {
            Self::_武斗(段_rc, &武文, line!());
        } else if let Some(特征后一笔) = 特征后一笔_opt {
            let 武斗候选 = {
                let 段2 = Self::取段(段_rc);
                let 基础序列_ref = 段2.基础序列.read();
                // Python: 序号 = cls._索引(段.基础序列, 特征后一笔)
                let 序号 = 线段::_索引(&基础序列_ref, &特征后一笔);
                if 序号 < 基础序列_ref.len() {
                    if 序号 < 段2.基础序列.read().len() - 1 {
                        let 下一笔 = Arc::clone(&段2.基础序列.read()[序号 + 1]);
                        if (段2.方向() == 相对方向::向上 && 段2.高() <= 下一笔.高())
                            || (段2.方向() == 相对方向::向下 && 段2.低() >= 下一笔.低())
                        {
                            Some(下一笔.武.read().clone())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            };
            if let Some(武) = 武斗候选 {
                Self::_武斗(段_rc, &武, line!());
            }
        }

        let 段3 = Self::取段(段_rc);
        let _ = Self::获取内部中枢序列_内部(段3, 配置);
    }

    /// 获取内部中枢序列 — 内部实现
    fn 获取内部中枢序列_内部(段: &虚线, _配置: &缠论配置) -> 中枢序列组 {
        if 段.模式.read().as_str() != "文武" {
            中枢::分析(
                &段.基础序列.read(),
                &mut 段.合_中枢序列.write(),
                true,
                &format!("{}_{}_合_", 段.标识.read(), 段.序号.load(Ordering::Relaxed)),
                0,
            );
            return (Vec::new(), Vec::new(), 段.合_中枢序列.read().clone());
        }

        // Use 分割序列 to get前/后
        let (前, 后, _, _) = Self::分割序列(段, None);

        中枢::分析(
            &前,
            &mut 段.实_中枢序列.write(),
            true,
            &format!("{}_{}_实_", 段.标识.read(), 段.序号.load(Ordering::Relaxed)),
            0,
        );
        中枢::分析(
            &后,
            &mut 段.虚_中枢序列.write(),
            true,
            &format!("{}_{}_虚_", 段.标识.read(), 段.序号.load(Ordering::Relaxed)),
            0,
        );
        中枢::分析(
            &段.基础序列.read(),
            &mut 段.合_中枢序列.write(),
            true,
            &format!("{}_{}_合_", 段.标识.read(), 段.序号.load(Ordering::Relaxed)),
            0,
        );

        (
            段.虚_中枢序列.read().clone(),
            段.实_中枢序列.read().clone(),
            段.合_中枢序列.read().clone(),
        )
    }

    /// 获取内部中枢序列
    pub fn 获取内部中枢序列(
        段_rc: &Arc<虚线>, 配置: &缠论配置
    ) -> 中枢序列组 {
        let 段 = Self::取段(段_rc);
        Self::获取内部中枢序列_内部(段, 配置)
    }

    // ================================================================
    // 线段序列管理
    // ================================================================

    /// _添加线段 — 向线段序列尾部添加线段（内部方法）
    pub fn _添加线段(
        线段序列: &mut Vec<Arc<虚线>>,
        mut 待添加线段: Arc<虚线>,
        _配置: &缠论配置,
        行号: u32,
        层级: i64,
    ) {
        {
            let seg = Arc::make_mut(&mut 待添加线段);
            *seg.模式.write() = "文武".into();

            if !线段序列.is_empty() {
                if let Some(前一个) = 线段序列.last()
                    && !前一个.之后是(seg)
                {
                    panic!(
                        "线段.向序列中添加 不连续[{}, {}] {} {}",
                        行号,
                        层级,
                        前一个.武.read(),
                        seg.文
                    );
                }

                let 之前线段 = 线段序列.last().unwrap();

                assert!(
                    之前线段.特征序列.read()[2].is_some()
                        || 之前线段.短路修正.load(Ordering::Relaxed),
                    "线段._向序列中添加[{}, {}], 之前线段.右 = None {}",
                    行号,
                    层级,
                    之前线段
                );

                if !seg
                    .基础序列
                    .read()
                    .iter()
                    .any(|x| Arc::ptr_eq(x, 之前线段.基础序列.read().last().unwrap()))
                    && !之前线段.短路修正.load(Ordering::Relaxed)
                {
                    panic!(
                        "线段._向序列中添加[{}, {}], 之前线段[-1] not in 待添加虚线! {}",
                        行号, 层级, 之前线段
                    );
                }

                seg.序号
                    .store(之前线段.序号.load(Ordering::Relaxed) + 1, Ordering::Relaxed);
                *seg.前一缺口.write() = if 之前线段.短路修正.load(Ordering::Relaxed) {
                    None
                } else {
                    Self::获取缺口(之前线段)
                };
                *seg.前一结束位置.write() =
                    Some(Arc::clone(之前线段.基础序列.read().last().unwrap()));

                if ["老阴", "老阳"].contains(&Self::四象(之前线段).as_str()) {
                    *seg.前一缺口.write() = None;
                }
            }
        }
        线段序列.push(待添加线段);
    }

    /// _弹出线段 — 从线段序列尾部弹出线段（内部方法）
    pub fn _弹出线段(
        线段序列: &mut Vec<Arc<虚线>>,
        待弹出线段: &Arc<虚线>,
        _配置: &缠论配置,
        行号: u32,
        层级: i64,
    ) -> Option<Arc<虚线>> {
        if 线段序列.is_empty() {
            return None;
        }

        if !Arc::ptr_eq(线段序列.last().unwrap(), 待弹出线段) {
            panic!("线段._从序列中删除 弹出数据不在列表中 {}", 待弹出线段);
        }

        {
            let 特序 = 待弹出线段.特征序列.read();
            let 左 = &特序[0];
            let 中 = &特序[1];
            let 右 = &特序[2];
            if let (Some(l), Some(m), Some(r)) = (左, 中, 右)
                && let Some(结构) = 分型结构::分析(&**l, &**m, &**r, true, true)
                && matches!(结构, 分型结构::顶 | 分型结构::底)
                && !相对方向::分析(l.高(), l.低(), m.高(), m.低()).是否缺口()
            {
                warn!(
                    "[警告<{}, {}>]: 线段._从序列中删除 发现分型完毕, 且特征序列无缺口 {}",
                    行号, 层级, 待弹出线段
                );
            }
        }

        let 弹出 = 线段序列.pop().unwrap();
        弹出.有效性.store(false, Ordering::Relaxed);
        *弹出.前一结束位置.write() = None;

        Some(弹出)
    }

    // ================================================================
    // 四种修正机制
    // ================================================================

    /// _缺口突破 — 老阳/老阴缺口突破修正
    pub fn _缺口突破(
        线段序列: &mut Vec<Arc<虚线>>, 配置: &缠论配置, 层级: i64
    ) -> bool {
        let 当前线段 = Arc::clone(线段序列.last().unwrap());
        let 基础序列 = 当前线段.基础序列.read();
        assert!(!基础序列.is_empty(), "缺口突破: 当前线段.基础序列为空！");
        let 当前虚线 = Arc::clone(&基础序列[基础序列.len() - 1]);
        drop(基础序列);
        let 四象 = Self::四象(&当前线段);
        let 同向 = 当前虚线.方向() == 当前线段.方向();

        // 条件1：不能同向
        if 同向 {
            return false;
        }

        // 条件2：四象必须是老阳或老阴
        if 四象 != "老阳" && 四象 != "老阴" {
            return false;
        }

        // 条件3：当前线段特征序列[2]必须为None
        if 当前线段.特征序列.read()[2].is_some() {
            return false;
        }

        // 条件4：具体突破方向判断
        let 突破 = (四象 == "老阳" && 当前虚线.低() < 当前线段.低())
            || (四象 == "老阴" && 当前虚线.高() > 当前线段.高());
        if !突破 {
            return false;
        }

        // 已被修正
        if 线段序列[线段序列.len() - 2]
            .短路修正
            .load(Ordering::Relaxed)
        {
            return false;
        }

        // 执行修正
        let 序列 = 当前线段.基础序列.read().clone();
        Self::_弹出线段(
            线段序列,
            &Arc::clone(线段序列.last().unwrap()),
            配置,
            line!(),
            层级,
        );

        assert!(!线段序列.is_empty(), "缺口突破: 线段序列为第二次空！");
        let 当前线段 = Arc::clone(线段序列.last().unwrap());

        assert!(当前线段.特征序列.read()[2].is_some());
        let (前, _, _, _) = Self::分割序列(&当前线段, None);
        let mut 当前线段基础序列 = 前;
        let 序列首 = Arc::clone(&序列[0]);
        assert!(
            当前线段基础序列.last().unwrap().之后是(&序列首),
            "缺口突破: 子序列不连续!"
        );
        当前线段基础序列.extend(序列);

        let idx = 线段序列.len() - 1;
        *线段序列[idx].基础序列.write() = 当前线段基础序列.clone();
        Self::_刷新(&线段序列[idx], 配置);
        true
    }

    /// _非缺口下穿刺 — 贯穿伤修复
    pub fn _非缺口下穿刺(
        线段序列: &mut Vec<Arc<虚线>>, 配置: &缠论配置, 层级: i64
    ) -> bool {
        assert!(!线段序列.is_empty(), "非缺口下穿刺: 线段序列为空！");

        let 当前线段 = Arc::clone(线段序列.last().unwrap());
        let 四象 = Self::四象(&当前线段);

        // 外层条件
        if !(配置.线段_非缺口下穿刺
            && (四象 == "小阳" || 四象 == "少阴")
            && 当前线段.特征序列.read()[2].is_none())
        {
            return false;
        }

        // 查找贯穿伤
        let 贯穿伤 = Self::查找贯穿伤(&当前线段);
        if 贯穿伤.is_none() {
            return false;
        }
        let 贯穿伤 = 贯穿伤.unwrap();

        // 切割基础序列
        // Python: 基础序列 = 当前线段.基础序列[cls._索引(当前线段.基础序列, 贯穿伤) :]
        let 贯穿伤_idx = {
            let guard = 当前线段.基础序列.read();
            线段::_索引(&guard, &贯穿伤)
        };
        let 基础序列: Vec<Arc<虚线>> = 当前线段.基础序列.read()[贯穿伤_idx..].to_vec();

        // 长度条件
        if !(基础序列.len() == 4 && 线段序列.len() >= 2) {
            return false;
        }

        let 左 = Arc::clone(&基础序列[基础序列.len() - 3]);
        let 中 = Arc::clone(&基础序列[基础序列.len() - 2]);
        let 右 = Arc::clone(&基础序列[基础序列.len() - 1]);

        // 方向条件
        if 相对方向::分析(左.高(), 左.低(), 右.高(), 右.低()) != 当前线段.方向()
        {
            return false;
        }

        // 执行修正
        warn!(
            "[警告<{}, {}>]: {}.修复贯穿伤, 序号:{} {} [{}]",
            line!(),
            层级,
            当前线段.标识.read(),
            当前线段.序号.load(Ordering::Relaxed),
            贯穿伤,
            基础序列
                .iter()
                .map(|s| format!("{}", s))
                .collect::<Vec<_>>()
                .join(", ")
        );

        // Reassign to full copy (matching Python pattern)
        let 基础序列 = 当前线段.基础序列.read().clone();
        Self::_弹出线段(
            线段序列,
            &Arc::clone(线段序列.last().unwrap()),
            配置,
            line!(),
            层级,
        );

        assert!(!线段序列.is_empty(), "非缺口下穿刺: 第二次线段序列为空！");

        let idx = 线段序列.len() - 1;
        线段序列[idx].特征序列.write()[2] = None;

        // Python: if 当前线段.基础序列[-1] not in 基础序列: 序号 = 0
        //        else: 序号 = cls._索引(基础序列, 当前线段.基础序列[-1]) + 1
        let 序号 = {
            let 开始笔 = Arc::clone(线段序列[idx].基础序列.read().last().unwrap());
            if !基础序列.iter().any(|x| Arc::ptr_eq(x, &开始笔)) {
                crate::error!("非缺口下穿刺: 当前线段.基础序列[-1] 不在 基础序列中！");
                0
            } else {
                线段::_索引(&基础序列, &开始笔) + 1
            }
        };

        for 筆 in 基础序列.iter().skip(序号) {
            Self::_添加虚线(&线段序列[idx], Arc::clone(筆));
        }
        Self::_刷新(&线段序列[idx], 配置);

        线段序列[idx].短路修正.store(true, Ordering::Relaxed);
        if 线段序列[idx].特征序列.read()[2].is_some() {
            let 段 = 虚线::创建线段(&[Arc::clone(&左), Arc::clone(&中), Arc::clone(&右)]);
            let 段_rc = Arc::new(段);
            Self::_添加线段(线段序列, 段_rc, 配置, line!(), 层级);

            // Set feature sequence [0]
            let 新段 = Self::取段(线段序列.last_mut().unwrap());
            新段.特征序列.write()[0] = Some(Arc::new(线段特征::新建(vec![中], 新段.方向())));
        }

        true
    }

    /// _缺口后紧急修正 — 老阴/老阳后的紧急修正
    pub fn _缺口后紧急修正(
        线段序列: &mut Vec<Arc<虚线>>,
        配置: &缠论配置,
        层级: i64,
    ) -> bool {
        assert!(!线段序列.is_empty(), "缺口后紧急修正: 线段序列为空！");

        let 当前线段 = Arc::clone(线段序列.last().unwrap());
        let 四象 = Self::四象(&当前线段);

        // 外层条件
        if !(配置.线段_缺口后紧急修正
            && !配置.线段_特征序列忽视老阴老阳
            && (四象 == "小阳" || 四象 == "少阴")
            && 当前线段.特征序列.read()[2].is_none())
        {
            return false;
        }

        // 内层条件
        if 线段序列.len() < 2 {
            return false;
        }
        let 前一线段_idx = 线段序列.len() - 2;
        if !["老阴", "老阳"].contains(&Self::四象(&线段序列[前一线段_idx]).as_str()) {
            return false;
        }

        let (_, 基础序列, _, _) = Self::分割序列(&当前线段, None);
        if 基础序列.len() < 3 {
            return false;
        }

        let mut 需要修正 = false;
        if 当前线段.方向() == 相对方向::向上 {
            if 相对方向::分析(
                基础序列[0].高(),
                基础序列[0].低(),
                基础序列[2].高(),
                基础序列[2].低(),
            ) == 相对方向::向下
            {
                需要修正 = true;
            }
        } else {
            if 相对方向::分析(
                基础序列[0].高(),
                基础序列[0].低(),
                基础序列[2].高(),
                基础序列[2].低(),
            ) == 相对方向::向上
            {
                需要修正 = true;
            }
        }

        if !需要修正 {
            return false;
        }

        // 执行修正 — set 短路修正 and create new segment
        let idx = 线段序列.len() - 1;
        线段序列[idx].短路修正.store(true, Ordering::Relaxed);

        let 新段 = 虚线::创建线段(&基础序列);
        let 新段_rc = Arc::new(新段);
        Self::_添加线段(线段序列, 新段_rc, 配置, line!(), 层级);
        true
    }

    /// _修正 — 通用线段修正（后段足够长时拆分）
    pub fn _修正(
        线段序列: &mut Vec<Arc<虚线>>, 配置: &缠论配置, 层级: i64
    ) -> bool {
        assert!(!线段序列.is_empty(), "修正: 线段序列为空！");

        let 当前线段 = Arc::clone(线段序列.last().unwrap());

        // 条件1
        if !(配置.线段_修正 && 当前线段.基础序列.read().len() >= 9) {
            return false;
        }

        let (_, 之后基础序列, _, _) = Self::分割序列(&当前线段, None);

        // 条件2
        if 之后基础序列.len() < 6 {
            return false;
        }
        if 之后基础序列.len() % 2 != 0 {
            return false;
        }

        let 前 = Arc::clone(&之后基础序列[之后基础序列.len() - 3]);
        let 后 = Arc::clone(&之后基础序列[之后基础序列.len() - 1]);

        // 条件3
        if 当前线段.方向() != 相对方向::分析(前.高(), 前.低(), 后.高(), 后.低())
        {
            return false;
        }

        // 执行修正
        let idx = 线段序列.len() - 1;
        线段序列[idx].短路修正.store(true, Ordering::Relaxed);

        // 第一个新段
        let 新段1 = 虚线::创建线段(&之后基础序列[..之后基础序列.len() - 3]);
        let 新段1_rc = Arc::new(新段1);
        // Set 短路修正
        新段1_rc.短路修正.store(true, Ordering::Relaxed);
        Self::_添加线段(线段序列, 新段1_rc, 配置, line!(), 层级);

        if ["老阴", "老阳"].contains(&Self::四象(&当前线段).as_str()) {
            *Self::取段(线段序列.last_mut().unwrap()).前一缺口.write() = None;
        }

        // 第二个新段
        let start = 之后基础序列.len() - 3;
        let 新段2 = 虚线::创建线段(&之后基础序列[start..]);
        let 新段2_rc = Arc::new(新段2);
        Self::_添加线段(线段序列, 新段2_rc, 配置, line!(), 层级);

        true
    }

    // ================================================================
    // 核心分析 — 使用显式栈（loop）模拟递归
    // ================================================================

    /// 分析 — 从笔序列生成线段序列
    ///
    /// 使用显式栈（loop + continue）模拟 Python 的递归调用，避免栈溢出。
    /// 线段划分核心递归算法 — 从笔序列递归生成线段（与 Python 线段.分析 对齐）
    pub fn 分析(
        笔序列: &[Arc<虚线>],
        线段序列: &mut Vec<Arc<虚线>>,
        配置: &缠论配置,
        层级: i64,
        关系序列: &[相对方向],
    ) {
        // 递归深度守卫（Python: if 层级 > 256: return）
        if 层级 > 256 {
            warn!("线段.分析 递归深度超出 256");
            return;
        }

        // Python: try: 笔序列[2] except IndexError: return
        if 笔序列.len() < 3 {
            return;
        }

        // -------------------- 1. 初始化第一个线段 --------------------
        // Python: if not 线段序列: for i in range(1, len-1): ...
        if 线段序列.is_empty() {
            for i in 1..笔序列.len() - 1 {
                let 左 = &笔序列[i - 1];
                let 中 = &笔序列[i];
                let 右 = &笔序列[i + 1];

                if !Self::_基础判断(左, 中, 右, 关系序列) {
                    continue;
                }
                let 段 = 虚线::创建线段(&[Arc::clone(左), Arc::clone(中), Arc::clone(右)]);
                let 段_rc = Arc::new(段);
                Self::_添加线段(线段序列, 段_rc, 配置, line!(), 层级);
                // Python: 段.特征序列[0] = 线段特征.新建([中], 段.方向)
                线段序列.last().unwrap().特征序列.write()[0] = Some(Arc::new(线段特征::新建(
                    vec![Arc::clone(中)],
                    线段序列.last().unwrap().方向(),
                )));
                break;
            }
            // Python: if not 线段序列: return
            if 线段序列.is_empty() {
                return;
            }
        }

        // -------------------- 2. 清理无效的尾部引用 --------------------
        // Python: while 线段序列 and 线段序列[-1].前一结束位置: ...
        while !线段序列.is_empty() && 线段序列.last().unwrap().前一结束位置.read().is_some()
        {
            let 前一结束 = Arc::clone(
                线段序列
                    .last()
                    .unwrap()
                    .前一结束位置
                    .read()
                    .as_ref()
                    .unwrap(),
            );
            // Python: if 前一结束位置 not in 笔序列: _弹出线段(...)
            let 首序号 = 笔序列[0].序号.load(Ordering::Relaxed);
            let idx = (前一结束.序号.load(Ordering::Relaxed) - 首序号) as usize;
            if idx >= 笔序列.len() || !Arc::ptr_eq(&笔序列[idx], &前一结束) {
                let 当前 = Arc::clone(线段序列.last().unwrap());
                Self::_弹出线段(线段序列, &当前, 配置, line!(), 层级);
            } else {
                break;
            }
        }

        // Python: if not 线段序列: return 线段递归分析(笔序列, 线段序列, 配置, 层级 + 1, 关系序列)
        if 线段序列.is_empty() {
            return Self::分析(笔序列, 线段序列, 配置, 层级 + 1, 关系序列);
        }

        // -------------------- 3. 确保当前线段有效 --------------------
        // Python: 当前线段 = 线段序列[-1]; _序列重置(当前线段, 笔序列)
        Self::序列重置(线段序列.last().unwrap(), 笔序列);

        // Python: if len(当前线段.基础序列) < 3: _弹出线段(...)
        if 线段序列.last().unwrap().基础序列.read().len() < 3 {
            let 当前 = Arc::clone(线段序列.last().unwrap());
            Self::_弹出线段(线段序列, &当前, 配置, line!(), 层级);
            // Python: if not 线段序列: return 线段递归分析(...)
            if 线段序列.is_empty() {
                return Self::分析(笔序列, 线段序列, 配置, 层级 + 1, 关系序列);
            }
        }

        // -------------------- 4. 特征序列已完整时的处理 --------------------
        // Python: 当前线段 = 线段序列[-1]
        // Python: if 当前线段.特征序列[2] is not None: ...
        {
            let 当前线段 = Arc::clone(线段序列.last().unwrap());
            if 当前线段.特征序列.read()[2].is_some() {
                // Python: 基础序列 = 线段.分割序列(当前线段)[1]
                let (_, 基础序列, _, _) = Self::分割序列(&当前线段, None);
                let 四象 = Self::四象(&当前线段);
                // Python: 新段 = 虚线.创建线段(基础序列)
                let 新段 = Arc::new(虚线::创建线段(&基础序列));
                // Python: 线段._添加线段(线段序列, 新段, 配置, ...)
                Self::_添加线段(线段序列, 新段, 配置, line!(), 层级);
                // Python: if 线段.四象(当前线段) in ("老阴", "老阳"): 新段.前一缺口 = None
                if ["老阴", "老阳"].contains(&四象.as_str()) {
                    *线段序列.last().unwrap().前一缺口.write() = None;
                }
            }
        }

        // Python: 当前线段 = 线段序列[-1]; 线段._刷新(当前线段, 配置)
        {
            let idx = 线段序列.len() - 1;
            Self::_刷新(&线段序列[idx], 配置);
        }

        // -------------------- 5. 调用一次全局修正（不循环） --------------------
        // Python: 四种修正各调用一次
        Self::_缺口突破(线段序列, 配置, 层级);
        Self::_非缺口下穿刺(线段序列, 配置, 层级);
        Self::_缺口后紧急修正(线段序列, 配置, 层级);
        Self::_修正(线段序列, 配置, 层级);

        // -------------------- 6. 循环处理后续的笔 --------------------
        // Python: 当前线段 = 线段序列[-1]
        let 当前线段_rc = Arc::clone(线段序列.last().unwrap());
        if 当前线段_rc.基础序列.read().is_empty() {
            panic!("线段.分析: 基础序列为空");
        }
        // Python: 起始索引 = cls._索引(笔序列, 当前线段.基础序列[-1]) + 1
        let 起始索引 = {
            let guard = 当前线段_rc.基础序列.read();
            线段::_索引(笔序列, &guard[guard.len() - 1]) + 1
        };

        // Python: for idx in range(起始索引, len(笔序列)):
        for 当前虚线_ref in &笔序列[起始索引..] {
            let 当前虚线 = Arc::clone(当前虚线_ref);
            // Python: 当前线段 = 线段序列[-1]; 四象 = 线段.四象(当前线段)
            let 当前线段 = Arc::clone(线段序列.last().unwrap());
            let 四象 = Self::四象(&当前线段);

            // Python: 线段._添加虚线(当前线段, 当前虚线)
            Self::_添加虚线(线段序列.last().unwrap(), Arc::clone(&当前虚线));
            // Python: 线段._刷新(当前线段, 配置)
            {
                let idx = 线段序列.len() - 1;
                Self::_刷新(&线段序列[idx], 配置);
            }

            // Python: 依次尝试四种修正，任意一个成功则跳过后续处理
            let mut 修正触发: Option<&str> = None;
            if Self::_缺口突破(线段序列, 配置, 层级) {
                修正触发 = Some("缺口突破");
            } else if Self::_非缺口下穿刺(线段序列, 配置, 层级) {
                修正触发 = Some("非缺口下穿刺");
            } else if Self::_缺口后紧急修正(线段序列, 配置, 层级) {
                修正触发 = Some("缺口后紧急修正");
            } else if Self::_修正(线段序列, 配置, 层级) {
                修正触发 = Some("修正");
            }
            if let Some(trigger) = 修正触发 {
                warn!(
                    "分析.修正触发={}, 笔序列长度={}, 线段序列长度={}",
                    trigger,
                    笔序列.len(),
                    线段序列.len()
                );
                continue;
            }

            // Python: 无修正触发，且特征序列[2]已存在 → 创建新段
            let 当前线段 = Arc::clone(线段序列.last().unwrap());
            if 当前线段.特征序列.read()[2].is_none() {
                continue;
            }

            // Python: 基础序列 = 线段.分割序列(当前线段)[1]; 新段 = 虚线.创建线段(基础序列)
            let (_, 基础序列, _, _) = Self::分割序列(&当前线段, None);
            let 新段 = Arc::new(虚线::创建线段(&基础序列));
            // Python: 线段._添加线段(线段序列, 新段, 配置, ...)
            Self::_添加线段(线段序列, 新段, 配置, line!(), 层级);
            // Python: if 四象 in ("老阴", "老阳"): 新段.前一缺口 = None
            if ["老阴", "老阳"].contains(&四象.as_str()) {
                *线段序列.last().unwrap().前一缺口.write() = None;
            }

            // Python: 检查新段与当前虚线的连续性
            let 新段_rc = Arc::clone(线段序列.last().unwrap());
            // Python: if 新段.基础序列[-1] is not 当前虚线
            if !Arc::ptr_eq(新段_rc.基础序列.read().last().unwrap(), &当前虚线) {
                // Python: if not 新段.基础序列[-1].之后是(当前虚线):
                //     return 线段递归分析(笔序列, 线段序列, 配置, 层级 + 1, 关系序列)
                if !新段_rc.基础序列.read().last().unwrap().之后是(&当前虚线) {
                    return Self::分析(笔序列, 线段序列, 配置, 层级 + 1, 关系序列);
                }
                // Python: 线段._添加虚线(新段, 当前虚线)
                Self::_添加虚线(线段序列.last().unwrap(), Arc::clone(&当前虚线));
            }

            // Python: 线段._刷新(新段, 配置)
            {
                let idx = 线段序列.len() - 1;
                Self::_刷新(&线段序列[idx], 配置);
            }
        }
    }

    // ================================================================
    // 扩展线段
    // ================================================================

    /// _添加扩展线段
    pub fn _添加扩展线段(
        线段序列: &mut Vec<Arc<虚线>>,
        mut 待添加线段: Arc<虚线>,
        行号: u32,
    ) {
        {
            let seg = Arc::make_mut(&mut 待添加线段);
            *seg.模式.write() = "高低".into();
            *seg.标识.write() = if seg.基础序列.read()[0].标识.read().as_str() != "笔" {
                format!("扩展{}", seg.标识.read())
            } else {
                "扩展线段".into()
            };

            if let Some(前一个) = 线段序列.last() {
                if !前一个.之后是(seg) {
                    panic!(
                        "线段.向序列中添加 不连续[{}] {} {}",
                        行号,
                        前一个.武.read(),
                        seg.文
                    );
                }
                let 之前线段 = 线段序列.last().unwrap();
                seg.序号
                    .store(之前线段.序号.load(Ordering::Relaxed) + 1, Ordering::Relaxed);
            }
        }
        线段序列.push(待添加线段);
    }

    /// _弹出扩展线段
    pub fn _弹出扩展线段(
        线段序列: &mut Vec<Arc<虚线>>,
        待弹出线段: &Arc<虚线>,
        _行号: u32,
    ) -> Option<Arc<虚线>> {
        if 线段序列.is_empty() {
            return None;
        }

        if Arc::ptr_eq(线段序列.last().unwrap(), 待弹出线段) {
            let drop = 线段序列.pop().unwrap();
            drop.有效性.store(false, Ordering::Relaxed);
            Some(drop)
        } else {
            panic!("线段._从序列中删除 弹出数据不在列表中 {}", 待弹出线段);
        }
    }

    /// 扩展分析 — 将笔视为线段进行同级别分析（与 Python 扩展分析 对齐，递归实现）
    pub fn 扩展分析(
        虚线序列: &[Arc<虚线>], 线段序列: &mut Vec<Arc<虚线>>, 配置: &缠论配置
    ) {
        Self::扩展分析_递归(虚线序列, 线段序列, 配置, 0);
    }

    fn 扩展分析_递归(
        虚线序列: &[Arc<虚线>],
        线段序列: &mut Vec<Arc<虚线>>,
        配置: &缠论配置,
        层级: i64,
    ) {
        // Python: if not 虚线序列: return; try: 虚线序列[2] except: return
        if 虚线序列.len() < 3 {
            return;
        }

        // Python: if 层级 > 256: return
        if 层级 > 256 {
            warn!("线段.扩展分析 递归深度超出 256");
            return;
        }

        // -------------------- 初始化第一个扩展线段 --------------------
        // Python: if not 线段序列: for i in range(1, len-1): ...
        if 线段序列.is_empty() {
            for i in 1..虚线序列.len() - 1 {
                let 左 = &虚线序列[i - 1];
                let 中 = &虚线序列[i];
                let 右 = &虚线序列[i + 1];
                // Python: 关系 = 相对方向.分析(左.端点高, 左.端点低, 右.端点高, 右.端点低)
                let 关系 = 相对方向::分析(左.端点高(), 左.端点低(), 右.端点高(), 右.端点低());
                // Python: if 关系 not in (向下, 向上, 顺, 逆, 同): continue
                if !matches!(
                    关系,
                    相对方向::向下
                        | 相对方向::向上
                        | 相对方向::顺
                        | 相对方向::逆
                        | 相对方向::同
                ) {
                    continue;
                }

                // Python: 段 = 虚线.创建线段([左, 中, 右]); _添加扩展线段(...)
                let 段 = Arc::new(虚线::创建线段(&[
                    Arc::clone(左),
                    Arc::clone(中),
                    Arc::clone(右),
                ]));
                Self::_添加扩展线段(线段序列, 段, line!());
                break;
            }

            // Python: if not 线段序列: return
            if 线段序列.is_empty() {
                return;
            }
        }

        // -------------------- 验证当前线段 --------------------
        // Python: 当前线段 = 线段序列[-1]; _验证序列(当前线段, 虚线序列)
        Self::_验证序列(线段序列.last().unwrap(), 虚线序列);

        // Python: if len(当前线段.基础序列) < 3: _弹出扩展线段(...); return 递归
        if 线段序列.last().unwrap().基础序列.read().len() < 3 {
            let 当前 = Arc::clone(线段序列.last().unwrap());
            Self::_弹出扩展线段(线段序列, &当前, line!());
            return Self::扩展分析_递归(虚线序列, 线段序列, 配置, 层级 + 1);
        }

        // -------------------- 非当下分析模式 --------------------
        // Python: if not 配置.扩展线段_当下分析: ...
        if !配置.扩展线段_当下分析 {
            let 当前线段 = 线段序列.last().unwrap();
            // Python: 左, 中, 右 = 当前线段.基础序列[:3]
            let 左 = Arc::clone(&当前线段.基础序列.read()[0]);
            let 右 = Arc::clone(&当前线段.基础序列.read()[2]);

            // Python: if not 相对方向.分析(左.端点高, 左.端点低, 右.端点高, 右.端点低).是否缺口():
            if !相对方向::分析(左.端点高(), 左.端点低(), 右.端点高(), 右.端点低()).是否缺口()
            {
                // Python: 当前线段.基础序列 = 当前线段.基础序列[:3]; _武终(当前线段, ...)
                let cur = Arc::make_mut(线段序列.last_mut().unwrap());
                let 前三个 = cur.基础序列.read()[..3].to_vec();
                *cur.基础序列.write() = 前三个;
                let seg_idx = 线段序列.len() - 1;
                Self::_武终(&线段序列[seg_idx], 0);
            } else {
                // Python: _弹出扩展线段(线段序列, 当前线段, ...); return 递归
                let 当前 = Arc::clone(线段序列.last().unwrap());
                Self::_弹出扩展线段(线段序列, &当前, line!());
                return Self::扩展分析_递归(虚线序列, 线段序列, 配置, 层级 + 1);
            }
        }

        // -------------------- 武终 --------------------
        // Python: 线段._武终(当前线段, ...)
        {
            let seg_idx = 线段序列.len() - 1;
            Self::_武终(&线段序列[seg_idx], 0);
        }

        // -------------------- 检查剩余元素是否够 --------------------
        // Python: if 当前线段.基础序列[-1].序号 + 3 > 虚线序列[-1].序号: return
        let 当前线段 = 线段序列.last().unwrap();
        if 当前线段
            .基础序列
            .read()
            .last()
            .unwrap()
            .序号
            .load(Ordering::Relaxed)
            + 3
            > 虚线序列.last().unwrap().序号.load(Ordering::Relaxed)
        {
            return;
        }

        // -------------------- 计算起始序号 --------------------
        // Python: 序号 = cls._索引(虚线序列, 当前线段.基础序列[-1]) + 1
        let 最后笔 = Arc::clone(当前线段.基础序列.read().last().unwrap());
        let 序号 = 线段::_索引(虚线序列, &最后笔) + 1;

        // Python: if 序号 >= len(虚线序列): return
        if 序号 >= 虚线序列.len() {
            return;
        }

        // -------------------- 循环处理后续虚线 --------------------
        // Python: for i in range(序号 + 1, len(虚线序列) - 1):
        for i in 序号 + 1..虚线序列.len() - 1 {
            let 左 = &虚线序列[i - 1];
            let 中 = &虚线序列[i];
            let 右 = &虚线序列[i + 1];
            let 相对关系 = 相对方向::分析(左.端点高(), 左.端点低(), 右.端点高(), 右.端点低());

            // Python: if 相对关系.是否缺口(): _添加虚线(当前线段, 左); _添加虚线(当前线段, 中); _武终; continue
            if 相对关系.是否缺口() {
                Self::_添加虚线(线段序列.last().unwrap(), Arc::clone(左));
                Self::_添加虚线(线段序列.last().unwrap(), Arc::clone(中));
                let seg_idx = 线段序列.len() - 1;
                Self::_武终(&线段序列[seg_idx], 0);
                continue;
            }

            // Python: if 左 in 当前线段.基础序列: continue
            if 线段序列
                .last()
                .unwrap()
                .基础序列
                .read()
                .iter()
                .any(|x| Arc::ptr_eq(x, 左))
            {
                continue;
            }

            // Python: 段 = 虚线.创建线段([左, 中, 右]); _添加扩展线段(...); return 递归
            let 段 = Arc::new(虚线::创建线段(&[
                Arc::clone(左),
                Arc::clone(中),
                Arc::clone(右),
            ]));
            Self::_添加扩展线段(线段序列, 段, line!());
            return Self::扩展分析_递归(虚线序列, 线段序列, 配置, 层级 + 1);
        }
    }

    // ================================================================
    // 背驰相关
    // ================================================================

    /// 判断线段内部是否背驰
    ///
    /// 分析线段的内部中枢和MACD柱分段，判断是否发生内部背驰
    pub fn 判断线段内部是否背驰(当前段: &虚线, 观察员: &观察者) -> bool {
        Self::判断线段内部是否背驰_impl(当前段, 观察员)
    }

    fn 判断线段内部是否背驰_impl(当前段: &虚线, 观察员: &观察者) -> bool {
        let 实 = &当前段.实_中枢序列;
        let (阳, 阴, _, _) = Self::分割序列(当前段, None);

        if !阴.is_empty() {
            // 阴不为空表示特征序列仍在合并中，不判断
        }
        let 笔之实数 = 阳.len();
        if 笔之实数 < 3 {
            return false;
        }

        let 进入段 = &阳[阳.len() - 3];
        let 离开段 = &阳[阳.len() - 1];
        assert!(
            进入段.序号.load(Ordering::Relaxed) < 离开段.序号.load(Ordering::Relaxed),
            "进入段.序号 >= 离开段.序号"
        );
        let 关系 = 相对方向::分析(进入段.高(), 进入段.低(), 离开段.高(), 离开段.低());
        let mut 背驰 = false;
        let mut 盘整背驰 = false;

        if ((进入段.方向().是否向上() && 关系.是否向上())
            || (进入段.方向().是否向下() && 关系.是否向下()))
            && crate::algorithm::divergence::背驰分析::背驰模式(
                进入段,
                离开段,
                &观察员.普通K线序列,
                &观察员.配置,
                &观察员.配置.线段内部背驰_模式,
            )
        {
            let k线序列 = K线::截取rc(
                &观察员.普通K线序列,
                &阳[阳.len() - 3].文.中.标的K线.read(),
                &阳[阳.len() - 1].武.read().中.标的K线.read(),
            );
            if 虚线::计算MACD柱子分段(&k线序列).len() >= 3 {
                盘整背驰 = true;
            }
        }

        let 实_ref = 实.read();
        if !实_ref.is_empty() {
            let 最后中枢 = &实_ref[实_ref.len() - 1];
            if 最后中枢
                .基础序列
                .read()
                .iter()
                .any(|b| Arc::ptr_eq(b, &阳[阳.len() - 1]))
            {
                // 最后一笔在最后一个中枢内
                // Python: 序号 = cls._索引(当前段.基础序列, 当前段.实_中枢序列[-1].基础序列[0])
                let 序号 = {
                    let guard = 当前段.基础序列.read();
                    线段::_索引(&guard, &最后中枢.基础序列.read()[0])
                };
                if 序号 > 0 && 序号 < 当前段.基础序列.read().len() {
                    let 进入段 = &当前段.基础序列.read()[序号 - 1];
                    let 离开段 = &阳[阳.len() - 1];
                    assert!(
                        进入段.序号.load(Ordering::Relaxed) < 离开段.序号.load(Ordering::Relaxed)
                    );
                    if 进入段.方向() != 离开段.方向() {
                        return crate::algorithm::divergence::背驰分析::测度背驰(
                            进入段, 离开段,
                        ) && 虚线::买卖意义(离开段, 观察员).0;
                    }
                    let 关系 =
                        相对方向::分析(进入段.高(), 进入段.低(), 离开段.高(), 离开段.低());
                    if ((进入段.方向().是否向上() && 关系.是否向上())
                        || (进入段.方向().是否向下() && 关系.是否向下()))
                        && crate::algorithm::divergence::背驰分析::背驰模式(
                            进入段,
                            离开段,
                            &观察员.普通K线序列,
                            &观察员.配置,
                            &观察员.配置.线段内部背驰_模式,
                        )
                    {
                        return true;
                    }
                }
            } else if 最后中枢.第三买卖线.read().is_some() {
                // 第三买卖点后盘整背驰
                let 进入段 = &阳[阳.len() - 3];
                let 离开段 = &阳[阳.len() - 1];
                assert!(进入段.序号.load(Ordering::Relaxed) < 离开段.序号.load(Ordering::Relaxed));
                if 进入段.方向() != 离开段.方向() {
                    return crate::algorithm::divergence::背驰分析::测度背驰(
                        进入段, 离开段,
                    ) && 虚线::买卖意义(离开段, 观察员).0;
                }
                let 关系 = 相对方向::分析(进入段.高(), 进入段.低(), 离开段.高(), 离开段.低());
                if ((进入段.方向().是否向上() && 关系.是否向上())
                    || (进入段.方向().是否向下() && 关系.是否向下()))
                    && crate::algorithm::divergence::背驰分析::背驰模式(
                        进入段,
                        离开段,
                        &观察员.普通K线序列,
                        &观察员.配置,
                        &观察员.配置.线段内部背驰_模式,
                    )
                {
                    return true;
                }
            }
        } else {
            // 没有中枢
            if 笔之实数 == 3 {
                背驰 = 盘整背驰;
            }
        }

        背驰 || 盘整背驰
    }

    /// 获取所有停顿位置 — 在线段范围内找出所有停顿位置
    pub fn 获取所有停顿位置(段: &虚线, 观察员: &观察者) -> Vec<虚线> {
        let cap = 段.基础序列.read().len() / 2;
        let mut 结果 = Vec::with_capacity(cap);
        if 段.模式.read().as_str() != "文武" || 段.标识.read().as_str() != "线段" {
            return 结果;
        }

        let (阳, _阴, _, _) = Self::分割序列(段, None);
        if 阳.len() < 3 {
            return 结果;
        }

        let mut 线段序列: Vec<Arc<虚线>> = Vec::new();
        let mut 笔序列: Vec<Arc<虚线>> = Vec::new();
        let mut 当前停顿: Option<std::sync::Arc<分型>> = None;

        for 筆 in &阳 {
            if 笔序列.len() >= 3 {
                let 筆停顿 = 笔::获取所有停顿位置(筆, 观察员);
                let mut 停顿列表: Vec<Arc<虚线>> = 筆停顿.into_iter().map(Arc::new).collect();
                停顿列表.push(Arc::clone(筆));

                for 停顿 in &停顿列表 {
                    笔序列.push(Arc::clone(停顿));
                    Self::分析(
                        &笔序列,
                        &mut 线段序列,
                        &观察员.配置,
                        0,
                        &[
                            相对方向::向下,
                            相对方向::向上,
                            相对方向::顺,
                            相对方向::逆,
                            相对方向::同,
                        ],
                    );

                    let 重复 = match (&线段序列.last(), &当前停顿) {
                        (Some(a), Some(b)) => Arc::ptr_eq(&*a.武.read(), b),
                        _ => false,
                    };
                    if !重复
                        && let Some(最后线段) = 线段序列.last()
                        && 最后线段.基础序列.read().len() % 2 == 1
                    {
                        let 新段 = 虚线::创建线段(&最后线段.基础序列.read());
                        新段
                            .序号
                            .store(段.序号.load(Ordering::Relaxed), Ordering::Relaxed);
                        let 新段_rc = Arc::new(新段);
                        Self::_刷新(&新段_rc, &观察员.配置);
                        let 新段_inner =
                            Arc::try_unwrap(新段_rc).unwrap_or_else(|rc| (*rc).clone());
                        if 新段_inner.方向() == 段.方向() {
                            当前停顿 = Some(Arc::clone(&*线段序列.last().unwrap().武.read()));
                            结果.push(新段_inner);
                        }
                    }

                    if !Arc::ptr_eq(停顿, 筆)
                        && let Some(popped) = 笔序列.pop()
                    {
                        popped.有效性.store(false, Ordering::Relaxed);
                    }
                }
            } else {
                笔序列.push(Arc::clone(筆));
            }
        }
        结果
    }

    /// 是否背驰过 — 判断线段是否在停顿位置出现过背驰
    pub fn 是否背驰过(当前段: &虚线, 观察员: &观察者) -> Vec<Arc<缠论K线>> {
        let 停顿位置 = Self::获取所有停顿位置(当前段, 观察员);
        let mut 结果 = Vec::new();

        for 段 in 停顿位置 {
            let 段_rc = Arc::new(段);
            Self::获取内部中枢序列(&段_rc, &观察员.配置);
            if Self::判断线段内部是否背驰(&段_rc, 观察员) {
                结果.push(Arc::clone(&段_rc.武.read().中));
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
    use crate::{error, info, warn};

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

    fn 辅助_创建顶分型(时间戳: i64, 高: f64) -> Arc<分型> {
        辅助_创建分型(
            辅助_创建缠K(
                时间戳 - 2,
                高 - 2.0,
                高 - 4.0,
                相对方向::向上,
                Some(分型结构::上),
            ),
            辅助_创建缠K(时间戳, 高, 高 - 3.0, 相对方向::向上, Some(分型结构::顶)),
            辅助_创建缠K(
                时间戳 + 2,
                高 - 2.0,
                高 - 4.0,
                相对方向::向下,
                Some(分型结构::下),
            ),
        )
    }

    fn 辅助_创建底分型(时间戳: i64, 低: f64) -> Arc<分型> {
        辅助_创建分型(
            辅助_创建缠K(
                时间戳 - 2,
                低 + 4.0,
                低 + 2.0,
                相对方向::向下,
                Some(分型结构::下),
            ),
            辅助_创建缠K(时间戳, 低 + 3.0, 低, 相对方向::向下, Some(分型结构::底)),
            辅助_创建缠K(
                时间戳 + 2,
                低 + 4.0,
                低 + 2.0,
                相对方向::向上,
                Some(分型结构::上),
            ),
        )
    }

    fn 辅助_创建笔(文: Arc<分型>, 武: Arc<分型>) -> Arc<虚线> {
        Arc::new(虚线::创建笔(文, 武, true))
    }

    // ========== 四象 测试 ==========

    #[test]
    fn test_四象_无缺口向上() {
        let 文 = 辅助_创建底分型(100, 90.0);
        let 武 = 辅助_创建顶分型(200, 110.0);
        let 段 = 辅助_创建笔(文, 武);
        assert_eq!(线段::四象(&段), "小阳");
    }

    #[test]
    fn test_四象_无缺口向下() {
        let 文 = 辅助_创建顶分型(100, 110.0);
        let 武 = 辅助_创建底分型(200, 90.0);
        let 段 = 辅助_创建笔(文, 武);
        assert_eq!(线段::四象(&段), "少阴");
    }

    #[test]
    fn test_四象_有缺口向上() {
        let 文 = 辅助_创建底分型(100, 90.0);
        let 武 = 辅助_创建顶分型(200, 110.0);
        let 段 = 辅助_创建笔(文, 武);
        *段.前一缺口.write() = Some(缺口 {
            高: 105.0,
            低: 95.0,
        });
        assert_eq!(线段::四象(&段), "老阳");
    }

    #[test]
    fn test_四象_有缺口向下() {
        let 文 = 辅助_创建顶分型(100, 110.0);
        let 武 = 辅助_创建底分型(200, 90.0);
        let 段 = 辅助_创建笔(文, 武);
        *段.前一缺口.write() = Some(缺口 {
            高: 105.0,
            低: 95.0,
        });
        assert_eq!(线段::四象(&段), "老阴");
    }

    // ========== 特征序列状态 测试 ==========

    #[test]
    fn test_特征序列状态_全部空() {
        let 文 = 辅助_创建底分型(100, 90.0);
        let 武 = 辅助_创建顶分型(200, 110.0);
        let 段 = 辅助_创建笔(文, 武);
        assert_eq!(线段::特征序列状态(&段), (false, false, false));
    }

    // ========== 获取缺口 测试 ==========

    #[test]
    fn test_获取缺口_模式非文武返回空() {
        let 文 = 辅助_创建底分型(100, 90.0);
        let 武 = 辅助_创建顶分型(200, 110.0);
        let 段 = 辅助_创建笔(文, 武);
        *段.模式.write() = "其他".into();
        assert!(线段::获取缺口(&段).is_none());
    }

    #[test]
    fn test_获取缺口_特征序列不足返回空() {
        let 文 = 辅助_创建底分型(100, 90.0);
        let 武 = 辅助_创建顶分型(200, 110.0);
        let 段 = 辅助_创建笔(文, 武);
        assert!(线段::获取缺口(&段).is_none());
    }

    // ========== 分割序列 测试 ==========

    #[test]
    fn test_分割序列_基本笔_无中枢() {
        let 文 = 辅助_创建底分型(100, 90.0);
        let 武 = 辅助_创建顶分型(200, 110.0);
        let 段 = 辅助_创建笔(文, 武);
        let (_前, 后, 第三买卖线, 贯穿伤) = 线段::分割序列(&段, None);
        // 独立笔无基础序列时，前也为空
        assert!(后.is_empty());
        assert!(第三买卖线.is_empty());
        assert!(贯穿伤.is_none());
    }
}
