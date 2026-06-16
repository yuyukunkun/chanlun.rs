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

use dashmap::DashMap;
use pyo3::prelude::*;
use pyo3::types::PySet;
/// 缓存模式：线程局部（默认，零锁）或全局（dashmap，跨线程共享）
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::OnceLock;

pub enum CacheMode {
    ThreadLocal,
    Global,
}

static CACHE_MODE: OnceLock<CacheMode> = OnceLock::new();

pub fn get_mode() -> &'static CacheMode {
    CACHE_MODE.get_or_init(|| match std::env::var("CHANLUN_CACHE_MODE").as_deref() {
        Ok("global") => CacheMode::Global,
        _ => CacheMode::ThreadLocal,
    })
}

pub fn peek_mode() -> Option<&'static CacheMode> {
    CACHE_MODE.get()
}

pub fn set_mode(mode: CacheMode) -> Result<(), String> {
    CACHE_MODE
        .set(mode)
        .map_err(|_| "缓存模式已初始化，请在创建任何观察者之前调用 set_cache_mode".into())
}

// ========== BAR_IDENTITY ==========
thread_local! {
    static BAR_LOCAL: RefCell<HashMap<usize, Py<super::kline_py::K线Py>>> = RefCell::new(HashMap::new());
}
static BAR_GLOBAL: std::sync::LazyLock<DashMap<usize, Py<super::kline_py::K线Py>>> =
    std::sync::LazyLock::new(DashMap::new);

pub fn bar_get(py: Python<'_>, key: usize) -> Option<Py<super::kline_py::K线Py>> {
    match get_mode() {
        CacheMode::ThreadLocal => BAR_LOCAL.with(|m| m.borrow().get(&key).map(|p| p.clone_ref(py))),
        CacheMode::Global => BAR_GLOBAL.get(&key).map(|p| p.clone_ref(py)),
    }
}
pub fn bar_insert(py: Python<'_>, key: usize, obj: &Py<super::kline_py::K线Py>) {
    match get_mode() {
        CacheMode::ThreadLocal => BAR_LOCAL.with(|m| {
            let mut m = m.borrow_mut();
            m.retain(|_, v| v.get_refcnt(py) > 1);
            m.insert(key, obj.clone_ref(py));
        }),
        CacheMode::Global => {
            BAR_GLOBAL.retain(|_, v| v.get_refcnt(py) > 1);
            BAR_GLOBAL.insert(key, obj.clone_ref(py));
        }
    }
}

// ========== KLINE_IDENTITY ==========
thread_local! {
    static KLINE_LOCAL: RefCell<HashMap<usize, Py<super::kline_py::缠论K线Py>>> = RefCell::new(HashMap::new());
}
static KLINE_GLOBAL: std::sync::LazyLock<DashMap<usize, Py<super::kline_py::缠论K线Py>>> =
    std::sync::LazyLock::new(DashMap::new);

pub fn kline_get(py: Python<'_>, key: usize) -> Option<Py<super::kline_py::缠论K线Py>> {
    match get_mode() {
        CacheMode::ThreadLocal => {
            KLINE_LOCAL.with(|m| m.borrow().get(&key).map(|p| p.clone_ref(py)))
        }
        CacheMode::Global => KLINE_GLOBAL.get(&key).map(|p| p.clone_ref(py)),
    }
}
pub fn kline_insert(py: Python<'_>, key: usize, obj: &Py<super::kline_py::缠论K线Py>) {
    match get_mode() {
        CacheMode::ThreadLocal => KLINE_LOCAL.with(|m| {
            let mut m = m.borrow_mut();
            m.retain(|_, v| v.get_refcnt(py) > 1);
            m.insert(key, obj.clone_ref(py));
        }),
        CacheMode::Global => {
            KLINE_GLOBAL.retain(|_, v| v.get_refcnt(py) > 1);
            KLINE_GLOBAL.insert(key, obj.clone_ref(py));
        }
    }
}

// ========== FRACTAL_IDENTITY ==========
use crate::structure_py::分型Py;
thread_local! {
    static FRACTAL_LOCAL: RefCell<HashMap<usize, Py<分型Py>>> = RefCell::new(HashMap::new());
}
static FRACTAL_GLOBAL: std::sync::LazyLock<DashMap<usize, Py<分型Py>>> =
    std::sync::LazyLock::new(DashMap::new);

pub fn fractal_get(py: Python<'_>, key: usize) -> Option<Py<分型Py>> {
    match get_mode() {
        CacheMode::ThreadLocal => {
            FRACTAL_LOCAL.with(|m| m.borrow().get(&key).map(|p| p.clone_ref(py)))
        }
        CacheMode::Global => FRACTAL_GLOBAL.get(&key).map(|p| p.clone_ref(py)),
    }
}
pub fn fractal_insert(py: Python<'_>, key: usize, obj: &Py<分型Py>) {
    match get_mode() {
        CacheMode::ThreadLocal => FRACTAL_LOCAL.with(|m| {
            let mut m = m.borrow_mut();
            m.retain(|_, v| v.get_refcnt(py) > 1);
            m.insert(key, obj.clone_ref(py));
        }),
        CacheMode::Global => {
            FRACTAL_GLOBAL.retain(|_, v| v.get_refcnt(py) > 1);
            FRACTAL_GLOBAL.insert(key, obj.clone_ref(py));
        }
    }
}

// ========== DASHED_IDENTITY ==========
use crate::structure_py::虚线Py;
thread_local! {
    static DASHED_LOCAL: RefCell<HashMap<usize, Py<虚线Py>>> = RefCell::new(HashMap::new());
}
static DASHED_GLOBAL: std::sync::LazyLock<DashMap<usize, Py<虚线Py>>> =
    std::sync::LazyLock::new(DashMap::new);

pub fn dashed_get(py: Python<'_>, key: usize) -> Option<Py<虚线Py>> {
    match get_mode() {
        CacheMode::ThreadLocal => {
            DASHED_LOCAL.with(|m| m.borrow().get(&key).map(|p| p.clone_ref(py)))
        }
        CacheMode::Global => DASHED_GLOBAL.get(&key).map(|p| p.clone_ref(py)),
    }
}
pub fn dashed_insert(py: Python<'_>, key: usize, obj: &Py<虚线Py>) {
    match get_mode() {
        CacheMode::ThreadLocal => DASHED_LOCAL.with(|m| {
            let mut m = m.borrow_mut();
            m.retain(|_, v| v.get_refcnt(py) > 1);
            m.insert(key, obj.clone_ref(py));
        }),
        CacheMode::Global => {
            DASHED_GLOBAL.retain(|_, v| v.get_refcnt(py) > 1);
            DASHED_GLOBAL.insert(key, obj.clone_ref(py));
        }
    }
}

// ========== HUB_IDENTITY ==========
use crate::algorithm_py::中枢Py;
thread_local! {
    static HUB_LOCAL: RefCell<HashMap<usize, Py<中枢Py>>> = RefCell::new(HashMap::new());
}
static HUB_GLOBAL: std::sync::LazyLock<DashMap<usize, Py<中枢Py>>> =
    std::sync::LazyLock::new(DashMap::new);

pub fn hub_get(py: Python<'_>, key: usize) -> Option<Py<中枢Py>> {
    match get_mode() {
        CacheMode::ThreadLocal => HUB_LOCAL.with(|m| m.borrow().get(&key).map(|p| p.clone_ref(py))),
        CacheMode::Global => HUB_GLOBAL.get(&key).map(|p| p.clone_ref(py)),
    }
}
pub fn hub_insert(py: Python<'_>, key: usize, obj: &Py<中枢Py>) {
    match get_mode() {
        CacheMode::ThreadLocal => HUB_LOCAL.with(|m| {
            let mut m = m.borrow_mut();
            m.retain(|_, v| v.get_refcnt(py) > 1);
            m.insert(key, obj.clone_ref(py));
        }),
        CacheMode::Global => {
            HUB_GLOBAL.retain(|_, v| v.get_refcnt(py) > 1);
            HUB_GLOBAL.insert(key, obj.clone_ref(py));
        }
    }
}

// ========== BSP_CACHE ==========
thread_local! {
    static BSP_LOCAL: RefCell<HashMap<usize, Py<PySet>>> = RefCell::new(HashMap::new());
}
static BSP_GLOBAL: std::sync::LazyLock<DashMap<usize, Py<PySet>>> =
    std::sync::LazyLock::new(DashMap::new);

pub fn bsp_get(py: Python<'_>, key: usize) -> Option<Py<PySet>> {
    match get_mode() {
        CacheMode::ThreadLocal => BSP_LOCAL.with(|m| m.borrow().get(&key).map(|p| p.clone_ref(py))),
        CacheMode::Global => BSP_GLOBAL.get(&key).map(|p| p.clone_ref(py)),
    }
}
pub fn bsp_insert(py: Python<'_>, key: usize, obj: Py<PySet>) {
    match get_mode() {
        CacheMode::ThreadLocal => BSP_LOCAL.with(|m| {
            m.borrow_mut().insert(key, obj);
        }),
        CacheMode::Global => {
            BSP_GLOBAL.insert(key, obj);
        }
    }
}
