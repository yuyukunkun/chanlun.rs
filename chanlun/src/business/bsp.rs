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
use crate::kline::chan_kline::缠论K线;
use crate::structure::fractal_obj::分型;
use crate::types::bsp_type::买卖点类型;
use crate::types::分型结构;
use std::sync::Arc;
use std::sync::atomic::Ordering;

/// 基础买卖点 — 买卖点的基础数据结构
///
/// 包含买卖点的完整信息：类型、关联分型/K线、失效与终结状态等。
#[derive(Debug, Clone)]
pub struct 基础买卖点 {
    /// 买卖点备注文本
    pub 备注: String,
    /// 买卖点类型（一买/一卖/二买/二卖/三买/三卖/T1/T2/T3 等）
    pub 类型: 买卖点类型,
    /// 买卖点对应的分型
    pub 买卖点分型: Arc<分型>,
    /// 买卖点对应的缠论K线（即分型的中缠K）
    pub 买卖点K线: Arc<缠论K线>,
    /// 当前K线（买卖点生成时的K线）
    pub 当前K线: Arc<K线>,
    /// 失效K线（买卖点失效时设置）
    pub 失效K线: Option<Arc<K线>>,
    /// 终结K线（买卖点终结时设置）
    pub 终结K线: Option<Arc<K线>>,
    /// 中枢破位值
    pub 破位值: f64,
    /// 分型结构（可选，用于补充确认）
    pub 结构: Option<分型结构>,
    /// 创建时的缠K序号，用于偏移计算（与买卖点K线.序号同尺度）
    /// None 时退化为使用当前K线.序号（bar序号，旧行为）
    pub 当前缠K序号: Option<i64>,
}

impl 基础买卖点 {
    /// 创建基础买卖点，买卖点K线自动取自买卖点分型的中缠K
    pub fn new(
        类型: 买卖点类型,
        当前K线: Arc<K线>,
        买卖点分型: Arc<分型>,
        备注: String,
        中枢破位值: f64,
    ) -> Self {
        let 买卖点K线 = Arc::clone(&买卖点分型.中);
        Self {
            备注,
            类型,
            买卖点分型,
            买卖点K线,
            当前K线,
            失效K线: None,
            终结K线: None,
            破位值: 中枢破位值,
            结构: None,
            当前缠K序号: None,
        }
    }

    /// 偏移 — 当前缠K序号与买卖点K线序号的差
    /// 如果设置了当前缠K序号（来自生成买卖点），使用缠K序号；否则退化为使用 bar序号
    pub fn 偏移(&self) -> i64 {
        match self.当前缠K序号 {
            Some(ck_idx) => ck_idx - self.买卖点K线.序号.load(Ordering::Relaxed),
            None => self.当前K线.序号 - self.买卖点K线.序号.load(Ordering::Relaxed),
        }
    }

    /// 失效偏移
    pub fn 失效偏移(&self) -> i64 {
        match &self.失效K线 {
            Some(k) => k.序号 - self.买卖点K线.序号.load(Ordering::Relaxed),
            None => -1,
        }
    }

    /// 有效性 — 失效K线是否存在
    pub fn 有效性(&self) -> bool {
        self.失效K线.is_some()
    }

    /// 与MACD柱子匹配
    pub fn 与MACD柱子匹配(&self) -> bool {
        self.买卖点K线.与MACD柱子匹配()
    }

    /// 与MACD柱子分型匹配
    pub fn 与MACD柱子分型匹配(&self) -> bool {
        self.买卖点分型.与MACD柱子分型匹配()
    }
}

impl std::fmt::Display for 基础买卖点 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}<{}, {}, {}>",
            self.类型,
            self.买卖点K线.as_ref(),
            self.偏移(),
            self.失效偏移(),
        )
    }
}

/// 买卖点 — 包含全部 18 种买卖点类型的工厂方法
pub struct 买卖点;

impl 买卖点 {
    /// 创建 一卖 类型的基础买卖点
    pub fn 一卖点(
        买卖点分型: Arc<分型>,
        当前K线: Arc<K线>,
        _标识: &str,
        备注: String,
        中枢破位值: f64,
    ) -> 基础买卖点 {
        基础买卖点::new(买卖点类型::一卖, 当前K线, 买卖点分型, 备注, 中枢破位值)
    }

    /// 创建 一买 类型的基础买卖点
    pub fn 一买点(
        买卖点分型: Arc<分型>,
        当前K线: Arc<K线>,
        _标识: &str,
        备注: String,
        中枢破位值: f64,
    ) -> 基础买卖点 {
        基础买卖点::new(买卖点类型::一买, 当前K线, 买卖点分型, 备注, 中枢破位值)
    }

    /// 创建 二卖 类型的基础买卖点
    pub fn 二卖点(
        买卖点分型: Arc<分型>,
        当前K线: Arc<K线>,
        _标识: &str,
        备注: String,
        中枢破位值: f64,
    ) -> 基础买卖点 {
        基础买卖点::new(买卖点类型::二卖, 当前K线, 买卖点分型, 备注, 中枢破位值)
    }

    /// 创建 二买 类型的基础买卖点
    pub fn 二买点(
        买卖点分型: Arc<分型>,
        当前K线: Arc<K线>,
        _标识: &str,
        备注: String,
        中枢破位值: f64,
    ) -> 基础买卖点 {
        基础买卖点::new(买卖点类型::二买, 当前K线, 买卖点分型, 备注, 中枢破位值)
    }

    /// 创建 三卖 类型的基础买卖点
    pub fn 三卖点(
        买卖点分型: Arc<分型>,
        当前K线: Arc<K线>,
        _标识: &str,
        备注: String,
        中枢破位值: f64,
    ) -> 基础买卖点 {
        基础买卖点::new(买卖点类型::三卖, 当前K线, 买卖点分型, 备注, 中枢破位值)
    }

    /// 创建 三买 类型的基础买卖点
    pub fn 三买点(
        买卖点分型: Arc<分型>,
        当前K线: Arc<K线>,
        _标识: &str,
        备注: String,
        中枢破位值: f64,
    ) -> 基础买卖点 {
        基础买卖点::new(买卖点类型::三买, 当前K线, 买卖点分型, 备注, 中枢破位值)
    }

    /// 创建 T1卖 类型的基础买卖点
    pub fn T1卖点(
        买卖点分型: Arc<分型>,
        当前K线: Arc<K线>,
        _标识: &str,
        备注: String,
        中枢破位值: f64,
    ) -> 基础买卖点 {
        基础买卖点::new(买卖点类型::T1卖, 当前K线, 买卖点分型, 备注, 中枢破位值)
    }

    /// 创建 T1买 类型的基础买卖点
    pub fn T1买点(
        买卖点分型: Arc<分型>,
        当前K线: Arc<K线>,
        _标识: &str,
        备注: String,
        中枢破位值: f64,
    ) -> 基础买卖点 {
        基础买卖点::new(买卖点类型::T1买, 当前K线, 买卖点分型, 备注, 中枢破位值)
    }

    /// 创建 T1P卖 类型的基础买卖点
    pub fn T1P卖点(
        买卖点分型: Arc<分型>,
        当前K线: Arc<K线>,
        _标识: &str,
        备注: String,
        中枢破位值: f64,
    ) -> 基础买卖点 {
        基础买卖点::new(买卖点类型::T1P卖, 当前K线, 买卖点分型, 备注, 中枢破位值)
    }

    /// 创建 T1P买 类型的基础买卖点
    pub fn T1P买点(
        买卖点分型: Arc<分型>,
        当前K线: Arc<K线>,
        _标识: &str,
        备注: String,
        中枢破位值: f64,
    ) -> 基础买卖点 {
        基础买卖点::new(买卖点类型::T1P买, 当前K线, 买卖点分型, 备注, 中枢破位值)
    }

    /// 创建 T2卖 类型的基础买卖点
    pub fn T2卖点(
        买卖点分型: Arc<分型>,
        当前K线: Arc<K线>,
        _标识: &str,
        备注: String,
        中枢破位值: f64,
    ) -> 基础买卖点 {
        基础买卖点::new(买卖点类型::T2卖, 当前K线, 买卖点分型, 备注, 中枢破位值)
    }

    /// 创建 T2买 类型的基础买卖点
    pub fn T2买点(
        买卖点分型: Arc<分型>,
        当前K线: Arc<K线>,
        _标识: &str,
        备注: String,
        中枢破位值: f64,
    ) -> 基础买卖点 {
        基础买卖点::new(买卖点类型::T2买, 当前K线, 买卖点分型, 备注, 中枢破位值)
    }

    /// 创建 T2S卖 类型的基础买卖点
    pub fn T2S卖点(
        买卖点分型: Arc<分型>,
        当前K线: Arc<K线>,
        _标识: &str,
        备注: String,
        中枢破位值: f64,
    ) -> 基础买卖点 {
        基础买卖点::new(买卖点类型::T2S卖, 当前K线, 买卖点分型, 备注, 中枢破位值)
    }

    /// 创建 T2S买 类型的基础买卖点
    pub fn T2S买点(
        买卖点分型: Arc<分型>,
        当前K线: Arc<K线>,
        _标识: &str,
        备注: String,
        中枢破位值: f64,
    ) -> 基础买卖点 {
        基础买卖点::new(买卖点类型::T2S买, 当前K线, 买卖点分型, 备注, 中枢破位值)
    }

    /// 创建 T3A卖 类型的基础买卖点
    pub fn T3A卖点(
        买卖点分型: Arc<分型>,
        当前K线: Arc<K线>,
        _标识: &str,
        备注: String,
        中枢破位值: f64,
    ) -> 基础买卖点 {
        基础买卖点::new(买卖点类型::T3A卖, 当前K线, 买卖点分型, 备注, 中枢破位值)
    }

    /// 创建 T3A买 类型的基础买卖点
    pub fn T3A买点(
        买卖点分型: Arc<分型>,
        当前K线: Arc<K线>,
        _标识: &str,
        备注: String,
        中枢破位值: f64,
    ) -> 基础买卖点 {
        基础买卖点::new(买卖点类型::T3A买, 当前K线, 买卖点分型, 备注, 中枢破位值)
    }

    /// 创建 T3B卖 类型的基础买卖点
    pub fn T3B卖点(
        买卖点分型: Arc<分型>,
        当前K线: Arc<K线>,
        _标识: &str,
        备注: String,
        中枢破位值: f64,
    ) -> 基础买卖点 {
        基础买卖点::new(买卖点类型::T3B卖, 当前K线, 买卖点分型, 备注, 中枢破位值)
    }

    /// 创建 T3B买 类型的基础买卖点
    pub fn T3B买点(
        买卖点分型: Arc<分型>,
        当前K线: Arc<K线>,
        _标识: &str,
        备注: String,
        中枢破位值: f64,
    ) -> 基础买卖点 {
        基础买卖点::new(买卖点类型::T3B买, 当前K线, 买卖点分型, 备注, 中枢破位值)
    }

    /// 生成买卖点 — 根据参数自动选择类型
    pub fn 生成买卖点(
        特征: &str,
        序号: &str,
        级别: &str,
        买卖点分型: Arc<分型>,
        当前缠K: Arc<缠论K线>,
    ) -> 基础买卖点 {
        let 买卖 = if matches!(买卖点分型.结构, 分型结构::底 | 分型结构::下) {
            "买"
        } else {
            "卖"
        };
        let 备注 = format!("{}_{}{}{}", 特征, 级别, 序号, 买卖);
        let 破位值 = 买卖点分型.分型特征值();

        // 当前K线 — 从缠K获取其标的K线
        let 当前K线 = Arc::clone(&*当前缠K.标的K线.read().unwrap());
        // 当前缠K序号 — 与买卖点K线（分型.中.序号）同尺度，用于偏移计算
        let 当前缠K序号 = 当前缠K.序号.load(Ordering::Relaxed);

        let 类型 = match (序号, 买卖) {
            ("一", "买") => 买卖点类型::一买,
            ("一", "卖") => 买卖点类型::一卖,
            ("二", "买") => 买卖点类型::二买,
            ("二", "卖") => 买卖点类型::二卖,
            ("三", "买") => 买卖点类型::三买,
            ("三", "卖") => 买卖点类型::三卖,
            ("T1", "买") => 买卖点类型::T1买,
            ("T1", "卖") => 买卖点类型::T1卖,
            ("T1P", "买") => 买卖点类型::T1P买,
            ("T1P", "卖") => 买卖点类型::T1P卖,
            ("T2", "买") => 买卖点类型::T2买,
            ("T2", "卖") => 买卖点类型::T2卖,
            ("T2S", "买") => 买卖点类型::T2S买,
            ("T2S", "卖") => 买卖点类型::T2S卖,
            ("T3A", "买") => 买卖点类型::T3A买,
            ("T3A", "卖") => 买卖点类型::T3A卖,
            ("T3B", "买") => 买卖点类型::T3B买,
            ("T3B", "卖") => 买卖点类型::T3B卖,
            _ => 买卖点类型::一买, // fallback
        };

        let mut bsp = 基础买卖点::new(类型, 当前K线, 买卖点分型, 备注, 破位值);
        bsp.当前缠K序号 = Some(当前缠K序号);
        bsp
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kline::bar::K线;
    use crate::kline::chan_kline::缠论K线;
    use crate::structure::fractal_obj::分型;
    use crate::types::分型结构;
    use crate::types::相对方向;
    use std::sync::Arc;
    use std::sync::atomic::Ordering;

    fn 辅助_创建普K(时间戳: i64, 序号: i64) -> Arc<K线> {
        Arc::new(K线 {
            时间戳,
            序号,
            高: 100.0,
            低: 90.0,
            开盘价: 95.0,
            收盘价: 95.0,
            ..Default::default()
        })
    }

    fn 辅助_创建缠K(
        序号: i64,
        时间戳: i64,
        高: f64,
        低: f64,
        分型: Option<分型结构>,
    ) -> Arc<缠论K线> {
        let 普K = 辅助_创建普K(时间戳, 0);
        Arc::new(缠论K线::创建缠K(
            时间戳,
            高,
            低,
            相对方向::向上,
            分型,
            序号,
            普K,
            None,
        ))
    }

    fn 辅助_创建底分型_中(序号: i64, 时间戳: i64) -> Arc<分型> {
        let 中 = 辅助_创建缠K(序号, 时间戳, 100.0, 90.0, Some(分型结构::底));
        中.序号.store(序号, Ordering::Relaxed);
        中.分型特征值.set(90.0);
        let 左 = 辅助_创建缠K(序号 - 1, 时间戳 - 100, 100.0, 92.0, Some(分型结构::下));
        左.序号.store(序号 - 1, Ordering::Relaxed);
        let 右 = 辅助_创建缠K(序号 + 1, 时间戳 + 100, 100.0, 92.0, Some(分型结构::上));
        右.序号.store(序号 + 1, Ordering::Relaxed);
        Arc::new(分型::new(Some(左), 中, Some(右)))
    }

    fn 辅助_创建顶分型_中(序号: i64, 时间戳: i64) -> Arc<分型> {
        let 中 = 辅助_创建缠K(序号, 时间戳, 100.0, 90.0, Some(分型结构::顶));
        中.序号.store(序号, Ordering::Relaxed);
        中.分型特征值.set(100.0);
        let 左 = 辅助_创建缠K(序号 - 1, 时间戳 - 100, 98.0, 88.0, Some(分型结构::上));
        左.序号.store(序号 - 1, Ordering::Relaxed);
        let 右 = 辅助_创建缠K(序号 + 1, 时间戳 + 100, 98.0, 88.0, Some(分型结构::下));
        右.序号.store(序号 + 1, Ordering::Relaxed);
        Arc::new(分型::new(Some(左), 中, Some(右)))
    }

    // ========== 基础买卖点 构造测试 ==========

    #[test]
    fn test_基础买卖点_new() {
        let 分型 = 辅助_创建底分型_中(10, 1000);
        let 当前K = 辅助_创建普K(1100, 0);
        let bsp = 基础买卖点::new(买卖点类型::一买, 当前K, 分型.clone(), "测试".into(), 90.0);
        assert_eq!(bsp.备注, "测试");
        assert_eq!(bsp.类型, 买卖点类型::一买);
        assert_eq!(bsp.破位值, 90.0);
        assert!(bsp.失效K线.is_none());
        assert!(!bsp.有效性());
    }

    // ========== 偏移 测试 ==========

    #[test]
    fn test_偏移_无缠K序号_退化为bar序号差值() {
        let 分型 = 辅助_创建底分型_中(10, 1000);
        let 当前K = 辅助_创建普K(1100, 15); // bar 序号=15
        let bsp = 基础买卖点::new(买卖点类型::一买, 当前K, 分型.clone(), "".into(), 0.0);
        // 无当前缠K序号: 偏移 = 15 - 10 = 5
        assert_eq!(bsp.偏移(), 5);
    }

    #[test]
    fn test_偏移_有缠K序号_使用缠K序号差值() {
        let 分型 = 辅助_创建底分型_中(10, 1000);
        let 当前K = 辅助_创建普K(1100, 100); // bar 序号(不使用)
        let mut bsp = 基础买卖点::new(买卖点类型::一买, 当前K, 分型.clone(), "".into(), 0.0);
        bsp.当前缠K序号 = Some(15);
        // 有当前缠K序号: 偏移 = 15 - 10 = 5
        assert_eq!(bsp.偏移(), 5);
    }

    // ========== 失效偏移 测试 ==========

    #[test]
    fn test_失效偏移_无失效K线返回负一() {
        let 分型 = 辅助_创建底分型_中(10, 1000);
        let 当前K = 辅助_创建普K(1100, 0);
        let bsp = 基础买卖点::new(买卖点类型::一买, 当前K, 分型.clone(), "".into(), 0.0);
        assert_eq!(bsp.失效偏移(), -1);
    }

    #[test]
    fn test_失效偏移_有失效K线() {
        let 分型 = 辅助_创建底分型_中(10, 1000);
        let 当前K = 辅助_创建普K(1100, 0);
        let mut bsp = 基础买卖点::new(买卖点类型::一买, 当前K, 分型.clone(), "".into(), 0.0);
        bsp.失效K线 = Some(辅助_创建普K(1200, 20)); // 序号=20
        // 失效偏移 = 20 - 10 = 10
        assert_eq!(bsp.失效偏移(), 10);
    }

    // ========== 有效性 测试 ==========

    #[test]
    fn test_有效性_无失效K线为false() {
        let 分型 = 辅助_创建底分型_中(10, 1000);
        let 当前K = 辅助_创建普K(1100, 0);
        let bsp = 基础买卖点::new(买卖点类型::一买, 当前K, 分型.clone(), "".into(), 0.0);
        assert!(!bsp.有效性());
    }

    #[test]
    fn test_有效性_有失效K线为true() {
        let 分型 = 辅助_创建底分型_中(10, 1000);
        let 当前K = 辅助_创建普K(1100, 0);
        let mut bsp = 基础买卖点::new(买卖点类型::一买, 当前K, 分型.clone(), "".into(), 0.0);
        bsp.失效K线 = Some(辅助_创建普K(1200, 0));
        assert!(bsp.有效性());
    }

    // ========== 生成买卖点 测试 ==========

    #[test]
    fn test_生成买卖点_一买() {
        let 分型 = 辅助_创建底分型_中(5, 1000);
        let 当前缠K = 分型.中.clone();
        let bsp = 买卖点::生成买卖点("特征A", "一", "本级", 分型.clone(), 当前缠K);
        assert_eq!(bsp.类型, 买卖点类型::一买);
        assert_eq!(bsp.当前缠K序号, Some(5));
    }

    #[test]
    fn test_生成买卖点_一卖() {
        let 分型 = 辅助_创建顶分型_中(5, 1000);
        let 当前缠K = 分型.中.clone();
        let bsp = 买卖点::生成买卖点("特征A", "一", "本级", 分型.clone(), 当前缠K);
        assert_eq!(bsp.类型, 买卖点类型::一卖);
    }

    #[test]
    fn test_生成买卖点_二买() {
        let 分型 = 辅助_创建底分型_中(5, 1000);
        let 当前缠K = 分型.中.clone();
        let bsp = 买卖点::生成买卖点("特征B", "二", "同级", 分型.clone(), 当前缠K);
        assert_eq!(bsp.类型, 买卖点类型::二买);
    }

    #[test]
    fn test_生成买卖点_三卖() {
        let 分型 = 辅助_创建顶分型_中(5, 1000);
        let 当前缠K = 分型.中.clone();
        let bsp = 买卖点::生成买卖点("特征C", "三", "本级", 分型.clone(), 当前缠K);
        assert_eq!(bsp.类型, 买卖点类型::三卖);
    }

    #[test]
    fn test_生成买卖点_T1买() {
        let 分型 = 辅助_创建底分型_中(5, 1000);
        let 当前缠K = 分型.中.clone();
        let bsp = 买卖点::生成买卖点("事后", "T1", "次级", 分型.clone(), 当前缠K);
        assert_eq!(bsp.类型, 买卖点类型::T1买);
    }

    #[test]
    fn test_生成买卖点_T2S卖() {
        let 分型 = 辅助_创建顶分型_中(5, 1000);
        let 当前缠K = 分型.中.clone();
        let bsp = 买卖点::生成买卖点("特征D", "T2S", "同级", 分型.clone(), 当前缠K);
        assert_eq!(bsp.类型, 买卖点类型::T2S卖);
    }

    #[test]
    fn test_生成买卖点_T3A买() {
        let 分型 = 辅助_创建底分型_中(5, 1000);
        let 当前缠K = 分型.中.clone();
        let bsp = 买卖点::生成买卖点("特征E", "T3A", "本级", 分型.clone(), 当前缠K);
        assert_eq!(bsp.类型, 买卖点类型::T3A买);
    }

    #[test]
    fn test_生成买卖点_T3B买() {
        let 分型 = 辅助_创建底分型_中(5, 1000);
        let 当前缠K = 分型.中.clone();
        let bsp = 买卖点::生成买卖点("特征F", "T3B", "本级", 分型.clone(), 当前缠K);
        assert_eq!(bsp.类型, 买卖点类型::T3B买);
    }

    #[test]
    fn test_生成买卖点_破位值来自分型特征值() {
        let 分型 = 辅助_创建底分型_中(5, 1000);
        let 当前缠K = 分型.中.clone();
        let bsp = 买卖点::生成买卖点("特征G", "一", "本级", 分型.clone(), 当前缠K);
        assert_eq!(bsp.破位值, 分型.分型特征值());
    }
}
