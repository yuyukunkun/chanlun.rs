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

use crate::business::observer::观察者;
use crate::business::synthesizer::K线合成器;
use crate::config::缠论配置;
use crate::kline::bar::K线;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;
use tracing::{error, info};

/// 立体分析器 — 多周期协调器
///
/// 包含一个K线合成器和每周期一个观察者。
/// 输入最小周期K线，合成大周期后分发到对应观察者。
pub struct 立体分析器 {
    pub 周期组: Vec<i64>,
    输入周期: i64,
    K线合成器: K线合成器,
    单体分析器: HashMap<i64, Arc<RwLock<观察者>>>,
}

impl 立体分析器 {
    /// 创建立体分析器，自动创建K线合成器 + 每周期一个观察者
    pub fn new(
        符号: String,
        周期组: Vec<i64>,
        配置: Option<缠论配置>,
        配置组: Option<HashMap<i64, 缠论配置>>,
    ) -> Self {
        let mut 周期组 = 周期组;
        周期组.sort();
        let 输入周期 = 周期组[0];
        let 显示周期 = 周期组[1];

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

        // 显示周期特殊配置
        {
            let 显示观察员 = 单体分析器.get(&显示周期).expect("显示周期观察者不存在");
            let mut guard = 显示观察员.write().unwrap();
            guard.配置.推送K线 = true;
            guard.配置.推送笔 = true;
            guard.配置.推送线段 = true;
            guard.配置.图表展示 = true;
            guard.重置基础序列();
        }

        // 非显示周期的基础缠K序列对齐至显示周期
        {
            let 显示缠K序列 = 单体分析器
                .get(&显示周期)
                .map(|o| o.read().unwrap().缠论K线序列.clone())
                .unwrap_or_default();

            for &周期 in &周期组 {
                if 周期 != 显示周期
                    && let Some(观察员) = 单体分析器.get(&周期)
                {
                    观察员.write().unwrap().基础缠K序列 = 显示缠K序列.clone();
                }
            }
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
            panic!(
                "立体分析器.投喂K线 周期不匹配 {} != {}",
                普K.周期, self.输入周期
            );
        }

        // Feed to synthesizer, get completion events
        let 完成事件 = self.K线合成器.投喂K线(普K);

        // Dispatch on completion events (matching Python's __K线回调)
        for (周期, 完成K线) in 完成事件 {
            if let Some(观察员) = self.单体分析器.get(&周期) {
                观察员.write().unwrap().增加原始K线(完成K线);
                if let Some(当前K线) = self.K线合成器.获取当前K线(周期) {
                    观察员.write().unwrap().增加原始K线(当前K线.clone());
                }
            }
        }
    }

    /// 获取指定周期的观察者
    pub fn 获取观察者(&self, 周期: i64) -> Option<Arc<RwLock<观察者>>> {
        self.单体分析器.get(&周期).cloned()
    }

    /// 测试_保存数据 — 多级别数据拆分保存
    /// 创建父目录 PyM_{标识}_{起始时间}_{结束时间}，各周期观察者保存到子目录
    pub fn 测试_保存数据(&self, root: Option<&str>) {
        let 根目录 = match root {
            Some(r) => std::path::PathBuf::from(r),
            None => std::env::var("CHANLUN_DATA_DIR")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|_| std::env::temp_dir()),
        };

        let 起始时间 = self
            .单体分析器
            .get(&self.输入周期)
            .and_then(|o| o.read().unwrap().普通K线序列.first().map(|k| k.时间戳))
            .unwrap_or(0);
        let 结束时间 = self
            .单体分析器
            .get(&self.输入周期)
            .and_then(|o| o.read().unwrap().普通K线序列.last().map(|k| k.时间戳))
            .unwrap_or(0);
        let 标识 = self
            .单体分析器
            .get(&self.输入周期)
            .map(|o| o.read().unwrap().符号.clone())
            .unwrap_or_default();

        let 周期 = self
            .单体分析器
            .get(&self.输入周期)
            .map(|o| o.read().unwrap().周期)
            .unwrap_or_default();

        let 目录标识 = format!("RustM_{}:{}_{}_{}", 标识, 周期, 起始时间, 结束时间);
        let 保存路径 = 根目录.join(&目录标识);

        if let Err(e) = std::fs::create_dir_all(&保存路径) {
            error!("创建目录失败: {} -> {}", 保存路径.display(), e);
            return;
        }

        for 周期 in &self.周期组 {
            if let Some(观察员) = self.单体分析器.get(周期) {
                观察员
                    .read()
                    .unwrap()
                    .测试_保存数据(Some(&保存路径.to_string_lossy()));
            }
        }

        info!("多级别数据拆分保存完成，目录：{}", 保存路径.display());
    }
}
