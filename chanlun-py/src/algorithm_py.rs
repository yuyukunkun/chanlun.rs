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

use pyo3::prelude::*;
use pyo3::types::PyType;
use std::rc::Rc;

use crate::business_py::观察者Py;
use crate::config_py::缠论配置Py;
use crate::kline_py::K线Py;
use crate::structure_py::{分型Py, 虚线Py};
use crate::types_py::相对方向Py;

// ========== 背驰分析 ==========

#[pyclass(name = "背驰分析")]
pub struct 背驰分析Py;

#[pymethods]
impl 背驰分析Py {
    #[classmethod]
    #[pyo3(signature = (进入段, 离开段, K线序列, 方式 = "总"))]
    fn MACD背驰(
        _cls: &Bound<'_, PyType>,
        进入段: &Bound<'_, 虚线Py>,
        离开段: &Bound<'_, 虚线Py>,
        K线序列: Vec<Py<K线Py>>,
        方式: &str,
        py: Python<'_>,
    ) -> bool {
        let rc_list: Vec<Rc<chanlun::kline::bar::K线>> = K线序列
            .iter()
            .map(|k| Rc::new(k.bind(py).borrow().inner.clone()))
            .collect();
        chanlun::algorithm::divergence::背驰分析::MACD背驰(
            &进入段.borrow().inner,
            &离开段.borrow().inner,
            &rc_list,
            方式,
        )
    }

    #[classmethod]
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
    fn 全量背驰(
        _cls: &Bound<'_, PyType>,
        进入段: &Bound<'_, 虚线Py>,
        离开段: &Bound<'_, 虚线Py>,
        普K序列: Vec<Py<K线Py>>,
        py: Python<'_>,
    ) -> bool {
        let rc_list: Vec<Rc<chanlun::kline::bar::K线>> = 普K序列
            .iter()
            .map(|k| Rc::new(k.bind(py).borrow().inner.clone()))
            .collect();
        chanlun::algorithm::divergence::背驰分析::全量背驰(
            &进入段.borrow().inner,
            &离开段.borrow().inner,
            &rc_list,
        )
    }

    #[classmethod]
    fn 任意背驰(
        _cls: &Bound<'_, PyType>,
        进入段: &Bound<'_, 虚线Py>,
        离开段: &Bound<'_, 虚线Py>,
        普K序列: Vec<Py<K线Py>>,
        py: Python<'_>,
    ) -> bool {
        let rc_list: Vec<Rc<chanlun::kline::bar::K线>> = 普K序列
            .iter()
            .map(|k| Rc::new(k.bind(py).borrow().inner.clone()))
            .collect();
        chanlun::algorithm::divergence::背驰分析::任意背驰(
            &进入段.borrow().inner,
            &离开段.borrow().inner,
            &rc_list,
        )
    }

    #[classmethod]
    fn 配置背驰(
        _cls: &Bound<'_, PyType>,
        进入段: &Bound<'_, 虚线Py>,
        离开段: &Bound<'_, 虚线Py>,
        普K序列: Vec<Py<K线Py>>,
        配置: &Bound<'_, 缠论配置Py>,
        py: Python<'_>,
    ) -> PyResult<bool> {
        let rc_list: Vec<Rc<chanlun::kline::bar::K线>> = 普K序列
            .iter()
            .map(|k| Rc::new(k.bind(py).borrow().inner.clone()))
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
    fn 任选背驰(
        _cls: &Bound<'_, PyType>,
        进入段: &Bound<'_, 虚线Py>,
        离开段: &Bound<'_, 虚线Py>,
        普K序列: Vec<Py<K线Py>>,
        py: Python<'_>,
    ) -> bool {
        let rc_list: Vec<Rc<chanlun::kline::bar::K线>> = 普K序列
            .iter()
            .map(|k| Rc::new(k.bind(py).borrow().inner.clone()))
            .collect();
        chanlun::algorithm::divergence::背驰分析::任选背驰(
            &进入段.borrow().inner,
            &离开段.borrow().inner,
            &rc_list,
        )
    }

    #[classmethod]
    fn 背驰模式(
        _cls: &Bound<'_, PyType>,
        进入段: &Bound<'_, 虚线Py>,
        离开段: &Bound<'_, 虚线Py>,
        普K序列: Vec<Py<K线Py>>,
        配置: &Bound<'_, 缠论配置Py>,
        模式: &str,
        py: Python<'_>,
    ) -> PyResult<bool> {
        let rc_list: Vec<Rc<chanlun::kline::bar::K线>> = 普K序列
            .iter()
            .map(|k| Rc::new(k.bind(py).borrow().inner.clone()))
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

#[pyclass(name = "笔")]
pub struct 笔Py;

#[pymethods]
impl 笔Py {
    #[classmethod]
    fn 获取缠K数量(
        _cls: &Bound<'_, PyType>,
        缠K序列: Vec<Py<crate::kline_py::缠论K线Py>>,
        笔序列: Vec<Py<虚线Py>>,
        配置: &Bound<'_, 缠论配置Py>,
        py: Python<'_>,
    ) -> PyResult<usize> {
        let ck_list: Vec<Rc<chanlun::kline::chan_kline::缠论K线>> = 缠K序列
            .iter()
            .map(|k| Rc::clone(&k.bind(py).borrow().inner))
            .collect();
        let bi_list: Vec<Rc<chanlun::structure::dash_line::虚线>> = 笔序列
            .iter()
            .map(|d| Rc::clone(&d.bind(py).borrow().inner))
            .collect();
        let config = 配置.borrow().to_rust_config(py)?;
        Ok(chanlun::algorithm::bi::笔::获取缠K数量(
            &ck_list, &bi_list, &config,
        ))
    }

    #[classmethod]
    fn 次高(
        _cls: &Bound<'_, PyType>,
        缠K序列: Vec<Py<crate::kline_py::缠论K线Py>>,
        取舍: bool,
        py: Python<'_>,
    ) -> Option<crate::kline_py::缠论K线Py> {
        let ck_list: Vec<Rc<chanlun::kline::chan_kline::缠论K线>> = 缠K序列
            .iter()
            .map(|k| Rc::clone(&k.bind(py).borrow().inner))
            .collect();
        chanlun::algorithm::bi::笔::次高(&ck_list, 取舍)
            .map(|inner| crate::kline_py::缠论K线Py { inner })
    }

    #[classmethod]
    fn 次低(
        _cls: &Bound<'_, PyType>,
        缠K序列: Vec<Py<crate::kline_py::缠论K线Py>>,
        取舍: bool,
        py: Python<'_>,
    ) -> Option<crate::kline_py::缠论K线Py> {
        let ck_list: Vec<Rc<chanlun::kline::chan_kline::缠论K线>> = 缠K序列
            .iter()
            .map(|k| Rc::clone(&k.bind(py).borrow().inner))
            .collect();
        chanlun::algorithm::bi::笔::次低(&ck_list, 取舍)
            .map(|inner| crate::kline_py::缠论K线Py { inner })
    }

    #[classmethod]
    fn 实际高点(
        _cls: &Bound<'_, PyType>,
        缠K序列: Vec<Py<crate::kline_py::缠论K线Py>>,
        取舍: bool,
        py: Python<'_>,
    ) -> Option<crate::kline_py::缠论K线Py> {
        let ck_list: Vec<Rc<chanlun::kline::chan_kline::缠论K线>> = 缠K序列
            .iter()
            .map(|k| Rc::clone(&k.bind(py).borrow().inner))
            .collect();
        chanlun::algorithm::bi::笔::实际高点(&ck_list, 取舍)
            .map(|inner| crate::kline_py::缠论K线Py { inner })
    }

    #[classmethod]
    fn 实际低点(
        _cls: &Bound<'_, PyType>,
        缠K序列: Vec<Py<crate::kline_py::缠论K线Py>>,
        取舍: bool,
        py: Python<'_>,
    ) -> Option<crate::kline_py::缠论K线Py> {
        let ck_list: Vec<Rc<chanlun::kline::chan_kline::缠论K线>> = 缠K序列
            .iter()
            .map(|k| Rc::clone(&k.bind(py).borrow().inner))
            .collect();
        chanlun::algorithm::bi::笔::实际低点(&ck_list, 取舍)
            .map(|inner| crate::kline_py::缠论K线Py { inner })
    }

    #[classmethod]
    fn 相对关系(
        _cls: &Bound<'_, PyType>,
        筆: &Bound<'_, 虚线Py>,
        配置: &Bound<'_, 缠论配置Py>,
        py: Python<'_>,
    ) -> PyResult<bool> {
        let config = 配置.borrow().to_rust_config(py)?;
        Ok(chanlun::algorithm::bi::笔::相对关系(
            &筆.borrow().inner,
            &config,
        ))
    }

    #[classmethod]
    fn 以文会友(
        _cls: &Bound<'_, PyType>,
        笔序列: Vec<Py<虚线Py>>,
        文: &Bound<'_, 分型Py>,
        py: Python<'_>,
    ) -> Option<虚线Py> {
        let bi_list: Vec<Rc<chanlun::structure::dash_line::虚线>> = 笔序列
            .iter()
            .map(|d| Rc::clone(&d.bind(py).borrow().inner))
            .collect();
        chanlun::algorithm::bi::笔::以文会友(&bi_list, &文.borrow().inner)
            .map(|inner| 虚线Py { inner })
    }

    #[classmethod]
    fn 以武会友(
        _cls: &Bound<'_, PyType>,
        笔序列: Vec<Py<虚线Py>>,
        武: &Bound<'_, 分型Py>,
        py: Python<'_>,
    ) -> Option<虚线Py> {
        let bi_list: Vec<Rc<chanlun::structure::dash_line::虚线>> = 笔序列
            .iter()
            .map(|d| Rc::clone(&d.bind(py).borrow().inner))
            .collect();
        chanlun::algorithm::bi::笔::以武会友(&bi_list, &武.borrow().inner)
            .map(|inner| 虚线Py { inner })
    }

    #[classmethod]
    #[pyo3(signature = (笔序列, 缠K, 偏移 = 1))]
    fn 根据缠K找笔(
        _cls: &Bound<'_, PyType>,
        笔序列: Vec<Py<虚线Py>>,
        缠K: &Bound<'_, crate::kline_py::缠论K线Py>,
        偏移: i64,
        py: Python<'_>,
    ) -> Option<虚线Py> {
        let bi_list: Vec<Rc<chanlun::structure::dash_line::虚线>> = 笔序列
            .iter()
            .map(|d| Rc::clone(&d.bind(py).borrow().inner))
            .collect();
        chanlun::algorithm::bi::笔::根据缠K找笔(&bi_list, &缠K.borrow().inner, 偏移)
            .map(|inner| 虚线Py { inner })
    }

    #[classmethod]
    fn 分析(
        _cls: &Bound<'_, PyType>,
        当前分型: &Bound<'_, 分型Py>,
        分型序列: Vec<Py<分型Py>>,
        笔序列: Vec<Py<虚线Py>>,
        缠K序列: Vec<Py<crate::kline_py::缠论K线Py>>,
        普K序列: Vec<Py<K线Py>>,
        配置: &Bound<'_, 缠论配置Py>,
        py: Python<'_>,
    ) -> PyResult<i64> {
        let mut fr_seq: Vec<Rc<chanlun::structure::fractal_obj::分型>> = 分型序列
            .iter()
            .map(|f| Rc::clone(&f.bind(py).borrow().inner))
            .collect();
        let mut bi_seq: Vec<Rc<chanlun::structure::dash_line::虚线>> = 笔序列
            .iter()
            .map(|d| Rc::clone(&d.bind(py).borrow().inner))
            .collect();
        let ck_list: Vec<Rc<chanlun::kline::chan_kline::缠论K线>> = 缠K序列
            .iter()
            .map(|k| Rc::clone(&k.bind(py).borrow().inner))
            .collect();
        let bar_list: Vec<Rc<chanlun::kline::bar::K线>> = 普K序列
            .iter()
            .map(|k| Rc::new(k.bind(py).borrow().inner.clone()))
            .collect();
        let config = 配置.borrow().to_rust_config(py)?;
        Ok(chanlun::algorithm::bi::笔::分析(
            Rc::clone(&当前分型.borrow().inner),
            &mut fr_seq,
            &mut bi_seq,
            &ck_list,
            &bar_list,
            &config,
        ))
    }

    #[classmethod]
    fn 分析递归(
        _cls: &Bound<'_, PyType>,
        当前分型: &Bound<'_, 分型Py>,
        分型序列: Vec<Py<分型Py>>,
        笔序列: Vec<Py<虚线Py>>,
        缠K序列: Vec<Py<crate::kline_py::缠论K线Py>>,
        普K序列: Vec<Py<K线Py>>,
        递归层次: i64,
        配置: &Bound<'_, 缠论配置Py>,
        py: Python<'_>,
    ) -> PyResult<i64> {
        let mut fr_seq: Vec<Rc<chanlun::structure::fractal_obj::分型>> = 分型序列
            .iter()
            .map(|f| Rc::clone(&f.bind(py).borrow().inner))
            .collect();
        let mut bi_seq: Vec<Rc<chanlun::structure::dash_line::虚线>> = 笔序列
            .iter()
            .map(|d| Rc::clone(&d.bind(py).borrow().inner))
            .collect();
        let ck_list: Vec<Rc<chanlun::kline::chan_kline::缠论K线>> = 缠K序列
            .iter()
            .map(|k| Rc::clone(&k.bind(py).borrow().inner))
            .collect();
        let bar_list: Vec<Rc<chanlun::kline::bar::K线>> = 普K序列
            .iter()
            .map(|k| Rc::new(k.bind(py).borrow().inner.clone()))
            .collect();
        let config = 配置.borrow().to_rust_config(py)?;
        Ok(chanlun::algorithm::bi::笔::分析递归(
            Rc::clone(&当前分型.borrow().inner),
            &mut fr_seq,
            &mut bi_seq,
            &ck_list,
            &bar_list,
            递归层次,
            &config,
        ))
    }

    #[classmethod]
    fn 自检(
        _cls: &Bound<'_, PyType>,
        筆: &Bound<'_, 虚线Py>,
        观察员: &Bound<'_, 观察者Py>,
    ) -> bool {
        let obs = 观察员.borrow();
        let obs_ref = obs.obs();
        chanlun::algorithm::bi::笔::自检(&筆.borrow().inner, &*obs_ref)
    }

    #[classmethod]
    fn 获取所有停顿位置(
        _cls: &Bound<'_, PyType>,
        筆: &Bound<'_, 虚线Py>,
        观察员: &Bound<'_, 观察者Py>,
    ) -> Vec<虚线Py> {
        let obs = 观察员.borrow();
        let obs_ref = obs.obs();
        chanlun::algorithm::bi::笔::获取所有停顿位置(&筆.borrow().inner, &*obs_ref)
            .into_iter()
            .map(|d| 虚线Py { inner: Rc::new(d) })
            .collect()
    }

    #[classmethod]
    fn 是否背驰过(
        _cls: &Bound<'_, PyType>,
        当前筆: &Bound<'_, 虚线Py>,
        观察员: &Bound<'_, 观察者Py>,
    ) -> Vec<分型Py> {
        let obs = 观察员.borrow();
        let obs_ref = obs.obs();
        chanlun::algorithm::bi::笔::是否背驰过(&当前筆.borrow().inner, &*obs_ref)
            .into_iter()
            .map(|f| 分型Py { inner: f })
            .collect()
    }
}

// ========== 线段 ==========

#[pyclass(name = "线段")]
pub struct 线段Py;

#[pymethods]
impl 线段Py {
    #[classmethod]
    fn 添加虚线(
        _cls: &Bound<'_, PyType>,
        段: &Bound<'_, 虚线Py>,
        筆: &Bound<'_, 虚线Py>,
    ) -> PyResult<()> {
        let bi_rc = Rc::clone(&筆.borrow().inner);
        let mut ref_mut = 段.borrow_mut();
        chanlun::algorithm::segment::线段::添加虚线(&mut ref_mut.inner, bi_rc);
        Ok(())
    }

    #[classmethod]
    fn 武斗(
        _cls: &Bound<'_, PyType>,
        段: &Bound<'_, 虚线Py>,
        武: &Bound<'_, 分型Py>,
        行号: u32,
    ) -> PyResult<()> {
        let mut ref_mut = 段.borrow_mut();
        chanlun::algorithm::segment::线段::武斗(
            &mut ref_mut.inner,
            &Rc::clone(&武.borrow().inner),
            行号,
        );
        Ok(())
    }

    #[classmethod]
    fn 武终(_cls: &Bound<'_, PyType>, 段: &Bound<'_, 虚线Py>, 行号: u32) -> PyResult<()> {
        let mut ref_mut = 段.borrow_mut();
        chanlun::algorithm::segment::线段::武终(&mut ref_mut.inner, 行号);
        Ok(())
    }

    #[classmethod]
    fn 验证序列(
        _cls: &Bound<'_, PyType>,
        段: &Bound<'_, 虚线Py>,
        序列: Vec<Py<虚线Py>>,
        py: Python<'_>,
    ) -> PyResult<()> {
        let mut ref_mut = 段.borrow_mut();
        let rc_list: Vec<Rc<chanlun::structure::dash_line::虚线>> = 序列
            .iter()
            .map(|d| Rc::clone(&d.bind(py).borrow().inner))
            .collect();
        chanlun::algorithm::segment::线段::验证序列(&mut ref_mut.inner, &rc_list);
        Ok(())
    }

    #[classmethod]
    fn 序列重置(
        _cls: &Bound<'_, PyType>,
        段: &Bound<'_, 虚线Py>,
        序列: Vec<Py<虚线Py>>,
        py: Python<'_>,
    ) -> PyResult<()> {
        let mut ref_mut = 段.borrow_mut();
        let rc_list: Vec<Rc<chanlun::structure::dash_line::虚线>> = 序列
            .iter()
            .map(|d| Rc::clone(&d.bind(py).borrow().inner))
            .collect();
        chanlun::algorithm::segment::线段::序列重置(&mut ref_mut.inner, &rc_list);
        Ok(())
    }

    #[classmethod]
    fn 基础判断(
        _cls: &Bound<'_, PyType>,
        左: &Bound<'_, 虚线Py>,
        中: &Bound<'_, 虚线Py>,
        右: &Bound<'_, 虚线Py>,
        关系序列: Vec<相对方向Py>,
    ) -> bool {
        let rel_list: Vec<chanlun::types::相对方向> = 关系序列.iter().map(|d| d.inner).collect();
        chanlun::algorithm::segment::线段::基础判断(
            &左.borrow().inner,
            &中.borrow().inner,
            &右.borrow().inner,
            &rel_list,
        )
    }

    #[classmethod]
    fn 四象(_cls: &Bound<'_, PyType>, 段: &Bound<'_, 虚线Py>) -> String {
        chanlun::algorithm::segment::线段::四象(&段.borrow().inner)
    }

    #[classmethod]
    fn 获取缺口(
        _cls: &Bound<'_, PyType>,
        段: &Bound<'_, 虚线Py>,
    ) -> Option<crate::types_py::缺口Py> {
        chanlun::algorithm::segment::线段::获取缺口(&段.borrow().inner)
            .map(|inner| crate::types_py::缺口Py { inner })
    }

    #[classmethod]
    fn 特征分型终结(_cls: &Bound<'_, PyType>, 段: &Bound<'_, 虚线Py>) -> bool {
        chanlun::algorithm::segment::线段::特征分型终结(&段.borrow().inner)
    }

    #[classmethod]
    fn 特征序列状态(
        _cls: &Bound<'_, PyType>, 段: &Bound<'_, 虚线Py>
    ) -> (bool, bool, bool) {
        chanlun::algorithm::segment::线段::特征序列状态(&段.borrow().inner)
    }

    #[classmethod]
    fn 设置特征序列(
        _cls: &Bound<'_, PyType>,
        段: &Bound<'_, 虚线Py>,
        序列: &Bound<'_, PyAny>,
        行号: u32,
    ) -> PyResult<()> {
        // 特征序列 contains Option<Rc<线段特征>>, skip for now
        let mut ref_mut = 段.borrow_mut();
        // Just pass empty vec for now — 线段特征Py not fully integrated
        chanlun::algorithm::segment::线段::设置特征序列(&mut ref_mut.inner, vec![], 行号);
        Ok(())
    }

    #[classmethod]
    fn 刷新特征序列(
        _cls: &Bound<'_, PyType>,
        段: &Bound<'_, 虚线Py>,
        配置: &Bound<'_, 缠论配置Py>,
        py: Python<'_>,
    ) -> PyResult<()> {
        let mut ref_mut = 段.borrow_mut();
        let config = 配置.borrow().to_rust_config(py)?;
        chanlun::algorithm::segment::线段::刷新特征序列(&mut ref_mut.inner, &config);
        Ok(())
    }

    #[classmethod]
    fn 查找贯穿伤(_cls: &Bound<'_, PyType>, 段: &Bound<'_, 虚线Py>) -> Option<虚线Py> {
        chanlun::algorithm::segment::线段::查找贯穿伤(&段.borrow().inner)
            .map(|inner| 虚线Py { inner })
    }

    #[classmethod]
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
        // 所属中枢 extraction is complex — pass None for now
        let (a, b, c, d) =
            chanlun::algorithm::segment::线段::分割序列(&段.borrow().inner, None);
        let wrap = |v: Vec<Rc<chanlun::structure::dash_line::虚线>>| -> PyResult<Vec<Py<虚线Py>>> {
            let mut result = Vec::new();
            for x in v {
                result.push(Py::new(py, 虚线Py { inner: x })?);
            }
            Ok(result)
        };
        Ok((
            wrap(a)?,
            wrap(b)?,
            wrap(c)?,
            d.map(|x| Py::new(py, 虚线Py { inner: x })).transpose()?,
        ))
    }

    #[classmethod]
    fn 刷新(
        _cls: &Bound<'_, PyType>,
        段: &Bound<'_, 虚线Py>,
        配置: &Bound<'_, 缠论配置Py>,
        py: Python<'_>,
    ) -> PyResult<()> {
        let mut ref_mut = 段.borrow_mut();
        let config = 配置.borrow().to_rust_config(py)?;
        chanlun::algorithm::segment::线段::刷新(&mut ref_mut.inner, &config);
        Ok(())
    }

    #[classmethod]
    fn 获取内部中枢序列(
        _cls: &Bound<'_, PyType>,
        段: &Bound<'_, 虚线Py>,
        配置: &Bound<'_, 缠论配置Py>,
        py: Python<'_>,
    ) -> PyResult<(Py<PyAny>, Py<PyAny>, Py<PyAny>)> {
        let mut ref_mut = 段.borrow_mut();
        let config = 配置.borrow().to_rust_config(py)?;
        let (a, b, c) = chanlun::algorithm::segment::线段::获取内部中枢序列(
            &mut ref_mut.inner,
            &config,
        );
        let pk_list = |v: Vec<Rc<chanlun::algorithm::hub::中枢>>| -> PyResult<Py<PyAny>> {
            let list = pyo3::types::PyList::empty(py);
            for h in v {
                list.append(Py::new(py, 中枢Py { inner: h })?)?;
            }
            Ok(list.into())
        };
        Ok((pk_list(a)?, pk_list(b)?, pk_list(c)?))
    }

    #[classmethod]
    fn _添加线段(
        _cls: &Bound<'_, PyType>,
        线段序列: &Bound<'_, PyAny>,
        待添加线段: &Bound<'_, 虚线Py>,
        配置: &Bound<'_, 缠论配置Py>,
        行号: String,
        py: Python<'_>,
    ) -> PyResult<()> {
        let config = 配置.borrow().to_rust_config(py)?;
        let mut seg_seq: Vec<Rc<chanlun::structure::dash_line::虚线>> = vec![];
        chanlun::algorithm::segment::线段::_添加线段(
            &mut seg_seq,
            Rc::clone(&待添加线段.borrow().inner),
            &config,
            行号,
        );
        Ok(())
    }

    #[classmethod]
    fn _弹出线段(
        _cls: &Bound<'_, PyType>,
        线段序列: &Bound<'_, PyAny>,
        待弹出线段: &Bound<'_, 虚线Py>,
        配置: &Bound<'_, 缠论配置Py>,
        行号: String,
        py: Python<'_>,
    ) -> PyResult<Option<虚线Py>> {
        let config = 配置.borrow().to_rust_config(py)?;
        let mut seg_seq: Vec<Rc<chanlun::structure::dash_line::虚线>> = vec![];
        let result = chanlun::algorithm::segment::线段::_弹出线段(
            &mut seg_seq,
            &Rc::clone(&待弹出线段.borrow().inner),
            &config,
            行号,
        );
        Ok(result.map(|inner| 虚线Py { inner }))
    }

    #[classmethod]
    fn _缺口突破(
        _cls: &Bound<'_, PyType>,
        线段序列: Vec<Py<虚线Py>>,
        配置: &Bound<'_, 缠论配置Py>,
        层级: i64,
        py: Python<'_>,
    ) -> PyResult<bool> {
        let mut seg_seq: Vec<Rc<chanlun::structure::dash_line::虚线>> = 线段序列
            .iter()
            .map(|d| Rc::clone(&d.bind(py).borrow().inner))
            .collect();
        let config = 配置.borrow().to_rust_config(py)?;
        Ok(chanlun::algorithm::segment::线段::_缺口突破(
            &mut seg_seq,
            &config,
            层级,
        ))
    }

    #[classmethod]
    fn _非缺口下穿刺(
        _cls: &Bound<'_, PyType>,
        线段序列: Vec<Py<虚线Py>>,
        配置: &Bound<'_, 缠论配置Py>,
        层级: i64,
        py: Python<'_>,
    ) -> PyResult<bool> {
        let mut seg_seq: Vec<Rc<chanlun::structure::dash_line::虚线>> = 线段序列
            .iter()
            .map(|d| Rc::clone(&d.bind(py).borrow().inner))
            .collect();
        let config = 配置.borrow().to_rust_config(py)?;
        Ok(chanlun::algorithm::segment::线段::_非缺口下穿刺(
            &mut seg_seq,
            &config,
            层级,
        ))
    }

    #[classmethod]
    fn _缺口后紧急修正(
        _cls: &Bound<'_, PyType>,
        线段序列: Vec<Py<虚线Py>>,
        配置: &Bound<'_, 缠论配置Py>,
        层级: i64,
        py: Python<'_>,
    ) -> PyResult<bool> {
        let mut seg_seq: Vec<Rc<chanlun::structure::dash_line::虚线>> = 线段序列
            .iter()
            .map(|d| Rc::clone(&d.bind(py).borrow().inner))
            .collect();
        let config = 配置.borrow().to_rust_config(py)?;
        Ok(chanlun::algorithm::segment::线段::_缺口后紧急修正(
            &mut seg_seq,
            &config,
            层级,
        ))
    }

    #[classmethod]
    fn _修正(
        _cls: &Bound<'_, PyType>,
        线段序列: Vec<Py<虚线Py>>,
        配置: &Bound<'_, 缠论配置Py>,
        层级: i64,
        py: Python<'_>,
    ) -> PyResult<bool> {
        let mut seg_seq: Vec<Rc<chanlun::structure::dash_line::虚线>> = 线段序列
            .iter()
            .map(|d| Rc::clone(&d.bind(py).borrow().inner))
            .collect();
        let config = 配置.borrow().to_rust_config(py)?;
        Ok(chanlun::algorithm::segment::线段::_修正(
            &mut seg_seq,
            &config,
            层级,
        ))
    }

    #[classmethod]
    #[pyo3(signature = (笔序列, 线段序列, 配置, 层级 = 0, 关系序列 = None))]
    fn 分析(
        _cls: &Bound<'_, PyType>,
        笔序列: Vec<Py<虚线Py>>,
        线段序列: Vec<Py<虚线Py>>,
        配置: &Bound<'_, 缠论配置Py>,
        层级: i64,
        关系序列: Option<Vec<相对方向Py>>,
        py: Python<'_>,
    ) -> PyResult<()> {
        let bi_list: Vec<Rc<chanlun::structure::dash_line::虚线>> = 笔序列
            .iter()
            .map(|d| Rc::clone(&d.bind(py).borrow().inner))
            .collect();
        let mut seg_seq: Vec<Rc<chanlun::structure::dash_line::虚线>> = 线段序列
            .iter()
            .map(|d| Rc::clone(&d.bind(py).borrow().inner))
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
    fn _添加扩展线段(
        _cls: &Bound<'_, PyType>,
        线段序列: Vec<Py<虚线Py>>,
        待添加线段: &Bound<'_, 虚线Py>,
        行号: u32,
        py: Python<'_>,
    ) -> PyResult<()> {
        let mut seg_seq: Vec<Rc<chanlun::structure::dash_line::虚线>> = 线段序列
            .iter()
            .map(|d| Rc::clone(&d.bind(py).borrow().inner))
            .collect();
        chanlun::algorithm::segment::线段::_添加扩展线段(
            &mut seg_seq,
            Rc::clone(&待添加线段.borrow().inner),
            行号,
        );
        Ok(())
    }

    #[classmethod]
    fn _弹出扩展线段(
        _cls: &Bound<'_, PyType>,
        线段序列: Vec<Py<虚线Py>>,
        待弹出线段: &Bound<'_, 虚线Py>,
        行号: u32,
        py: Python<'_>,
    ) -> PyResult<Option<虚线Py>> {
        let mut seg_seq: Vec<Rc<chanlun::structure::dash_line::虚线>> = 线段序列
            .iter()
            .map(|d| Rc::clone(&d.bind(py).borrow().inner))
            .collect();
        let result = chanlun::algorithm::segment::线段::_弹出扩展线段(
            &mut seg_seq,
            &Rc::clone(&待弹出线段.borrow().inner),
            行号,
        );
        Ok(result.map(|inner| 虚线Py { inner }))
    }

    #[classmethod]
    fn 扩展分析(
        _cls: &Bound<'_, PyType>,
        虚线序列: Vec<Py<虚线Py>>,
        线段序列: Vec<Py<虚线Py>>,
        配置: &Bound<'_, 缠论配置Py>,
        py: Python<'_>,
    ) -> PyResult<()> {
        let dash_list: Vec<Rc<chanlun::structure::dash_line::虚线>> = 虚线序列
            .iter()
            .map(|d| Rc::clone(&d.bind(py).borrow().inner))
            .collect();
        let mut seg_seq: Vec<Rc<chanlun::structure::dash_line::虚线>> = 线段序列
            .iter()
            .map(|d| Rc::clone(&d.bind(py).borrow().inner))
            .collect();
        let config = 配置.borrow().to_rust_config(py)?;
        chanlun::algorithm::segment::线段::扩展分析(&dash_list, &mut seg_seq, &config);
        Ok(())
    }

    #[classmethod]
    fn 判断线段内部是否背驰(
        _cls: &Bound<'_, PyType>,
        当前段: &Bound<'_, 虚线Py>,
        观察员: &Bound<'_, 观察者Py>,
    ) -> bool {
        let obs = 观察员.borrow();
        let obs_ref = obs.obs();
        chanlun::algorithm::segment::线段::判断线段内部是否背驰(
            &当前段.borrow().inner,
            &*obs_ref,
        )
    }

    #[classmethod]
    fn 段获取所有停顿位置(
        _cls: &Bound<'_, PyType>,
        段: &Bound<'_, 虚线Py>,
        观察员: &Bound<'_, 观察者Py>,
    ) -> Vec<虚线Py> {
        let obs = 观察员.borrow();
        let obs_ref = obs.obs();
        chanlun::algorithm::segment::线段::段获取所有停顿位置(
            &段.borrow().inner,
            &*obs_ref,
        )
        .into_iter()
        .map(|d| 虚线Py { inner: Rc::new(d) })
        .collect()
    }

    #[classmethod]
    fn 是否背驰过(
        _cls: &Bound<'_, PyType>,
        当前段: &Bound<'_, 虚线Py>,
        观察员: &Bound<'_, 观察者Py>,
    ) -> Vec<分型Py> {
        let obs = 观察员.borrow();
        let obs_ref = obs.obs();
        chanlun::algorithm::segment::线段::是否背驰过(&当前段.borrow().inner, &*obs_ref)
            .into_iter()
            .map(|f| 分型Py { inner: f })
            .collect()
    }
}

// ========== 中枢 ==========

#[pyclass(name = "中枢", unsendable)]
#[derive(Clone)]
pub struct 中枢Py {
    pub(crate) inner: Rc<chanlun::algorithm::hub::中枢>,
}

#[pymethods]
impl 中枢Py {
    #[new]
    fn new(
        序号: i64, 标识: String, 级别: i64, 基础序列: Vec<Py<虚线Py>>, py: Python<'_>
    ) -> Self {
        let rc_list: Vec<Rc<chanlun::structure::dash_line::虚线>> = 基础序列
            .iter()
            .map(|d| Rc::clone(&d.bind(py).borrow().inner))
            .collect();
        Self {
            inner: Rc::new(chanlun::algorithm::hub::中枢::new(
                序号, 标识, 级别, rc_list,
            )),
        }
    }

    #[getter]
    fn 序号(&self) -> i64 {
        self.inner.序号
    }

    #[getter]
    fn 标识(&self) -> String {
        self.inner.标识.clone()
    }

    #[getter]
    fn 级别(&self) -> i64 {
        self.inner.级别
    }

    #[getter]
    fn 基础序列(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let list = pyo3::types::PyList::empty(py);
        for d in &self.inner.基础序列 {
            list.append(虚线Py {
                inner: Rc::clone(d),
            })?;
        }
        Ok(list.into())
    }

    #[getter]
    fn 第三买卖线(&self) -> Option<虚线Py> {
        self.inner.第三买卖线.as_ref().map(|d| 虚线Py {
            inner: Rc::clone(d),
        })
    }

    #[getter]
    fn 本级_第三买卖线(&self) -> Option<虚线Py> {
        self.inner.本级_第三买卖线.as_ref().map(|d| 虚线Py {
            inner: Rc::clone(d),
        })
    }

    fn 添加虚线(&mut self, 实线: &Bound<'_, 虚线Py>) {
        let inner = Rc::make_mut(&mut self.inner);
        inner.添加虚线(Rc::clone(&实线.borrow().inner));
    }

    #[getter]
    fn 图表标题(&self) -> String {
        self.inner.图表标题()
    }

    #[getter]
    fn 离开段(&self) -> 虚线Py {
        虚线Py {
            inner: self.inner.离开段(),
        }
    }

    #[getter]
    fn 方向(&self) -> 相对方向Py {
        相对方向Py {
            inner: self.inner.方向(),
        }
    }

    #[getter]
    fn 高(&self) -> f64 {
        self.inner.高()
    }

    #[getter]
    fn 低(&self) -> f64 {
        self.inner.低()
    }

    #[getter]
    fn 高高(&self) -> f64 {
        self.inner.高高()
    }

    #[getter]
    fn 低低(&self) -> f64 {
        self.inner.低低()
    }

    #[getter]
    fn 文(&self) -> 分型Py {
        分型Py {
            inner: self.inner.文(),
        }
    }

    #[getter]
    fn 武(&self) -> 分型Py {
        分型Py {
            inner: self.inner.武(),
        }
    }

    fn 设置第三买卖线(&mut self, 线: &Bound<'_, 虚线Py>) {
        let inner = Rc::make_mut(&mut self.inner);
        inner.设置第三买卖线(Rc::clone(&线.borrow().inner));
    }

    fn 获取序列(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let list = pyo3::types::PyList::empty(py);
        for d in self.inner.获取序列() {
            list.append(虚线Py { inner: d })?;
        }
        Ok(list.into())
    }

    fn 获取数据文本(&self) -> String {
        self.inner.获取数据文本()
    }

    fn 校验合法性(&mut self, 序列: Vec<Py<虚线Py>>, py: Python<'_>) -> bool {
        let rc_list: Vec<Rc<chanlun::structure::dash_line::虚线>> = 序列
            .iter()
            .map(|d| Rc::clone(&d.bind(py).borrow().inner))
            .collect();
        let inner = Rc::make_mut(&mut self.inner);
        inner.校验合法性(&rc_list)
    }

    #[pyo3(signature = (虚实 = "合"))]
    fn 完整性(&self, 虚实: &str) -> bool {
        self.inner.完整性(虚实)
    }

    fn 获取扩展中枢(&self, 扩展中枢: Vec<Py<Self>>, py: Python<'_>) -> PyResult<()> {
        let mut hub_seq: Vec<Rc<chanlun::algorithm::hub::中枢>> = 扩展中枢
            .iter()
            .map(|h| Rc::clone(&h.bind(py).borrow().inner))
            .collect();
        // Need config for this call — use default
        let config = chanlun::config::缠论配置::default();
        self.inner.获取扩展中枢(&mut hub_seq, &config);
        Ok(())
    }

    #[getter]
    fn 当前状态(&self) -> String {
        self.inner.当前状态().to_string()
    }

    fn __str__(&self) -> String {
        format!("{}", self.inner)
    }

    fn __repr__(&self) -> String {
        self.__str__()
    }

    // ---- classmethods ----

    #[classmethod]
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
    fn 创建(
        _cls: &Bound<'_, PyType>,
        左: &Bound<'_, 虚线Py>,
        中: &Bound<'_, 虚线Py>,
        右: &Bound<'_, 虚线Py>,
        级别: i64,
        标识: &str,
    ) -> Self {
        Self {
            inner: Rc::new(chanlun::algorithm::hub::中枢::创建(
                Rc::clone(&左.borrow().inner),
                Rc::clone(&中.borrow().inner),
                Rc::clone(&右.borrow().inner),
                级别,
                标识,
            )),
        }
    }

    #[classmethod]
    fn 从序列中获取中枢(
        _cls: &Bound<'_, PyType>,
        虚线序列: Vec<Py<虚线Py>>,
        起始方向: &Bound<'_, 相对方向Py>,
        标识: &str,
        py: Python<'_>,
    ) -> Option<Self> {
        let rc_list: Vec<Rc<chanlun::structure::dash_line::虚线>> = 虚线序列
            .iter()
            .map(|d| Rc::clone(&d.bind(py).borrow().inner))
            .collect();
        chanlun::algorithm::hub::中枢::从序列中获取中枢(
            &rc_list,
            起始方向.borrow().inner,
            标识,
        )
        .map(|inner| Self { inner })
    }

    #[classmethod]
    fn 向中枢序列尾部添加(
        _cls: &Bound<'_, PyType>,
        中枢序列: &Bound<'_, PyAny>,
        待添加中枢: &Bound<'_, Self>,
    ) -> PyResult<()> {
        let inner = Rc::clone(&待添加中枢.borrow().inner);
        let wrapper = Py::new(中枢序列.py(), Self { inner })?;
        中枢序列.call_method1("append", (wrapper,))?;
        Ok(())
    }

    #[classmethod]
    fn 从中枢序列尾部弹出(
        _cls: &Bound<'_, PyType>,
        中枢序列: &Bound<'_, PyAny>,
        待弹出中枢: &Bound<'_, Self>,
    ) -> PyResult<Option<Self>> {
        let result = 中枢序列.call_method1("pop", ())?;
        if result.is_none() {
            return Ok(None);
        }
        let bound: Bound<'_, Self> = result.extract()?;
        Ok(Some(Self {
            inner: Rc::clone(&bound.borrow().inner),
        }))
    }

    #[classmethod]
    #[pyo3(signature = (虚线序列, 中枢序列, 跳过首部 = true, 标识 = "", 层级 = 0))]
    fn 分析(
        _cls: &Bound<'_, PyType>,
        虚线序列: Vec<Py<虚线Py>>,
        中枢序列: Vec<Py<Self>>,
        跳过首部: bool,
        标识: &str,
        层级: i64,
        py: Python<'_>,
    ) -> PyResult<()> {
        let rc_list: Vec<Rc<chanlun::structure::dash_line::虚线>> = 虚线序列
            .iter()
            .map(|d| Rc::clone(&d.bind(py).borrow().inner))
            .collect();
        let mut hub_seq: Vec<Rc<chanlun::algorithm::hub::中枢>> = 中枢序列
            .iter()
            .map(|h| Rc::clone(&h.bind(py).borrow().inner))
            .collect();
        chanlun::algorithm::hub::中枢::分析(&rc_list, &mut hub_seq, 跳过首部, 标识, 层级);
        Ok(())
    }
}

pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<背驰分析Py>()?;
    m.add_class::<笔Py>()?;
    m.add_class::<线段Py>()?;
    m.add_class::<中枢Py>()?;
    Ok(())
}
