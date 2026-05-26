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
use std::rc::Rc;

/// 线段特征 — 特征序列元素（内部是虚线的集合）
#[derive(Debug, Clone)]
pub struct 线段特征 {
    pub 序号: i64,
    pub 标识: String,
    pub 线段方向: 相对方向,
    pub 元素: Vec<Rc<虚线>>,
}

impl 线段特征 {
    pub fn new(标识: String, 基础序列: Vec<Rc<虚线>>, 线段方向: 相对方向) -> Self {
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
    pub fn 文(&self) -> Rc<分型> {
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
                .map(|x| Rc::clone(&x.文))
                .unwrap_or_else(|| Rc::clone(&self.元素[0].文))
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
                .map(|x| Rc::clone(&x.文))
                .unwrap_or_else(|| Rc::clone(&self.元素[0].文))
        }
    }

    /// 武 — 取特征序列元素中分型特征值最大/最小的武分型
    /// tiebreaker: later时间戳 wins when特征值 equal (matches Python)
    pub fn 武(&self) -> Rc<分型> {
        if self.线段方向.是否向上() {
            self.元素
                .iter()
                .max_by(|a, b| {
                    a.武
                        .分型特征值
                        .partial_cmp(&b.武.分型特征值)
                        .unwrap_or(std::cmp::Ordering::Equal)
                        .then_with(|| a.武.时间戳.cmp(&b.武.时间戳))
                })
                .map(|x| Rc::clone(&x.武))
                .unwrap_or_else(|| Rc::clone(&self.元素[0].武))
        } else {
            self.元素
                .iter()
                .min_by(|a, b| {
                    a.武
                        .分型特征值
                        .partial_cmp(&b.武.分型特征值)
                        .unwrap_or(std::cmp::Ordering::Equal)
                        .then_with(|| b.武.时间戳.cmp(&a.武.时间戳))
                })
                .map(|x| Rc::clone(&x.武))
                .unwrap_or_else(|| Rc::clone(&self.元素[0].武))
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
    pub fn 添加(&mut self, 待添加虚线: Rc<虚线>) -> Result<(), String> {
        if 待添加虚线.方向() == self.线段方向 {
            return Err("添加方向与线段方向相同".into());
        }
        self.元素.push(待添加虚线);
        Ok(())
    }

    /// 从特征序列元素中删除虚线
    pub fn 删除(&mut self, 待删除虚线: &Rc<虚线>) -> Result<(), String> {
        if 待删除虚线.方向() == self.方向() {
            return Err("删除方向与特征序列方向相同".into());
        }
        if let Some(pos) = self
            .元素
            .iter()
            .position(|x| Rc::as_ptr(x) == Rc::as_ptr(待删除虚线))
        {
            self.元素.remove(pos);
            Ok(())
        } else {
            Err("待删除虚线不在特征序列中".into())
        }
    }

    /// 新建特征序列元素
    pub fn 新建(虚线序列: Vec<Rc<虚线>>, 线段方向: 相对方向) -> Self {
        let 标识 = format!("特征<虚线>");
        Self::new(标识, 虚线序列, 线段方向)
    }

    /// 静态分析 — 从虚线序列生成特征序列元素列表
    pub fn 静态分析(
        虚线序列: &[Rc<虚线>],
        线段方向: 相对方向,
        四象: &str,
        是否忽视: bool,
    ) -> Vec<Rc<线段特征>> {
        let mut 结果: Vec<Rc<线段特征>> = Vec::new();

        // 需要被合并的方向集合
        let 需要合并: Vec<相对方向> = match 四象 {
            "老阳" | "老阴" if !是否忽视 => vec![相对方向::顺, 相对方向::逆, 相对方向::同],
            _ => vec![相对方向::顺, 相对方向::同],
        };

        for 虚线 in 虚线序列 {
            // 情况1：方向相同（可能触发分型替换）
            if 虚线.方向() == 线段方向 {
                if 结果.len() >= 3 {
                    let 左 = Rc::clone(&结果[结果.len() - 3]);
                    let 中 = Rc::clone(&结果[结果.len() - 2]);
                    let 右 = Rc::clone(&结果[结果.len() - 1]);

                    if let Some(结构) = 分型结构::分析(&*左, &*中, &*右, true, true) {
                        let 应替换 = (线段方向 == 相对方向::向上
                            && 结构 == 分型结构::顶
                            && 虚线.高() > 中.高())
                            || (线段方向 == 相对方向::向下
                                && 结构 == 分型结构::底
                                && 虚线.低() < 中.低());

                        if 应替换 {
                            let 小号虚线 = 中.元素.iter().min_by_key(|o| o.序号).unwrap();
                            let 大号虚线 = 右.元素.iter().max_by_key(|o| o.序号).unwrap();
                            let fake = 虚线::创建笔(
                                Rc::clone(&小号虚线.文),
                                Rc::clone(&大号虚线.武),
                                false,
                            );
                            结果.pop();
                            let idx = 结果.len() - 1;
                            结果[idx] = Rc::new(Self::新建(vec![Rc::new(fake)], 线段方向));
                        }
                    }
                }
                continue;
            }

            // 情况2：方向不同（执行特征序列的合并/添加）
            if 结果.is_empty() {
                结果.push(Rc::new(Self::新建(vec![Rc::clone(虚线)], 线段方向)));
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
                let _ = 新特征.添加(Rc::clone(虚线));
                结果[最后_idx] = Rc::new(新特征);
            } else {
                结果.push(Rc::new(Self::新建(vec![Rc::clone(虚线)], 线段方向)));
            }
        }

        结果
    }

    /// 获取分型序列
    pub fn 获取分型序列(特征序列: &[Rc<线段特征>]) -> Vec<特征分型> {
        let mut 结果 = Vec::new();
        if 特征序列.len() < 3 {
            return 结果;
        }
        for i in 2..特征序列.len() {
            let 左 = Rc::clone(&特征序列[i - 2]);
            let 中 = Rc::clone(&特征序列[i - 1]);
            let 右 = Rc::clone(&特征序列[i]);

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
