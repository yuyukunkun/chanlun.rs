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
use crate::structure::feat_fractal::特征分型;
use crate::structure::fractal_obj::分型;
use crate::types::{分型结构, 相对方向};
use std::sync::atomic::Ordering;
use std::sync::Arc;

/// 线段特征 — 特征序列元素（内部是虚线的集合）
#[derive(Debug, Clone)]
pub struct 线段特征 {
    pub 序号: i64,
    pub 标识: String,
    pub 线段方向: 相对方向,
    pub 元素: Vec<Arc<虚线>>,
}

impl 线段特征 {
    pub fn new(标识: String, 基础序列: Vec<Arc<虚线>>, 线段方向: 相对方向) -> Self {
        Self {
            序号: 0,
            标识,
            线段方向,
            元素: 基础序列,
        }
    }

    pub fn 图表标题(&self) -> String {
        self.标识.clone()
    }

    /// 文 — 取特征序列元素中分型特征值最大/最小的文分型
    /// tiebreaker: later时间戳 wins when特征值 equal (matches Python)
    pub fn 文(&self) -> Arc<分型> {
        if self.线段方向.是否向上() {
            self.元素
                .iter()
                .max_by(|a, b| {
                    a.文
                        .分型特征值
                        .partial_cmp(&b.文.分型特征值)
                        .unwrap_or(std::cmp::Ordering::Equal)
                        .then_with(|| a.文.时间戳.cmp(&b.文.时间戳))
                })
                .map(|x| Arc::clone(&x.文))
                .unwrap_or_else(|| Arc::clone(&self.元素[0].文))
        } else {
            self.元素
                .iter()
                .min_by(|a, b| {
                    a.文
                        .分型特征值
                        .partial_cmp(&b.文.分型特征值)
                        .unwrap_or(std::cmp::Ordering::Equal)
                        .then_with(|| b.文.时间戳.cmp(&a.文.时间戳))
                })
                .map(|x| Arc::clone(&x.文))
                .unwrap_or_else(|| Arc::clone(&self.元素[0].文))
        }
    }

    /// 武 — 取特征序列元素中分型特征值最大/最小的武分型
    /// tiebreaker: later时间戳 wins when特征值 equal (matches Python)
    pub fn 武(&self) -> Arc<分型> {
        if self.线段方向.是否向上() {
            self.元素
                .iter()
                .max_by(|a, b| {
                    a.武
                        .read()
                        .unwrap()
                        .分型特征值
                        .partial_cmp(&b.武.read().unwrap().分型特征值)
                        .unwrap_or(std::cmp::Ordering::Equal)
                        .then_with(|| {
                            a.武
                                .read()
                                .unwrap()
                                .时间戳
                                .cmp(&b.武.read().unwrap().时间戳)
                        })
                })
                .map(|x| x.武.read().unwrap().clone())
                .unwrap_or_else(|| self.元素[0].武.read().unwrap().clone())
        } else {
            self.元素
                .iter()
                .min_by(|a, b| {
                    a.武
                        .read()
                        .unwrap()
                        .分型特征值
                        .partial_cmp(&b.武.read().unwrap().分型特征值)
                        .unwrap_or(std::cmp::Ordering::Equal)
                        .then_with(|| {
                            b.武
                                .read()
                                .unwrap()
                                .时间戳
                                .cmp(&a.武.read().unwrap().时间戳)
                        })
                })
                .map(|x| x.武.read().unwrap().clone())
                .unwrap_or_else(|| self.元素[0].武.read().unwrap().clone())
        }
    }

    pub fn 高(&self) -> f64 {
        let 文 = self.文();
        let 武 = self.武();
        文.分型特征值.max(武.分型特征值)
    }

    pub fn 低(&self) -> f64 {
        let 文 = self.文();
        let 武 = self.武();
        文.分型特征值.min(武.分型特征值)
    }

    /// 方向 — 线段方向的翻转
    pub fn 方向(&self) -> 相对方向 {
        self.线段方向.翻转()
    }

    /// 向特征序列元素中添加虚线
    pub fn 添加(&mut self, 待添加虚线: Arc<虚线>) -> Result<(), String> {
        if 待添加虚线.方向() == self.线段方向 {
            return Err("添加方向与线段方向相同".into());
        }
        self.元素.push(待添加虚线);
        Ok(())
    }

    /// 从特征序列元素中删除虚线
    pub fn 删除(&mut self, 待删除虚线: &Arc<虚线>) -> Result<(), String> {
        if 待删除虚线.方向() == self.方向() {
            return Err("删除方向与特征序列方向相同".into());
        }
        if let Some(pos) = self
            .元素
            .iter()
            .position(|x| Arc::as_ptr(x) == Arc::as_ptr(待删除虚线))
        {
            self.元素.remove(pos);
            Ok(())
        } else {
            Err("待删除虚线不在特征序列中".into())
        }
    }

    /// 新建特征序列元素
    pub fn 新建(虚线序列: Vec<Arc<虚线>>, 线段方向: 相对方向) -> Self {
        let 标识 = format!("特征<虚线>");
        Self::new(标识, 虚线序列, 线段方向)
    }

    /// 静态分析 — 从虚线序列生成特征序列元素列表
    pub fn 静态分析(
        虚线序列: &[Arc<虚线>],
        线段方向: 相对方向,
        四象: &str,
        是否忽视: bool,
    ) -> Vec<Arc<线段特征>> {
        let mut 结果: Vec<Arc<线段特征>> = Vec::new();

        // 需要被合并的方向集合
        let 需要合并: Vec<相对方向> = match 四象 {
            "老阳" | "老阴" if !是否忽视 => vec![相对方向::顺, 相对方向::逆, 相对方向::同],
            _ => vec![相对方向::顺, 相对方向::同],
        };

        for 虚线 in 虚线序列 {
            // 情况1：方向相同（可能触发分型替换）
            if 虚线.方向() == 线段方向 {
                if 结果.len() >= 3 {
                    let 左 = Arc::clone(&结果[结果.len() - 3]);
                    let 中 = Arc::clone(&结果[结果.len() - 2]);
                    let 右 = Arc::clone(&结果[结果.len() - 1]);

                    if let Some(结构) = 分型结构::分析(&*左, &*中, &*右, true, true) {
                        let 应替换 = (线段方向 == 相对方向::向上
                            && 结构 == 分型结构::顶
                            && 虚线.高() > 中.高())
                            || (线段方向 == 相对方向::向下
                                && 结构 == 分型结构::底
                                && 虚线.低() < 中.低());

                        if 应替换 {
                            let 小号虚线 = 中
                                .元素
                                .iter()
                                .min_by_key(|o| o.序号.load(Ordering::Relaxed))
                                .unwrap();
                            let 大号虚线 = 右
                                .元素
                                .iter()
                                .max_by_key(|o| o.序号.load(Ordering::Relaxed))
                                .unwrap();
                            let fake = 虚线::创建笔(
                                Arc::clone(&小号虚线.文),
                                大号虚线.武.read().unwrap().clone(),
                                false,
                            );
                            结果.pop();
                            let idx = 结果.len() - 1;
                            结果[idx] = Arc::new(Self::新建(vec![Arc::new(fake)], 线段方向));
                        }
                    }
                }
                continue;
            }

            // 情况2：方向不同（执行特征序列的合并/添加）
            if 结果.is_empty() {
                结果.push(Arc::new(Self::新建(vec![Arc::clone(虚线)], 线段方向)));
                continue;
            }

            // 检查与最后一个特征序列的方向关系
            let 最后_idx = 结果.len() - 1;
            let 之前线段特征 = &*结果[最后_idx];
            if 需要合并.contains(&相对方向::分析(
                之前线段特征.高(),
                之前线段特征.低(),
                虚线.高(),
                虚线.低(),
            )) {
                // Clone-modify-replace
                let mut 新特征 = (*结果[最后_idx]).clone();
                let _ = 新特征.添加(Arc::clone(虚线));
                结果[最后_idx] = Arc::new(新特征);
            } else {
                结果.push(Arc::new(Self::新建(vec![Arc::clone(虚线)], 线段方向)));
            }
        }

        结果
    }

    /// 获取分型序列
    pub fn 获取分型序列(特征序列: &[Arc<线段特征>]) -> Vec<特征分型> {
        let mut 结果 = Vec::new();
        if 特征序列.len() < 3 {
            return 结果;
        }
        for i in 2..特征序列.len() {
            let 左 = Arc::clone(&特征序列[i - 2]);
            let 中 = Arc::clone(&特征序列[i - 1]);
            let 右 = Arc::clone(&特征序列[i]);

            let 结构 = 分型结构::分析_对象(
                &*左 as &dyn crate::types::fractal::有高低,
                &*中 as &dyn crate::types::fractal::有高低,
                &*右 as &dyn crate::types::fractal::有高低,
                true,
                true,
            );
            结果.push(特征分型::new(左, 中, 右, 结构.unwrap_or(分型结构::散)));
        }
        结果
    }
}

impl crate::types::fractal::有高低 for 线段特征 {
    fn 高(&self) -> f64 {
        self.高()
    }
    fn 低(&self) -> f64 {
        self.低()
    }
}

impl std::fmt::Display for 线段特征 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.元素.is_empty() {
            write!(f, "{}<{}, 空>", self.标识, self.线段方向)
        } else {
            write!(
                f,
                "{}<{}, {}, {}, {}>",
                self.标识,
                self.线段方向,
                self.文(),
                self.武(),
                self.元素.len()
            )
        }
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
        let mut k = K线::default();
        k.时间戳 = 时间戳;
        k.高 = 高;
        k.低 = 低;
        k.开盘价 = 开;
        k.收盘价 = 收;
        k
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
    // 文 — 取特征值最大/最小的分型
    // 特征序列元素方向与线段方向相反:
    //   向上线段 → 元素为向下笔(顶→底) → 文=顶分型 → 取max
    //   向下线段 → 元素为向上笔(底→顶) → 文=底分型 → 取min
    // ============================================================

    #[test]
    fn test_文_向上线段取最大特征值分型() {
        // 向上线段，元素用向下笔(顶→底)，文=顶分型
        // 笔1: 顶(特征值=100)→底(80), 文=顶(100)
        let 笔1 = 辅助_创建笔(100, 100.0, 90.0, 200, 90.0, 80.0);
        // 笔2: 顶(特征值=110)→底(90), 文=顶(110)
        let 笔2 = 辅助_创建笔(200, 110.0, 100.0, 300, 90.0, 80.0);

        let feat = 线段特征::new(
            "测试".into(),
            vec![Arc::clone(&笔1), Arc::clone(&笔2)],
            相对方向::向上,
        );

        // 向上取max → 笔2.文=110
        let 文 = feat.文();
        assert!((文.分型特征值 - 110.0).abs() < 0.01);
    }

    #[test]
    fn test_文_向下线段取最小特征值分型() {
        // 向下线段，元素用向上笔(底→顶)，文=底分型
        // 笔1: 底(特征值=80)→顶(100), 文=底(80)
        let 底1 = 辅助_创建底分型(100, 90.0, 80.0, 5);
        let 顶1 = 辅助_创建顶分型(200, 100.0, 90.0, 10);
        let 笔1 = Arc::new(虚线::创建笔(底1, 顶1, true));

        // 笔2: 底(特征值=70)→顶(95), 文=底(70)
        let 底2 = 辅助_创建底分型(200, 80.0, 70.0, 15);
        let 顶2 = 辅助_创建顶分型(300, 95.0, 85.0, 20);
        let 笔2 = Arc::new(虚线::创建笔(底2, 顶2, true));

        let feat = 线段特征::new(
            "测试".into(),
            vec![Arc::clone(&笔1), Arc::clone(&笔2)],
            相对方向::向下,
        );

        // 向下取min → 笔2.文=70
        let 文 = feat.文();
        assert!((文.分型特征值 - 70.0).abs() < 0.01);
    }

    // ============================================================
    // 武 — 取特征值最大/最小的分型
    //   向上线段 → 元素为向下笔 → 武=底分型 → 取max
    //   向下线段 → 元素为向上笔 → 武=顶分型 → 取min
    // ============================================================

    #[test]
    fn test_武_向上线段取最大特征值分型() {
        // 向上线段，元素用向下笔(顶→底)，武=底分型
        // 笔1: 顶(100)→底(特征值=80), 武=底(80)
        let 笔1 = 辅助_创建笔(100, 100.0, 90.0, 200, 90.0, 80.0);
        // 笔2: 顶(110)→底(特征值=90), 武=底(90)
        let 笔2 = 辅助_创建笔(200, 110.0, 100.0, 300, 100.0, 90.0);

        let feat = 线段特征::new(
            "测试".into(),
            vec![Arc::clone(&笔1), Arc::clone(&笔2)],
            相对方向::向上,
        );

        // 向上取max → 笔2.武=90
        let 武 = feat.武();
        assert!((武.分型特征值 - 90.0).abs() < 0.01);
    }

    #[test]
    fn test_武_向下线段取最小特征值分型() {
        // 向下线段，元素用向上笔(底→顶)，武=顶分型
        // 笔1: 底(80)→顶(特征值=100), 武=顶(100)
        let 底1 = 辅助_创建底分型(100, 90.0, 80.0, 5);
        let 顶1 = 辅助_创建顶分型(200, 100.0, 90.0, 10);
        let 笔1 = Arc::new(虚线::创建笔(底1, 顶1, true));

        // 笔2: 底(60)→顶(特征值=85), 武=顶(85)
        let 底2 = 辅助_创建底分型(200, 70.0, 60.0, 15);
        let 顶2 = 辅助_创建顶分型(300, 85.0, 75.0, 20);
        let 笔2 = Arc::new(虚线::创建笔(底2, 顶2, true));

        let feat = 线段特征::new(
            "测试".into(),
            vec![Arc::clone(&笔1), Arc::clone(&笔2)],
            相对方向::向下,
        );

        // 向下取min → 笔2.武=85
        let 武 = feat.武();
        assert!((武.分型特征值 - 85.0).abs() < 0.01);
    }

    // ============================================================
    // 文/武 tiebreaker — 同特征值取后时间戳
    // ============================================================

    #[test]
    fn test_文_同特征值取后时间戳() {
        // 两个笔的文特征值相同=100，但时间戳不同
        let 顶1 = 辅助_创建顶分型(100, 100.0, 90.0, 5);
        let 底1 = 辅助_创建底分型(200, 90.0, 80.0, 10);
        let 笔1 = Arc::new(虚线::创建笔(顶1, 底1, true));

        let 顶2 = 辅助_创建顶分型(300, 100.0, 90.0, 15); // 同特征值，后时间戳
        let 底2 = 辅助_创建底分型(400, 80.0, 70.0, 20);
        let 笔2 = Arc::new(虚线::创建笔(顶2, 底2, true));

        let feat = 线段特征::new("测试".into(), vec![笔1, 笔2], 相对方向::向上);

        let 文 = feat.文();
        // 向上取最大特征值：都是100 → tiebreaker取后时间戳 → 笔2.文(300)
        assert_eq!(文.时间戳, 300);
    }

    #[test]
    fn test_武_同特征值取后时间戳_向上() {
        let 顶1 = 辅助_创建顶分型(100, 100.0, 90.0, 5);
        let 底1 = 辅助_创建底分型(200, 80.0, 70.0, 10); // 特征值80
        let 笔1 = Arc::new(虚线::创建笔(顶1, 底1, true));

        let 顶2 = 辅助_创建顶分型(300, 80.0, 70.0, 15); // 特征值80
        let 底2 = 辅助_创建底分型(400, 80.0, 70.0, 20); // 特征值80
        let 笔2 = Arc::new(虚线::创建笔(顶2, 底2, true));

        let feat = 线段特征::new("测试".into(), vec![笔1, 笔2], 相对方向::向上);

        let 武 = feat.武();
        // 向上取最大特征值：都是80 → tiebreaker取后时间戳 → 笔2.武(400)
        assert_eq!(武.时间戳, 400);
    }

    // ============================================================
    // 添加/删除 操作
    // ============================================================

    #[test]
    fn test_添加方向与线段方向相反的虚线可成功() {
        // 向下笔(顶→底,方向=向下) 添加到 向上线段(方向=向上) → 方向不同, 可添加
        let 笔1 = 辅助_创建笔(100, 100.0, 90.0, 200, 90.0, 80.0);
        let mut feat = 线段特征::new("测试".into(), vec![], 相对方向::向上);

        let result = feat.添加(Arc::clone(&笔1));
        assert!(result.is_ok());
    }

    #[test]
    fn test_添加方向与线段方向相同的虚线应报错() {
        // 向下笔(顶→底,方向=向下) 添加到 向下线段(方向=向下) → 方向相同, 应报错
        let 笔1 = 辅助_创建笔(100, 100.0, 90.0, 200, 90.0, 80.0);
        let mut feat = 线段特征::new("测试".into(), vec![], 相对方向::向下);

        let result = feat.添加(Arc::clone(&笔1));
        assert!(result.is_err());
    }

    #[test]
    fn test_空线段特征文返回第一个元素的文() {
        let 笔1 = 辅助_创建笔(100, 100.0, 90.0, 200, 90.0, 80.0);
        let feat = 线段特征::new("测试".into(), vec![Arc::clone(&笔1)], 相对方向::向上);

        let 文 = feat.文();
        assert_eq!(Arc::as_ptr(&文), Arc::as_ptr(&笔1.文));
    }
}
