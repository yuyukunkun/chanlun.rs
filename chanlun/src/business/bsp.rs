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
use std::rc::Rc;

/// 基础买卖点 — 买卖点的基础数据结构
#[derive(Debug, Clone)]
pub struct 基础买卖点 {
    pub 备注: String,
    pub 类型: 买卖点类型,
    pub 买卖点分型: Rc<分型>,
    pub 买卖点K线: Rc<缠论K线>,
    pub 当前K线: Rc<K线>,
    pub 失效K线: Option<Rc<K线>>,
    pub 终结K线: Option<Rc<K线>>,
    pub 破位值: f64,
    pub 结构: Option<分型结构>,
}

impl 基础买卖点 {
    pub fn new(
        类型: 买卖点类型,
        当前K线: Rc<K线>,
        买卖点分型: Rc<分型>,
        备注: String,
        中枢破位值: f64,
    ) -> Self {
        let 买卖点K线 = Rc::clone(&买卖点分型.中);
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
        }
    }

    /// 偏移 — 当前K线与买卖点K线的序号差
    pub fn 偏移(&self) -> i64 {
        self.当前K线.序号 - self.买卖点K线.序号
    }

    /// 失效偏移
    pub fn 失效偏移(&self) -> i64 {
        match &self.失效K线 {
            Some(k) => k.序号 - self.买卖点K线.序号,
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

/// 买卖点 — 包含一二三类买卖点的工厂方法
pub struct 买卖点;

impl 买卖点 {
    pub fn 一卖点(
        买卖点分型: Rc<分型>,
        当前K线: Rc<K线>,
        _标识: &str,
        备注: String,
        中枢破位值: f64,
    ) -> 基础买卖点 {
        基础买卖点::new(买卖点类型::一卖, 当前K线, 买卖点分型, 备注, 中枢破位值)
    }

    pub fn 一买点(
        买卖点分型: Rc<分型>,
        当前K线: Rc<K线>,
        _标识: &str,
        备注: String,
        中枢破位值: f64,
    ) -> 基础买卖点 {
        基础买卖点::new(买卖点类型::一买, 当前K线, 买卖点分型, 备注, 中枢破位值)
    }

    pub fn 二卖点(
        买卖点分型: Rc<分型>,
        当前K线: Rc<K线>,
        _标识: &str,
        备注: String,
        中枢破位值: f64,
    ) -> 基础买卖点 {
        基础买卖点::new(买卖点类型::二卖, 当前K线, 买卖点分型, 备注, 中枢破位值)
    }

    pub fn 二买点(
        买卖点分型: Rc<分型>,
        当前K线: Rc<K线>,
        _标识: &str,
        备注: String,
        中枢破位值: f64,
    ) -> 基础买卖点 {
        基础买卖点::new(买卖点类型::二买, 当前K线, 买卖点分型, 备注, 中枢破位值)
    }

    pub fn 三卖点(
        买卖点分型: Rc<分型>,
        当前K线: Rc<K线>,
        _标识: &str,
        备注: String,
        中枢破位值: f64,
    ) -> 基础买卖点 {
        基础买卖点::new(买卖点类型::三卖, 当前K线, 买卖点分型, 备注, 中枢破位值)
    }

    pub fn 三买点(
        买卖点分型: Rc<分型>,
        当前K线: Rc<K线>,
        _标识: &str,
        备注: String,
        中枢破位值: f64,
    ) -> 基础买卖点 {
        基础买卖点::new(买卖点类型::三买, 当前K线, 买卖点分型, 备注, 中枢破位值)
    }

    /// 生成买卖点 — 根据参数自动选择类型
    pub fn 生成买卖点(
        特征: &str,
        序号: &str,
        级别: &str,
        买卖点分型: Rc<分型>,
        当前缠K: Rc<缠论K线>,
    ) -> 基础买卖点 {
        let 买卖 = if matches!(买卖点分型.结构, 分型结构::底 | 分型结构::下) {
            "买"
        } else {
            "卖"
        };
        let 备注 = format!("{}_{}{}{}", 特征, 级别, 序号, 买卖);
        let 破位值 = 买卖点分型.分型特征值;

        // 当前K线 — 从缠K获取其标的K线
        let 当前K线 = Rc::clone(&当前缠K.标的K线);

        let 类型 = match (序号, 买卖) {
            ("一", "买") => 买卖点类型::一买,
            ("一", "卖") => 买卖点类型::一卖,
            ("二", "买") => 买卖点类型::二买,
            ("二", "卖") => 买卖点类型::二卖,
            ("三", "买") => 买卖点类型::三买,
            ("三", "卖") => 买卖点类型::三卖,
            _ => 买卖点类型::一买, // fallback
        };

        基础买卖点::new(类型, 当前K线, 买卖点分型, 备注, 破位值)
    }
}
