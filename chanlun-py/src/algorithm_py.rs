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

use crate::kline_py::chan_kline_to_py;
use crate::structure_py::{dashed_to_py, fractal_to_py};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyType};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;
use std::sync::atomic::Ordering;

// 使用全局 static 而非 thread_local!，保证跨线程对象标识一致性
static HUB_IDENTITY: std::sync::LazyLock<RwLock<HashMap<usize, Py<中枢Py>>>> =
    std::sync::LazyLock::new(|| RwLock::new(HashMap::new()));

pub(crate) fn hub_to_py(
    py: Python<'_>, inner: Arc<chanlun::algorithm::hub::中枢>
) -> Py<中枢Py> {
    let key = Arc::as_ptr(&inner) as usize;
    if let Some(cached) = HUB_IDENTITY
        .read()
        .unwrap()
        .get(&key)
        .map(|p| p.clone_ref(py))
    {
        return cached;
    }
    HUB_IDENTITY
        .write()
        .unwrap()
        .retain(|_, v| v.get_refcnt(py) > 1);
    let obj = Py::new(py, 中枢Py { inner }).unwrap();
    HUB_IDENTITY.write().unwrap().insert(key, obj.clone_ref(py));
    obj
}

use crate::business_py::观察者Py;
use crate::config_py::缠论配置Py;
use crate::kline_py::{K线Py, 缠论K线Py};
use crate::structure_py::{分型Py, 虚线Py};
use crate::types_py::相对方向Py;

// ========== 背驰分析 ==========

/// 背驰分析 — 静态方法容器，提供背驰/背离检测算法。
///
/// 所有方法均为 staticmethod，直接调用无需实例化。
///
/// 方法:
///   MACD背驰(虚线段, 线段序列, 中枢序列, 配置) -> bool
///   MACD趋势背驰(虚线段, 线段序列, 中枢序列, 配置) -> bool
///   MACD盘整背驰(虚线段, 线段序列, 中枢序列, 配置) -> bool
///   MACD柱子面积背驰(虚线段, 线段序列, 中枢序列, 配置) -> bool
///   MACD柱子高度背驰(虚线段, 线段序列, 中枢序列, 配置) -> bool
#[pyclass(name = "背驰分析", module = "chanlun._chanlun")]
pub struct 背驰分析Py;

#[pymethods]
impl 背驰分析Py {
    #[classmethod]
    #[pyo3(signature = (进入段, 离开段, K线序列, 方式 = "总"))]
    /// MACD柱状线面积背驰
    fn MACD背驰(
        _cls: &Bound<'_, PyType>,
        进入段: &Bound<'_, 虚线Py>,
        离开段: &Bound<'_, 虚线Py>,
        K线序列: Vec<Py<K线Py>>,
        方式: &str,
        py: Python<'_>,
    ) -> bool {
        let rc_list: Vec<Arc<chanlun::kline::bar::K线>> = K线序列
            .iter()
            .map(|k| k.bind(py).borrow().inner.clone())
            .collect();
        chanlun::algorithm::divergence::背驰分析::MACD背驰(
            &进入段.borrow().inner,
            &离开段.borrow().inner,
            &rc_list,
            方式,
        )
    }

    #[classmethod]
    /// 价格斜率背驰
    fn 斜率背驰(
        _cls: &Bound<'_, PyType>,
        进入段: &Bound<'_, 虚线Py>,
        离开段: &Bound<'_, 虚线Py>,
    ) -> bool {
        chanlun::algorithm::divergence::背驰分析::斜率背驰(
            &进入段.borrow().inner,
            &离开段.borrow().inner,
        )
    }

    #[classmethod]
    /// 价格测度背驰（欧氏距离）
    fn 测度背驰(
        _cls: &Bound<'_, PyType>,
        进入段: &Bound<'_, 虚线Py>,
        离开段: &Bound<'_, 虚线Py>,
    ) -> bool {
        chanlun::algorithm::divergence::背驰分析::测度背驰(
            &进入段.borrow().inner,
            &离开段.borrow().inner,
        )
    }

    #[classmethod]
    /// 判断是否满足全部三种背驰条件（MACD + 测度 + 斜率）
    fn 全量背驰(
        _cls: &Bound<'_, PyType>,
        进入段: &Bound<'_, 虚线Py>,
        离开段: &Bound<'_, 虚线Py>,
        普K序列: Vec<Py<K线Py>>,
        py: Python<'_>,
    ) -> bool {
        let rc_list: Vec<Arc<chanlun::kline::bar::K线>> = 普K序列
            .iter()
            .map(|k| k.bind(py).borrow().inner.clone())
            .collect();
        chanlun::algorithm::divergence::背驰分析::全量背驰(
            &进入段.borrow().inner,
            &离开段.borrow().inner,
            &rc_list,
        )
    }

    #[classmethod]
    /// 判断是否满足任一背驰条件
    fn 任意背驰(
        _cls: &Bound<'_, PyType>,
        进入段: &Bound<'_, 虚线Py>,
        离开段: &Bound<'_, 虚线Py>,
        普K序列: Vec<Py<K线Py>>,
        py: Python<'_>,
    ) -> bool {
        let rc_list: Vec<Arc<chanlun::kline::bar::K线>> = 普K序列
            .iter()
            .map(|k| k.bind(py).borrow().inner.clone())
            .collect();
        chanlun::algorithm::divergence::背驰分析::任意背驰(
            &进入段.borrow().inner,
            &离开段.borrow().inner,
            &rc_list,
        )
    }

    #[classmethod]
    /// 根据配置选择对应的背驰检测组合
    fn 配置背驰(
        _cls: &Bound<'_, PyType>,
        进入段: &Bound<'_, 虚线Py>,
        离开段: &Bound<'_, 虚线Py>,
        普K序列: Vec<Py<K线Py>>,
        配置: &Bound<'_, 缠论配置Py>,
        py: Python<'_>,
    ) -> PyResult<bool> {
        let rc_list: Vec<Arc<chanlun::kline::bar::K线>> = 普K序列
            .iter()
            .map(|k| k.bind(py).borrow().inner.clone())
            .collect();
        let config = 配置.borrow().to_rust_config(py)?;
        Ok(chanlun::algorithm::divergence::背驰分析::配置背驰(
            &进入段.borrow().inner,
            &离开段.borrow().inner,
            &rc_list,
            &config,
        ))
    }

    #[classmethod]
    /// 三个背驰条件中至少两个满足即视为背驰
    fn 任选背驰(
        _cls: &Bound<'_, PyType>,
        进入段: &Bound<'_, 虚线Py>,
        离开段: &Bound<'_, 虚线Py>,
        普K序列: Vec<Py<K线Py>>,
        py: Python<'_>,
    ) -> bool {
        let rc_list: Vec<Arc<chanlun::kline::bar::K线>> = 普K序列
            .iter()
            .map(|k| k.bind(py).borrow().inner.clone())
            .collect();
        chanlun::algorithm::divergence::背驰分析::任选背驰(
            &进入段.borrow().inner,
            &离开段.borrow().inner,
            &rc_list,
        )
    }

    #[classmethod]
    /// 根据模式字符串选择背驰检测策略
    fn 背驰模式(
        _cls: &Bound<'_, PyType>,
        进入段: &Bound<'_, 虚线Py>,
        离开段: &Bound<'_, 虚线Py>,
        普K序列: Vec<Py<K线Py>>,
        配置: &Bound<'_, 缠论配置Py>,
        模式: &str,
        py: Python<'_>,
    ) -> PyResult<bool> {
        let rc_list: Vec<Arc<chanlun::kline::bar::K线>> = 普K序列
            .iter()
            .map(|k| k.bind(py).borrow().inner.clone())
            .collect();
        let config = 配置.borrow().to_rust_config(py)?;
        Ok(chanlun::algorithm::divergence::背驰分析::背驰模式(
            &进入段.borrow().inner,
            &离开段.borrow().inner,
            &rc_list,
            &config,
            模式,
        ))
    }
}

// ========== 笔 ==========

/// 笔 — 静态方法容器，提供笔的创建与分析算法。
///
/// 所有方法均为 staticmethod/classmethod，直接调用无需实例化。
///
/// 方法:
///   :meth:`以文会友` — 根据起始分型找笔
///   :meth:`以武会友` — 根据结束分型找笔
///   :meth:`根据缠K找笔` — 判断缠K是否在笔的文武序号之间
///   :meth:`分析` — 笔划分核心递归算法
///   :meth:`获取所有停顿位置` — 获取笔内所有可能的停顿位置（用于背驰检测）
///   :meth:`是否背驰过` — 判断笔内是否发生过MACD趋向背驰
#[pyclass(name = "笔", module = "chanlun._chanlun")]
pub struct 笔Py;

#[pymethods]
impl 笔Py {
    #[classmethod]
    /// 以文会友
    fn 以文会友(
        _cls: &Bound<'_, PyType>,
        笔序列: Vec<Py<虚线Py>>,
        文: &Bound<'_, 分型Py>,
        py: Python<'_>,
    ) -> Option<Py<虚线Py>> {
        let bi_list: Vec<Arc<chanlun::structure::dash_line::虚线>> = 笔序列
            .iter()
            .map(|d| Arc::clone(&d.bind(py).borrow().inner))
            .collect();
        chanlun::algorithm::bi::笔::以文会友(&bi_list, &文.borrow().inner)
            .map(|inner| dashed_to_py(py, inner))
    }

    #[classmethod]
    /// 以武会友
    fn 以武会友(
        _cls: &Bound<'_, PyType>,
        笔序列: Vec<Py<虚线Py>>,
        武: &Bound<'_, 分型Py>,
        py: Python<'_>,
    ) -> Option<Py<虚线Py>> {
        let bi_list: Vec<Arc<chanlun::structure::dash_line::虚线>> = 笔序列
            .iter()
            .map(|d| Arc::clone(&d.bind(py).borrow().inner))
            .collect();
        chanlun::algorithm::bi::笔::以武会友(&bi_list, &武.borrow().inner)
            .map(|inner| dashed_to_py(py, inner))
    }

    #[classmethod]
    #[pyo3(signature = (笔序列, 缠K, 偏移 = 1))]
    /// 根据缠K找笔
    fn 根据缠K找笔(
        _cls: &Bound<'_, PyType>,
        笔序列: Vec<Py<虚线Py>>,
        缠K: &Bound<'_, crate::kline_py::缠论K线Py>,
        偏移: i64,
        py: Python<'_>,
    ) -> Option<Py<虚线Py>> {
        let bi_list: Vec<Arc<chanlun::structure::dash_line::虚线>> = 笔序列
            .iter()
            .map(|d| Arc::clone(&d.bind(py).borrow().inner))
            .collect();
        chanlun::algorithm::bi::笔::根据缠K找笔(&bi_list, &缠K.borrow().inner, 偏移)
            .map(|inner| dashed_to_py(py, inner))
    }

    #[classmethod]
    #[pyo3(signature = (当前分型, 分型序列, 笔序列, 缠K序列, 普K序列, 递归层次, 配置))]
    /// 笔划分核心递归算法
    fn 分析(
        _cls: &Bound<'_, PyType>,
        当前分型: Option<&Bound<'_, 分型Py>>,
        分型序列: Vec<Py<分型Py>>,
        笔序列: Vec<Py<虚线Py>>,
        缠K序列: Vec<Py<crate::kline_py::缠论K线Py>>,
        普K序列: Vec<Py<K线Py>>,
        递归层次: i64,
        配置: &Bound<'_, 缠论配置Py>,
        py: Python<'_>,
    ) -> PyResult<i64> {
        let _ = 递归层次; // Python API 兼容参数，核心从0开始计数
        let 当前分型_rc = 当前分型.map(|f| Arc::clone(&f.borrow().inner));
        let mut fr_seq: Vec<Arc<chanlun::structure::fractal_obj::分型>> = 分型序列
            .iter()
            .map(|f| Arc::clone(&f.bind(py).borrow().inner))
            .collect();
        let mut bi_seq: Vec<Arc<chanlun::structure::dash_line::虚线>> = 笔序列
            .iter()
            .map(|d| Arc::clone(&d.bind(py).borrow().inner))
            .collect();
        let ck_list: Vec<Arc<chanlun::kline::chan_kline::缠论K线>> = 缠K序列
            .iter()
            .map(|k| Arc::clone(&k.bind(py).borrow().inner))
            .collect();
        let bar_list: Vec<Arc<chanlun::kline::bar::K线>> = 普K序列
            .iter()
            .map(|k| k.bind(py).borrow().inner.clone())
            .collect();
        let config = 配置.borrow().to_rust_config(py)?;
        match 当前分型_rc {
            Some(fr) => Ok(chanlun::algorithm::bi::笔::分析(
                fr,
                &mut fr_seq,
                &mut bi_seq,
                &ck_list,
                &bar_list,
                递归层次,
                &config,
            )),
            None => Ok(递归层次),
        }
    }

    #[classmethod]
    /// 获取笔内所有可能的停顿位置（用于背驰检测）
    fn 获取所有停顿位置(
        _cls: &Bound<'_, PyType>,
        筆: &Bound<'_, 虚线Py>,
        观察员: &Bound<'_, 观察者Py>,
        py: Python<'_>,
    ) -> Vec<Py<虚线Py>> {
        let obs = 观察员.borrow();
        let obs_ref = obs.obs();
        chanlun::algorithm::bi::笔::获取所有停顿位置(&筆.borrow().inner, &obs_ref)
            .into_iter()
            .map(|d| dashed_to_py(py, Arc::new(d)))
            .collect()
    }

    #[classmethod]
    /// 判断笔内是否发生过MACD趋向背驰
    fn 是否背驰过(
        _cls: &Bound<'_, PyType>,
        当前筆: &Bound<'_, 虚线Py>,
        观察员: &Bound<'_, 观察者Py>,
        py: Python<'_>,
    ) -> Vec<Py<缠论K线Py>> {
        let obs = 观察员.borrow();
        let obs_ref = obs.obs();
        chanlun::algorithm::bi::笔::是否背驰过(&当前筆.borrow().inner, &obs_ref)
            .into_iter()
            .map(|ck| chan_kline_to_py(py, ck))
            .collect()
    }
}

// ========== 线段 ==========

/// 线段 — 静态方法容器，提供线段的创建与分析算法。
///
/// 所有方法均为 staticmethod/classmethod，直接调用无需实例化。
///
/// 核心方法:
///   创建(左分型, 右分型, 级别, 序号?, 标识?) -> 虚线
///   分析(笔序列, 配置, 级别?) -> (list[虚线], list[线段特征])
///      — 从笔序列中划分线段，返回(线段序列, 特征序列)
///
/// 辅助方法:
///   扩展分析(基础序列, 扩展结果容器, 配置) — 从基础序列生成扩展分析
///   四象(虚线段) -> str — 返回"少阳"/"少阴"/"老阳"/"老阴"
///   特征序列状态(虚线段) -> str
#[pyclass(name = "线段", module = "chanlun._chanlun")]
pub struct 线段Py;

#[pymethods]
impl 线段Py {
    #[classmethod]
    /// 判断线段的四象属性
    fn 四象(_cls: &Bound<'_, PyType>, 段: &Bound<'_, 虚线Py>) -> String {
        chanlun::algorithm::segment::线段::四象(&段.borrow().inner)
    }

    #[classmethod]
    /// 获取线段特征序列第一二元素间的缺口
    fn 获取缺口(
        _cls: &Bound<'_, PyType>,
        段: &Bound<'_, 虚线Py>,
    ) -> Option<crate::types_py::缺口Py> {
        chanlun::algorithm::segment::线段::获取缺口(&段.borrow().inner)
            .map(|inner| crate::types_py::缺口Py { inner })
    }

    #[classmethod]
    /// 是否符合特征序列正常分型终结
    fn 特征分型终结(_cls: &Bound<'_, PyType>, 段: &Bound<'_, 虚线Py>) -> bool {
        chanlun::algorithm::segment::线段::特征分型终结(&段.borrow().inner)
    }

    #[classmethod]
    /// :param 段: 线段
    fn 特征序列状态(
        _cls: &Bound<'_, PyType>, 段: &Bound<'_, 虚线Py>
    ) -> (bool, bool, bool) {
        chanlun::algorithm::segment::线段::特征序列状态(&段.borrow().inner)
    }

    #[classmethod]
    /// 查找贯穿伤
    fn 查找贯穿伤(
        _cls: &Bound<'_, PyType>, 段: &Bound<'_, 虚线Py>
    ) -> Option<Py<虚线Py>> {
        let py = _cls.py();
        chanlun::algorithm::segment::线段::查找贯穿伤(&段.borrow().inner)
            .map(|inner| dashed_to_py(py, inner))
    }

    #[classmethod]
    #[pyo3(signature = (段, 所属中枢 = None))]
    /// 将线段基础序列分割为前/后/第三买卖/贯穿伤四部分
    #[allow(clippy::type_complexity)]
    fn 分割序列(
        _cls: &Bound<'_, PyType>,
        段: &Bound<'_, 虚线Py>,
        所属中枢: Option<&Bound<'_, PyAny>>,
        py: Python<'_>,
    ) -> PyResult<(
        Vec<Py<虚线Py>>,
        Vec<Py<虚线Py>>,
        Vec<Py<虚线Py>>,
        Option<Py<虚线Py>>,
    )> {
        let borrowed = 段.borrow();
        let (a, b, c, d) = if let Some(hub_bound) = 所属中枢 {
            if let Ok(hub_ref) = hub_bound.extract::<PyRef<'_, 中枢Py>>() {
                chanlun::algorithm::segment::线段::分割序列(
                    &borrowed.inner,
                    Some(&hub_ref.inner),
                )
            } else {
                chanlun::algorithm::segment::线段::分割序列(&borrowed.inner, None)
            }
        } else {
            chanlun::algorithm::segment::线段::分割序列(&borrowed.inner, None)
        };
        let wrap = |v: Vec<Arc<chanlun::structure::dash_line::虚线>>| -> Vec<Py<虚线Py>> {
            v.into_iter().map(|x| dashed_to_py(py, x)).collect()
        };
        Ok((wrap(a), wrap(b), wrap(c), d.map(|x| dashed_to_py(py, x))))
    }

    #[classmethod]
    /// 获取内部中枢序列
    fn 获取内部中枢序列(
        _cls: &Bound<'_, PyType>,
        段: &Bound<'_, 虚线Py>,
        配置: &Bound<'_, 缠论配置Py>,
        py: Python<'_>,
    ) -> PyResult<(Py<PyAny>, Py<PyAny>, Py<PyAny>)> {
        let ref_mut = 段.borrow_mut();
        let config = 配置.borrow().to_rust_config(py)?;
        let (a, b, c) =
            chanlun::algorithm::segment::线段::获取内部中枢序列(&ref_mut.inner, &config);
        let pk_list = |v: Vec<Arc<chanlun::algorithm::hub::中枢>>| -> PyResult<Py<PyAny>> {
            let list = pyo3::types::PyList::empty(py);
            for h in v {
                list.append(hub_to_py(py, h))?;
            }
            Ok(list.into())
        };
        Ok((pk_list(a)?, pk_list(b)?, pk_list(c)?))
    }

    #[classmethod]
    /// 线段划分核心递归算法
    fn 分析(
        _cls: &Bound<'_, PyType>,
        笔序列: Vec<Py<虚线Py>>,
        线段序列: Vec<Py<虚线Py>>,
        配置: &Bound<'_, 缠论配置Py>,
        层级: i64,
        关系序列: Option<Vec<相对方向Py>>,
        py: Python<'_>,
    ) -> PyResult<()> {
        let bi_list: Vec<Arc<chanlun::structure::dash_line::虚线>> = 笔序列
            .iter()
            .map(|d| Arc::clone(&d.bind(py).borrow().inner))
            .collect();
        let mut seg_seq: Vec<Arc<chanlun::structure::dash_line::虚线>> = 线段序列
            .iter()
            .map(|d| Arc::clone(&d.bind(py).borrow().inner))
            .collect();
        let config = 配置.borrow().to_rust_config(py)?;
        let default_rel = vec![
            chanlun::types::相对方向::向上,
            chanlun::types::相对方向::向下,
        ];
        let rel_list: Vec<chanlun::types::相对方向> = 关系序列
            .map(|v| v.into_iter().map(|d| d.inner).collect())
            .unwrap_or(default_rel);
        chanlun::algorithm::segment::线段::分析(
            &bi_list,
            &mut seg_seq,
            &config,
            层级,
            &rel_list,
        );
        Ok(())
    }

    #[classmethod]
    /// 即同级别分析
    fn 扩展分析(
        _cls: &Bound<'_, PyType>,
        虚线序列: Vec<Py<虚线Py>>,
        线段序列: Vec<Py<虚线Py>>,
        配置: &Bound<'_, 缠论配置Py>,
        py: Python<'_>,
    ) -> PyResult<()> {
        let dash_list: Vec<Arc<chanlun::structure::dash_line::虚线>> = 虚线序列
            .iter()
            .map(|d| Arc::clone(&d.bind(py).borrow().inner))
            .collect();
        let mut seg_seq: Vec<Arc<chanlun::structure::dash_line::虚线>> = 线段序列
            .iter()
            .map(|d| Arc::clone(&d.bind(py).borrow().inner))
            .collect();
        let config = 配置.borrow().to_rust_config(py)?;
        chanlun::algorithm::segment::线段::扩展分析(&dash_list, &mut seg_seq, &config);
        Ok(())
    }

    #[classmethod]
    /// 判断线段内部是否发生背驰（基于内部中枢和MACD）
    fn 判断线段内部是否背驰(
        _cls: &Bound<'_, PyType>,
        当前段: &Bound<'_, 虚线Py>,
        观察员: &Bound<'_, 观察者Py>,
    ) -> bool {
        let obs = 观察员.borrow();
        let obs_ref = obs.obs();
        chanlun::algorithm::segment::线段::判断线段内部是否背驰(
            &当前段.borrow().inner,
            &obs_ref,
        )
    }

    #[classmethod]
    /// 获取所有停顿位置
    fn 获取所有停顿位置(
        _cls: &Bound<'_, PyType>,
        段: &Bound<'_, 虚线Py>,
        观察员: &Bound<'_, 观察者Py>,
        py: Python<'_>,
    ) -> Vec<Py<虚线Py>> {
        let obs = 观察员.borrow();
        let obs_ref = obs.obs();
        chanlun::algorithm::segment::线段::获取所有停顿位置(&段.borrow().inner, &obs_ref)
            .into_iter()
            .map(|d| dashed_to_py(py, Arc::new(d)))
            .collect()
    }

    #[classmethod]
    /// 判断线段内是否发生过背驰（遍历所有停顿位置）
    fn 是否背驰过(
        _cls: &Bound<'_, PyType>,
        当前段: &Bound<'_, 虚线Py>,
        观察员: &Bound<'_, 观察者Py>,
        py: Python<'_>,
    ) -> Vec<Py<缠论K线Py>> {
        let obs = 观察员.borrow();
        let obs_ref = obs.obs();
        chanlun::algorithm::segment::线段::是否背驰过(&当前段.borrow().inner, &obs_ref)
            .into_iter()
            .map(|ck| chan_kline_to_py(py, ck))
            .collect()
    }
}

// ========== 中枢 ==========

/// 中枢 — 三段虚线重叠区间构成的价格中枢，支持延伸/扩展。
///
/// 属性 (只读):
///   序号: int / 标识: str (如 "中枢<笔>" / "中枢<线段>") / 级别: int
///   基础序列: list[虚线] — 构成中枢的三段虚线
///   第三买卖线: 虚线|None — 第三类买卖点确认线
///   本级_第三买卖线: 虚线|None
///
/// 计算属性:
///   高: float — 中枢上沿（三段虚线高点的最小值）
///   低: float — 中枢下沿（三段虚线低点的最大值）
///   高高: float — 所有虚线高点的最大值
///   低低: float — 所有虚线低点的最小值
///   方向: 相对方向 — 中枢方向（与第一段虚线方向相反）
///   文: 分型 — 中枢起点分型（第一段虚线的起点）
///   武: 分型 — 中枢终点分型（最后一段虚线的终点）
///   离开段: 虚线 — 中枢最后一段虚线
///   图表标题: str
///
/// 方法:
///   添加虚线(虚线) — 向中枢追加虚线（延伸）
///   获取序列() -> list[虚线] — 基础序列 + 第三买卖线
///   获取扩展中枢(扩展中枢列表, 配置) — 基础序列 >= 9 时生成扩展中枢
///   设置第三买卖线(虚线|None) — 设置第三类买卖确认线
///   完整性(虚实="合") -> bool — 中枢是否完整（已有第三买卖线确认）
///   当前状态() -> str — "中枢之上"/"中枢之下"/"中枢之中"
///   获取数据文本() -> str
///
/// 类方法:
///   基础检查(左虚线, 中虚线, 右虚线) -> bool — 检查三段虚线能否构成中枢
///   创建(左虚线, 中虚线, 右虚线, 级别, 标识?) -> 中枢 — 创建新中枢
///   从序列中获取中枢(虚线序列, 允许重叠?, 标识前缀?, 高级中枢容器?...) -> list[中枢]
///      — 扫描虚线序列提取所有中枢
///   分析(虚线序列, 中枢序列容器, 允许重叠?, 标识前缀?, ...) — 中枢分析主流程
#[pyclass(name = "中枢", module = "chanlun._chanlun", from_py_object)]
#[derive(Clone)]
pub struct 中枢Py {
    pub(crate) inner: Arc<chanlun::algorithm::hub::中枢>,
}

#[pymethods]
impl 中枢Py {
    #[new]
    fn new(
        序号: i64, 标识: String, 级别: i64, 基础序列: Vec<Py<虚线Py>>, py: Python<'_>
    ) -> Self {
        let rc_list: Vec<Arc<chanlun::structure::dash_line::虚线>> = 基础序列
            .iter()
            .map(|d| Arc::clone(&d.bind(py).borrow().inner))
            .collect();
        Self {
            inner: Arc::new(chanlun::algorithm::hub::中枢::new(
                序号, 标识, 级别, rc_list,
            )),
        }
    }

    #[getter]
    fn 序号(&self) -> i64 {
        self.inner.序号.load(Ordering::Relaxed)
    }

    #[getter]
    fn 标识(&self) -> String {
        self.inner.标识.read().unwrap().clone()
    }

    #[getter]
    fn 级别(&self) -> i64 {
        self.inner.级别.load(Ordering::Relaxed)
    }

    #[getter]
    fn 基础序列(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let list = pyo3::types::PyList::empty(py);
        for d in self.inner.基础序列.read().unwrap().iter() {
            list.append(dashed_to_py(py, Arc::clone(d)))?;
        }
        Ok(list.into())
    }

    #[getter]
    fn 第三买卖线(&self, py: Python<'_>) -> Option<Py<虚线Py>> {
        self.inner
            .第三买卖线
            .read()
            .unwrap()
            .as_ref()
            .map(|d| dashed_to_py(py, Arc::clone(d)))
    }

    #[getter]
    fn 本级_第三买卖线(&self, py: Python<'_>) -> Option<Py<虚线Py>> {
        self.inner
            .本级_第三买卖线
            .read()
            .unwrap()
            .as_ref()
            .map(|d| dashed_to_py(py, Arc::clone(d)))
    }

    #[getter]
    /// :return: 图表标题
    fn 图表标题(&self) -> String {
        self.inner.图表标题()
    }

    #[getter]
    /// :return: 最后一条虚线
    fn 离开段(&self, py: Python<'_>) -> Py<虚线Py> {
        dashed_to_py(py, self.inner.离开段())
    }

    #[getter]
    /// :return: 中枢方向（首条虚线的方向翻转）
    fn 方向(&self, py: Python<'_>) -> Py<相对方向Py> {
        crate::types_py::获取相对方向单例(py, self.inner.方向())
    }

    #[getter]
    /// :return: 中枢上沿（前三段中虚线高点的最小值）
    fn 高(&self) -> f64 {
        self.inner.高()
    }

    #[getter]
    /// :return: 中枢下沿（前三段中虚线低点的最大值）
    fn 低(&self) -> f64 {
        self.inner.低()
    }

    #[getter]
    /// :return: 全区间最高价
    fn 高高(&self) -> f64 {
        self.inner.高高()
    }

    #[getter]
    /// :return: 全区间最低价
    fn 低低(&self) -> f64 {
        self.inner.低低()
    }

    #[getter]
    /// :return: 起点分型
    fn 文(&self, py: Python<'_>) -> Py<分型Py> {
        fractal_to_py(py, self.inner.文())
    }

    #[getter]
    /// :return: 终点分型
    fn 武(&self, py: Python<'_>) -> Py<分型Py> {
        fractal_to_py(py, self.inner.武())
    }

    /// 设置第三类买卖点关联虚线
    fn 设置第三买卖线(&mut self, 线: &Bound<'_, 虚线Py>) {
        let inner = Arc::make_mut(&mut self.inner);
        inner.设置第三买卖线(Some(Arc::clone(&线.borrow().inner)));
    }

    /// 获取中枢的完整虚线序列（基础序列+第三买卖线）
    fn 获取序列(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let list = pyo3::types::PyList::empty(py);
        for d in self.inner.获取序列() {
            list.append(dashed_to_py(py, d))?;
        }
        Ok(list.into())
    }

    /// 获取用于保存的数据文本
    fn 获取数据文本(&self) -> String {
        self.inner.获取数据文本()
    }

    #[pyo3(signature = (虚实 = "合"))]
    /// 判断中枢是否完整（是否有第三买卖点或内部中枢离开）
    fn 完整性(&self, 虚实: &str) -> bool {
        self.inner.完整性(虚实)
    }

    /// 当基础序列>=9时，从中枢中提取扩展线段中枢
    fn 获取扩展中枢(
        &self,
        扩展中枢: Vec<Py<Self>>,
        配置: &Bound<'_, crate::config_py::缠论配置Py>,
        py: Python<'_>,
    ) -> PyResult<()> {
        let mut hub_seq: Vec<Arc<chanlun::algorithm::hub::中枢>> = 扩展中枢
            .iter()
            .map(|h| Arc::clone(&h.bind(py).borrow().inner))
            .collect();
        let config = 配置.borrow().to_rust_config(配置.py())?;
        self.inner.获取扩展中枢(&mut hub_seq, &config);
        Ok(())
    }

    /// 获取中枢当前状态：中枢之中/中枢之上/中枢之下
    fn 当前状态(&self) -> String {
        self.inner.当前状态().to_string()
    }

    /// pandas 兼容 — 返回关键标量字段构成的字典
    #[getter]
    fn __dict__(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let dict = PyDict::new(py);
        dict.set_item("序号", self.序号())?;
        dict.set_item("标识", self.标识())?;
        dict.set_item("级别", self.级别())?;
        dict.set_item("图表标题", self.图表标题())?;
        dict.set_item("方向", self.方向(py))?;
        dict.set_item("高", self.高())?;
        dict.set_item("低", self.低())?;
        dict.set_item("高高", self.高高())?;
        dict.set_item("低低", self.低低())?;
        dict.set_item("当前状态", self.当前状态())?;
        Ok(dict.into())
    }

    fn __str__(&self) -> String {
        format!("{}", self.inner)
    }

    fn __repr__(&self) -> String {
        self.__str__()
    }

    fn __eq__(&self, other: &Bound<'_, PyAny>) -> bool {
        if let Ok(other) = other.extract::<PyRef<'_, Self>>() {
            return Arc::as_ptr(&self.inner) == Arc::as_ptr(&other.inner);
        }
        false
    }

    fn __hash__(&self) -> u64 {
        Arc::as_ptr(&self.inner) as u64
    }

    // ---- classmethods ----

    #[classmethod]
    /// 检查三条虚线是否构成中枢（连续且重叠）
    fn 基础检查(
        _cls: &Bound<'_, PyType>,
        左: &Bound<'_, 虚线Py>,
        中: &Bound<'_, 虚线Py>,
        右: &Bound<'_, 虚线Py>,
    ) -> bool {
        chanlun::algorithm::hub::中枢::基础检查(
            &左.borrow().inner,
            &中.borrow().inner,
            &右.borrow().inner,
        )
    }

    #[classmethod]
    #[pyo3(signature = (左, 中, 右, 级别, 标识 = ""))]
    /// 从三条连续且重叠的虚线创建中枢
    fn 创建(
        _cls: &Bound<'_, PyType>,
        左: &Bound<'_, 虚线Py>,
        中: &Bound<'_, 虚线Py>,
        右: &Bound<'_, 虚线Py>,
        级别: i64,
        标识: &str,
    ) -> Py<中枢Py> {
        let inner = Arc::new(chanlun::algorithm::hub::中枢::创建(
            Arc::clone(&左.borrow().inner),
            Arc::clone(&中.borrow().inner),
            Arc::clone(&右.borrow().inner),
            级别,
            标识,
        ));
        hub_to_py(_cls.py(), inner)
    }

    #[classmethod]
    #[pyo3(signature = (虚线序列, 中枢序列, 跳过首部 = true, 标识 = "", 层级 = 0))]
    /// 中枢识别核心递归算法
    fn 分析(
        _cls: &Bound<'_, PyType>,
        虚线序列: Vec<Py<虚线Py>>,
        中枢序列: Vec<Py<Self>>,
        跳过首部: bool,
        标识: &str,
        层级: i64,
        py: Python<'_>,
    ) -> PyResult<()> {
        let rc_list: Vec<Arc<chanlun::structure::dash_line::虚线>> = 虚线序列
            .iter()
            .map(|d| Arc::clone(&d.bind(py).borrow().inner))
            .collect();
        let mut hub_seq: Vec<Arc<chanlun::algorithm::hub::中枢>> = 中枢序列
            .iter()
            .map(|h| Arc::clone(&h.bind(py).borrow().inner))
            .collect();
        chanlun::algorithm::hub::中枢::分析(&rc_list, &mut hub_seq, 跳过首部, 标识, 层级);
        Ok(())
    }

    #[classmethod]
    /// 从虚线序列中按起始方向查找第一个中枢
    fn 从序列中获取中枢(
        _cls: &Bound<'_, PyType>,
        虚线序列: Vec<Py<虚线Py>>,
        起始方向: &Bound<'_, 相对方向Py>,
        标识: &str,
        py: Python<'_>,
    ) -> Option<Py<中枢Py>> {
        let rc_list: Vec<Arc<chanlun::structure::dash_line::虚线>> = 虚线序列
            .iter()
            .map(|d| Arc::clone(&d.bind(py).borrow().inner))
            .collect();
        chanlun::algorithm::hub::中枢::_从序列中获取中枢(
            &rc_list,
            起始方向.borrow().inner,
            标识,
        )
        .map(|inner| hub_to_py(py, inner))
    }
}

pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<背驰分析Py>()?;
    m.add_class::<笔Py>()?;
    m.add_class::<线段Py>()?;
    m.add_class::<中枢Py>()?;
    Ok(())
}
