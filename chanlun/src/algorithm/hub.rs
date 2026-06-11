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

use crate::structure::dash_line::虚线;
use crate::structure::fractal_obj::分型;
use crate::types::相对方向;
use parking_lot::RwLock;
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};

/// 中枢 — 三段虚线重叠区间构成的价格中枢
///
/// 可变字段使用 AtomicI64 / RwLock 实现内部可变性，确保多 Arc 共享时可修改。
///
/// 字段:
/// - 序号: 中枢序号，同一级别内递增
/// - 标识: 中枢标识，格式如 "笔中枢<0>" 或 "线段中枢<1>"
/// - 级别: 中枢级别（笔中枢=1，线段中枢=2 等）
/// - 基础序列: 构成中枢的虚线序列（至少 3 根，延伸后可多至 9 根甚至更多）
/// - 第三买卖线: 第三类买卖点对应的虚线（离开中枢后不回中枢的虚线）
/// - 本级_第三买卖线: 本级第三类买卖点对应的虚线
#[derive(Debug)]
pub struct 中枢 {
    /// 中枢序号，同一级别内递增
    pub 序号: AtomicI64,
    /// 中枢标识，格式如 "笔中枢<0>" 或 "线段中枢<1>"
    pub 标识: RwLock<String>,
    /// 中枢级别（笔中枢=1，线段中枢=2 等）
    pub 级别: AtomicI64,
    /// 构成中枢的虚线序列（至少 3 根，延伸后可多至 9+ 根）
    pub 基础序列: RwLock<Vec<Arc<虚线>>>,
    /// 第三类买卖点对应的虚线（离开中枢后不回中枢的虚线）
    pub 第三买卖线: RwLock<Option<Arc<虚线>>>,
    /// 本级第三类买卖点对应的虚线
    pub 本级_第三买卖线: RwLock<Option<Arc<虚线>>>,
}

impl Clone for 中枢 {
    fn clone(&self) -> Self {
        Self {
            序号: AtomicI64::new(self.序号.load(Ordering::Relaxed)),
            标识: RwLock::new(self.标识.read().clone()),
            级别: AtomicI64::new(self.级别.load(Ordering::Relaxed)),
            基础序列: RwLock::new(self.基础序列.read().clone()),
            第三买卖线: RwLock::new(self.第三买卖线.read().clone()),
            本级_第三买卖线: RwLock::new(self.本级_第三买卖线.read().clone()),
        }
    }
}

impl 中枢 {
    /// 创建新中枢（最多取前 3 根虚线作为基础序列）
    pub fn new(序号: i64, 标识: String, 级别: i64, 基础序列: Vec<Arc<虚线>>) -> Self {
        Self {
            序号: AtomicI64::new(序号),
            标识: RwLock::new(标识),
            级别: AtomicI64::new(级别),
            基础序列: RwLock::new(基础序列.into_iter().take(3).collect()),
            第三买卖线: RwLock::new(None),
            本级_第三买卖线: RwLock::new(None),
        }
    }

    /// 向基础序列尾部添加虚线（中枢延伸），并清除第三买卖线
    pub fn _添加虚线(&self, 实线: Arc<虚线>) {
        self.基础序列.write().push(实线);
        *self.本级_第三买卖线.write() = None;
        *self.第三买卖线.write() = None;
    }

    /// 返回图表标题字符串，格式为 "文.标识:文.周期:中枢标识:序号"
    pub fn 图表标题(&self) -> String {
        format!(
            "{}:{}:{}:{}",
            self.文().中.标识,
            self.文().中.周期,
            self.标识.read(),
            self.序号.load(Ordering::Relaxed)
        )
    }

    /// 返回基础序列的最后一根虚线（当前离开段）
    pub fn 离开段(&self) -> Arc<虚线> {
        let guard = self.基础序列.read();
        Arc::clone(&guard[guard.len() - 1])
    }

    /// 返回中枢方向（与基础序列第一段方向相反）
    pub fn 方向(&self) -> 相对方向 {
        self.基础序列.read()[0].方向().翻转()
    }

    /// 中枢上沿 = min(前三段的高)
    pub fn 高(&self) -> f64 {
        self.基础序列.read()[..3]
            .iter()
            .map(|x| x.高())
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0)
    }

    /// 中枢下沿 = max(前三段的低)
    pub fn 低(&self) -> f64 {
        self.基础序列.read()[..3]
            .iter()
            .map(|x| x.低())
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0)
    }

    /// 中枢最高点 = max(所有段的高)
    pub fn 高高(&self) -> f64 {
        self.基础序列
            .read()
            .iter()
            .map(|x| x.高())
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0)
    }

    /// 中枢最低点 = min(所有段的低)
    pub fn 低低(&self) -> f64 {
        self.基础序列
            .read()
            .iter()
            .map(|x| x.低())
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0)
    }

    /// 返回基础序列第一段的起点分型
    pub fn 文(&self) -> Arc<分型> {
        Arc::clone(&self.基础序列.read()[0].文)
    }

    /// 返回基础序列最后一段的终点分型
    pub fn 武(&self) -> Arc<分型> {
        let guard = self.基础序列.read();
        Arc::clone(&*guard[guard.len() - 1].武.read())
    }

    /// 设置第三类买卖点对应的虚线
    pub fn 设置第三买卖线(&self, 线: Option<Arc<虚线>>) {
        *self.第三买卖线.write() = 线;
    }

    /// 获取序列 — 基础序列 + 第三买卖线（若有）
    pub fn 获取序列(&self) -> Vec<Arc<虚线>> {
        let mut 序列: Vec<Arc<虚线>> = self.基础序列.read().clone();
        if let Some(ref 三买) = *self.第三买卖线.read() {
            序列.push(Arc::clone(三买));
        }
        序列
    }

    /// 获取基础序列最后一个元素
    pub fn 最后(&self) -> Option<Arc<虚线>> {
        self.基础序列.read().last().cloned()
    }

    /// 返回序列化数据文本，用于调试和存储
    pub fn 获取数据文本(&self) -> String {
        let 第三买卖线_str = match &*self.第三买卖线.read() {
            Some(x) => format!("{}", x),
            None => "None".to_string(),
        };
        let 本级_第三买卖线_str = match &*self.本级_第三买卖线.read() {
            Some(x) => format!("{}", x),
            None => "None".to_string(),
        };
        format!(
            "{}, {}, {}, 文:({},{}), 武:({},{}), {}, {}",
            self.标识.read(),
            self.序号.load(Ordering::Relaxed),
            self.级别.load(Ordering::Relaxed),
            self.文().时间戳(),
            crate::utils::format_f64_g(self.文().分型特征值),
            self.武().时间戳(),
            crate::utils::format_f64_g(self.武().分型特征值),
            第三买卖线_str,
            本级_第三买卖线_str,
        )
    }

    /// 校验中枢合法性
    pub fn _校验合法性(&self, 序列: &[Arc<虚线>]) -> bool {
        let guard = self.基础序列.read();
        let mut 有效序列 = guard.clone();
        let mut 无效序列: Vec<Arc<虚线>> = Vec::new();
        let 首序号 = 序列[0].序号.load(Ordering::Relaxed);
        for 元素 in guard.iter() {
            let idx = (元素.序号.load(Ordering::Relaxed) - 首序号) as usize;
            if idx >= 序列.len() || !Arc::ptr_eq(&序列[idx], 元素) {
                无效序列.push(Arc::clone(元素));
            }
        }
        if !无效序列.is_empty() {
            let 无效 = &无效序列[0];
            // Python: 序号 = 线段._索引(self.基础序列, 无效)
            let pos = crate::algorithm::segment::线段::_索引(&guard, 无效);
            有效序列 = guard[..pos].to_vec();
        }
        drop(guard);

        if 有效序列.len() < 3 {
            self.设置第三买卖线(None);
            *self.本级_第三买卖线.write() = None;
            return false;
        }

        *self.基础序列.write() = 有效序列;

        let 中枢高 = self.高();
        let 中枢低 = self.低();
        有效序列 = Vec::new();
        for 元素 in self.基础序列.read().iter() {
            if crate::types::相对方向::分析(中枢高, 中枢低, 元素.高(), 元素.低()).是否缺口()
            {
                break;
            }
            有效序列.push(Arc::clone(元素));
        }
        *self.基础序列.write() = 有效序列;

        let 是缺口 = {
            let guard = self.基础序列.read();
            if guard.len() < 3 {
                return false;
            }
            for i in 1..guard.len() {
                if !guard[i - 1].之后是(&guard[i]) {
                    return false;
                }
            }
            crate::types::相对方向::分析(
                guard[0].高(),
                guard[0].低(),
                guard[2].高(),
                guard[2].低(),
            )
            .是否缺口()
        };
        if !是缺口 {
            let 重叠高 = self.高();
            let 重叠低 = self.低();
            if 重叠低 > 重叠高 {
                return false;
            }
        }

        let 三买线_opt = self.第三买卖线.read().clone();
        if let Some(ref 三买线) = 三买线_opt {
            let 首序号 = 序列[0].序号.load(Ordering::Relaxed);
            let idx = (三买线.序号.load(Ordering::Relaxed) - 首序号) as usize;
            if idx < 序列.len() && Arc::ptr_eq(&序列[idx], 三买线) {
                if !self.基础序列.read().last().unwrap().之后是(三买线) {
                    self.设置第三买卖线(None);
                } else if !crate::types::相对方向::分析(
                    self.高(),
                    self.低(),
                    三买线.高(),
                    三买线.低(),
                )
                .是否缺口()
                {
                    self._添加虚线(Arc::clone(三买线));
                    self.设置第三买卖线(None);
                }
            } else {
                self.设置第三买卖线(None);
            }
        }
        true
    }

    /// 完整性 — 详见教你炒股票43：有关背驰的补习课
    /// 不完整时下一个中枢大概率会与当前中枢发生扩展
    pub fn 完整性(&self, 虚实: &str) -> bool {
        if *self.基础序列.read()[0].标识.read() == "笔" {
            return self.第三买卖线.read().is_some();
        }

        let 基础序列_ref = self.基础序列.read();
        let 最后段 = 基础序列_ref.last().unwrap();
        let 内部中枢_vec = if 虚实 == "合" {
            最后段.合_中枢序列.read()
        } else {
            最后段.实_中枢序列.read()
        };
        for 内部中枢 in 内部中枢_vec.iter() {
            if crate::types::相对方向::分析(
                self.高(),
                self.低(),
                内部中枢.高(),
                内部中枢.低(),
            )
            .是否缺口()
            {
                return true;
            }
        }
        false
    }

    /// 获取扩展中枢 — 当基础序列 >= 9 时生成扩展中枢
    pub fn 获取扩展中枢(
        &self,
        扩展中枢: &mut Vec<Arc<中枢>>,
        配置: &crate::config::缠论配置,
    ) {
        if self.基础序列.read().len() >= 9 {
            let mut 扩展线段: Vec<Arc<虚线>> = Vec::new();
            let 基础序列_ref = self.基础序列.read();
            crate::algorithm::segment::线段::扩展分析(&基础序列_ref, &mut 扩展线段, 配置);
            let 标识 = format!("{}_扩展中枢_", self.标识.read());
            中枢::分析(&扩展线段, 扩展中枢, false, &标识, 0);
        }
    }

    /// 当前状态 — 详见教你炒股票49：利润率最大的操作模式
    /// 返回当前中枢最后一段所处的位置关系：中枢之中/中枢之上/中枢之下
    pub fn 当前状态(&self) -> &str {
        let 基础序列_ref = self.基础序列.read();
        let 最后 = Arc::clone(基础序列_ref.last().unwrap());
        let 尾部 = 最后.获取_武();
        let 关系 = crate::types::相对方向::分析(
            self.高(),
            self.低(),
            尾部.中.高.get(),
            尾部.中.低.get(),
        );
        if 关系 == crate::types::相对方向::向上缺口 {
            "中枢之上"
        } else if 关系 == crate::types::相对方向::向下缺口 {
            "中枢之下"
        } else {
            "中枢之中"
        }
    }

    // ---- 关联函数 ----

    /// 基础检查 — 三根虚线是否能形成中枢
    pub fn 基础检查(左: &虚线, 中: &虚线, 右: &虚线) -> bool {
        if !左.之后是(中) || !中.之后是(右) {
            return false;
        }
        let 关系 = crate::types::相对方向::分析(左.高(), 左.低(), 右.高(), 右.低());
        matches!(
            关系,
            crate::types::相对方向::向下
                | crate::types::相对方向::向上
                | crate::types::相对方向::顺
                | crate::types::相对方向::逆
                | crate::types::相对方向::同
        )
    }

    /// 创建中枢
    pub fn 创建(
        左: Arc<虚线>, 中: Arc<虚线>, 右: Arc<虚线>, 级别: i64, 标识: &str
    ) -> Self {
        assert!(Self::基础检查(&左, &中, &右), "中枢.创建 基础检查失败");
        Self::new(
            0,
            format!("{}中枢<{}>", 标识, 中.标识.read()),
            级别,
            vec![左, 中, 右],
        )
    }

    /// _从序列中获取中枢
    pub fn _从序列中获取中枢(
        虚线序列: &[Arc<虚线>],
        起始方向: 相对方向,
        标识: &str,
    ) -> Option<Arc<中枢>> {
        for i in 2..虚线序列.len() {
            let 左 = &虚线序列[i - 2];
            let 中 = &虚线序列[i - 1];
            let 右 = &虚线序列[i];
            if Self::基础检查(左, 中, 右) && 左.方向() == 起始方向 {
                let 中枢 = Self::创建(Arc::clone(左), Arc::clone(中), Arc::clone(右), 0, 标识);
                return Some(Arc::new(中枢));
            }
        }
        None
    }

    /// _向中枢序列尾部添加
    pub fn _向中枢序列尾部添加(
        中枢序列: &mut Vec<Arc<中枢>>, 待添加中枢: Arc<中枢>
    ) {
        if let Some(前一个) = 中枢序列.last() {
            待添加中枢
                .序号
                .store(前一个.序号.load(Ordering::Relaxed) + 1, Ordering::Relaxed);
            let 前_last_序号 = 前一个.最后().unwrap().序号.load(Ordering::Relaxed);
            let new_last_序号 = 待添加中枢.最后().unwrap().序号.load(Ordering::Relaxed);
            if 前_last_序号 > new_last_序号 {
                panic!(
                    "向中枢序列尾部添加 序号错误 前last={} > new_last={}",
                    前_last_序号, new_last_序号
                );
            }
        }
        中枢序列.push(待添加中枢);
    }

    /// 从中枢序列尾部弹出
    pub fn _从中枢序列尾部弹出(
        中枢序列: &mut Vec<Arc<中枢>>,
        待弹出: &Arc<中枢>,
    ) -> Option<Arc<中枢>> {
        if 中枢序列.last().map(Arc::as_ptr) == Some(Arc::as_ptr(待弹出)) {
            中枢序列.pop()
        } else {
            None
        }
    }

    /// 中枢分析 — 从虚线序列生成中枢序列（增量算法）
    ///
    /// 每收到新的虚线序列数据后调用，更新中枢序列
    pub fn 分析(
        虚线序列: &[Arc<虚线>],
        中枢序列: &mut Vec<Arc<中枢>>,
        跳过首部: bool,
        标识: &str,
        _层级: i64,
    ) {
        if 虚线序列.len() < 3 {
            return;
        }

        // 初始化第一个中枢
        if 中枢序列.is_empty() {
            for i in 1..虚线序列.len() - 1 {
                let 左 = &虚线序列[i - 1];
                let 中 = &虚线序列[i];
                let 右 = &虚线序列[i + 1];

                if Self::基础检查(左, 中, 右) {
                    // Python: 序号 = 线段._索引(虚线序列, 左)
                    let 序号: usize = crate::algorithm::segment::线段::_索引(虚线序列, 左);
                    if 跳过首部 && (左.序号.load(Ordering::Relaxed) == 0 || 序号 == 0) {
                        continue;
                    }
                    if 序号 >= 2 {
                        let 前 = &虚线序列[序号 - 2];
                        let 同向相对关系 =
                            crate::types::相对方向::分析(前.高(), 前.低(), 左.高(), 左.低());
                        if 同向相对关系.是否向上() && 左.方向().是否向上() {
                            continue;
                        }
                        if 同向相对关系.是否向下() && 左.方向().是否向下() {
                            continue;
                        }
                    }
                    let 新中枢 = Arc::new(Self::创建(
                        Arc::clone(左),
                        Arc::clone(中),
                        Arc::clone(右),
                        中.级别.load(Ordering::Relaxed),
                        标识,
                    ));
                    Self::_向中枢序列尾部添加(中枢序列, 新中枢);
                    // Python: return 中枢递归分析(虚线序列, 中枢序列, ...)
                    Self::分析(虚线序列, 中枢序列, 跳过首部, 标识, _层级);
                    return;
                }
            }
            return;
        }

        // 增量更新
        let mut 当前中枢_idx = 中枢序列.len() - 1;

        // Validate via shared reference (中枢 uses RwLock internally)
        let needs_pop = !中枢序列[当前中枢_idx]._校验合法性(虚线序列);
        if needs_pop {
            let 当前中枢 = Arc::clone(&中枢序列[当前中枢_idx]);
            Self::_从中枢序列尾部弹出(中枢序列, &当前中枢);
            Self::分析(虚线序列, 中枢序列, 跳过首部, 标识, _层级);
            return;
        }

        // Python: 序号 = 线段._索引(虚线序列, 当前中枢.基础序列[-1]) + 1
        let 起始索引 = {
            let cur = &中枢序列[当前中枢_idx];
            let guard = cur.基础序列.read();
            crate::algorithm::segment::线段::_索引(虚线序列, &guard[guard.len() - 1]) + 1
        };

        let mut 中枢高 = 中枢序列[当前中枢_idx].高();
        let mut 中枢低 = 中枢序列[当前中枢_idx].低();
        let mut 候选序列 = Vec::with_capacity(3);

        for 当前虚线_ref in &虚线序列[起始索引..] {
            let 当前虚线 = Arc::clone(当前虚线_ref);

            // 检查是否超出中枢范围（缺口）
            if crate::types::相对方向::分析(中枢高, 中枢低, 当前虚线.高(), 当前虚线.低()).是否缺口()
            {
                候选序列.push(当前虚线.clone());

                // Python: if 当前中枢.基础序列[-1].之后是(当前虚线):
                let needs_三买 = {
                    let cur = &中枢序列[当前中枢_idx];
                    cur.基础序列.read().last().unwrap().之后是(&当前虚线)
                };
                if needs_三买 {
                    中枢序列[当前中枢_idx].设置第三买卖线(Some(当前虚线.clone()));
                }
            } else {
                if 候选序列.is_empty() {
                    // 仍在范围内：延伸中枢
                    assert!(
                        中枢序列[当前中枢_idx]
                            .基础序列
                            .read()
                            .last()
                            .unwrap()
                            .之后是(&当前虚线),
                        "中枢延伸: 不连续 {}, {}",
                        中枢序列[当前中枢_idx].基础序列.read().last().unwrap(),
                        当前虚线
                    );
                    中枢序列[当前中枢_idx]._添加虚线(当前虚线);
                } else {
                    候选序列.push(当前虚线);
                }
            }

            // 候选序列积满3个：尝试创建新中枢
            while 候选序列.len() >= 3 {
                let 起始方向 = 中枢序列[当前中枢_idx]
                    .基础序列
                    .read()
                    .last()
                    .unwrap()
                    .方向()
                    .翻转();
                match Self::_从序列中获取中枢(&候选序列, 起始方向, 标识) {
                    Some(新中枢) => {
                        Self::_向中枢序列尾部添加(中枢序列, 新中枢);
                        // Python: 当前中枢 = 新中枢
                        当前中枢_idx = 中枢序列.len() - 1;
                        中枢高 = 中枢序列[当前中枢_idx].高();
                        中枢低 = 中枢序列[当前中枢_idx].低();
                        候选序列.clear();
                    }
                    None => {
                        候选序列.remove(0); // 滑动窗口
                    }
                }
            }
        }
    }

    /// 结构化相等校验 — 递归校验基础序列虚线和第三买卖线，返回 (是否相等, 差异描述)
    pub fn 相等(&self, other: &Self, 浮点容差: f64) -> (bool, String) {
        if self.序号.load(Ordering::Relaxed) != other.序号.load(Ordering::Relaxed) {
            return (
                false,
                format!(
                    "中枢: [序号] 不等 A={},B={}",
                    self.序号.load(Ordering::Relaxed),
                    other.序号.load(Ordering::Relaxed)
                ),
            );
        }
        if *self.标识.read() != *other.标识.read() {
            return (
                false,
                format!(
                    "中枢: [标识] 不等 A={},B={}",
                    self.标识.read(),
                    other.标识.read()
                ),
            );
        }
        if self.级别.load(Ordering::Relaxed) != other.级别.load(Ordering::Relaxed) {
            return (
                false,
                format!(
                    "中枢: [级别] 不等 A={},B={}",
                    self.级别.load(Ordering::Relaxed),
                    other.级别.load(Ordering::Relaxed)
                ),
            );
        }
        // 基础序列
        let a_seq = self.基础序列.read();
        let b_seq = other.基础序列.read();
        if a_seq.len() != b_seq.len() {
            return (
                false,
                format!(
                    "中枢: [基础序列] 长度不一致 A={},B={}",
                    a_seq.len(),
                    b_seq.len()
                ),
            );
        }
        for (idx, (a, b)) in a_seq.iter().zip(b_seq.iter()).enumerate() {
            let (eq, msg) = a.相等(b, 浮点容差);
            if !eq {
                return (false, format!("中枢: 基础序列[{idx}]虚线异常 >> {msg}"));
            }
        }
        // 第三买卖线
        let 检查单个 = |名: &str,
                        a: &Option<Arc<虚线>>,
                        b: &Option<Arc<虚线>>,
                        容差: f64|
         -> Result<(), String> {
            match (a, b) {
                (None, None) => Ok(()),
                (Some(x), Some(y)) => {
                    let (eq, msg) = x.相等(y, 容差);
                    if eq {
                        Ok(())
                    } else {
                        Err(format!("中枢: [{名}]子虚线异常 >> {msg}"))
                    }
                }
                _ => Err(format!(
                    "中枢: [{名}]空值不一致 A={},B={}",
                    a.is_some(),
                    b.is_some()
                )),
            }
        };
        检查单个(
            "第三买卖线",
            &self.第三买卖线.read(),
            &other.第三买卖线.read(),
            浮点容差,
        )
        .map_err(|e| (false, e))
        .ok();
        检查单个(
            "本级_第三买卖线",
            &self.本级_第三买卖线.read(),
            &other.本级_第三买卖线.read(),
            浮点容差,
        )
        .map_err(|e| (false, e))
        .ok();
        (true, "中枢: 全部字段一致".into())
    }
}

impl std::fmt::Display for 中枢 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let guard = self.基础序列.read();
        let len = guard.len();
        let 序列_str = if let Some((first, rest)) = guard.split_first() {
            let mut s = format!("{}", first);
            for d in rest {
                use std::fmt::Write;
                write!(&mut s, ", {}", d).unwrap();
            }
            s
        } else {
            String::new()
        };
        drop(guard);
        write!(
            f,
            "{}({}, {}, 元素数量: {}, [{}], {} ===>>> {})",
            self.标识.read(),
            crate::utils::format_f64_g(self.高()),
            crate::utils::format_f64_g(self.低()),
            len,
            序列_str,
            self.文(),
            self.武(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kline::bar::K线;
    use crate::kline::chan_kline::缠论K线;
    use crate::structure::fractal_obj::分型;
    use crate::types::分型结构;

    fn 辅助_创建K线(时间戳: i64, 高: f64, 低: f64, 开: f64, 收: f64) -> K线 {
        K线 {
            时间戳,
            高,
            低,
            开盘价: 开,
            收盘价: 收,
            ..Default::default()
        }
    }

    fn 辅助_创建缠K(
        时间戳: i64,
        高: f64,
        低: f64,
        方向: 相对方向,
        结构: Option<分型结构>,
        序号: i64,
    ) -> Arc<缠论K线> {
        let 普K = Arc::new(辅助_创建K线(时间戳, 高, 低, 低, 高));
        Arc::new(缠论K线::创建缠K(
            时间戳, 高, 低, 方向, 结构, 序号, 普K, None,
        ))
    }

    fn 辅助_创建顶分型(时间戳: i64, 高: f64, 低: f64, 序号: i64) -> Arc<分型> {
        let 左 = 辅助_创建缠K(
            时间戳 - 2,
            高 - 2.0,
            低 - 2.0,
            相对方向::向上,
            Some(分型结构::上),
            序号 - 2,
        );
        let 中 = 辅助_创建缠K(时间戳, 高, 低, 相对方向::向上, Some(分型结构::顶), 序号);
        let 右 = 辅助_创建缠K(
            时间戳 + 2,
            高 - 1.0,
            低 - 1.0,
            相对方向::向下,
            Some(分型结构::下),
            序号 + 2,
        );
        Arc::new(分型::new(Some(左), 中, Some(右)))
    }

    fn 辅助_创建底分型(时间戳: i64, 高: f64, 低: f64, 序号: i64) -> Arc<分型> {
        let 左 = 辅助_创建缠K(
            时间戳 - 2,
            高 + 2.0,
            低 + 2.0,
            相对方向::向下,
            Some(分型结构::下),
            序号 - 2,
        );
        let 中 = 缠论K线::创建缠K(
            时间戳,
            高,
            低,
            相对方向::向下,
            Some(分型结构::底),
            序号,
            Arc::new(辅助_创建K线(时间戳, 高, 低, 低, 高)),
            None,
        );
        中.分型特征值.set(低);
        let 中 = Arc::new(中);
        let 右 = 辅助_创建缠K(
            时间戳 + 2,
            高 + 1.0,
            低 + 1.0,
            相对方向::向上,
            Some(分型结构::上),
            序号 + 2,
        );
        Arc::new(分型::new(Some(左), 中, Some(右)))
    }

    fn 辅助_创建笔(
        文时间戳: i64,
        文高: f64,
        文低: f64,
        武时间戳: i64,
        武高: f64,
        武低: f64,
    ) -> Arc<虚线> {
        let 顶 = 辅助_创建顶分型(文时间戳, 文高, 文低, 1);
        let 底 = 辅助_创建底分型(武时间戳, 武高, 武低, 2);
        Arc::new(虚线::创建笔(顶, 底, true))
    }

    // ============================================================
    // 中枢 Cell/RefCell 字段读写
    // ============================================================

    #[test]
    fn test_中枢创建后字段初始值正确() {
        let 笔1 = 辅助_创建笔(100, 50.0, 40.0, 200, 30.0, 20.0);
        let 笔2 = 辅助_创建笔(200, 30.0, 20.0, 300, 55.0, 45.0);
        let 笔3 = 辅助_创建笔(300, 55.0, 45.0, 400, 25.0, 15.0);

        let 中枢 = 中枢::new(
            1,
            "测试中枢".into(),
            1,
            vec![Arc::clone(&笔1), Arc::clone(&笔2), Arc::clone(&笔3)],
        );

        assert_eq!(中枢.序号.load(Ordering::Relaxed), 1);
        assert_eq!(*中枢.标识.read(), "测试中枢");
        assert_eq!(中枢.级别.load(Ordering::Relaxed), 1);
        assert_eq!(中枢.基础序列.read().len(), 3);
        assert!(中枢.第三买卖线.read().is_none());
        assert!(中枢.本级_第三买卖线.read().is_none());
    }

    #[test]
    fn test_中枢CellRefCell字段读写() {
        let 笔1 = 辅助_创建笔(100, 50.0, 40.0, 200, 30.0, 20.0);
        let 笔2 = 辅助_创建笔(200, 30.0, 20.0, 300, 55.0, 45.0);
        let 笔3 = 辅助_创建笔(300, 55.0, 45.0, 400, 25.0, 15.0);

        let 中枢 = 中枢::new(
            0,
            "测试".into(),
            1,
            vec![Arc::clone(&笔1), Arc::clone(&笔2), Arc::clone(&笔3)],
        );

        // Cell 序号读写
        中枢.序号.store(99, Ordering::Relaxed);
        assert_eq!(中枢.序号.load(Ordering::Relaxed), 99);

        // RefCell 第三买卖线读写
        中枢.设置第三买卖线(Some(Arc::clone(&笔1)));
        assert!(中枢.第三买卖线.read().is_some());
        assert_eq!(
            Arc::as_ptr(中枢.第三买卖线.read().as_ref().unwrap()),
            Arc::as_ptr(&笔1)
        );

        // 本级_第三买卖线
        assert!(中枢.本级_第三买卖线.read().is_none());
        *中枢.本级_第三买卖线.write() = Some(Arc::clone(&笔3));
        assert!(中枢.本级_第三买卖线.read().is_some());
    }

    // ============================================================
    // 中枢 添加虚线
    // ============================================================

    #[test]
    fn test_中枢添加虚线后基础序列扩展() {
        let 笔1 = 辅助_创建笔(100, 50.0, 40.0, 200, 30.0, 20.0);
        let 笔2 = 辅助_创建笔(200, 30.0, 20.0, 300, 55.0, 45.0);
        let 笔3 = 辅助_创建笔(300, 55.0, 45.0, 400, 25.0, 15.0);
        let 笔4 = 辅助_创建笔(400, 25.0, 15.0, 500, 60.0, 50.0);

        let 中枢 = 中枢::new(
            0,
            "测试".into(),
            1,
            vec![Arc::clone(&笔1), Arc::clone(&笔2), Arc::clone(&笔3)],
        );
        assert_eq!(中枢.基础序列.read().len(), 3);

        中枢._添加虚线(Arc::clone(&笔4));
        assert_eq!(中枢.基础序列.read().len(), 4);
        assert_eq!(Arc::as_ptr(&中枢.基础序列.read()[3]), Arc::as_ptr(&笔4));
    }

    #[test]
    fn test_中枢添加虚线后清除第三买卖线() {
        let 笔1 = 辅助_创建笔(100, 50.0, 40.0, 200, 30.0, 20.0);
        let 笔2 = 辅助_创建笔(200, 30.0, 20.0, 300, 55.0, 45.0);
        let 笔3 = 辅助_创建笔(300, 55.0, 45.0, 400, 25.0, 15.0);
        let 笔4 = 辅助_创建笔(400, 25.0, 15.0, 500, 60.0, 50.0);

        let 中枢 = 中枢::new(
            0,
            "测试".into(),
            1,
            vec![Arc::clone(&笔1), Arc::clone(&笔2), Arc::clone(&笔3)],
        );
        中枢.设置第三买卖线(Some(Arc::clone(&笔1)));
        *中枢.本级_第三买卖线.write() = Some(Arc::clone(&笔2));
        assert!(中枢.第三买卖线.read().is_some());
        assert!(中枢.本级_第三买卖线.read().is_some());

        中枢._添加虚线(Arc::clone(&笔4));
        // 添加虚线后第三买卖线被清除
        assert!(中枢.第三买卖线.read().is_none());
        assert!(中枢.本级_第三买卖线.read().is_none());
    }

    // ============================================================
    // Clone 后 Rc 指针身份一致
    // ============================================================

    #[test]
    fn test_中枢Clone后基础序列Rc指针一致() {
        let 笔1 = 辅助_创建笔(100, 50.0, 40.0, 200, 30.0, 20.0);
        let 笔2 = 辅助_创建笔(200, 30.0, 20.0, 300, 55.0, 45.0);
        let 笔3 = 辅助_创建笔(300, 55.0, 45.0, 400, 25.0, 15.0);

        let 中枢 = 中枢::new(
            0,
            "测试".into(),
            1,
            vec![Arc::clone(&笔1), Arc::clone(&笔2), Arc::clone(&笔3)],
        );
        中枢.设置第三买卖线(Some(Arc::clone(&笔1)));

        let 克隆 = 中枢.clone();

        // 基础序列中的 Rc 指针应一致
        for i in 0..3 {
            assert_eq!(
                Arc::as_ptr(&中枢.基础序列.read()[i]),
                Arc::as_ptr(&克隆.基础序列.read()[i])
            );
        }

        // 第三买卖线 Rc 指针应一致
        assert_eq!(
            Arc::as_ptr(中枢.第三买卖线.read().as_ref().unwrap()),
            Arc::as_ptr(克隆.第三买卖线.read().as_ref().unwrap())
        );
    }

    // ============================================================
    // 中枢 高/低/高高/低低计算
    // ============================================================

    #[test]
    fn test_中枢高低计算正确() {
        // 笔1: 顶(高=50,低=45) →底(高=40,低=30) = 向下笔, 高=50, 低=30
        let 笔1 = 辅助_创建笔(100, 50.0, 45.0, 200, 40.0, 30.0);

        // 笔2: 底(高=40,低=30) →顶(高=55,低=50) = 向上笔, 高=55, 低=30
        let 底2 = 辅助_创建底分型(200, 40.0, 30.0, 10);
        let 顶2 = 辅助_创建顶分型(300, 55.0, 50.0, 15);
        let 笔2 = Arc::new(虚线::创建笔(底2, 顶2, true));

        // 笔3: 顶(高=55,低=50) →底(高=35,低=25) = 向下笔, 高=55, 低=25
        let 笔3 = 辅助_创建笔(300, 55.0, 50.0, 400, 35.0, 25.0);

        let 中枢 = 中枢::new(
            0,
            "测试".into(),
            1,
            vec![Arc::clone(&笔1), Arc::clone(&笔2), Arc::clone(&笔3)],
        );

        // 高 = min(笔1高, 笔2高, 笔3高) = min(50, 55, 55) = 50
        assert!((中枢.高() - 50.0).abs() < 0.01, "中枢高={}", 中枢.高());
        // 低 = max(笔1低, 笔2低, 笔3低) = max(30, 30, 25) = 30
        assert!((中枢.低() - 30.0).abs() < 0.01, "中枢低={}", 中枢.低());
    }

    // ============================================================
    // 多 Rc 共享下修改可见性
    // ============================================================

    #[test]
    fn test_多Rc共享中枢修改对所有引用可见() {
        let 笔1 = 辅助_创建笔(100, 50.0, 40.0, 200, 30.0, 20.0);
        let 笔2 = 辅助_创建笔(200, 30.0, 20.0, 300, 55.0, 45.0);
        let 笔3 = 辅助_创建笔(300, 55.0, 45.0, 400, 25.0, 15.0);
        let 笔4 = 辅助_创建笔(400, 25.0, 15.0, 500, 60.0, 50.0);

        let 中枢1 = Arc::new(中枢::new(
            0,
            "测试".into(),
            1,
            vec![Arc::clone(&笔1), Arc::clone(&笔2), Arc::clone(&笔3)],
        ));
        let 中枢2 = Arc::clone(&中枢1);

        // 通过 rc1 修改序号
        中枢1.序号.store(88, Ordering::Relaxed);
        assert_eq!(中枢2.序号.load(Ordering::Relaxed), 88);

        // 通过 rc1 添加虚线
        中枢1._添加虚线(Arc::clone(&笔4));
        assert_eq!(中枢2.基础序列.read().len(), 4);

        // 验证共享的 Arc<虚线> 指针一致
        assert_eq!(
            Arc::as_ptr(&中枢1.基础序列.read()[3]),
            Arc::as_ptr(&中枢2.基础序列.read()[3])
        );
    }
}
