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

use chrono::DateTime;

/// 将多种类型统一转为时间戳 (Unix epoch 秒)
pub fn 转化为时间戳(ts: &str) -> Option<i64> {
    // 尝试解析为整数时间戳
    if let Ok(v) = ts.parse::<i64>() {
        return Some(v);
    }
    // 尝试解析为浮点时间戳
    if let Ok(v) = ts.parse::<f64>() {
        return Some(v as i64);
    }
    // 尝试 ISO 格式日期字符串
    if let Ok(dt) = DateTime::parse_from_rfc3339(ts) {
        return Some(dt.timestamp());
    }
    // 尝试 "YYYY-MM-DD HH:MM:SS" 格式
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(ts, "%Y-%m-%d %H:%M:%S") {
        return Some(dt.and_utc().timestamp());
    }
    None
}

/// 将多种类型转为时间戳数字 (i64)
pub fn 转化为时间戳_数字(ts: &str) -> Option<i64> {
    转化为时间戳(ts)
}
