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

//! 信号原语 — k1_k2_k3_v1_v2_v3_score 七段字符串。

use crate::signal::{信号字典, 匹配值, 缺键错误};

pub(crate) const 任意: &str = "任意";

/// 单个信号。字段与 Python `chan_external.Signal` 一致。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Signal {
    pub signal: String,
    pub score: i32,
    pub k1: String,
    pub k2: String,
    pub k3: String,
    pub v1: String,
    pub v2: String,
    pub v3: String,
}

impl Signal {
    /// 创建"空"信号（v1=v2=v3="任意"，score=0）。
    /// 对应 Python `create_single_signal(k1=k1, k2=k2, k3=k3)` 的默认返回值。
    pub fn new_empty(k1: &str, k2: &str, k3: &str) -> Self {
        let signal = format!("{k1}_{k2}_{k3}_任意_任意_任意_0");
        Self {
            signal,
            score: 0,
            k1: k1.to_string(),
            k2: k2.to_string(),
            k3: k3.to_string(),
            v1: "任意".to_string(),
            v2: "任意".to_string(),
            v3: "任意".to_string(),
        }
    }

    /// 创建带分类值的信号（便捷构造器，score 自动钳制到 0..100）。
    pub fn new(k1: &str, k2: &str, k3: &str, v1: &str, v2: &str, v3: &str, score: i32) -> Self {
        let score = score.clamp(0, 100);
        let signal = format!("{k1}_{k2}_{k3}_{v1}_{v2}_{v3}_{score}");
        Self {
            signal,
            score,
            k1: k1.to_string(),
            k2: k2.to_string(),
            k3: k3.to_string(),
            v1: v1.to_string(),
            v2: v2.to_string(),
            v3: v3.to_string(),
        }
    }

    /// key — k1/k2/k3 中非「任意」部分用 _ 连接。
    pub fn key(&self) -> String {
        [&self.k1, &self.k2, &self.k3]
            .iter()
            .filter(|k| k.as_str() != 任意)
            .map(|k| k.as_str())
            .collect::<Vec<_>>()
            .join("_")
    }

    /// value — v1_v2_v3_score。
    pub fn value(&self) -> String {
        format!("{}_{}_{}_{}", self.v1, self.v2, self.v3, self.score)
    }

    /// 纯值匹配 — 给定信号字典里取到的 value 串（v1_v2_v3_score），判断是否匹配。
    pub fn is_match_value(&self, 值: &str) -> bool {
        let parts: Vec<&str> = 值.split('_').collect();
        if parts.len() != 4 {
            return false;
        }
        let (v1, v2, v3, score_str) = (parts[0], parts[1], parts[2], parts[3]);
        let score: i32 = score_str.parse().unwrap_or(0);
        score >= self.score
            && (v1 == self.v1 || self.v1 == 任意)
            && (v2 == self.v2 || self.v2 == 任意)
            && (v3 == self.v3 || self.v3 == 任意)
    }

    /// 在信号字典中匹配。缺键 → Err（对应 Python raise ValueError），
    /// 非字符串值 → Ok(false)，字符串值 → 走 is_match_value。
    pub fn is_match(&self, 字典: &信号字典) -> Result<bool, 缺键错误> {
        let key = self.key();
        match 字典.get(&key) {
            None => Err(缺键错误(key)),
            Some(匹配值::非字符串) => Ok(false),
            Some(匹配值::字符串(v)) => Ok(self.is_match_value(v)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_score_越界_钳制() {
        // new() 自动将越界 score 钳制到 0~100
        let s = Signal::new("a", "b", "c", "d", "e", "f", 101);
        assert_eq!(s.score, 100);
        let s = Signal::new("a", "b", "c", "d", "e", "f", -1);
        assert_eq!(s.score, 0);
    }

    #[test]
    fn test_key_过滤任意() {
        let s = Signal::new("14400", "D1MO3", "中枢", 任意, "三买", 任意, 0);
        assert_eq!(s.key(), "14400_D1MO3_中枢");

        // k1=任意 时 key 只剩 k2_k3
        let s2 = Signal::new(任意, "D1MO3", "中枢", 任意, "三买", 任意, 0);
        assert_eq!(s2.key(), "D1MO3_中枢");

        // 全「任意」时 key 为空串
        let s3 = Signal::new(任意, 任意, 任意, 任意, 任意, 任意, 0);
        assert_eq!(s3.key(), "");
    }

    #[test]
    fn test_value() {
        let s = Signal::new("k1", "k2", "k3", "v1", "v2", "v3", 88);
        assert_eq!(s.value(), "v1_v2_v3_88");
    }

    #[test]
    fn test_is_match_缺键_报错() {
        let s = Signal::new("14400", "D1MO3", "中枢", 任意, "三买", 任意, 0);
        let 字典: HashMap<String, 匹配值> = HashMap::new();
        assert!(s.is_match(&字典).is_err());
    }

    #[test]
    fn test_is_match_非字符串_false() {
        let s = Signal::new("14400", "D1MO3", "中枢", 任意, "三买", 任意, 0);
        let mut 字典 = HashMap::new();
        字典.insert("14400_D1MO3_中枢".to_string(), 匹配值::非字符串);
        assert_eq!(s.is_match(&字典).unwrap(), false);
    }

    #[test]
    fn test_is_match_命中() {
        let s = Signal::new("14400", "D1MO3", "中枢", 任意, "三买", 任意, 0);
        let mut 字典 = HashMap::new();
        字典.insert(
            "14400_D1MO3_中枢".to_string(),
            匹配值::字符串("中枢段DEA穿越2_三买_偏移0_100".into()),
        );
        assert_eq!(s.is_match(&字典).unwrap(), true);
    }

    #[test]
    fn test_is_match_v2不符_未命中() {
        let s = Signal::new("14400", "D1MO3", "中枢", 任意, "三买", 任意, 0);
        let mut 字典 = HashMap::new();
        字典.insert(
            "14400_D1MO3_中枢".to_string(),
            匹配值::字符串("中枢段DEA穿越2_三卖_偏移0_100".into()),
        );
        assert_eq!(s.is_match(&字典).unwrap(), false);
    }
}
