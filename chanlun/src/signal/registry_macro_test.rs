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

//! 端到端：#[signal] 宏 + inventory 注册表协同。
//! 放在 chanlun lib 内（非 tests/ 外部集成测试），因 #[signal] emit 的是
//! `crate::signal::registry::` 路径，只有在 chanlun crate 内才能解析。
#![cfg(test)]

use std::collections::HashMap;

use crate::business::observer::观察者;
use crate::signal::Signal;
use crate::signal::registry::{get_signal, get_template, list_signal_names};
use chanlun_signal_macros::signal;
use serde_json::Value;

/// 探针信号函数：贴 #[signal] 后应被自动注册进 SIGNAL_REGISTRY。
#[signal(
    name = "test_probe_signal_V230101",
    template = "{freq}_D1MO{max_overlap}_test_probe_signalV230101"
)]
fn test_probe_signal_V230101(_obs: &观察者, _params: &HashMap<String, Value>) -> Vec<Signal> {
    Vec::new()
}

#[test]
fn test_signal_宏自动注册到全局表() {
    assert!(
        get_signal("test_probe_signal_V230101").is_some(),
        "#[signal] 应把探针函数注册进 SIGNAL_REGISTRY"
    );
    assert_eq!(
        get_template("test_probe_signal_V230101"),
        Some("{freq}_D1MO{max_overlap}_test_probe_signalV230101".to_string())
    );
    assert!(list_signal_names().contains(&"test_probe_signal_V230101".to_string()));
}

#[test]
fn test_未注册信号返回None() {
    assert!(get_signal("不存在的信号_V999999").is_none());
}
