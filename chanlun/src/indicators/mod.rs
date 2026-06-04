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

pub mod boll;
pub mod calculator;
pub mod container;
pub mod kdj;
pub mod macd;
pub mod rsi;

pub use boll::布林带;
pub use calculator::指标计算器;
pub use container::{指标值, 指标容器};
pub use kdj::随机指标;
pub use macd::平滑异同移动平均线;
pub use rsi::相对强弱指数;

/// K线取值 —— 根据计算方式从K线提取对应的价格
pub fn K线取值(开盘价: f64, 高: f64, 低: f64, 收盘价: f64, 计算方式: &str) -> f64 {
    match 计算方式 {
        "开" => 开盘价,
        "高" => 高,
        "低" => 低,
        "收" => 收盘价,
        "高低均值" => (高 + 低) / 2.0,
        "高低收均值" => (高 + 低 + 收盘价) / 3.0,
        "开高低收均值" => (高 + 低 + 开盘价 + 收盘价) / 4.0,
        _ => 收盘价,
    }
}
