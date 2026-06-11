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
use crate::{error, warn};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

/// 立体分析器 — 多周期协调器
pub struct 立体分析器 {
    pub 周期组: Vec<i64>,
    输入周期: i64,
    pub K线合成器: K线合成器,
    pub 单体分析器: HashMap<i64, Arc<RwLock<观察者>>>,
}

impl 立体分析器 {
    /// 创建立体分析器 — 对应 Python 立体分析器.__init__
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

        let mut 单体分析器: HashMap<i64, Arc<RwLock<观察者>>> = HashMap::new();
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
            let mut guard = 显示观察员.write();
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
                .map(|o| o.read().缠论K线序列.clone())
                .unwrap_or_default();

            for &周期 in &周期组 {
                if 周期 != 显示周期
                    && let Some(观察员) = 单体分析器.get(&周期)
                {
                    观察员.write().基础缠K序列 = 显示缠K序列.clone();
                }
            }
        }

        // 对应 Python: K线合成器(符号, 周期组, self.__K线回调)
        let 单体分析器_回调 = 单体分析器.clone();
        let K线合成器 = K线合成器::new(
            符号.clone(),
            周期组.clone(),
            Some(Box::new(
                move |_信号类型: String, _标识: String, 周期: i64, 完成K线: K线| {
                    立体分析器::__K线回调_调度(&单体分析器_回调, 周期, 完成K线);
                },
            )),
        );

        Self {
            周期组,
            输入周期,
            K线合成器,
            单体分析器,
        }
    }

    /// __K线回调 — 对应 Python 立体分析器.__K线回调
    fn __K线回调(&self, _信号类型: String, _标识: String, 周期: i64, 完成K线: K线) {
        if let Some(观察员) = self.单体分析器.get(&周期) {
            let mut obs = 观察员.write();
            obs.增加原始K线(完成K线);
            // 对应 Python: if 当前K线 := self._K线合成器.获取当前K线(周期)
            // _完成K线刚清空当前K线，获取当前K线返回 None，所以这里不添加
        }
    }

    /// 静态调度版本 — 用于回调闭包
    fn __K线回调_调度(
        单体分析器: &HashMap<i64, Arc<RwLock<观察者>>>,
        周期: i64,
        完成K线: K线,
    ) {
        if let Some(观察员) = 单体分析器.get(&周期) {
            观察员.write().增加原始K线(完成K线);
        }
    }

    /// 投喂K线 — 对应 Python 立体分析器.投喂K线
    pub fn 投喂K线(&mut self, 普K: K线) {
        if 普K.周期 != self.输入周期 {
            panic!(
                "立体分析器.投喂K线 周期不匹配 {} != {}",
                普K.周期, self.输入周期
            );
        }
        self.K线合成器.投喂K线(普K);
    }

    /// 获取指定周期的观察者
    pub fn 获取观察者(&self, 周期: i64) -> Option<Arc<RwLock<观察者>>> {
        self.单体分析器.get(&周期).cloned()
    }

    /// 测试_保存数据 — 对应 Python 立体分析器.测试_保存数据
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
            .and_then(|o| o.read().普通K线序列.first().map(|k| k.时间戳))
            .unwrap_or(0);
        let 结束时间 = self
            .单体分析器
            .get(&self.输入周期)
            .and_then(|o| o.read().普通K线序列.last().map(|k| k.时间戳))
            .unwrap_or(0);
        let 标识 = self
            .单体分析器
            .get(&self.输入周期)
            .map(|o| o.read().符号.clone())
            .unwrap_or_default();
        let 周期 = self
            .单体分析器
            .get(&self.输入周期)
            .map(|o| o.read().周期)
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
                    .测试_保存数据(Some(&保存路径.to_string_lossy()));
            }
        }

        warn!("多级别数据拆分保存完成，目录：{}", 保存路径.display());
    }

    /// 相等 — 各周期观察者全量比对，对应 Python `立体分析器相等`
    pub fn 相等(&self, other: &Self, 浮点容差: f64) -> (bool, String) {
        let 标签 = format!("立体分析器校验[A={:?},B={:?}]", self.周期组, other.周期组);

        if self.周期组 != other.周期组 {
            return (false, format!("{标签}: 周期组不一致"));
        }

        for 周期 in &self.周期组 {
            let a_obs = match self.单体分析器.get(周期) {
                Some(o) => o.read(),
                None => return (false, format!("{标签}: 周期{周期} 观察者不存在 (A)")),
            };
            let b_obs = match other.单体分析器.get(周期) {
                Some(o) => o.read(),
                None => return (false, format!("{标签}: 周期{周期} 观察者不存在 (B)")),
            };
            let (eq, msg) = a_obs.相等(&b_obs, 浮点容差);
            if !eq {
                return (false, format!("{标签}: 周期{周期} >> {msg}"));
            }
        }

        (true, format!("{标签}：所有周期观察者全量校验全部一致"))
    }
}
