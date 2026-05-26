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
    K线合成器: K线合成器,
    单体分析器: HashMap<i64, 观察者>,
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

        let 默认配置 = 配置.unwrap_or_default();
        let 配置组 = 配置组.unwrap_or_default();

        let K线合成器 = K线合成器::new(符号.clone(), 周期组.clone());

        let mut 单体分析器 = HashMap::new();
        for &周期 in &周期组 {
            let mut 当前配置 = 配置组
                .get(&周期)
                .cloned()
                .unwrap_or_else(|| 默认配置.clone());
            当前配置.推送K线 = false;
            当前配置.推送线段 = false;
            当前配置.标识 = 符号.clone();

            let 观察员 = 观察者::new(符号.clone(), 周期, 当前配置);
            单体分析器.insert(周期, 观察员);
        }

        Self {
            周期组,
            输入周期,
            K线合成器,
            单体分析器,
        }
    }

    /// 投喂K线 — 统一入口，接收最小周期K线
    /// 匹配 Python __K线回调：合成器完成K线时喂给观察者
    pub fn 投喂K线(&mut self, 普K: K线) {
        if 普K.周期 != self.输入周期 {
            eprintln!(
                "立体分析器.投喂K线 周期不匹配 {} != {}",
                普K.周期, self.输入周期
            );
            return;
        }

        // Feed to synthesizer, get completion events
        let 完成事件 = self.K线合成器.投喂K线(普K);

        // Dispatch on completion events (matching Python's __K线回调)
        for (周期, 完成K线) in 完成事件 {
            if let Some(观察员) = self.单体分析器.get_mut(&周期) {
                观察员.增加原始K线(完成K线);
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

    /// 测试_保存数据 — 多级别数据拆分保存
    /// 创建父目录 PyM_{标识}_{起始时间}_{结束时间}，各周期观察者保存到子目录
    pub fn 测试_保存数据(&self) {
        let 根目录 = std::env::var("CHANLUN_DATA_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| std::env::temp_dir());

        let 起始时间 = self
            .单体分析器
            .get(&self.输入周期)
            .and_then(|o| o.普通K线序列.first())
            .map(|k| k.时间戳)
            .unwrap_or(0);
        let 结束时间 = self
            .单体分析器
            .get(&self.输入周期)
            .and_then(|o| o.普通K线序列.last())
            .map(|k| k.时间戳)
            .unwrap_or(0);
        let 标识 = self
            .单体分析器
            .get(&self.输入周期)
            .map(|o| o.符号.clone())
            .unwrap_or_default();

        let 周期 = self
            .单体分析器
            .get(&self.输入周期)
            .map(|o| o.周期)
            .unwrap_or_default();

        let 目录标识 = format!("RustM_{}:{}_{}_{}", 标识, 周期, 起始时间, 结束时间);
        let 保存路径 = 根目录.join(&目录标识);

        if let Err(e) = std::fs::create_dir_all(&保存路径) {
            eprintln!("创建目录失败: {} -> {}", 保存路径.display(), e);
            return;
        }

        for 周期 in &self.周期组 {
            if let Some(观察员) = self.单体分析器.get(周期) {
                观察员.测试_保存数据(Some(&保存路径.to_string_lossy()));
            }
        }

        println!("多级别数据拆分保存完成，目录：{}", 保存路径.display());
    }
}
