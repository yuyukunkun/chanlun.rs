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
use crate::warn;
use std::collections::HashMap;

/// 事件回调类型 — fn(信号类型, 标识, 周期, 完成K线)
type 合成器事件回调 = Box<dyn Fn(String, String, i64, K线) + Send + Sync>;

/// K线合成器 — 将小周期K线合成为大周期K线
pub struct K线合成器 {
    pub 标识: String,
    pub 周期组: Vec<i64>,
    pub 当前K线: HashMap<i64, Option<K线>>,
    pub 合成K线列表: HashMap<i64, Vec<K线>>,
    /// 事件回调 — K线完成时触发，对应 Python K线合成器.事件回调
    /// 签名: fn(信号类型: str, 标识: str, 周期: i64, 完成K线: K线)
    /// 在 _完成K线 清空当前K线后、新K线创建前触发
    事件回调: Option<合成器事件回调>,
}

impl K线合成器 {
    /// 创建K线合成器 — 对应 Python K线合成器.__init__(标识, 周期组, 事件回调=None)
    pub fn new(
        标识: String, 周期组: Vec<i64>, 事件回调: Option<合成器事件回调>
    ) -> Self {
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
            事件回调,
        }
    }

    /// 设置事件回调 — 对应 Python `设置事件回调`
    pub fn 设置事件回调(&mut self, 回调: 合成器事件回调) {
        self.事件回调 = Some(回调);
    }

    /// 投喂 — 便捷入口，直接从 OHLCV 创建 K线 并投喂
    pub fn 投喂(&mut self, 时间戳: i64, 开: f64, 高: f64, 低: f64, 收: f64, 量: f64) {
        let 普K = K线::创建普K(&self.标识, 时间戳, 开, 高, 低, 收, 量, 0, 0);
        self.投喂K线(普K);
    }

    /// 投喂K线 — 输入最小周期K线，合成为所有目标周期
    pub fn 投喂K线(&mut self, 普K: K线) {
        let 周期组 = self.周期组.clone();
        for 周期 in 周期组 {
            self._处理单个周期(周期, &普K);
        }
    }

    fn _处理单个周期(&mut self, 周期: i64, 普K: &K线) {
        let 目标时间戳 = self._对齐时间戳(普K.时间戳, 周期);
        let 相同时间 = self.当前K线[&周期]
            .as_ref()
            .map(|k| k.时间戳 == 目标时间戳)
            .unwrap_or(false);

        if self.当前K线[&周期].is_none() {
            let 新K线 = self._创建新K线(周期, 目标时间戳, 普K);
            self.当前K线.insert(周期, Some(新K线));
        } else if 相同时间 {
            let ent = self.当前K线.get_mut(&周期).unwrap();
            Self::_更新K线(ent.as_mut().unwrap(), 普K);
        } else {
            self._完成K线(周期);
            let 新K线 = self._创建新K线(周期, 目标时间戳, 普K);
            self.当前K线.insert(周期, Some(新K线));
        }
    }

    /// 对齐时间戳到周期边界 — 对应 Python `_对齐时间戳`
    fn _对齐时间戳(&self, 时间戳: i64, 周期: i64) -> i64 {
        if 周期 == 0 {
            panic!("_对齐时间戳: 周期不能为0");
        }
        (时间戳 / 周期) * 周期
    }

    /// 创建新K线 — 对应 Python `_创建新K线`
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

    /// 更新K线 — 对应 Python `_更新K线`
    fn _更新K线(当前K线: &mut K线, 新数据: &K线) {
        当前K线.高 = 当前K线.高.max(新数据.高);
        当前K线.低 = 当前K线.低.min(新数据.低);
        当前K线.收盘价 = 新数据.收盘价;
        当前K线.成交量 += 新数据.成交量;
    }

    /// 完成K线 — 对应 Python `_完成K线`
    /// 清空当前K线后，触发事件回调（此时获取当前K线返回 None）
    fn _完成K线(&mut self, 周期: i64) {
        let ent = self.当前K线.get_mut(&周期).unwrap();
        let mut k线 = match ent.take() {
            Some(k) => k,
            None => return,
        };
        k线.序号 = self
            .合成K线列表
            .get(&周期)
            .and_then(|list| list.last())
            .map(|k| k.序号 + 1)
            .unwrap_or(0);

        let 完成K线 = k线.clone();
        self.合成K线列表.get_mut(&周期).unwrap().push(k线);

        // 对应 Python _完成K线：清空当前K线后、新K线创建前触发回调
        self._产生完成K线信号(周期, 完成K线);
    }

    /// 产生完成K线信号 — 对应 Python `_产生完成K线信号`
    /// 异常安全：若回调 panic，捕获并记录错误，不中断管线
    fn _产生完成K线信号(&self, 周期: i64, 完成K线: K线) {
        if let Some(ref cb) = self.事件回调 {
            let 标识 = self.标识.clone();
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                cb("K线完成".into(), 标识, 周期, 完成K线);
            }));
            if let Err(e) = result {
                let msg = e
                    .downcast_ref::<&str>()
                    .map(|s| s.to_string())
                    .or_else(|| e.downcast_ref::<String>().cloned())
                    .unwrap_or_else(|| "未知错误".into());
                warn!("K线合成器 事件回调 异常: {}", msg);
            }
        }
    }

    /// 获取指定周期当前正在合成的K线 — 对应 Python `获取当前K线`
    pub fn 获取当前K线(&self, 周期: i64) -> Option<&K线> {
        self.当前K线.get(&周期).and_then(|k| k.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_创建合成器_初始状态正确() {
        let synth = K线合成器::new("btcusd".into(), vec![60, 300], None);
        assert_eq!(synth.标识, "btcusd");
        assert_eq!(synth.周期组, vec![60, 300]);
        assert!(synth.事件回调.is_none());
        assert!(synth.当前K线[&60].is_none());
        assert!(synth.当前K线[&300].is_none());
    }

    #[test]
    fn test_设置事件回调() {
        let mut synth = K线合成器::new("btcusd".into(), vec![60], None);
        assert!(synth.事件回调.is_none());
        synth.设置事件回调(Box::new(|_, _, _, _| {}));
        assert!(synth.事件回调.is_some());
    }

    #[test]
    fn test_对齐时间戳() {
        let synth = K线合成器::new("t".into(), vec![300], None);
        assert_eq!(synth._对齐时间戳(1218124800, 300), 1218124800);
        assert_eq!(synth._对齐时间戳(1218124801, 300), 1218124800);
        assert_eq!(synth._对齐时间戳(1218125099, 300), 1218124800);
        assert_eq!(synth._对齐时间戳(1218125100, 300), 1218125100);
    }

    #[test]
    fn test_创建新K线_序号递进() {
        let mut synth = K线合成器::new("btcusd".into(), vec![300], None);
        {
            let first = K线::创建普K("btcusd", 0, 100.0, 110.0, 90.0, 105.0, 1000.0, 0, 300);
            synth.合成K线列表.get_mut(&300).unwrap().push(first);
        }
        let new_bar = K线::创建普K("btcusd", 100, 200.0, 210.0, 190.0, 205.0, 500.0, 0, 60);
        let created = synth._创建新K线(300, 300, &new_bar);
        assert_eq!(created.序号, 1);
        assert_eq!(created.时间戳, 300);
        assert_eq!(created.开盘价, 200.0);
    }

    #[test]
    fn test_更新K线_高低更新() {
        let mut current = K线::创建普K("t", 0, 100.0, 110.0, 90.0, 105.0, 100.0, 0, 300);
        let new_data = K线::创建普K("t", 0, 102.0, 115.0, 85.0, 108.0, 50.0, 0, 60);
        K线合成器::_更新K线(&mut current, &new_data);
        assert_eq!(current.高, 115.0);
        assert_eq!(current.低, 85.0);
        assert_eq!(current.收盘价, 108.0);
        assert_eq!(current.成交量, 150.0);
    }

    #[test]
    fn test_完成K线_返回完成K并将当前置空() {
        let mut synth = K线合成器::new("btcusd".into(), vec![300], None);
        let bar = K线::创建普K("btcusd", 300, 100.0, 110.0, 90.0, 105.0, 1000.0, 0, 300);
        synth.当前K线.insert(300, Some(bar));
        synth._完成K线(300);
        assert!(synth.当前K线[&300].is_none());
        assert_eq!(synth.合成K线列表[&300].len(), 1);
    }

    #[test]
    fn test_完成K线_事件回调触发() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicBool, Ordering};

        let callback_fired = Arc::new(AtomicBool::new(false));
        let cb_flag = Arc::clone(&callback_fired);

        let mut synth = K线合成器::new(
            "btcusd".into(),
            vec![300],
            Some(Box::new(move |信号类型, 标识, 周期, _完成K线| {
                assert_eq!(信号类型, "K线完成");
                assert_eq!(标识, "btcusd");
                assert_eq!(周期, 300);
                cb_flag.store(true, Ordering::SeqCst);
            })),
        );

        let bar1 = K线::创建普K("btcusd", 0, 100.0, 110.0, 90.0, 105.0, 1000.0, 0, 300);
        synth.当前K线.insert(300, Some(bar1));
        let bar2 = K线::创建普K("btcusd", 400, 200.0, 210.0, 190.0, 205.0, 500.0, 0, 60);
        synth.投喂K线(bar2);
        assert!(callback_fired.load(Ordering::SeqCst));
    }

    #[test]
    fn test_投喂K线_多周期合成() {
        let mut synth = K线合成器::new("btcusd".into(), vec![60, 300], None);
        synth.投喂K线(K线::创建普K(
            "btcusd", 60, 100.0, 110.0, 90.0, 105.0, 100.0, 0, 60,
        ));
        assert!(synth.获取当前K线(60).is_some());
        assert!(synth.获取当前K线(300).is_some());
    }

    #[test]
    fn test_投喂_便捷方法() {
        let mut synth = K线合成器::new("btcusd".into(), vec![300], None);
        synth.投喂(1218124800, 100.0, 110.0, 90.0, 105.0, 1000.0);
        assert!(synth.获取当前K线(300).is_some());
    }
}
