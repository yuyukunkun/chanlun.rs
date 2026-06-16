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

//! C-ABI 导出 — 供动态加载的 .so 插件调用。
//!
//! 插件编译为 cdylib (`.so`)，由 Python `ctypes.CDLL` 或 Rust `libloading` 加载。
//! 加载后插件调用 `chanlun_register_signal` 向宿主进程的 `DYNAMIC_REGISTRY` 注册信号。
//!
//! # 插件约定
//!
//! 1. 插件 .so 的构造函数中调用 `chanlun_register_signal(name, template, func)`
//! 2. `func` 是 `SignalFn` 类型的函数指针（`fn(&观察者, &HashMap<String, Value>) -> Vec<Signal>`）
//! 3. 插件和宿主必须用相同 Rust 编译器版本编译

use std::ffi::CStr;
use std::os::raw::c_char;

use crate::signal::registry::{self, SignalFn};

/// 宿主导出：供外部动态库调用的注册入口。
///
/// - `name`: 信号名（C 字符串）
/// - `template`: 参数模板（C 字符串）
/// - `func`: 函数指针（Rust 调用约定，插件与宿主须同编译器版本）
///
/// 返回 0 成功，非 0 失败。
///
/// # Safety
///
/// `name` 和 `template` 必须是非空的合法 UTF-8 C 字符串指针。
/// `func` 必须是合法的 `SignalFn` 函数指针（Rust 调用约定）。
#[unsafe(no_mangle)]
#[allow(improper_ctypes_definitions)]
pub unsafe extern "C" fn chanlun_register_signal(
    name: *const c_char,
    template: *const c_char,
    func: SignalFn,
) -> i32 {
    if name.is_null() || template.is_null() {
        return 1;
    }
    let name_str = unsafe { CStr::from_ptr(name) }.to_string_lossy();
    let template_str = unsafe { CStr::from_ptr(template) }.to_string_lossy();
    match registry::register_signal(&name_str, &template_str, func) {
        Ok(()) => 0,
        Err(_) => 2,
    }
}

/// 宿主导出：从动态注册表移除信号。
///
/// 返回 0 成功，非 0 失败。
///
/// # Safety
///
/// `name` 必须是非空的合法 UTF-8 C 字符串指针。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chanlun_unregister_signal(name: *const c_char) -> i32 {
    if name.is_null() {
        return 1;
    }
    let name_str = unsafe { CStr::from_ptr(name) }.to_string_lossy();
    match registry::unregister_signal(&name_str) {
        Ok(()) => 0,
        Err(_) => 2,
    }
}

/// 查询已注册信号总数（编译时 + 动态）。
///
/// # Safety
///
/// 此函数不接受任何指针参数，调用始终安全。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chanlun_list_signal_count() -> i32 {
    registry::list_signal_names().len() as i32
}
