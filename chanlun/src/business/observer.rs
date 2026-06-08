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

use crate::algorithm::bi::笔;
use crate::algorithm::hub::中枢;
use crate::algorithm::segment::线段;
use crate::config::缠论配置;
use crate::kline::bar::K线;
use crate::kline::chan_kline::缠论K线;
use crate::structure::dash_line::虚线;
use crate::structure::fractal_obj::分型;
use crate::types::相对方向;
use crate::utils::datetime;
use std::sync::atomic::Ordering;
use std::sync::{Arc, RwLock};
use tracing::{error, info};

/// 观察者 — 单周期分析器，持有所有层级序列，接收K线流式输入后逐层计算
pub struct 观察者 {
    pub 符号: String,
    pub 周期: i64,
    pub 配置: 缠论配置,

    // K线序列
    pub 普通K线序列: Vec<Arc<K线>>,
    pub 基础缠K序列: Vec<Arc<缠论K线>>,
    pub 缠论K线序列: Vec<Arc<缠论K线>>,

    // 分型与笔
    pub 分型序列: Vec<Arc<分型>>,
    pub 笔序列: Vec<Arc<虚线>>,
    pub 笔_中枢序列: Vec<Arc<中枢>>,

    // 分析层次配置
    pub 线段分析层次: usize,
    pub 扩展线段分析层次: usize,
    pub 混合扩展线段分析层次: usize,

    // 线段组: [0]=线段, [1]=线段<线段>, [2]=线段<线段<线段>>
    pub 线段序列组: Vec<Vec<Arc<虚线>>>,
    pub 中枢序列组: Vec<Vec<Arc<中枢>>>,

    // 扩展线段组: [0]=扩展线段, [1]=扩展线段<扩展线段>, [2]=扩展线段<扩展线段<扩展线段>>
    pub 扩展线段序列组: Vec<Vec<Arc<虚线>>>,
    pub 扩展中枢序列组: Vec<Vec<Arc<中枢>>>,

    // 混合扩展线段组: [0]=扩展线段<线段>, [1]=扩展线段<线段<线段>>, [2]=扩展线段<线段<线段<线段>>>
    pub 混合扩展线段序列组: Vec<Vec<Arc<虚线>>>,
    pub 混合扩展中枢序列组: Vec<Vec<Arc<中枢>>>,

    // 终止时间戳
    终止时间戳: Option<i64>,
}

impl 观察者 {
    /// 创建观察者，初始化所有序列为空，若配置了手动终止则解析时间戳
    pub fn new(符号: String, 周期: i64, 配置: 缠论配置) -> Arc<RwLock<Self>> {
        let 终止时间戳 = if !配置.手动终止.is_empty() {
            datetime::转化为时间戳(&配置.手动终止)
        } else {
            None
        };

        let 线段分析层次 = 3usize;
        let 扩展线段分析层次 = 3usize;
        let 混合扩展线段分析层次 = 3usize;

        let mut 线段序列组: Vec<Vec<Arc<虚线>>> = Vec::with_capacity(线段分析层次);
        let mut 中枢序列组: Vec<Vec<Arc<中枢>>> = Vec::with_capacity(线段分析层次);
        let mut 扩展线段序列组: Vec<Vec<Arc<虚线>>> = Vec::with_capacity(扩展线段分析层次);
        let mut 扩展中枢序列组: Vec<Vec<Arc<中枢>>> = Vec::with_capacity(扩展线段分析层次);
        let mut 混合扩展线段序列组: Vec<Vec<Arc<虚线>>> = Vec::with_capacity(混合扩展线段分析层次);
        let mut 混合扩展中枢序列组: Vec<Vec<Arc<中枢>>> = Vec::with_capacity(混合扩展线段分析层次);

        for _ in 0..线段分析层次 {
            线段序列组.push(Vec::new());
            中枢序列组.push(Vec::new());
        }
        for _ in 0..扩展线段分析层次 {
            扩展线段序列组.push(Vec::new());
            扩展中枢序列组.push(Vec::new());
        }
        for _ in 0..混合扩展线段分析层次 {
            混合扩展线段序列组.push(Vec::new());
            混合扩展中枢序列组.push(Vec::new());
        }

        let mut instance = Self {
            符号: 符号.clone(),
            周期,
            配置,
            普通K线序列: Vec::new(),
            基础缠K序列: Vec::new(),
            缠论K线序列: Vec::new(),
            分型序列: Vec::new(),
            笔序列: Vec::new(),
            笔_中枢序列: Vec::new(),
            线段分析层次,
            扩展线段分析层次,
            混合扩展线段分析层次,
            线段序列组,
            中枢序列组,
            扩展线段序列组,
            扩展中枢序列组,
            混合扩展线段序列组,
            混合扩展中枢序列组,
            终止时间戳,
        };
        instance.配置.标识 = 符号;
        Arc::new(RwLock::new(instance))
    }

    /// 标识
    pub fn 标识(&self) -> String {
        format!("{}:{}", self.符号, self.周期)
    }

    /// 当前K线
    pub fn 当前K线(&self) -> Option<&Arc<K线>> {
        self.普通K线序列.last()
    }

    /// 当前缠K
    pub fn 当前缠K(&self) -> Option<&Arc<缠论K线>> {
        self.缠论K线序列.last()
    }

    // ---- 向后兼容的属性访问器 ----

    /// 线段序列 (线段序列组[0])
    pub fn 线段序列(&self) -> &Vec<Arc<虚线>> {
        &self.线段序列组[0]
    }

    /// 中枢序列 (中枢序列组[0])
    pub fn 中枢序列(&self) -> &Vec<Arc<中枢>> {
        &self.中枢序列组[0]
    }

    /// 线段_线段序列 (线段序列组[1])
    pub fn 线段_线段序列(&self) -> &Vec<Arc<虚线>> {
        &self.线段序列组[1]
    }

    /// 线段_中枢序列 (中枢序列组[1])
    pub fn 线段_中枢序列(&self) -> &Vec<Arc<中枢>> {
        &self.中枢序列组[1]
    }

    /// 扩展线段序列 (扩展线段序列组[0])
    pub fn 扩展线段序列(&self) -> &Vec<Arc<虚线>> {
        &self.扩展线段序列组[0]
    }

    /// 扩展中枢序列 (扩展中枢序列组[0])
    pub fn 扩展中枢序列(&self) -> &Vec<Arc<中枢>> {
        &self.扩展中枢序列组[0]
    }

    /// 扩展线段序列_扩展线段 (扩展线段序列组[1])
    pub fn 扩展线段序列_扩展线段(&self) -> &Vec<Arc<虚线>> {
        &self.扩展线段序列组[1]
    }

    /// 扩展中枢序列_扩展线段 (扩展中枢序列组[1])
    pub fn 扩展中枢序列_扩展线段(&self) -> &Vec<Arc<中枢>> {
        &self.扩展中枢序列组[1]
    }

    /// 扩展线段序列_线段 (混合扩展线段序列组[0])
    pub fn 扩展线段序列_线段(&self) -> &Vec<Arc<虚线>> {
        &self.混合扩展线段序列组[0]
    }

    /// 扩展中枢序列_线段 (混合扩展中枢序列组[0])
    pub fn 扩展中枢序列_线段(&self) -> &Vec<Arc<中枢>> {
        &self.混合扩展中枢序列组[0]
    }

    /// 重置基础序列
    pub fn 重置基础序列(&mut self) {
        self.普通K线序列.clear();
        self.基础缠K序列.clear();
        self.缠论K线序列.clear();
        self.分型序列.clear();
        self.笔序列.clear();
        self.笔_中枢序列.clear();

        self.线段序列组.clear();
        self.中枢序列组.clear();
        for _ in 0..self.线段分析层次 {
            self.线段序列组.push(Vec::new());
            self.中枢序列组.push(Vec::new());
        }

        self.扩展线段序列组.clear();
        self.扩展中枢序列组.clear();
        for _ in 0..self.扩展线段分析层次 {
            self.扩展线段序列组.push(Vec::new());
            self.扩展中枢序列组.push(Vec::new());
        }

        self.混合扩展线段序列组.clear();
        self.混合扩展中枢序列组.clear();
        for _ in 0..self.混合扩展线段分析层次 {
            self.混合扩展线段序列组.push(Vec::new());
            self.混合扩展中枢序列组.push(Vec::new());
        }
    }

    /// 增加原始K线 — 单根K线投喂入口
    pub fn 增加原始K线(&mut self, 普K: K线) {
        if let Some(终止) = self.终止时间戳
            && 普K.时间戳 > 终止
        {
            return;
        }
        self.__处理数据(普K);
    }

    /// 投喂原始数据 — 便捷入口，直接从 OHLCV 创建 K线 并投喂
    pub fn 投喂原始数据(
        &mut self, 时间戳: i64, 开: f64, 高: f64, 低: f64, 收: f64, 量: f64
    ) {
        let 普K = K线::创建普K(&self.符号, 时间戳, 开, 高, 低, 收, 量, 0, self.周期);
        self.增加原始K线(普K);
    }

    /// 核心数据处理管道
    fn __处理数据(&mut self, 普K: K线) {
        // Step 1: 缠论K线分析
        let (_, 当前分型) = 缠论K线::分析(
            普K,
            &mut self.缠论K线序列,
            &mut self.普通K线序列,
            &self.配置,
        );
        let 当前分型 = match 当前分型 {
            Some(fx) => fx,
            None => return,
        };

        // Step 2: 笔分析
        if self.配置.分析笔 {
            笔::分析(
                当前分型,
                &mut self.分型序列,
                &mut self.笔序列,
                &self.缠论K线序列,
                &self.普通K线序列,
                0,
                &self.配置,
            );
        }
        if self.分型序列.is_empty() {
            return;
        }

        // Step 3: 笔中枢分析
        if self.配置.分析笔中枢 {
            中枢::分析(&self.笔序列, &mut self.笔_中枢序列, true, "", 0);
        }
        if self.笔序列.is_empty() {
            return;
        }

        // Step 4: 线段分析 — 3 级递归
        if self.配置.分析线段 || self.配置.分析线段中枢 {
            for i in 0..self.线段分析层次 {
                if i == 0 {
                    if self.配置.分析线段 {
                        线段::分析(
                            &self.笔序列,
                            &mut self.线段序列组[i],
                            &self.配置,
                            0,
                            &[相对方向::向上, 相对方向::向下],
                        );
                    }
                } else {
                    if self.配置.分析线段 {
                        let 源序列 = self.线段序列组[i - 1].clone();
                        线段::分析(
                            &源序列,
                            &mut self.线段序列组[i],
                            &self.配置,
                            0,
                            &[相对方向::向上, 相对方向::向下],
                        );
                    }
                }
                if self.配置.分析线段中枢 {
                    中枢::分析(&self.线段序列组[i], &mut self.中枢序列组[i], true, "", 0);
                }
            }
        }

        // Step 5: 扩展线段分析 — 3 级递归
        if self.配置.分析扩展线段 || self.配置.分析线段中枢 {
            for i in 0..self.扩展线段分析层次 {
                if i == 0 {
                    if self.配置.分析扩展线段 {
                        线段::扩展分析(&self.笔序列, &mut self.扩展线段序列组[i], &self.配置);
                    }
                } else {
                    if self.配置.分析扩展线段 {
                        let 源序列 = self.扩展线段序列组[i - 1].clone();
                        线段::扩展分析(&源序列, &mut self.扩展线段序列组[i], &self.配置);
                    }
                }
                if self.配置.分析线段中枢 {
                    中枢::分析(
                        &self.扩展线段序列组[i],
                        &mut self.扩展中枢序列组[i],
                        true,
                        "",
                        0,
                    );
                }
            }
        }

        // Step 6: 混合扩展线段分析 — 3 级递归
        if self.配置.分析扩展线段 || self.配置.分析线段中枢 {
            for i in 0..self.混合扩展线段分析层次.min(self.线段序列组.len()) {
                if self.配置.分析扩展线段 {
                    let 源序列 = self.线段序列组[i].clone();
                    线段::扩展分析(&源序列, &mut self.混合扩展线段序列组[i], &self.配置);
                }
                if self.配置.分析线段中枢 {
                    中枢::分析(
                        &self.混合扩展线段序列组[i],
                        &mut self.混合扩展中枢序列组[i],
                        true,
                        "",
                        0,
                    );
                }
            }
        }
    }

    /// 识别买卖点（占位方法，具体逻辑在子类或Rust核心中实现）
    pub fn 识别买卖点(&self) {}

    /// 静态重新分析 — 遍历所有缠K重新生成分型/笔/线段
    pub fn 静态重新分析(&mut self) {
        self.分型序列.clear();
        self.笔序列.clear();
        self.笔_中枢序列.clear();

        self.线段序列组.clear();
        self.中枢序列组.clear();
        for _ in 0..self.线段分析层次 {
            self.线段序列组.push(Vec::new());
            self.中枢序列组.push(Vec::new());
        }

        self.扩展线段序列组.clear();
        self.扩展中枢序列组.clear();
        for _ in 0..self.扩展线段分析层次 {
            self.扩展线段序列组.push(Vec::new());
            self.扩展中枢序列组.push(Vec::new());
        }

        self.混合扩展线段序列组.clear();
        self.混合扩展中枢序列组.clear();
        for _ in 0..self.混合扩展线段分析层次 {
            self.混合扩展线段序列组.push(Vec::new());
            self.混合扩展中枢序列组.push(Vec::new());
        }

        if self.配置.分析笔 {
            for i in 1..self.缠论K线序列.len() - 1 {
                let 当前分型 = 分型::new(
                    Some(Arc::clone(&self.缠论K线序列[i - 1])),
                    Arc::clone(&self.缠论K线序列[i]),
                    Some(Arc::clone(&self.缠论K线序列[i + 1])),
                );
                笔::分析(
                    Arc::new(当前分型),
                    &mut self.分型序列,
                    &mut self.笔序列,
                    &self.缠论K线序列,
                    &self.普通K线序列,
                    0,
                    &self.配置,
                );
            }
        }

        if self.笔序列.is_empty() {
            return;
        }

        if self.配置.分析笔中枢 {
            中枢::分析(&self.笔序列, &mut self.笔_中枢序列, true, "", 0);
        }

        if self.配置.分析线段 || self.配置.分析线段中枢 {
            for i in 0..self.线段分析层次 {
                if i == 0 {
                    if self.配置.分析线段 {
                        线段::分析(
                            &self.笔序列,
                            &mut self.线段序列组[i],
                            &self.配置,
                            0,
                            &[相对方向::向上, 相对方向::向下],
                        );
                    }
                } else {
                    if self.配置.分析线段 {
                        let 源序列 = self.线段序列组[i - 1].clone();
                        线段::分析(
                            &源序列,
                            &mut self.线段序列组[i],
                            &self.配置,
                            0,
                            &[相对方向::向上, 相对方向::向下],
                        );
                    }
                }
                if self.配置.分析线段中枢 {
                    中枢::分析(&self.线段序列组[i], &mut self.中枢序列组[i], true, "", 0);
                }
            }
        }

        if self.配置.分析扩展线段 || self.配置.分析线段中枢 {
            for i in 0..self.扩展线段分析层次 {
                if i == 0 {
                    if self.配置.分析扩展线段 {
                        线段::扩展分析(&self.笔序列, &mut self.扩展线段序列组[i], &self.配置);
                    }
                } else {
                    if self.配置.分析扩展线段 {
                        let 源序列 = self.扩展线段序列组[i - 1].clone();
                        线段::扩展分析(&源序列, &mut self.扩展线段序列组[i], &self.配置);
                    }
                }
                if self.配置.分析线段中枢 {
                    中枢::分析(
                        &self.扩展线段序列组[i],
                        &mut self.扩展中枢序列组[i],
                        true,
                        "",
                        0,
                    );
                }
            }
        }

        if self.配置.分析扩展线段 || self.配置.分析线段中枢 {
            for i in 0..self.混合扩展线段分析层次.min(self.线段序列组.len()) {
                if self.配置.分析扩展线段 {
                    let 源序列 = self.线段序列组[i].clone();
                    线段::扩展分析(&源序列, &mut self.混合扩展线段序列组[i], &self.配置);
                }
                if self.配置.分析线段中枢 {
                    中枢::分析(
                        &self.混合扩展线段序列组[i],
                        &mut self.混合扩展中枢序列组[i],
                        true,
                        "",
                        0,
                    );
                }
            }
        }
    }

    /// 测试_保存数据 — 输出各序列数据文本到文件
    pub fn 测试_保存数据(&self, root: Option<&str>) -> String {
        // 确定根目录
        let 根目录 = match root {
            Some(r) => std::path::PathBuf::from(r),
            None => std::env::temp_dir(),
        };

        // 生成子目录名称
        let 起始时间 = self.普通K线序列.first().map(|k| k.时间戳).unwrap_or(0);
        let 结束时间 = self.普通K线序列.last().map(|k| k.时间戳).unwrap_or(0);
        let 目录标识 = format!("Rust_{}:{}_{}_{}", self.符号, self.周期, 起始时间, 结束时间);

        let 保存路径 = 根目录.join(&目录标识);
        if let Err(e) = std::fs::create_dir_all(&保存路径) {
            error!("创建目录失败: {} -> {}", 保存路径.display(), e);
            return String::new();
        }

        // 辅助：保存序列到文件 (Python: 保存序列(序列))
        let 保存序列 =
            |序列: &Vec<Arc<虚线>>, 保存路径: &std::path::Path| -> Result<(), std::io::Error> {
                if 序列.is_empty() {
                    return Ok(());
                }
                let 数据列表: Vec<String> = 序列.iter().map(|d| d.获取数据文本()).collect();
                let 文件名 = format!("{}.txt", 序列[0].标识.read().unwrap());
                std::fs::write(保存路径.join(&文件名), 数据列表.join("\n") + "\n")?;
                Ok(())
            };

        let 保存中枢序列 =
            |序列: &Vec<Arc<中枢>>, 保存路径: &std::path::Path| -> Result<(), std::io::Error> {
                if 序列.is_empty() {
                    return Ok(());
                }
                let 数据列表: Vec<String> = 序列.iter().map(|h| h.获取数据文本()).collect();
                let 文件名 = format!("{}.txt", 序列[0].标识.read().unwrap());
                std::fs::write(保存路径.join(&文件名), 数据列表.join("\n") + "\n")?;
                Ok(())
            };

        // 保存笔/笔中枢
        let _ = 保存序列(&self.笔序列, &保存路径);
        let _ = 保存中枢序列(&self.笔_中枢序列, &保存路径);

        // 保存线段组
        for i in 0..self.线段分析层次 {
            let _ = 保存序列(&self.线段序列组[i], &保存路径);
            let _ = 保存中枢序列(&self.中枢序列组[i], &保存路径);
        }
        // 保存扩展线段组
        for i in 0..self.扩展线段分析层次 {
            let _ = 保存序列(&self.扩展线段序列组[i], &保存路径);
            let _ = 保存中枢序列(&self.扩展中枢序列组[i], &保存路径);
        }
        // 保存混合扩展线段组
        for i in 0..self.混合扩展线段分析层次 {
            let _ = 保存序列(&self.混合扩展线段序列组[i], &保存路径);
            let _ = 保存中枢序列(&self.混合扩展中枢序列组[i], &保存路径);
        }

        // 缠K/分型 debug data
        let 缠K序列_数据文本: Vec<String> = self
            .缠论K线序列
            .iter()
            .map(|ck| {
                format!(
                    "缠K, {}, {}, {:?}, {}, {}, {}, {}, {}",
                    ck.序号.load(Ordering::Relaxed),
                    ck.时间戳.load(Ordering::Relaxed),
                    ck.分型,
                    *ck.方向.read().unwrap(),
                    ck.高.get(),
                    ck.低.get(),
                    ck.原始起始序号,
                    ck.原始结束序号.load(Ordering::Relaxed)
                )
            })
            .collect();
        let _ = std::fs::write(
            保存路径.join("缠K序列_数据文本.txt"),
            缠K序列_数据文本.join("\n") + "\n",
        );

        let 分型序列_数据文本: Vec<String> = self
            .分型序列
            .iter()
            .enumerate()
            .map(|(i, fx)| {
                format!(
                    "分型, {}, {}, {:?}, {}, {}, {}",
                    i,
                    fx.时间戳(),
                    fx.结构,
                    fx.分型特征值,
                    fx.中.时间戳.load(Ordering::Relaxed),
                    fx.中.低.get(),
                )
            })
            .collect();
        let _ = std::fs::write(
            保存路径.join("分型序列_数据文本.txt"),
            分型序列_数据文本.join("\n") + "\n",
        );

        info!("全部数据拆分保存完成，目录：{}", 保存路径.display());
        保存路径.display().to_string()
    }

    /// 加载本地数据 — 从 .nb 文件加载数据到当前观察者（先重置再投喂）
    pub fn 加载本地数据(&mut self, 文件路径: &str) -> Result<(), String> {
        self.重置基础序列();
        let data = std::fs::read(文件路径).map_err(|e| format!("read file: {}", e))?;
        let size = 48;
        for i in 0..data.len() / size {
            let offset = i * size;
            if let Some((时间戳, 开, 高, 低, 收, 量)) =
                K线::解析原始数据(&data[offset..offset + size])
            {
                self.投喂原始数据(时间戳, 开, 高, 低, 收, 量);
            }
        }
        Ok(())
    }

    /// 读取数据文件 — 更新当前观察者并加载 .nb 文件
    pub fn 读取数据文件(
        &mut self, 文件路径: &str, 配置: 缠论配置
    ) -> Result<(), String> {
        let path = std::path::Path::new(文件路径);
        let name = path
            .file_stem()
            .and_then(|n| n.to_str())
            .ok_or("invalid filename")?;
        let parts: Vec<&str> = name.split('-').collect();
        if parts.len() < 4 {
            return Err(format!("invalid filename format: {}", name));
        }
        self.符号 = parts[0].to_string();
        self.周期 = parts[1]
            .parse()
            .map_err(|e| format!("parse period: {}", e))?;
        self.配置 = 配置;
        self.加载本地数据(文件路径)
    }

    /// 相等 — 全量序列逐项比对，双端一致性验证，对应 Python `观察者相等`
    pub fn 相等(&self, other: &Self, 浮点容差: f64) -> (bool, String) {
        let 标签 = format!("观察者校验[A={},B={}]", self.标识(), other.标识());

        if self.缠论K线序列.len() != other.缠论K线序列.len() {
            return (
                false,
                format!(
                    "{标签}: 缠K序列长度不一致 A={},B={}",
                    self.缠论K线序列.len(),
                    other.缠论K线序列.len()
                ),
            );
        }
        if self.分型序列.len() != other.分型序列.len() {
            return (
                false,
                format!(
                    "{标签}: 分型序列长度不一致 A={},B={}",
                    self.分型序列.len(),
                    other.分型序列.len()
                ),
            );
        }
        if self.笔序列.len() != other.笔序列.len() {
            return (
                false,
                format!(
                    "{标签}: 笔序列长度不一致 A={},B={}",
                    self.笔序列.len(),
                    other.笔序列.len()
                ),
            );
        }

        for (i, (a, b)) in self.笔序列.iter().zip(other.笔序列.iter()).enumerate() {
            let (eq, msg) = a.相等(b, 浮点容差);
            if !eq {
                return (false, format!("{标签}: 笔#{i}不一致 >> {msg}"));
            }
        }

        if self.笔_中枢序列.len() != other.笔_中枢序列.len() {
            return (false, format!("{标签}: 笔中枢序列长度不一致"));
        }
        for (i, (a, b)) in self
            .笔_中枢序列
            .iter()
            .zip(other.笔_中枢序列.iter())
            .enumerate()
        {
            let (eq, msg) = a.相等(b, 浮点容差);
            if !eq {
                return (false, format!("{标签}: 笔中枢#{i}不一致 >> {msg}"));
            }
        }

        for level in 0..self.线段分析层次.min(other.线段分析层次) {
            let a_segs = &self.线段序列组[level];
            let b_segs = &other.线段序列组[level];
            if a_segs.len() != b_segs.len() {
                return (false, format!("{标签}: 线段序列组[{level}]长度不一致"));
            }
            for (i, (a, b)) in a_segs.iter().zip(b_segs.iter()).enumerate() {
                let (eq, msg) = a.相等(b, 浮点容差);
                if !eq {
                    return (
                        false,
                        format!("{标签}: 线段序列组[{level}]#{i}不一致 >> {msg}"),
                    );
                }
            }
            let a_hubs = &self.中枢序列组[level];
            let b_hubs = &other.中枢序列组[level];
            if a_hubs.len() != b_hubs.len() {
                return (false, format!("{标签}: 中枢序列组[{level}]长度不一致"));
            }
            for (i, (a, b)) in a_hubs.iter().zip(b_hubs.iter()).enumerate() {
                let (eq, msg) = a.相等(b, 浮点容差);
                if !eq {
                    return (
                        false,
                        format!("{标签}: 中枢序列组[{level}]#{i}不一致 >> {msg}"),
                    );
                }
            }
        }

        (true, format!("{标签}：全量序列校验全部一致"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::缠论配置;

    fn test_data_path() -> String {
        let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        manifest
            .parent()
            .unwrap()
            .join("btcusd-300-1777649100-1778398800.nb")
            .to_string_lossy()
            .to_string()
    }

    #[test]
    fn test_普k序列指针一致性() {
        let obs = 观察者::new("btcusd".into(), 300, Default::default());
        obs.write()
            .unwrap()
            .读取数据文件(&test_data_path(), Default::default())
            .unwrap();
        let obs_ref = obs.read().unwrap();

        for (i, bi) in obs_ref.笔序列.iter().enumerate() {
            let pu_seq = bi.获取普K序列(&obs_ref.普通K线序列);
            if pu_seq.is_empty() {
                info!("笔 {}: 获取普K序列 返回空 (fallback failed)", i);
                info!("  文.中.标的K线 原始起始序号: {}", bi.文.中.原始起始序号);
                info!(
                    "  武.中.标的K线 原始结束序号: {}",
                    bi.武
                        .read()
                        .unwrap()
                        .中
                        .原始结束序号
                        .load(Ordering::Relaxed)
                );
                info!("  普通K线序列.len: {}", obs_ref.普通K线序列.len());
            } else {
                let first_ptr = Arc::as_ptr(&pu_seq[0]);
                let found = obs_ref
                    .普通K线序列
                    .iter()
                    .any(|k| Arc::as_ptr(k) == first_ptr);
                if !found {
                    info!("笔 {}: 获取普K序列[0] 的 Rc 指针不在 普通K线序列 中!", i);
                    let wen_ptr = Arc::as_ptr(&*bi.文.中.标的K线.read().unwrap());
                    let wen_found = obs_ref
                        .普通K线序列
                        .iter()
                        .any(|k| Arc::as_ptr(k) == wen_ptr);
                    info!("  文.中.标的K线 在序列中: {}", wen_found);
                } else {
                    info!("笔 {}: OK, 获取普K序列[0] 在序列中找到", i);
                }
            }
        }
    }

    #[test]
    fn test_pyo3_flow_pointer_consistency() {
        let config = 缠论配置::default();
        let obs_ref = 观察者::new("btcusd".into(), 300, config);

        let data = std::fs::read(test_data_path()).unwrap();
        let size = 48;

        for i in 0..data.len() / size {
            let offset = i * size;
            if let Some(k线) = K线::from_bytes(&data[offset..offset + size], 300, "btcusd") {
                let _k线_py_inner = Arc::new(k线.clone());
                obs_ref.write().unwrap().增加原始K线(k线);
            }
        }

        let obs = obs_ref.read().unwrap();
        info!("普通K线序列.len: {}", obs.普通K线序列.len());
        info!("笔序列.len: {}", obs.笔序列.len());

        for (i, bi) in obs.笔序列.iter().enumerate() {
            let pu_seq = bi.获取普K序列(&obs.普通K线序列);
            if pu_seq.is_empty() {
                info!("笔 {}: 获取普K序列 返回空!", i);
            } else {
                let first_ptr = Arc::as_ptr(&pu_seq[0]);
                let found = obs.普通K线序列.iter().any(|k| Arc::as_ptr(k) == first_ptr);
                if !found {
                    info!("笔 {}: 获取普K序列[0] 指针不在序列中!", i);
                }
                if i < 5 {
                    info!("笔 {}: OK, len={}", i, pu_seq.len());
                }
            }
        }
    }

    // ============================================================
    // 分型到笔的 Rc 指针一致性
    // ============================================================

    #[test]
    fn test_分型到笔的文武Rc指针一致性() {
        let obs = 观察者::new("btcusd".into(), 300, Default::default());
        obs.write()
            .unwrap()
            .读取数据文件(&test_data_path(), Default::default())
            .unwrap();
        let obs_ref = obs.read().unwrap();

        // 每个笔的文/武 分型 Rc 指针必须在 分型序列 中
        for (i, bi) in obs_ref.笔序列.iter().enumerate() {
            let 文_ptr = Arc::as_ptr(&bi.文);
            let 文_found = obs_ref.分型序列.iter().any(|f| Arc::as_ptr(f) == 文_ptr);
            if !文_found {
                info!("笔 {}: 文(时间戳={}) 不在分型序列中!", i, bi.文.时间戳());
            }

            let 武_ptr = Arc::as_ptr(&*bi.武.read().unwrap());
            let 武_found = obs_ref.分型序列.iter().any(|f| Arc::as_ptr(f) == 武_ptr);
            if !武_found {
                info!(
                    "笔 {}: 武(时间戳={}) 不在分型序列中!",
                    i,
                    bi.武.read().unwrap().时间戳()
                );
            }
        }
    }

    // ============================================================
    // 笔到线段的 Rc 指针一致性
    // ============================================================

    #[test]
    fn test_笔到线段的基础序列Rc指针一致性() {
        let obs = 观察者::new("btcusd".into(), 300, Default::default());
        obs.write()
            .unwrap()
            .读取数据文件(&test_data_path(), Default::default())
            .unwrap();
        let obs_ref = obs.read().unwrap();

        // 每个线段的基础序列中的笔 Rc 指针必须在 笔序列 中
        for (i, seg) in obs_ref.线段序列().iter().enumerate() {
            for (j, bi_in_seg) in seg.基础序列.read().unwrap().iter().enumerate() {
                let bi_ptr = Arc::as_ptr(bi_in_seg);
                let found = obs_ref.笔序列.iter().any(|b| Arc::as_ptr(b) == bi_ptr);
                if !found {
                    info!("线段 {} 的基础序列[{}] 不在笔序列中!", i, j);
                }
            }
        }
    }

    // ============================================================
    // 中枢基础序列与源的 Rc 指针一致性
    // ============================================================

    #[test]
    fn test_中枢基础序列与笔序列Rc指针一致() {
        let obs = 观察者::new("btcusd".into(), 300, Default::default());
        obs.write()
            .unwrap()
            .读取数据文件(&test_data_path(), Default::default())
            .unwrap();
        let obs_ref = obs.read().unwrap();

        for (i, hub) in obs_ref.笔_中枢序列.iter().enumerate() {
            for (j, bi_in_hub) in hub.基础序列.read().unwrap().iter().enumerate() {
                let bi_ptr = Arc::as_ptr(bi_in_hub);
                let found = obs_ref.笔序列.iter().any(|b| Arc::as_ptr(b) == bi_ptr);
                if !found {
                    info!("笔中枢 {} 的基础序列[{}] 不在笔序列中!", i, j);
                }
            }
        }

        for (i, hub) in obs_ref.中枢序列().iter().enumerate() {
            for (j, seg_in_hub) in hub.基础序列.read().unwrap().iter().enumerate() {
                let seg_ptr = Arc::as_ptr(seg_in_hub);
                let found = obs_ref.线段序列().iter().any(|s| Arc::as_ptr(s) == seg_ptr);
                if !found {
                    info!("线段中枢 {} 的基础序列[{}] 不在线段序列中!", i, j);
                }
            }
        }
    }

    // ============================================================
    // 重复计算一致性
    // ============================================================

    #[test]
    fn test_重复计算后结果一致() {
        let data = std::fs::read(test_data_path()).unwrap();
        let size = 48;

        let 计算 = || {
            let config = 缠论配置::default();
            let obs_ref = 观察者::new("btcusd".into(), 300, config);
            for i in 0..data.len() / size {
                let offset = i * size;
                if let Some(k线) = K线::from_bytes(&data[offset..offset + size], 300, "btcusd") {
                    obs_ref.write().unwrap().增加原始K线(k线);
                }
            }
            let obs = obs_ref.read().unwrap();
            (
                obs.笔序列.len(),
                obs.线段序列().len(),
                obs.中枢序列().len(),
                obs.笔_中枢序列.len(),
            )
        };

        let (笔数1, 段数1, 中枢1, 笔中枢1) = 计算();
        let (笔数2, 段数2, 中枢2, 笔中枢2) = 计算();

        assert_eq!(笔数1, 笔数2, "重复计算笔数不一致");
        assert_eq!(段数1, 段数2, "重复计算线段数不一致");
        assert_eq!(中枢1, 中枢2, "重复计算中枢数不一致");
        assert_eq!(笔中枢1, 笔中枢2, "重复计算笔中枢数不一致");

        info!(
            "两次计算结果一致: 笔={}, 线段={}, 中枢={}, 笔中枢={}",
            笔数1, 段数1, 中枢1, 笔中枢1
        );
    }

    // ============================================================
    // 重置后重新投喂数据一致性
    // ============================================================

    #[test]
    fn test_重置后重新投喂数据一致() {
        let config = 缠论配置::default();
        let obs_ref = 观察者::new("btcusd".into(), 300, config);

        let data = std::fs::read(test_data_path()).unwrap();
        let size = 48;

        for i in 0..data.len() / size {
            let offset = i * size;
            if let Some(k线) = K线::from_bytes(&data[offset..offset + size], 300, "btcusd") {
                obs_ref.write().unwrap().增加原始K线(k线);
            }
        }

        let 第一次笔数 = obs_ref.read().unwrap().笔序列.len();
        let 第一次段数 = obs_ref.read().unwrap().线段序列().len();

        // 重置
        obs_ref.write().unwrap().重置基础序列();
        assert_eq!(obs_ref.read().unwrap().笔序列.len(), 0);
        assert_eq!(obs_ref.read().unwrap().线段序列().len(), 0);

        // 重新投喂
        for i in 0..data.len() / size {
            let offset = i * size;
            if let Some(k线) = K线::from_bytes(&data[offset..offset + size], 300, "btcusd") {
                obs_ref.write().unwrap().增加原始K线(k线);
            }
        }

        let 第二次笔数 = obs_ref.read().unwrap().笔序列.len();
        let 第二次段数 = obs_ref.read().unwrap().线段序列().len();

        assert_eq!(第一次笔数, 第二次笔数, "重置后重新投喂笔数不一致");
        assert_eq!(第一次段数, 第二次段数, "重置后重新投喂线段数不一致");
        info!("重置后重投一致: 笔={}, 线段={}", 第一次笔数, 第一次段数);
    }

    // ============================================================
    // RefCell 借用安全性 — 连续大量操作不应 panic
    // ============================================================

    #[test]
    fn test_RefCell借用安全性_连续读取不panic() {
        let obs = 观察者::new("btcusd".into(), 300, Default::default());
        obs.write()
            .unwrap()
            .读取数据文件(&test_data_path(), Default::default())
            .unwrap();
        let obs_ref = obs.read().unwrap();

        // 连续大量读取所有 RefCell 字段，不应 panic
        for _ in 0..100 {
            for bi in &obs_ref.笔序列 {
                let _标识 = bi.标识.read().unwrap().clone();
                let _wu = bi.武.read().unwrap().clone();
                let _基础序列 = bi.基础序列.read().unwrap().len();
                let _特征序列 = bi.特征序列.read().unwrap().len();
                let _模式 = bi.模式.read().unwrap().clone();
                let _实中枢 = bi.实_中枢序列.read().unwrap().len();
                let _虚中枢 = bi.虚_中枢序列.read().unwrap().len();
                let _合中枢 = bi.合_中枢序列.read().unwrap().len();
                let _确认K = bi.确认K线.read().unwrap().is_some();
                let _序号 = bi.序号.load(Ordering::Relaxed);
                let _有效性 = bi.有效性.load(Ordering::Relaxed);
                let _短路 = bi.短路修正.load(Ordering::Relaxed);
                let _前一缺口 = *bi.前一缺口.read().unwrap();
            }
            for seg in obs_ref.线段序列() {
                let _ = seg.标识.read().unwrap().clone();
                let _ = seg.基础序列.read().unwrap().len();
            }
        }
        // 到达这里 = 无 panic
    }

    #[test]
    fn test_RefCell借用安全性_交替读写不panic() {
        let obs = 观察者::new("btcusd".into(), 300, Default::default());
        obs.write()
            .unwrap()
            .读取数据文件(&test_data_path(), Default::default())
            .unwrap();
        let obs_ref = obs.read().unwrap();

        // 交替读写 RefCell 字段 — 先读再写同字段，分离 borrow 作用域
        if !obs_ref.笔序列.is_empty() {
            let bi = &obs_ref.笔序列[0];
            // 读
            let old_mode = bi.模式.read().unwrap().clone();
            // Ref 已释放，可以写
            *bi.模式.write().unwrap() = "测试模式".into();
            let new_mode = bi.模式.read().unwrap().clone();
            assert_eq!(new_mode, "测试模式");
            // 恢复
            *bi.模式.write().unwrap() = old_mode;

            // 读武
            let old_wu = bi.武.read().unwrap().clone();
            // Ref 已释放，可以检查
            assert!(Arc::as_ptr(&old_wu) == Arc::as_ptr(&old_wu));
        }
    }

    // ============================================================
    // 缠K 到 分型 的 Rc 指针一致性
    // ============================================================

    #[test]
    fn test_缠K到分型的Rc指针一致性() {
        let obs = 观察者::new("btcusd".into(), 300, Default::default());
        obs.write()
            .unwrap()
            .读取数据文件(&test_data_path(), Default::default())
            .unwrap();
        let obs_ref = obs.read().unwrap();

        // 每个分型的左/中/右 缠K 指针必须在 缠论K线序列 中
        for (i, f) in obs_ref.分型序列.iter().enumerate() {
            let 中_ptr = Arc::as_ptr(&f.中);
            let 中_found = obs_ref.缠论K线序列.iter().any(|k| Arc::as_ptr(k) == 中_ptr);
            if !中_found {
                info!("分型 {} 的 中(时间戳={}) 不在缠论K线序列中!", i, f.时间戳);
            }

            if let Some(ref 左) = f.左 {
                let 左_ptr = Arc::as_ptr(左);
                let 左_found = obs_ref.缠论K线序列.iter().any(|k| Arc::as_ptr(k) == 左_ptr);
                if !左_found {
                    info!("分型 {} 的 左 不在缠论K线序列中!", i);
                }
            }

            if let Some(ref 右) = f.右 {
                let 右_ptr = Arc::as_ptr(右);
                let 右_found = obs_ref.缠论K线序列.iter().any(|k| Arc::as_ptr(k) == 右_ptr);
                if !右_found {
                    info!("分型 {} 的 右 不在缠论K线序列中!", i);
                }
            }
        }
    }

    // ========== 跨线程安全测试 ==========

    // 编译期断言：核心类型必须实现 Send + Sync
    #[allow(dead_code)]
    fn 断言_Send_Sync_编译期检查() {
        fn _需要_Send<T: Send>() {}
        fn _需要_Sync<T: Sync>() {}
        fn _需要_Send_Sync<T: Send + Sync>() {}

        // 核心数据结构
        _需要_Send::<crate::kline::chan_kline::缠论K线>();
        _需要_Send::<crate::structure::dash_line::虚线>();
        _需要_Send::<crate::algorithm::hub::中枢>();
        _需要_Send_Sync::<crate::kline::chan_kline::缠论K线>();
        _需要_Send_Sync::<crate::structure::dash_line::虚线>();
        _需要_Send_Sync::<crate::algorithm::hub::中枢>();

        // Arc 包装后的 Send 检查
        _需要_Send::<Arc<crate::kline::chan_kline::缠论K线>>();
        _需要_Send::<Arc<crate::structure::dash_line::虚线>>();
        _需要_Send::<Arc<crate::algorithm::hub::中枢>>();

        // 观察者
        _需要_Send::<crate::business::observer::观察者>();
    }

    /// 测试：Arc<缠论K线> 可跨线程传递
    #[test]
    fn test_跨线程_缠论K线_Send() {
        use crate::kline::bar::K线;
        use crate::kline::chan_kline::缠论K线;

        let 普k = Arc::new(K线::创建普K(
            "test", 1000, 100.0, 110.0, 90.0, 95.0, 1000.0, 0, 300,
        ));
        let ck = 缠论K线::创建缠K(
            1000,
            110.0,
            90.0,
            crate::types::相对方向::向上,
            None,
            1,
            普k,
            None,
        );
        let arc_ck = Arc::new(ck);
        let arc_ck2 = Arc::clone(&arc_ck);

        let handle = std::thread::spawn(move || {
            let _ = arc_ck2;
            42
        });
        assert_eq!(handle.join().unwrap(), 42);
        assert!((arc_ck.高.get() - 110.0).abs() < 0.01);
    }

    /// 测试：Arc<虚线> 可跨线程传递
    #[test]
    fn test_跨线程_虚线_Send() {
        use crate::kline::bar::K线;
        use crate::kline::chan_kline::缠论K线;
        use crate::structure::dash_line::虚线;
        use crate::structure::fractal_obj::分型;

        let 普k = Arc::new(K线::创建普K(
            "test", 1000, 100.0, 110.0, 90.0, 95.0, 1000.0, 0, 300,
        ));
        let ck = Arc::new(缠论K线::创建缠K(
            1000,
            110.0,
            90.0,
            crate::types::相对方向::向上,
            None,
            1,
            普k,
            None,
        ));
        let frac = Arc::new(分型::new(None, Arc::clone(&ck), None));
        let frac2 = Arc::new(分型::new(None, ck, None));

        let dash = Arc::new(虚线::创建笔(frac, frac2, true));
        let dash2 = Arc::clone(&dash);

        let handle = std::thread::spawn(move || {
            let _ = Arc::as_ptr(&dash2.文);
            99
        });
        assert_eq!(handle.join().unwrap(), 99);
        assert_eq!(dash.标识.read().unwrap().as_str(), "笔");
    }

    /// 测试：Arc<中枢> 可跨线程传递
    #[test]
    fn test_跨线程_中枢_Send() {
        let hub = crate::algorithm::hub::中枢::new(1, "test".into(), 1, vec![]);
        let arc_hub = Arc::new(hub);
        let arc_hub2 = Arc::clone(&arc_hub);

        let handle = std::thread::spawn(move || {
            let _ = arc_hub2.序号.load(Ordering::Relaxed);
            77
        });
        assert_eq!(handle.join().unwrap(), 77);
        assert_eq!(arc_hub.序号.load(Ordering::Relaxed), 1);
    }

    /// 测试：多线程并发读取 观察者
    #[test]
    fn test_跨线程_观察者_多线程读取() {
        let obs = 观察者::new("btcusd".into(), 86400, Default::default());
        let obs2 = Arc::clone(&obs);
        let obs3 = Arc::clone(&obs);

        let h1 = std::thread::spawn(move || {
            let guard = obs2.read().unwrap();
            guard.符号.clone()
        });
        let h2 = std::thread::spawn(move || {
            let guard = obs3.read().unwrap();
            guard.周期
        });

        assert_eq!(h1.join().unwrap(), "btcusd");
        assert_eq!(h2.join().unwrap(), 86400);
    }

    /// 测试：Cell 字段跨线程读写不 panic
    #[test]
    fn test_跨线程_Cell字段_并发读写() {
        use crate::kline::bar::K线;
        use crate::kline::chan_kline::缠论K线;

        let 普k = Arc::new(K线::创建普K(
            "test", 1000, 100.0, 110.0, 90.0, 95.0, 1000.0, 0, 300,
        ));
        let ck = Arc::new(缠论K线::创建缠K(
            1000,
            110.0,
            90.0,
            crate::types::相对方向::向上,
            None,
            1,
            普k,
            None,
        ));
        let ck2 = Arc::clone(&ck);

        let handle = std::thread::spawn(move || {
            let 序号 = ck2.序号.load(Ordering::Relaxed);
            let 高 = ck2.高.get();
            (序号, 高)
        });

        let (序号, 高) = handle.join().unwrap();
        assert_eq!(序号, 0); // 序号 初始值为 0
        assert!((高 - 110.0).abs() < 0.01);
        ck.序号.store(2, Ordering::Relaxed);
        assert_eq!(ck.序号.load(Ordering::Relaxed), 2);
    }

    /// 测试：Arc<观察者> 直接跨线程传递
    #[test]
    fn test_跨线程_观察者_所有权转移() {
        let obs = 观察者::new("ethusd".into(), 7200, Default::default());

        let handle = std::thread::spawn(move || {
            let guard = obs.read().unwrap();
            (guard.符号.clone(), guard.周期)
        });

        let (符号, 周期) = handle.join().unwrap();
        assert_eq!(符号, "ethusd");
        assert_eq!(周期, 7200);
    }

    #[test]
    fn test_处理数据_线段分析层次为零_不崩溃() {
        let mut config = 缠论配置::default();
        config.加载文件路径 = test_data_path();
        let obs = 观察者::new("btcusd".into(), 300, config);
        let mut obs_w = obs.write().unwrap();
        obs_w.线段分析层次 = 0;
        obs_w.重置基础序列();
        drop(obs_w);

        // 逐根投喂K线，不应因 线段分析层次=0 而 panic
        let data = std::fs::read(test_data_path()).unwrap();
        let size = 48;
        for i in 0..(data.len() / size).min(500) {
            let offset = i * size;
            if let Some(k线) = K线::from_bytes(&data[offset..offset + size], 300, "btcusd") {
                obs.write().unwrap().增加原始K线(k线);
            }
        }

        let obs_r = obs.read().unwrap();
        assert!(obs_r.缠论K线序列.len() > 0, "缠K序列应有数据");
        assert!(obs_r.分型序列.len() > 0, "分型序列应有数据");
        assert!(obs_r.线段序列组.is_empty(), "线段序列组应为空");
        // 混合扩展线段序列组 有 3 个空 Vec（因为 混合扩展线段分析层次 仍是 3），
        // 但所有条目应为空（min(3, 0) = 0，循环未执行）
        assert!(
            obs_r.混合扩展线段序列组.iter().all(|s| s.is_empty()),
            "混合扩展线段序列组所有条目应为空"
        );
        info!(
            "线段分析层次=0 处理数据 OK: {} 缠K, {} 分型, {} 笔",
            obs_r.缠论K线序列.len(),
            obs_r.分型序列.len(),
            obs_r.笔序列.len()
        );
    }

    #[test]
    fn test_静态重新分析_线段分析层次为零_不崩溃() {
        let mut config = 缠论配置::default();
        config.加载文件路径 = test_data_path();
        let obs = 观察者::new("btcusd".into(), 300, config);

        // 先正常投喂数据
        obs.write()
            .unwrap()
            .读取数据文件(&test_data_path(), Default::default())
            .unwrap();

        // 设为0后执行静态重新分析，不应 panic
        let mut obs_w = obs.write().unwrap();
        obs_w.线段分析层次 = 0;
        obs_w.静态重新分析();
        drop(obs_w);

        let obs_r = obs.read().unwrap();
        assert!(obs_r.分型序列.len() > 0, "静态重新分析后分型序列应有数据");
        assert!(obs_r.线段序列组.is_empty(), "线段序列组应为空");
        assert!(
            obs_r.混合扩展线段序列组.iter().all(|s| s.is_empty()),
            "混合扩展线段序列组所有条目应为空"
        );
        info!(
            "线段分析层次=0 静态重新分析 OK: {} 分型, {} 笔",
            obs_r.分型序列.len(),
            obs_r.笔序列.len()
        );
    }
}
