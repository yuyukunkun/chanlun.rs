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

use crate::algorithm::hub::中枢;
use crate::config::缠论配置;
use crate::kline::bar::K线;
use crate::kline::chan_kline::缠论K线;
use crate::structure::fractal_obj::分型;
use crate::structure::segment_feat::线段特征;
use crate::types::{分型结构, 相对方向, 缺口};
use std::rc::Rc;

/// 虚线 — 笔和线段的通用数据结构
///
/// 笔和线段共享此 struct，通过 `标识` 字段区分 ("笔"/"线段"/"扩展线段"等)
#[derive(Debug, Clone)]
pub struct 虚线 {
    pub 标识: String,
    pub 序号: i64,
    pub 级别: i64,
    pub 文: Rc<分型>,
    pub 武: Rc<分型>,
    pub 有效性: bool,
    pub 基础序列: Vec<Rc<虚线>>,
    pub 特征序列: Vec<Option<Rc<线段特征>>>,
    pub 实_中枢序列: Vec<Rc<中枢>>,
    pub 虚_中枢序列: Vec<Rc<中枢>>,
    pub 合_中枢序列: Vec<Rc<中枢>>,
    pub 确认K线: Option<Rc<缠论K线>>,
    pub 模式: String,
    pub _特征序列_显示: bool,
    pub 前一缺口: Option<缺口>,
    pub 前一结束位置: Option<Rc<虚线>>,
    pub 短路修正: bool,
}

/// MACD行为统计 — 统计MACD行为 方法的返回类型
#[derive(Debug, Clone)]
pub struct MACD行为统计 {
    pub DIF上穿0: i64,
    pub DIF下穿0: i64,
    pub DEA上穿0: i64,
    pub DEA下穿0: i64,
    pub 金叉次数: i64,
    pub 死叉次数: i64,
    pub 密集交叉区域: Vec<(usize, usize, usize)>,
}

impl 虚线 {
    pub fn new(
        序号: i64,
        标识: String,
        文: Rc<分型>,
        武: Rc<分型>,
        级别: i64,
        有效性: bool,
    ) -> Self {
        Self {
            序号,
            标识,
            级别,
            文,
            武,
            有效性,
            基础序列: Vec::new(),
            特征序列: Vec::new(),
            实_中枢序列: Vec::new(),
            虚_中枢序列: Vec::new(),
            合_中枢序列: Vec::new(),
            确认K线: None,
            模式: "文武".into(),
            _特征序列_显示: false,
            前一缺口: None,
            前一结束位置: None,
            短路修正: false,
        }
    }

    /// 笔序列（基础序列的别名）
    pub fn 笔序列(&self) -> &Vec<Rc<虚线>> {
        &self.基础序列
    }

    pub fn 图表标题(&self) -> String {
        format!(
            "{}:{}:{}:{}",
            self.文.中.标识, self.文.中.周期, self.标识, self.序号
        )
    }

    /// 方向 — 文到武的方向
    pub fn 方向(&self) -> 相对方向 {
        match (self.文.结构, self.武.结构) {
            (分型结构::顶, 分型结构::底) => 相对方向::向下,
            (分型结构::顶, 分型结构::下) => 相对方向::向下,
            (分型结构::上, 分型结构::底) => 相对方向::向下,
            (分型结构::上, 分型结构::下) => 相对方向::向下,
            _ => 相对方向::向上,
        }
    }

    /// 虚线高
    pub fn 高(&self) -> f64 {
        if self.方向() == 相对方向::向下 {
            self.文.中.高
        } else {
            self.武.中.高
        }
    }

    /// 虚线低
    pub fn 低(&self) -> f64 {
        if self.方向() == 相对方向::向下 {
            self.武.中.低
        } else {
            self.文.中.低
        }
    }

    /// 判断两个虚线是否首尾相连
    pub fn 之前是(&self, 之前: &虚线) -> bool {
        if self.标识 != 之前.标识 {
            return false;
        }
        Rc::as_ptr(&之前.武) == Rc::as_ptr(&self.文)
    }

    /// 判断两个虚线是否首尾相连
    pub fn 之后是(&self, 之后: &虚线) -> bool {
        if self.标识 != 之后.标识 {
            return false;
        }
        Rc::as_ptr(&self.武) == Rc::as_ptr(&之后.文)
    }

    /// 获取该虚线范围内的普K序列
    pub fn 获取普K序列(&self, 普K序列: &[Rc<K线>]) -> Vec<Rc<K线>> {
        // 使用指针查找（与 Python list.index 身份匹配行为一致），
        // 而非序号切片——因为序号可能与实际位置不一致。
        let 始 = 普K序列
            .iter()
            .position(|k| Rc::as_ptr(k) == Rc::as_ptr(&self.文.中.标的K线));
        let 终 = 普K序列
            .iter()
            .position(|k| Rc::as_ptr(k) == Rc::as_ptr(&self.武.中.标的K线));
        match (始, 终) {
            (Some(s), Some(e)) if s <= e => 普K序列[s..=e].to_vec(),
            _ => {
                // 指针查找失败时回退到序号方式
                println!("[警告]虚线.获取普K序列 <指针查找失败时回退到序号方式>");
                let 始 = self.文.中.原始起始序号 as usize;
                let 终 = self.武.中.原始结束序号 as usize;
                if 始 < 普K序列.len() && 终 < 普K序列.len() && 始 <= 终 {
                    普K序列[始..=终].to_vec()
                } else {
                    Vec::new()
                }
            }
        }
    }

    /// 获取该虚线范围内的缠K序列
    pub fn 获取缠K序列(&self, 缠K序列: &[Rc<缠论K线>]) -> Vec<Rc<缠论K线>> {
        缠论K线::截取(缠K序列, &self.文.中, &self.武.中).unwrap_or_default()
    }

    /// 获取_武 — 递归获取虚线的终点分型（笔直接返回武，线段递归到底层笔的武）
    pub fn 获取_武(&self) -> Rc<分型> {
        if self.标识 == "笔" {
            return Rc::clone(&self.武);
        }
        let mut current: &虚线 = self;
        while current.标识 != "笔" {
            current = &*current.基础序列.last().unwrap();
        }
        Rc::clone(&current.武)
    }

    /// 获取数据文本（用于保存/调试）
    pub fn 获取数据文本(&self) -> String {
        use crate::utils::format_f64_g;
        if self.标识 == "笔" {
            return format!(
                "{}, {}, {}, 文:({},{}), 武:({},{}), {}",
                self.标识,
                self.序号,
                self.级别,
                self.文.时间戳,
                format_f64_g(self.文.分型特征值),
                self.武.时间戳,
                format_f64_g(self.武.分型特征值),
                if self.有效性 { "True" } else { "False" },
            );
        }

        // 非笔：线段/扩展线段等，完整输出
        let (前, 后, 三, 贯穿伤) = crate::algorithm::segment::线段::分割序列(self, None);
        let (特征_a, 特征_b, 特征_c) = crate::algorithm::segment::线段::特征序列状态(self);
        let 特征_bool = |b: bool| -> &str {
            if b {
                "True"
            } else {
                "False"
            }
        };

        let 前一缺口_str = match &self.前一缺口 {
            Some(g) => format!("{}", g),
            None => "None".to_string(),
        };
        let 前一结束位置_str = match &self.前一结束位置 {
            Some(d) => format!("{}", d),
            None => "None".to_string(),
        };

        // Format中枢序列 as Python-style list representations
        let 实_str = format!(
            "[{}]",
            self.实_中枢序列
                .iter()
                .map(|h| format!("{}", h))
                .collect::<Vec<_>>()
                .join(", ")
        );
        let 虚_str = format!(
            "[{}]",
            self.虚_中枢序列
                .iter()
                .map(|h| format!("{}", h))
                .collect::<Vec<_>>()
                .join(", ")
        );
        let 合_str = format!(
            "[{}]",
            self.合_中枢序列
                .iter()
                .map(|h| format!("{}", h))
                .collect::<Vec<_>>()
                .join(", ")
        );

        let 前_str = format!(
            "[{}]",
            前.iter()
                .map(|d| format!("{}", d))
                .collect::<Vec<_>>()
                .join(", ")
        );
        let 后_str = format!(
            "[{}]",
            后.iter()
                .map(|d| format!("{}", d))
                .collect::<Vec<_>>()
                .join(", ")
        );
        let 三_str = format!(
            "[{}]",
            三.iter()
                .map(|d| format!("{}", d))
                .collect::<Vec<_>>()
                .join(", ")
        );

        format!(
            "{}, {}, {}, 文:({},{}), 武:({},{}), {}, {}, ({}, {}, {}), (前: {}, 后: {}, 三: {}, 伤: {}), 实: {}, 虚: {}, 合: {}, {}, {}, {}, {}",
            self.标识,
            self.序号,
            self.级别,
            self.文.时间戳,
            format_f64_g(self.文.分型特征值),
            self.武.时间戳,
            format_f64_g(self.武.分型特征值),
            if self.有效性 { "True" } else { "False" },
            self.基础序列.len(),
            特征_bool(特征_a),
            特征_bool(特征_b),
            特征_bool(特征_c),
            前_str,
            后_str,
            三_str,
            match &贯穿伤 { Some(d) => format!("{}", d), None => "None".to_string() },
            实_str,
            虚_str,
            合_str,
            self.模式,
            前一缺口_str,
            前一结束位置_str,
            if self.短路修正 { "True" } else { "False" },
        )
    }

    // ---- 关联函数（静态工厂方法） ----

    /// 创建笔
    pub fn 创建笔(文: Rc<分型>, 武: Rc<分型>, 有效性: bool) -> Self {
        Self::new(0, "笔".into(), 文, 武, 1, 有效性)
    }

    /// 创建线段
    pub fn 创建线段(虚线序列: &[Rc<虚线>]) -> Self {
        let 文 = Rc::clone(&虚线序列[0].文);
        let 武 = Rc::clone(&虚线序列[虚线序列.len() - 1].武);
        let 标识 = if 虚线序列[0].标识 == "笔" {
            "线段".into()
        } else {
            format!("线段<{}>", 虚线序列[0].标识)
        };
        let 级别 = 虚线序列[0].级别 + 1;
        let mut 段 = Self::new(0, 标识, 文, 武, 级别, true);
        段.基础序列 = 虚线序列.to_vec();
        段.模式 = "文武".into();
        段
    }

    // ---- 买卖点模式匹配 ----

    /// 缠K买卖点模式 — 根据模式字符串选择匹配方法
    pub fn 缠K买卖点模式(模式: &str, 缠K: &缠论K线, 配置: &缠论配置) -> bool {
        match 模式 {
            "全量" => Self::买卖点全量匹配(缠K),
            "任意" => Self::买卖点任意匹配(缠K),
            "配置" => Self::买卖点配置匹配(缠K, 配置),
            "相对" => Self::买卖点相对匹配(缠K),
            _ => false,
        }
    }

    /// 买卖点配置匹配 — 根据配置中的指标开关组合判断
    pub fn 买卖点配置匹配(缠K: &缠论K线, 配置: &缠论配置) -> bool {
        match (
            配置.买卖点_指标匹配_MACD,
            配置.买卖点_指标匹配_KDJ,
            配置.买卖点_指标匹配_RSI,
        ) {
            (true, true, true) => 缠K.与MACD柱子匹配() && 缠K.与KDJ匹配() && 缠K.与RSI匹配(),
            (false, false, false) => false,
            (true, false, true) => 缠K.与MACD柱子匹配() && 缠K.与RSI匹配(),
            (false, true, false) => 缠K.与KDJ匹配(),
            (true, false, false) => 缠K.与MACD柱子匹配(),
            (false, true, true) => 缠K.与KDJ匹配() && 缠K.与RSI匹配(),
            (false, false, true) => 缠K.与RSI匹配(),
            (true, true, false) => 缠K.与MACD柱子匹配() && 缠K.与KDJ匹配(),
        }
    }

    /// 买卖点任意匹配 — 任一指标匹配
    pub fn 买卖点任意匹配(缠K: &缠论K线) -> bool {
        缠K.与MACD柱子匹配() || 缠K.与KDJ匹配() || 缠K.与RSI匹配()
    }

    /// 买卖点全量匹配 — 全部指标匹配
    pub fn 买卖点全量匹配(缠K: &缠论K线) -> bool {
        缠K.与MACD柱子匹配() && 缠K.与KDJ匹配() && 缠K.与RSI匹配()
    }

    /// 买卖点相对匹配 — 至少两个指标匹配
    pub fn 买卖点相对匹配(缠K: &缠论K线) -> bool {
        let 混沌槽 = [缠K.与MACD柱子匹配(), 缠K.与KDJ匹配(), 缠K.与RSI匹配()];
        混沌槽.iter().filter(|&&x| x).count() >= 2
    }

    // ---- MACD柱子均值计算 ----

    /// 计算MACD柱子均值 — 虚线范围内所有MACD柱的绝对值均值
    pub fn 计算MACD柱子均值(普K序列: &[Rc<K线>], 实线: &虚线) -> f64 {
        let K线序列 = K线::截取rc(普K序列, &实线.文.中.标的K线, &实线.武.中.标的K线);
        if K线序列.is_empty() {
            return 0.0;
        }
        let 总: f64 = K线序列
            .iter()
            .filter_map(|k| k.macd.as_ref())
            .map(|m| m.MACD柱.abs())
            .sum();
        总 / K线序列.len() as f64
    }

    /// 计算MACD柱子均值_阴 — 负柱的绝对值均值
    pub fn 计算MACD柱子均值_阴(普K序列: &[Rc<K线>], 实线: &虚线) -> Option<f64> {
        let K线序列 = K线::截取rc(普K序列, &实线.文.中.标的K线, &实线.武.中.标的K线);
        let 总: Vec<f64> = K线序列
            .iter()
            .filter_map(|k| k.macd.as_ref())
            .filter(|m| m.MACD柱 < 0.0)
            .map(|m| m.MACD柱.abs())
            .collect();
        if 总.is_empty() {
            None
        } else {
            Some(总.iter().sum::<f64>() / 总.len() as f64)
        }
    }

    /// 计算MACD柱子均值_阳 — 正柱的绝对值均值
    pub fn 计算MACD柱子均值_阳(普K序列: &[Rc<K线>], 实线: &虚线) -> Option<f64> {
        let K线序列 = K线::截取rc(普K序列, &实线.文.中.标的K线, &实线.武.中.标的K线);
        let 总: Vec<f64> = K线序列
            .iter()
            .filter_map(|k| k.macd.as_ref())
            .filter(|m| m.MACD柱 > 0.0)
            .map(|m| m.MACD柱.abs())
            .collect();
        if 总.is_empty() {
            None
        } else {
            Some(总.iter().sum::<f64>() / 总.len() as f64)
        }
    }

    // ---- 武之MACD比较 ----

    /// 武之全量MACD均值 — 武端MACD柱是否小于均值（背驰）
    pub fn 武之全量MACD均值(普K序列: &[Rc<K线>], 实线: &虚线) -> bool {
        let 武_MACD = match &实线.武.中.标的K线.macd {
            Some(m) => m.MACD柱.abs(),
            None => return false,
        };
        武_MACD < Self::计算MACD柱子均值(普K序列, 实线)
    }

    /// 武之MACD均值 — 按方向选择阴/阳均值比对
    pub fn 武之MACD均值(普K序列: &[Rc<K线>], 实线: &虚线) -> bool {
        if 实线.方向() == 相对方向::向上 {
            Self::武之MACD均值_阳(普K序列, 实线)
        } else {
            Self::武之MACD均值_阴(普K序列, 实线)
        }
    }

    /// 武之MACD均值_阴 — 武端负柱是否小于阴均值
    pub fn 武之MACD均值_阴(普K序列: &[Rc<K线>], 实线: &虚线) -> bool {
        let 武_MACD = match &实线.武.中.标的K线.macd {
            Some(m) => m.MACD柱.abs(),
            None => return false,
        };
        match Self::计算MACD柱子均值_阴(普K序列, 实线) {
            Some(均值) => 武_MACD < 均值.abs(),
            None => false,
        }
    }

    /// 武之MACD均值_阳 — 武端正柱是否小于阳均值
    pub fn 武之MACD均值_阳(普K序列: &[Rc<K线>], 实线: &虚线) -> bool {
        let 武_MACD = match &实线.武.中.标的K线.macd {
            Some(m) => m.MACD柱.abs(),
            None => return false,
        };
        match Self::计算MACD柱子均值_阳(普K序列, 实线) {
            Some(均值) => 武_MACD < 均值,
            None => false,
        }
    }

    /// 武之MACD极值 — 武端MACD柱是否为区间极值
    pub fn 武之MACD极值(普K序列: &[Rc<K线>], 实线: &虚线) -> bool {
        let 武_MACD = match &实线.武.中.标的K线.macd {
            Some(m) => m.MACD柱,
            None => return false,
        };
        let K线序列 = K线::截取rc(普K序列, &实线.文.中.标的K线, &实线.武.中.标的K线);
        let 所有柱子: Vec<f64> = K线序列
            .iter()
            .filter_map(|k| k.macd.as_ref())
            .map(|m| m.MACD柱)
            .collect();
        if 所有柱子.is_empty() {
            return false;
        }
        if 武_MACD > 0.0 {
            let 极值 = 所有柱子.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            极值 == 武_MACD
        } else {
            let 极值 = 所有柱子.iter().cloned().fold(f64::INFINITY, f64::min);
            极值 == 武_MACD
        }
    }

    // ---- MACD趋向背驰 ----

    /// 计算K线序列MACD趋向背驰 — 分析 MACD柱/DIF/DEA 三项背驰信号
    pub fn 计算K线序列MACD趋向背驰(
        普K序列: &[Rc<K线>], 方向: 相对方向
    ) -> [bool; 3] {
        if 普K序列.is_empty() {
            return [false, false, false];
        }
        let 最后 = &普K序列[普K序列.len() - 1];

        if 方向 == 相对方向::向上 {
            let 柱子序列: Vec<&Rc<K线>> = 普K序列
                .iter()
                .filter(|k| k.macd.as_ref().map_or(false, |m| m.MACD柱 > 0.0))
                .collect();
            if 柱子序列.is_empty() {
                return [false, false, false];
            }

            let mut 结果 = [false; 3];

            // MACD柱背驰
            let 最高柱子 = 柱子序列
                .iter()
                .max_by(|a, b| {
                    a.macd
                        .as_ref()
                        .unwrap()
                        .MACD柱
                        .partial_cmp(&b.macd.as_ref().unwrap().MACD柱)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .unwrap();
            let mut 柱对 = vec![Rc::clone(*最高柱子), Rc::clone(最后)];
            柱对.sort_by_key(|k| k.时间戳);
            if let (Some(m0), Some(m1)) = (柱对[0].macd.as_ref(), 柱对[1].macd.as_ref()) {
                if m0.MACD柱 > m1.MACD柱 && 柱对[0].高 < 柱对[1].高 {
                    结果[0] = true;
                }
            }

            // DIF背驰 (no sort — Python compares peak vs last directly)
            let 最高离差值 = 柱子序列
                .iter()
                .max_by(|a, b| {
                    let da = a.macd.as_ref().and_then(|m| m.DIF).unwrap_or(0.0);
                    let db = b.macd.as_ref().and_then(|m| m.DIF).unwrap_or(0.0);
                    da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
                })
                .unwrap();
            if let (Some(m0), Some(m1)) = (最高离差值.macd.as_ref(), 最后.macd.as_ref()) {
                let dif0 = m0.DIF.unwrap_or(0.0);
                let dif1 = m1.DIF.unwrap_or(0.0);
                if dif0 > dif1 && 最高离差值.高 < 最后.高 {
                    结果[1] = true;
                }
            }

            // DEA背驰 (no sort — Python compares peak vs last directly)
            let 最高信号线 = 柱子序列
                .iter()
                .max_by(|a, b| {
                    let da = a.macd.as_ref().and_then(|m| m.DEA).unwrap_or(0.0);
                    let db = b.macd.as_ref().and_then(|m| m.DEA).unwrap_or(0.0);
                    da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
                })
                .unwrap();
            if let (Some(m0), Some(m1)) = (最高信号线.macd.as_ref(), 最后.macd.as_ref()) {
                let dea0 = m0.DEA.unwrap_or(0.0);
                let dea1 = m1.DEA.unwrap_or(0.0);
                if dea0 > dea1 && 最高信号线.高 < 最后.高 {
                    结果[2] = true;
                }
            }

            结果
        } else {
            let 柱子序列: Vec<&Rc<K线>> = 普K序列
                .iter()
                .filter(|k| k.macd.as_ref().map_or(false, |m| m.MACD柱 < 0.0))
                .collect();
            if 柱子序列.is_empty() {
                return [false, false, false];
            }

            let mut 结果 = [false; 3];

            // MACD柱背驰 (负向: absolute value comparison)
            let 最高柱子 = 柱子序列
                .iter()
                .max_by(|a, b| {
                    a.macd
                        .as_ref()
                        .unwrap()
                        .MACD柱
                        .abs()
                        .partial_cmp(&b.macd.as_ref().unwrap().MACD柱.abs())
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .unwrap();
            let mut 柱对 = vec![Rc::clone(*最高柱子), Rc::clone(最后)];
            柱对.sort_by_key(|k| k.时间戳);
            if let (Some(m0), Some(m1)) = (柱对[0].macd.as_ref(), 柱对[1].macd.as_ref()) {
                if m0.MACD柱 < m1.MACD柱 && 柱对[0].低 > 柱对[1].低 {
                    结果[0] = true;
                }
            }

            // DIF背驰 (no sort — Python compares peak vs last directly)
            let 最高离差值 = 柱子序列
                .iter()
                .max_by(|a, b| {
                    let da = a.macd.as_ref().and_then(|m| m.DIF).unwrap_or(0.0).abs();
                    let db = b.macd.as_ref().and_then(|m| m.DIF).unwrap_or(0.0).abs();
                    da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
                })
                .unwrap();
            if let (Some(m0), Some(m1)) = (最高离差值.macd.as_ref(), 最后.macd.as_ref()) {
                let dif0 = m0.DIF.unwrap_or(0.0);
                let dif1 = m1.DIF.unwrap_or(0.0);
                if dif0 < dif1 && 最高离差值.低 > 最后.低 {
                    结果[1] = true;
                }
            }

            // DEA背驰 (no sort — Python compares peak vs last directly)
            let 最高信号线 = 柱子序列
                .iter()
                .max_by(|a, b| {
                    let da = a.macd.as_ref().and_then(|m| m.DEA).unwrap_or(0.0).abs();
                    let db = b.macd.as_ref().and_then(|m| m.DEA).unwrap_or(0.0).abs();
                    da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
                })
                .unwrap();
            if let (Some(m0), Some(m1)) = (最高信号线.macd.as_ref(), 最后.macd.as_ref()) {
                let dea0 = m0.DEA.unwrap_or(0.0);
                let dea1 = m1.DEA.unwrap_or(0.0);
                if dea0 < dea1 && 最高信号线.低 > 最后.低 {
                    结果[2] = true;
                }
            }

            结果
        }
    }

    // ---- MACD柱子分段 ----

    /// 计算MACD柱子分段 — 按正负号将MACD柱子分段
    pub fn 计算MACD柱子分段(k线序列: &[Rc<K线>]) -> Vec<Vec<f64>> {
        if k线序列.is_empty() {
            return Vec::new();
        }

        let 符号 = |x: f64| -> &str {
            if x > 0.0 {
                "正"
            } else {
                "负"
            }
        };

        let 首_MACD = match &k线序列[0].macd {
            Some(m) => m.MACD柱,
            None => return Vec::new(),
        };
        let mut 当前符号 = 符号(首_MACD);
        let mut 当前段 = vec![首_MACD];
        let mut 结果 = Vec::new();

        for k线 in &k线序列[1..] {
            let macd = match &k线.macd {
                Some(m) => m.MACD柱,
                None => continue,
            };
            let 新符号 = 符号(macd);
            if 新符号 == 当前符号 {
                当前段.push(macd);
            } else {
                结果.push(std::mem::take(&mut 当前段));
                当前段.push(macd);
                当前符号 = 新符号;
            }
        }
        if !当前段.is_empty() {
            结果.push(当前段);
        }
        结果
    }

    // ---- 密集区域按间隔 ----

    /// 密集区域按间隔 — 找出交叉标记中的密集区域
    pub fn 密集区域按间隔(
        交叉标记: &[i32],
        最大间隔: usize,
        最少交叉数: usize,
    ) -> Vec<(usize, usize, usize)> {
        let 交叉索引: Vec<usize> = (0..交叉标记.len()).filter(|&i| 交叉标记[i] != 0).collect();
        if 交叉索引.is_empty() {
            return Vec::new();
        }

        let mut 密集区 = Vec::new();
        let mut 当前块起始 = 交叉索引[0];
        let mut 当前块交叉数 = 1;

        for i in 1..交叉索引.len() {
            let prev = 交叉索引[i - 1];
            let curr = 交叉索引[i];
            if curr - prev <= 最大间隔 {
                当前块交叉数 += 1;
            } else {
                if 当前块交叉数 >= 最少交叉数 {
                    密集区.push((当前块起始, prev, 当前块交叉数));
                }
                当前块起始 = curr;
                当前块交叉数 = 1;
            }
        }

        if 当前块交叉数 >= 最少交叉数 {
            密集区.push((当前块起始, 交叉索引[交叉索引.len() - 1], 当前块交叉数));
        }

        密集区
    }

    // ---- 统计MACD行为 ----

    /// 统计MACD行为 — 分析DIF/DEA穿零轴和金叉死叉
    pub fn 统计MACD行为(
        普K序列: &[Rc<K线>],
        最大间隔: usize,
        最少交叉数: usize,
    ) -> MACD行为统计 {
        let mut dif_up = 0;
        let mut dif_down = 0;
        let mut dea_up = 0;
        let mut dea_down = 0;

        for i in 1..普K序列.len() {
            let pre = &普K序列[i - 1].macd;
            let cur = &普K序列[i].macd;
            if pre.is_none() || cur.is_none() {
                continue;
            }
            let (pre_dif, cur_dif) = (pre.as_ref().unwrap().DIF, cur.as_ref().unwrap().DIF);
            let (pre_dea, cur_dea) = (pre.as_ref().unwrap().DEA, cur.as_ref().unwrap().DEA);

            if let (Some(pd), Some(cd)) = (pre_dif, cur_dif) {
                if pd < 0.0 && cd >= 0.0 {
                    dif_up += 1;
                }
                if pd > 0.0 && cd <= 0.0 {
                    dif_down += 1;
                }
            }
            if let (Some(pd), Some(cd)) = (pre_dea, cur_dea) {
                if pd < 0.0 && cd >= 0.0 {
                    dea_up += 1;
                }
                if pd > 0.0 && cd <= 0.0 {
                    dea_down += 1;
                }
            }
        }

        let mut golden = 0;
        let mut death = 0;
        let mut 交叉标记 = vec![0i32];

        for i in 1..普K序列.len() {
            let pre = &普K序列[i - 1].macd;
            let cur = &普K序列[i].macd;
            if pre.is_none() || cur.is_none() {
                交叉标记.push(0);
                continue;
            }
            let pre_dif = pre.as_ref().unwrap().DIF;
            let pre_dea = pre.as_ref().unwrap().DEA;
            let cur_dif = cur.as_ref().unwrap().DIF;
            let cur_dea = cur.as_ref().unwrap().DEA;

            if let (Some(pd), Some(cd), Some(pe), Some(ce)) = (pre_dif, cur_dif, pre_dea, cur_dea) {
                if pd <= pe && cd > ce {
                    golden += 1;
                    交叉标记.push(1);
                } else if pd >= pe && cd < ce {
                    death += 1;
                    交叉标记.push(-1);
                } else {
                    交叉标记.push(0);
                }
            } else {
                交叉标记.push(0);
            }
        }

        let 密集交叉区域 = Self::密集区域按间隔(&交叉标记, 最大间隔, 最少交叉数);

        MACD行为统计 {
            DIF上穿0: dif_up,
            DIF下穿0: dif_down,
            DEA上穿0: dea_up,
            DEA下穿0: dea_down,
            金叉次数: golden,
            死叉次数: death,
            密集交叉区域,
        }
    }

    // ---- 买卖意义 ----

    /// 买卖意义 — 核心买卖点判断逻辑
    ///
    /// 返回 (是否有意义, 原因字符串)
    pub fn 买卖意义(
        实线: &虚线, 观察员: &crate::business::observer::观察者
    ) -> (bool, String) {
        let 普K序列 = &观察员.普通K线序列;
        let 配置 = &观察员.配置;

        if 实线.标识 != "笔" && 实线.标识 != "线段" && !实线.标识.starts_with("线段<")
        {
            return (false, "标识不在范围内".into());
        }

        // KDJ指标完整性检查
        match &实线.武.中.标的K线.kdj {
            Some(kdj) if kdj.K.is_some() && kdj.D.is_some() && kdj.J.is_some() => {}
            _ => return (false, "KDJ指标不完整".into()),
        }

        let 意义 = Self::缠K买卖点模式(&配置.买卖点_指标模式, &实线.武.中, 配置);
        let 结果 = false;

        let 背驰过: Vec<Rc<缠论K线>> = if 实线.标识 == "笔" {
            crate::algorithm::bi::笔::是否背驰过(实线, 观察员)
        } else {
            crate::algorithm::segment::线段::是否背驰过(实线, 观察员)
        };

        if 意义 {
            if 实线.标识 == "笔" {
                if Self::武之MACD均值(普K序列, 实线) {
                    return (true, "武之MACD均值".into());
                }
                if Self::武之MACD极值(普K序列, 实线) && !背驰过.is_empty() {
                    return (true, "背驰过且极值".into());
                } else if 实线.武.与MACD柱子分型匹配() {
                    return (
                        true,
                        format!(
                            "背驰过:{},极值:{},柱子分型匹配",
                            背驰过.len(),
                            if Self::武之MACD极值(普K序列, 实线) {
                                "True"
                            } else {
                                "False"
                            }
                        ),
                    );
                }
            }
            if 实线.标识 != "笔"
                && crate::algorithm::segment::线段::判断线段内部是否背驰(实线, 观察员)
            {
                return (true, "线段内部背驰".into());
            }
        }

        if !结果 && 意义 && 实线.武.中.与MACD柱子匹配() {
            if Self::武之MACD极值(普K序列, 实线) && 背驰过.len() > 2 {
                return (true, "没结果, 极值, 柱子分型匹配, 背驰过大于2次".into());
            }
        }

        (结果, "".into())
    }
}

impl std::fmt::Display for 虚线 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.标识 == "笔" {
            write!(
                f,
                "笔({}, {}, {}, {}, 周期: {}, 数量: {})",
                self.序号,
                self.方向(),
                self.文,
                self.武,
                self.文.中.周期,
                self.武.中.序号 - self.文.中.序号 + 1
            )
        } else {
            let 四象 = crate::algorithm::segment::线段::四象(self);
            let 缺口 = crate::algorithm::segment::线段::获取缺口(self);
            let 缺口_str = match 缺口 {
                Some(g) => format!("{}", g),
                None => "None".to_string(),
            };
            let 确认K线_str = match &self.确认K线 {
                Some(k) => format!("{}", k),
                None => "None".to_string(),
            };
            write!(
                f,
                "{}<{}, {}, {}, {}, {}, 数量: {}, 缺口: {}, {}>",
                self.标识,
                self.序号,
                四象,
                self.方向(),
                self.文,
                self.武,
                self.基础序列.len(),
                缺口_str,
                确认K线_str,
            )
        }
    }
}
