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

    /// 投喂K线 — 输入最小周期K线，合成为所有目标周期
    pub fn 投喂K线(&mut self, 普K: K线) {
        let 周期组 = self.周期组.clone();
        for 周期 in 周期组 {
            self._处理单个周期(周期, &普K);
        }
    }

    fn _处理单个周期(&mut self, 周期: i64, 普K: &K线) {
        let 目标时间戳 = self._对齐时间戳(普K.时间戳, 周期);
        let 当前K线 = self.当前K线.get(&周期).unwrap().clone();

        if 当前K线.is_none() {
            // 创建新K线
            let 新K线 = self._创建新K线(周期, 目标时间戳, 普K);
            *self.当前K线.get_mut(&周期).unwrap() = Some(新K线);
        } else if 当前K线.as_ref().unwrap().时间戳 == 目标时间戳 {
            // 更新当前K线
            let mut k线 = 当前K线.unwrap();
            self._更新K线(&mut k线, 普K);
            *self.当前K线.get_mut(&周期).unwrap() = Some(k线);
        } else {
            // 完成当前K线，创建新K线
            self._完成K线(周期);
            let 新K线 = self._创建新K线(周期, 目标时间戳, 普K);
            *self.当前K线.get_mut(&周期).unwrap() = Some(新K线);
        }
    }

    fn _对齐时间戳(&self, 时间戳: i64, 周期: i64) -> i64 {
        if 周期 == 0 {
            return 时间戳;
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

    fn _更新K线(&self, 当前K线: &mut K线, 新数据: &K线) {
        当前K线.高 = 当前K线.高.max(新数据.高);
        当前K线.低 = 当前K线.低.min(新数据.低);
        当前K线.收盘价 = 新数据.收盘价;
        当前K线.成交量 += 新数据.成交量;
    }

    fn _完成K线(&mut self, 周期: i64) {
        let 当前K线 = self.当前K线.get(&周期).and_then(|k| k.clone());
        if let Some(mut k线) = 当前K线 {
            k线.序号 = self
                .合成K线列表
                .get(&周期)
                .and_then(|list| list.last())
                .map(|k| k.序号 + 1)
                .unwrap_or(0);

            self.合成K线列表.get_mut(&周期).unwrap().push(k线);
            *self.当前K线.get_mut(&周期).unwrap() = None;
        }
    }

    /// 获取指定周期当前正在合成的K线
    pub fn 获取当前K线(&self, 周期: i64) -> Option<&K线> {
        self.当前K线.get(&周期).and_then(|k| k.as_ref())
    }
}
