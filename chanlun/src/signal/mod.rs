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

//! 信号匹配原语层。
//!
//! 第三方代码声明：本模块的 Signal/Factor/Event/Position/Operate 匹配框架
//! 摘录自 czsc 项目（https://github.com/waditu/czsc），Apache License 2.0 授权，
//! 已做中文命名适配与 Rust 重写。

use std::collections::HashMap;

pub mod engine;
pub mod event;
pub mod factor;
pub mod ffi;
pub mod functions;
pub mod operate;
pub mod params;
pub mod position;
pub mod registry;
#[cfg(test)]
mod registry_macro_test;
#[allow(clippy::module_inception)]
pub mod signal;

pub use event::Event;
pub use factor::Factor;
pub use operate::Operate;
pub use position::Position;
pub use signal::Signal;

/// 信号字典中某个 key 对应的值。区分「字符串」与「非字符串」，
/// 以在纯 Rust 内表达 Python `is_match` 的三态：缺键 / 非 str / str。
#[derive(Clone, Debug)]
pub enum 匹配值 {
    字符串(String),
    非字符串,
}

/// 信号字典类型别名。
pub type 信号字典 = HashMap<String, 匹配值>;

/// 缺键错误 — `is_match` 在信号字典中找不到 key 时返回。
#[derive(Debug, Clone)]
pub struct 缺键错误(pub String);

/// 对任意字节串算 sha256，取大写十六进制前 4 位（= 前 2 字节）。
/// 对应 Python `hashlib.sha256(...).hexdigest().upper()[:4]`。
pub(crate) fn sha256前4(输入: &str) -> String {
    use sha2::{Digest, Sha256};
    let 摘要 = Sha256::digest(输入.as_bytes());
    format!("{:02X}{:02X}", 摘要[0], 摘要[1])
}
