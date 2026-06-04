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
use std::collections::HashMap;

/// K线合成器 — 将小周期K线合成为大周期K线
pub struct K线合成器 {
    pub 标识: String,
    pub 周期组: Vec<i64>,
    pub 当前K线: HashMap<i64, Option<K线>>,
    pub 合成K线列表: HashMap<i64, Vec<K线>>,
}

impl K线合成器 {
    /// 创建K线合成器，按周期升序排列，初始化当前K线和合成K线列表
    pub fn new(标识: String, 周期组: Vec<i64>) -> Self {
        let mut 周期组 = 周期组;
        周期组.sort();

        let mut 当前K线 = HashMap::new();
        let mut 合成K线列表 = HashMap::new();
        for &周期 in &周期组 {
            当前K线.insert(周期, None);
            合成K线列表.insert(周期, Vec::new());
        }

        Self {
            标识,
            周期组,
            当前K线,
            合成K线列表,
        }
    }

    /// 投喂 — 便捷入口，直接从 OHLCV 创建 K线 并投喂
    pub fn 投喂(
        &mut self,
        时间戳: i64,
        开: f64,
        高: f64,
        低: f64,
        收: f64,
        量: f64,
    ) -> Vec<(i64, K线)> {
        let 普K = K线::创建普K(&self.标识, 时间戳, 开, 高, 低, 收, 量, 0, 0);
        self.投喂K线(普K)
    }

    /// 投喂K线 — 输入最小周期K线，合成为所有目标周期
    /// 返回本次投喂完成了哪些周期的K线（周期 → 完成K线）
    pub fn 投喂K线(&mut self, 普K: K线) -> Vec<(i64, K线)> {
        let mut 完成记录 = Vec::new();
        let 周期组 = self.周期组.clone();
        for 周期 in 周期组 {
            if let Some(完成K线) = self._处理单个周期(周期, &普K) {
                完成记录.push((周期, 完成K线));
            }
        }
        完成记录
    }

    fn _处理单个周期(&mut self, 周期: i64, 普K: &K线) -> Option<K线> {
        let 目标时间戳 = self._对齐时间戳(普K.时间戳, 周期);
        let 相同时间 = self.当前K线[&周期]
            .as_ref()
            .map(|k| k.时间戳 == 目标时间戳)
            .unwrap_or(false);

        if self.当前K线[&周期].is_none() {
            let 新K线 = self._创建新K线(周期, 目标时间戳, 普K);
            self.当前K线.insert(周期, Some(新K线));
            None
        } else if 相同时间 {
            let ent = self.当前K线.get_mut(&周期).unwrap();
            Self::_更新K线(ent.as_mut().unwrap(), 普K);
            None
        } else {
            let 完成K线 = self._完成K线(周期);
            let 新K线 = self._创建新K线(周期, 目标时间戳, 普K);
            self.当前K线.insert(周期, Some(新K线));
            完成K线
        }
    }

    fn _对齐时间戳(&self, 时间戳: i64, 周期: i64) -> i64 {
        if 周期 == 0 {
            panic!("_对齐时间戳: 周期不能为0");
        }
        (时间戳 / 周期) * 周期
    }

    fn _创建新K线(&self, 周期: i64, 时间戳: i64, 普K: &K线) -> K线 {
        let 序号 = self
            .合成K线列表
            .get(&周期)
            .and_then(|list| list.last())
            .map(|k| k.序号 + 1)
            .unwrap_or(0);

        K线::创建普K(
            &self.标识,
            时间戳,
            普K.开盘价,
            普K.高,
            普K.低,
            普K.收盘价,
            普K.成交量,
            序号,
            周期,
        )
    }

    fn _更新K线(当前K线: &mut K线, 新数据: &K线) {
        当前K线.高 = 当前K线.高.max(新数据.高);
        当前K线.低 = 当前K线.低.min(新数据.低);
        当前K线.收盘价 = 新数据.收盘价;
        当前K线.成交量 += 新数据.成交量;
    }

    fn _完成K线(&mut self, 周期: i64) -> Option<K线> {
        let ent = self.当前K线.get_mut(&周期).unwrap();
        let mut k线 = ent.take()?;
        k线.序号 = self
            .合成K线列表
            .get(&周期)
            .and_then(|list| list.last())
            .map(|k| k.序号 + 1)
            .unwrap_or(0);

        let 完成K线 = k线.clone();
        self.合成K线列表.get_mut(&周期).unwrap().push(k线);
        Some(完成K线)
    }

    /// 获取指定周期当前正在合成的K线
    pub fn 获取当前K线(&self, 周期: i64) -> Option<&K线> {
        self.当前K线.get(&周期).and_then(|k| k.as_ref())
    }
}
