use crate::business::observer::观察者;
use crate::business::synthesizer::K线合成器;
use crate::config::缠论配置;
use crate::kline::bar::K线;
use std::collections::HashMap;

/// 立体分析器 — 多周期协调器
///
/// 包含一个K线合成器和每周期一个观察者。
/// 输入最小周期K线，合成大周期后分发到对应观察者。
pub struct 立体分析器 {
    pub 周期组: Vec<i64>,
    输入周期: i64,
    显示周期: i64,
    K线合成器: K线合成器,
    单体分析器: HashMap<i64, 观察者>,
    合成K线计数: HashMap<i64, usize>,
}

impl 立体分析器 {
    pub fn new(
        符号: String,
        周期组: Vec<i64>,
        配置: Option<缠论配置>,
        配置组: Option<HashMap<i64, 缠论配置>>,
    ) -> Self {
        let mut 周期组 = 周期组;
        周期组.sort();
        let 输入周期 = 周期组[0];
        let 显示周期 = if 周期组.len() > 1 {
            周期组[1]
        } else {
            周期组[0]
        };

        let 默认配置 = 配置.unwrap_or_default();
        let 配置组 = 配置组.unwrap_or_default();

        let K线合成器 = K线合成器::new(符号.clone(), 周期组.clone());

        let mut 单体分析器 = HashMap::new();
        for &周期 in &周期组 {
            let mut 当前配置 = 配置组.get(&周期).cloned().unwrap_or_else(|| 默认配置.clone());
            当前配置.推送K线 = false;
            当前配置.推送线段 = false;
            // Set 标识 to match 符号
            当前配置.标识 = 符号.clone();

            let 观察员 = 观察者::new(符号.clone(), 周期, 当前配置);
            单体分析器.insert(周期, 观察员);
        }

        // Configure display period
        if let Some(显示观察员) = 单体分析器.get_mut(&显示周期) {
            显示观察员.配置.推送K线 = true;
            显示观察员.配置.推送笔 = true;
            显示观察员.配置.推送线段 = true;
            显示观察员.配置.图表展示 = true;
            显示观察员.重置基础序列();
        }

        // Align other periods to display period's 缠K序列
        // (in practice, this is done during data loading)
        for &周期 in &周期组 {
            if 周期 != 显示周期 {
                // Other periods will reference display period's 缠论K线序列
                // This is done during the K-line callback flow
            }
        }

        let mut 合成K线计数 = HashMap::new();
        for &周期 in &周期组 {
            合成K线计数.insert(周期, 0);
        }

        Self {
            周期组,
            输入周期,
            显示周期,
            K线合成器,
            单体分析器,
            合成K线计数,
        }
    }

    /// 投喂K线 — 统一入口，接收最小周期K线
    pub fn 投喂K线(&mut self, 普K: K线) {
        if 普K.周期 != self.输入周期 {
            eprintln!(
                "立体分析器.投喂K线 周期不匹配 {} != {}",
                普K.周期, self.输入周期
            );
            return;
        }

        // Record current K-line counts before feeding
        let mut 之前计数 = HashMap::new();
        for &周期 in &self.周期组 {
            之前计数.insert(周期, self.K线合成器.合成K线列表.get(&周期).map(|v| v.len()).unwrap_or(0));
        }

        // Feed to synthesizer
        self.K线合成器.投喂K线(普K);

        // Dispatch new K-lines to observers
        for &周期 in &self.周期组 {
            let 新K线列表 = self.K线合成器.合成K线列表.get(&周期);
            let 之前计数 = 之前计数[&周期];
            if let Some(列表) = 新K线列表 {
                for k线 in 列表.iter().skip(之前计数) {
                    // Clone the K-line for feeding to observer
                    let 完成K线 = k线.clone();
                    if let Some(观察员) = self.单体分析器.get_mut(&周期) {
                        观察员.增加原始K线(完成K线);
                    }
                }
            }

            // Also feed the current in-progress K-line (if any)
            if let Some(Some(当前K线)) = self.K线合成器.当前K线.get(&周期).cloned() {
                if let Some(观察员) = self.单体分析器.get_mut(&周期) {
                    // Feed a clone of current in-progress K-line for real-time updates
                    // Note: this may cause duplicate data, matching Python behavior
                    观察员.增加原始K线(当前K线);
                }
            }
        }
    }

    /// 获取指定周期的观察者
    pub fn 获取观察者(&self, 周期: i64) -> Option<&观察者> {
        self.单体分析器.get(&周期)
    }

    /// 获取指定周期的观察者（可变）
    pub fn 获取观察者_mut(&mut self, 周期: i64) -> Option<&mut 观察者> {
        self.单体分析器.get_mut(&周期)
    }

    /// 测试_保存数据
    pub fn 测试_保存数据(&self) {
        for 周期 in &self.周期组 {
            if let Some(观察员) = self.单体分析器.get(周期) {
                观察员.测试_保存数据(None);
            }
        }
    }
}
