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

use crate::config::缠论配置;
use crate::indicators::{
    平滑异同移动平均线, 相对强弱指数, 随机指标, K线取值
};
use crate::kline::bar::K线;
use crate::structure::fractal_obj::分型;
use crate::types::分型结构;
use crate::types::相对方向;
use crate::types::SyncF64;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, RwLock};

/// 缠论K线 — 经包含处理过后的K线
///
/// 部分字段使用 Cell/RefCell 实现内部可变性，确保包含处理原地修改时
/// Rc 指针不变，所有持有该 Rc 的引用（如分型.右）能看到最新数据。
#[derive(Debug)]
pub struct 缠论K线 {
    pub 序号: AtomicI64,
    pub 时间戳: AtomicI64,
    pub 高: SyncF64,
    pub 低: SyncF64,
    pub 方向: RwLock<相对方向>,
    pub 分型: RwLock<Option<分型结构>>,
    pub 周期: i64,
    pub 标识: String,
    pub 分型特征值: SyncF64,
    pub 原始起始序号: i64,
    pub 原始结束序号: AtomicI64,
    pub 标的K线: RwLock<Arc<K线>>,
    pub 买卖点信息: RwLock<Vec<String>>,
}

impl Clone for 缠论K线 {
    fn clone(&self) -> Self {
        Self {
            序号: AtomicI64::new(self.序号.load(Ordering::Relaxed)),
            时间戳: AtomicI64::new(self.时间戳.load(Ordering::Relaxed)),
            高: SyncF64::new(self.高.get()),
            低: SyncF64::new(self.低.get()),
            方向: RwLock::new(*self.方向.read().unwrap()),
            分型: RwLock::new(*self.分型.read().unwrap()),
            周期: self.周期,
            标识: self.标识.clone(),
            分型特征值: SyncF64::new(self.分型特征值.get()),
            原始起始序号: self.原始起始序号,
            原始结束序号: AtomicI64::new(self.原始结束序号.load(Ordering::Relaxed)),
            标的K线: RwLock::new(Arc::clone(&self.标的K线.read().unwrap())),
            买卖点信息: RwLock::new(self.买卖点信息.read().unwrap().clone()),
        }
    }
}

impl std::fmt::Display for 缠论K线 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use crate::utils::format_f64_g;
        write!(
            f,
            "{}<{}, {}, {}, {}, {}, {}, {}>",
            self.标识,
            self.序号.load(Ordering::Relaxed),
            self.分型
                .read()
                .unwrap()
                .map_or("None".to_string(), |fx| fx.to_string()),
            self.周期,
            *self.方向.read().unwrap(),
            self.时间戳.load(Ordering::Relaxed),
            format_f64_g(self.高.get()),
            format_f64_g(self.低.get())
        )
    }
}

impl 缠论K线 {
    /// 创建镜像（浅拷贝 Rc 引用）
    pub fn 镜像(&self) -> Self {
        Self {
            序号: AtomicI64::new(self.序号.load(Ordering::Relaxed)),
            时间戳: AtomicI64::new(self.时间戳.load(Ordering::Relaxed)),
            高: SyncF64::new(self.高.get()),
            低: SyncF64::new(self.低.get()),
            方向: RwLock::new(*self.方向.read().unwrap()),
            分型: RwLock::new(*self.分型.read().unwrap()),
            周期: self.周期,
            标识: self.标识.clone(),
            分型特征值: SyncF64::new(self.分型特征值.get()),
            原始起始序号: self.原始起始序号,
            原始结束序号: AtomicI64::new(self.原始结束序号.load(Ordering::Relaxed)),
            标的K线: RwLock::new(Arc::clone(&self.标的K线.read().unwrap())),
            买卖点信息: RwLock::new(self.买卖点信息.read().unwrap().clone()),
        }
    }

    /// 与MACD柱子匹配 — 底分型时MACD柱应<0, 顶分型时>0
    pub fn 与MACD柱子匹配(&self) -> bool {
        match *self.分型.read().unwrap() {
            Some(分型结构::底) | Some(分型结构::下) => {
                if let Some(ref macd) = self.标的K线.read().unwrap().macd {
                    macd.MACD柱 < 0.0
                } else {
                    false
                }
            }
            Some(分型结构::顶) | Some(分型结构::上) => {
                if let Some(ref macd) = self.标的K线.read().unwrap().macd {
                    macd.MACD柱 > 0.0
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// 与RSI匹配 — 底分型时RSI应低于SMA, 顶分型时高于SMA
    pub fn 与RSI匹配(&self) -> bool {
        match *self.分型.read().unwrap() {
            Some(分型结构::底) | Some(分型结构::下) => {
                if let Some(ref rsi) = self.标的K线.read().unwrap().rsi {
                    match (rsi.RSI, rsi.RSI_SMA) {
                        (Some(r), Some(sma)) => r < sma,
                        _ => false,
                    }
                } else {
                    false
                }
            }
            Some(分型结构::顶) | Some(分型结构::上) => {
                if let Some(ref rsi) = self.标的K线.read().unwrap().rsi {
                    match (rsi.RSI, rsi.RSI_SMA) {
                        (Some(r), Some(sma)) => r > sma,
                        _ => false,
                    }
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// 与KDJ匹配 — 底分型时K应低于D(死叉后), 顶分型时K应高于D(金叉后)
    pub fn 与KDJ匹配(&self) -> bool {
        match *self.分型.read().unwrap() {
            Some(分型结构::底) | Some(分型结构::下) => {
                if let Some(ref kdj) = self.标的K线.read().unwrap().kdj {
                    match (kdj.K, kdj.D) {
                        (Some(k), Some(d)) => k < d,
                        _ => false,
                    }
                } else {
                    false
                }
            }
            Some(分型结构::顶) | Some(分型结构::上) => {
                if let Some(ref kdj) = self.标的K线.read().unwrap().kdj {
                    match (kdj.K, kdj.D) {
                        (Some(k), Some(d)) => k > d,
                        _ => false,
                    }
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// 时间戳对齐 — 从基线序列中找匹配的时间戳
    pub fn 时间戳对齐(基线: &[Arc<缠论K线>], k线: &缠论K线) -> i64 {
        if let Some(基) = 基线.first() {
            for k in 基线.iter().rev() {
                if 基.周期 < k线.周期 {
                    if k线.时间戳.load(Ordering::Relaxed) <= k.时间戳.load(Ordering::Relaxed)
                        && k.时间戳.load(Ordering::Relaxed)
                            <= k线.时间戳.load(Ordering::Relaxed) + k线.周期
                        && (k线.分型特征值.get() - k.分型特征值.get()).abs() < f64::EPSILON
                    {
                        return k.时间戳.load(Ordering::Relaxed);
                    }
                } else if k.时间戳.load(Ordering::Relaxed) <= k线.时间戳.load(Ordering::Relaxed)
                    && k线.时间戳.load(Ordering::Relaxed)
                        <= k.时间戳.load(Ordering::Relaxed) + k.周期
                    && (k线.分型特征值.get() - k.分型特征值.get()).abs() < f64::EPSILON
                {
                    return k.时间戳.load(Ordering::Relaxed);
                }
            }
        }
        k线.时间戳.load(Ordering::Relaxed)
    }

    /// 创建缠K
    #[allow(clippy::too_many_arguments)]
    pub fn 创建缠K(
        时间戳: i64,
        高: f64,
        低: f64,
        方向: 相对方向,
        结构: Option<分型结构>,
        原始序号: i64,
        普k: Arc<K线>,
        之前: Option<&缠论K线>,
    ) -> Self {
        if 高.is_nan() || 低.is_nan() {
            panic!("缠K高/低不能为NaN: 高={高}, 低={低}");
        }
        assert!(高 >= 低, "缠K高必须>=低: 高={高}, 低={低}");

        let 周期 = 普k.周期;
        let 标识 = 普k.标识.clone();

        let 当前 = Self {
            序号: AtomicI64::new(0),
            时间戳: AtomicI64::new(时间戳),
            高: SyncF64::new(高),
            低: SyncF64::new(低),
            方向: RwLock::new(方向),
            分型: RwLock::new(结构),
            周期,
            标识,
            分型特征值: SyncF64::new(高),
            原始起始序号: 原始序号,
            原始结束序号: AtomicI64::new(原始序号),
            标的K线: RwLock::new(普k),
            买卖点信息: RwLock::new(Vec::new()),
        };

        if let Some(之前) = 之前 {
            当前
                .序号
                .store(之前.序号.load(Ordering::Relaxed) + 1, Ordering::Relaxed);
            let 关系 =
                相对方向::分析(之前.高.get(), 之前.低.get(), 当前.高.get(), 当前.低.get());
            if 关系.是否包含() {
                panic!(
                    "创建缠K 包含关系: {:?}\n  之前: {}\n  当前: {}",
                    关系, 之前, 当前
                );
            }
        }
        当前
    }

    /// 兼并（合并）处理 — 缠论包含处理的核心算法
    ///
    /// 返回 (新缠K, 模式) — 模式: "添加"/"替换"/None
    pub fn 兼并(
        之前缠K: Option<&缠论K线>,
        当前缠K: &缠论K线,
        当前普K: &Arc<K线>,
        配置: &缠论配置,
    ) -> (Option<Arc<缠论K线>>, Option<String>) {
        let 关系 = 相对方向::分析(当前缠K.高.get(), 当前缠K.低.get(), 当前普K.高, 当前普K.低);

        // 无包含关系 — 创建新元素追加
        if !关系.是否包含() {
            let 结构 = if 关系.是否向下() {
                Some(分型结构::下)
            } else {
                Some(分型结构::上)
            };
            let 新缠K = Self::创建缠K(
                当前普K.时间戳,
                当前普K.高,
                当前普K.低,
                当前普K.方向(),
                结构,
                当前普K.序号,
                Arc::clone(当前普K),
                Some(当前缠K),
            );
            新缠K
                .序号
                .store(当前缠K.序号.load(Ordering::Relaxed) + 1, Ordering::Relaxed);
            return (Some(Arc::new(新缠K)), Some("添加".into()));
        }

        // 重复提交检测 — 当序号相同时认为是重复提交K线
        if 当前普K.序号 == 当前缠K.原始结束序号.load(Ordering::Relaxed) {
            return (None, None);
        }

        // 序号连续性检查
        if 当前普K.序号 - 1 != 当前缠K.原始结束序号.load(Ordering::Relaxed)
            && 当前普K.序号 != 当前缠K.原始结束序号.load(Ordering::Relaxed)
        {
            panic!(
                "兼并: 不可追加不连续元素 缠K.原始结束序号: {}, 当前普K.序号: {}",
                当前缠K.原始结束序号.load(Ordering::Relaxed),
                当前普K.序号
            );
        }

        // 包含关系 — 原地合并到当前缠K
        let 取值函数: fn(f64, f64) -> f64 = if let Some(之前) = 之前缠K {
            if 相对方向::分析(
                之前.高.get(),
                之前.低.get(),
                当前缠K.高.get(),
                当前缠K.低.get(),
            )
            .是否向下()
            {
                f64::min
            } else {
                f64::max
            }
        } else {
            f64::max
        };

        // 逆序包含时更新时间和标的K线
        if 关系 != 相对方向::顺 {
            当前缠K.时间戳.store(当前普K.时间戳, Ordering::Relaxed);
            *当前缠K.标的K线.write().unwrap() = Arc::clone(当前普K);
        }
        当前缠K.高.set(取值函数(当前缠K.高.get(), 当前普K.高));
        当前缠K.低.set(取值函数(当前缠K.低.get(), 当前普K.低));
        当前缠K.原始结束序号.store(当前普K.序号, Ordering::Relaxed);
        *当前缠K.方向.write().unwrap() = 当前普K.方向();

        if let Some(之前) = 之前缠K {
            当前缠K
                .序号
                .store(之前.序号.load(Ordering::Relaxed) + 1, Ordering::Relaxed);
        }

        if 配置.缠K合并替换 {
            (Some(Arc::new(当前缠K.镜像())), Some("替换".into()))
        } else {
            (None, None)
        }
    }

    /// 完整的缠K分析 — 普K → 缠K + 分型
    ///
    /// 返回 (状态, 形态)
    pub fn 分析(
        mut 当前K线: K线,
        缠K序列: &mut Vec<Arc<缠论K线>>,
        普K序列: &mut Vec<Arc<K线>>,
        配置: &缠论配置,
    ) -> (String, Option<Arc<分型>>) {
        当前K线.标识 = 配置.标识.clone();

        // ---- 阶段1: 普K序列管理 + 指标增量计算 ----
        if 普K序列.is_empty() {
            if 配置.计算指标 {
                当前K线.macd = Some(平滑异同移动平均线::首次计算(
                    K线取值(
                        当前K线.开盘价,
                        当前K线.高,
                        当前K线.低,
                        当前K线.收盘价,
                        &配置.指标计算方式,
                    ),
                    当前K线.时间戳,
                    配置.平滑异同移动平均线_快线周期,
                    配置.平滑异同移动平均线_慢线周期,
                    配置.平滑异同移动平均线_信号周期,
                ));
                当前K线.rsi = Some(相对强弱指数::首次计算(
                    K线取值(
                        当前K线.开盘价,
                        当前K线.高,
                        当前K线.低,
                        当前K线.收盘价,
                        &配置.指标计算方式,
                    ),
                    当前K线.时间戳,
                    配置.相对强弱指数_周期,
                    配置.相对强弱指数_超买阈值,
                    配置.相对强弱指数_超卖阈值,
                    Some(配置.相对强弱指数_移动平均线周期),
                ));
                当前K线.kdj = Some(随机指标::首次计算(
                    当前K线.高,
                    当前K线.低,
                    当前K线.收盘价,
                    当前K线.时间戳,
                    配置.随机指标_RSV周期,
                    配置.随机指标_K值平滑周期,
                    配置.随机指标_D值平滑周期,
                    配置.随机指标_超买阈值,
                    配置.随机指标_超卖阈值,
                ));
            }
            let 当前K线_rc = Arc::new(当前K线);
            普K序列.push(当前K线_rc);
        } else {
            let 之前普K = 普K序列.last().unwrap();
            if 之前普K.时间戳 == 当前K线.时间戳 {
                // 同时间戳更新
                当前K线.序号 = 之前普K.序号;
                if 配置.计算指标 && 普K序列.len() >= 2 {
                    if let Some(ref prev_macd) = 普K序列[普K序列.len() - 2].macd {
                        当前K线.macd = Some(平滑异同移动平均线::增量计算(
                            prev_macd,
                            K线取值(
                                当前K线.开盘价,
                                当前K线.高,
                                当前K线.低,
                                当前K线.收盘价,
                                &配置.指标计算方式,
                            ),
                            当前K线.时间戳,
                        ));
                    }
                    if let Some(ref prev_rsi) = 普K序列[普K序列.len() - 2].rsi {
                        当前K线.rsi = Some(相对强弱指数::增量计算(
                            prev_rsi,
                            K线取值(
                                当前K线.开盘价,
                                当前K线.高,
                                当前K线.低,
                                当前K线.收盘价,
                                &配置.指标计算方式,
                            ),
                            当前K线.时间戳,
                        ));
                    }
                    if let Some(ref prev_kdj) = 普K序列[普K序列.len() - 2].kdj {
                        当前K线.kdj = Some(随机指标::增量计算(
                            prev_kdj,
                            当前K线.高,
                            当前K线.低,
                            当前K线.收盘价,
                            当前K线.时间戳,
                        ));
                    }
                }
                普K序列.pop();
                普K序列.push(Arc::new(当前K线));
            } else {
                if 之前普K.时间戳 > 当前K线.时间戳 {
                    panic!("时序错误: 之前={}, 当前={}", 之前普K.时间戳, 当前K线.时间戳);
                }
                当前K线.序号 = 之前普K.序号 + 1;
                if 配置.计算指标 {
                    if let Some(ref prev_macd) = 之前普K.macd {
                        当前K线.macd = Some(平滑异同移动平均线::增量计算(
                            prev_macd,
                            K线取值(
                                当前K线.开盘价,
                                当前K线.高,
                                当前K线.低,
                                当前K线.收盘价,
                                &配置.指标计算方式,
                            ),
                            当前K线.时间戳,
                        ));
                    }
                    if let Some(ref prev_rsi) = 之前普K.rsi {
                        当前K线.rsi = Some(相对强弱指数::增量计算(
                            prev_rsi,
                            K线取值(
                                当前K线.开盘价,
                                当前K线.高,
                                当前K线.低,
                                当前K线.收盘价,
                                &配置.指标计算方式,
                            ),
                            当前K线.时间戳,
                        ));
                    }
                    if let Some(ref prev_kdj) = 之前普K.kdj {
                        当前K线.kdj = Some(随机指标::增量计算(
                            prev_kdj,
                            当前K线.高,
                            当前K线.低,
                            当前K线.收盘价,
                            当前K线.时间戳,
                        ));
                    }
                }
                普K序列.push(Arc::new(当前K线));
            }
        }

        // ---- 阶段2: 缠K合并 ----
        let 状态: String;
        let 当前K线_ref: &Arc<K线> = 普K序列.last().unwrap();

        if !缠K序列.is_empty() {
            let len = 缠K序列.len();
            let (左边, 右边) = 缠K序列.split_at_mut(len - 1);
            let 之前缠K: Option<&缠论K线> = 左边.last().map(Arc::as_ref);
            let 最后一个缠K = &*右边[0];
            let (新缠K, 模式) = Self::兼并(之前缠K, 最后一个缠K, 当前K线_ref, 配置);

            if let Some(k) = 新缠K {
                match 模式.as_deref() {
                    Some("添加") => {
                        缠K序列.push(k);
                        状态 = "创建".into();
                    }
                    Some("替换") => {
                        // Cell::set 已原地更新数据，无需 pop+push 打破 Rc 身份
                        状态 = "兼并".into();
                    }
                    _ => {
                        状态 = "兼并".into();
                    }
                }
            } else {
                状态 = "兼并".into();
            }
        } else {
            let 新缠K = Self::创建缠K(
                当前K线_ref.时间戳,
                当前K线_ref.高,
                当前K线_ref.低,
                当前K线_ref.方向(),
                None,
                当前K线_ref.序号,
                Arc::clone(普K序列.last().unwrap()),
                None,
            );
            缠K序列.push(Arc::new(新缠K));
            状态 = "新建".into();
        }

        // ---- 阶段3: 分型识别 ----
        if 缠K序列.len() < 3 {
            return (状态, None);
        }

        let idx = 缠K序列.len();
        let 左 = Arc::clone(&缠K序列[idx - 3]);
        let 中 = Arc::clone(&缠K序列[idx - 2]);
        let 右 = Arc::clone(&缠K序列[idx - 1]);

        let 结构 = 分型结构::分析(&*左, &*中, &*右, false, false);

        // 对齐 Python：无条件设置 中.分型、中.分型特征值、右.分型特征值、右.分型
        *缠K序列[idx - 2].分型.write().unwrap() = 结构;

        if let Some(结构) = 结构 {
            match 结构 {
                分型结构::底 => {
                    缠K序列[idx - 2].分型特征值.set(缠K序列[idx - 2].低.get());
                    缠K序列[idx - 1].分型特征值.set(缠K序列[idx - 1].高.get());
                    *缠K序列[idx - 1].分型.write().unwrap() = Some(分型结构::顶);
                }
                分型结构::顶 => {
                    缠K序列[idx - 2].分型特征值.set(缠K序列[idx - 2].高.get());
                    缠K序列[idx - 1].分型特征值.set(缠K序列[idx - 1].低.get());
                    *缠K序列[idx - 1].分型.write().unwrap() = Some(分型结构::底);
                }
                分型结构::上 => {
                    缠K序列[idx - 2].分型特征值.set(缠K序列[idx - 2].高.get());
                    缠K序列[idx - 1].分型特征值.set(缠K序列[idx - 1].高.get());
                    *缠K序列[idx - 1].分型.write().unwrap() = Some(分型结构::顶);
                }
                分型结构::下 => {
                    缠K序列[idx - 2].分型特征值.set(缠K序列[idx - 2].低.get());
                    缠K序列[idx - 1].分型特征值.set(缠K序列[idx - 1].低.get());
                    *缠K序列[idx - 1].分型.write().unwrap() = Some(分型结构::底);
                }
                分型结构::散 => {}
            }

            let 形态 = if matches!(结构, 分型结构::上 | 分型结构::下) {
                // Python: 形态 = 分型(中, 右, None) — 左=中K线, 中=右K线, 右=None
                Arc::new(分型::new(
                    Some(Arc::clone(&缠K序列[idx - 2])),
                    Arc::clone(&缠K序列[idx - 1]),
                    None,
                ))
            } else {
                Arc::new(分型::new(
                    Some(Arc::clone(&缠K序列[idx - 3])),
                    Arc::clone(&缠K序列[idx - 2]),
                    Some(Arc::clone(&缠K序列[idx - 1])),
                ))
            };

            return (状态, Some(形态));
        }

        // 对齐 Python：结构为 None 时仍创建并返回分型
        let 形态 = Arc::new(分型::new(
            Some(Arc::clone(&缠K序列[idx - 3])),
            Arc::clone(&缠K序列[idx - 2]),
            Some(Arc::clone(&缠K序列[idx - 1])),
        ));
        (状态, Some(形态))
    }

    /// 截取缠K序列从始到终
    pub fn 截取(
        序列: &[Arc<缠论K线>],
        始: &缠论K线,
        终: &缠论K线,
    ) -> Option<Vec<Arc<缠论K线>>> {
        let 始_idx = 序列.iter().position(|k| std::ptr::eq(Arc::as_ptr(k), 始))?;
        let 终_idx = 序列.iter().position(|k| std::ptr::eq(Arc::as_ptr(k), 终))?;
        Some(序列[始_idx..=终_idx].to_vec())
    }
}

impl crate::types::fractal::有高低 for 缠论K线 {
    fn 高(&self) -> f64 {
        self.高.get()
    }
    fn 低(&self) -> f64 {
        self.低.get()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::缠论配置;

    fn make_普K(时间戳: i64, 开: f64, 高: f64, 低: f64, 收: f64, 序号: i64) -> K线 {
        K线::创建普K("test", 时间戳, 开, 高, 低, 收, 1000.0, 序号, 60)
    }

    #[test]
    fn test_创建缠K_basic() {
        let pk = Arc::new(make_普K(1000, 100.0, 110.0, 95.0, 105.0, 0));
        let ck = 缠论K线::创建缠K(1000, 110.0, 95.0, 相对方向::向上, None, 0, pk, None);
        assert_eq!(ck.高.get(), 110.0);
        assert_eq!(ck.低.get(), 95.0);
        assert_eq!(ck.序号.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_分析_empty_sequence() {
        let config = 缠论配置::default();
        let pk = make_普K(1000, 100.0, 110.0, 95.0, 105.0, 0);
        let mut 缠K序列 = Vec::new();
        let mut 普K序列 = Vec::new();

        let (状态, 形态) = 缠论K线::分析(pk, &mut 缠K序列, &mut 普K序列, &config);
        assert_eq!(状态, "新建");
        assert_eq!(缠K序列.len(), 1);
        assert!(形态.is_none()); // 不够3根
    }

    #[test]
    fn test_分析_three_bars_fractal() {
        let config = 缠论配置::default();
        let mut 缠K序列 = Vec::new();
        let mut 普K序列 = Vec::new();

        // 三根形成顶分型: 低高 → 更高高 → 低高
        let pk1 = make_普K(1000, 100.0, 110.0, 95.0, 105.0, 0);
        let 状态1 = 缠论K线::分析(pk1, &mut 缠K序列, &mut 普K序列, &config);
        assert_eq!(状态1.0, "新建");

        let pk2 = make_普K(1001, 105.0, 115.0, 102.0, 112.0, 1);
        let 状态2 = 缠论K线::分析(pk2, &mut 缠K序列, &mut 普K序列, &config);
        assert!(状态2.1.is_none()); // 仍不够

        let pk3 = make_普K(1002, 112.0, 113.0, 100.0, 103.0, 2);
        let (_状态3, 形态) = 缠论K线::分析(pk3, &mut 缠K序列, &mut 普K序列, &config);
        assert!(形态.is_some()); // 分型产生了
    }
}
