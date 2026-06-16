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

//! 信号函数参数提取辅助 — 从 `HashMap<String, Value>` 中提取类型化参数。

use serde_json::Value;
use std::collections::HashMap;

/// 提取字符串参数，缺失或类型不对时返回默认值。
pub fn get_string(params: &HashMap<String, Value>, key: &str, default: &str) -> String {
    params
        .get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| default.to_string())
}

/// 提取 i64 参数。
pub fn get_int(params: &HashMap<String, Value>, key: &str, default: i64) -> i64 {
    params.get(key).and_then(|v| v.as_i64()).unwrap_or(default)
}

/// 提取 f64 参数。
pub fn get_f64(params: &HashMap<String, Value>, key: &str, default: f64) -> f64 {
    params.get(key).and_then(|v| v.as_f64()).unwrap_or(default)
}

/// 提取字符串引用（零拷贝），缺失时返回默认值。
pub fn get_str<'a>(params: &'a HashMap<String, Value>, key: &str, default: &'a str) -> &'a str {
    params.get(key).and_then(|v| v.as_str()).unwrap_or(default)
}
