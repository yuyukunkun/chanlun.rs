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

//! 因子 — signals_all 全满足 + signals_any 任一满足 + signals_not 全不满足。

use crate::signal::signal::Signal;
use crate::signal::{sha256前4, 信号字典, 缺键错误};

#[derive(Clone, Debug)]
pub struct Factor {
    pub signals_all: Vec<Signal>,
    pub signals_any: Vec<Signal>,
    pub signals_not: Vec<Signal>,
    pub name: String,
}

impl Factor {
    /// 构造。signals_all 为空 → Err。name 自动补确定性哈希后缀。
    pub fn 新建(
        signals_all: Vec<Signal>,
        signals_any: Vec<Signal>,
        signals_not: Vec<Signal>,
        name: String,
    ) -> Result<Self, String> {
        if signals_all.is_empty() {
            return Err("signals_all 不能为空".to_string());
        }
        let hash = Self::计算哈希(&signals_all, &signals_any, &signals_not);
        let 前缀 = name.split('#').next().unwrap_or("").to_string();
        let name = format!("{前缀}#{hash}");
        Ok(Self {
            signals_all,
            signals_any,
            signals_not,
            name,
        })
    }

    /// 确定性哈希 — 拼接三组 signals 串后算 sha256 前4。
    fn 计算哈希(all: &[Signal], any: &[Signal], not: &[Signal]) -> String {
        let 取串 = |v: &[Signal]| {
            v.iter()
                .map(|s| s.signal.clone())
                .collect::<Vec<_>>()
                .join(",")
        };
        let 规范 = format!(
            "all=[{}]|any=[{}]|not=[{}]",
            取串(all),
            取串(any),
            取串(not)
        );
        sha256前4(&规范)
    }

    pub fn unique_signals(&self) -> Vec<String> {
        let mut 集合 = std::collections::BTreeSet::new();
        for s in self
            .signals_all
            .iter()
            .chain(&self.signals_any)
            .chain(&self.signals_not)
        {
            集合.insert(s.signal.clone());
        }
        集合.into_iter().collect()
    }

    /// 因子匹配。任一信号缺键 → Err 向上传播。
    pub fn is_match(&self, 字典: &信号字典) -> Result<bool, 缺键错误> {
        for s in &self.signals_not {
            if s.is_match(字典)? {
                return Ok(false);
            }
        }
        for s in &self.signals_all {
            if !s.is_match(字典)? {
                return Ok(false);
            }
        }
        if self.signals_any.is_empty() {
            return Ok(true);
        }
        for s in &self.signals_any {
            if s.is_match(字典)? {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signal::匹配值;
    use std::collections::HashMap;

    fn 信号(k3: &str, v2: &str) -> Signal {
        Signal::new("14400", "D1MO3", k3, "任意", v2, "任意", 0)
    }
    fn 字典(k3: &str, v2: &str) -> HashMap<String, 匹配值> {
        let mut m = HashMap::new();
        m.insert(
            format!("14400_D1MO3_{k3}"),
            匹配值::字符串(format!("x_{v2}_y_100")),
        );
        m
    }

    #[test]
    fn test_signals_all_为空_报错() {
        assert!(Factor::新建(vec![], vec![], vec![], "".into()).is_err());
    }

    #[test]
    fn test_name_含哈希后缀() {
        let f = Factor::新建(vec![信号("中枢", "三买")], vec![], vec![], "测试".into()).unwrap();
        assert!(f.name.starts_with("测试#"));
        assert_eq!(f.name.len(), "测试#".len() + 4);
    }

    #[test]
    fn test_name_确定性() {
        let f1 = Factor::新建(vec![信号("中枢", "三买")], vec![], vec![], "".into()).unwrap();
        let f2 = Factor::新建(vec![信号("中枢", "三买")], vec![], vec![], "".into()).unwrap();
        assert_eq!(f1.name, f2.name);
    }

    #[test]
    fn test_all_命中() {
        let f = Factor::新建(vec![信号("中枢", "三买")], vec![], vec![], "".into()).unwrap();
        assert_eq!(f.is_match(&字典("中枢", "三买")).unwrap(), true);
    }

    #[test]
    fn test_not_命中则false() {
        let f = Factor::新建(
            vec![信号("中枢", "三买")],
            vec![],
            vec![信号("中枢", "三买")],
            "".into(),
        )
        .unwrap();
        assert_eq!(f.is_match(&字典("中枢", "三买")).unwrap(), false);
    }

    #[test]
    fn test_缺键传播错误() {
        let f = Factor::新建(vec![信号("中枢", "三买")], vec![], vec![], "".into()).unwrap();
        let m: HashMap<String, 匹配值> = HashMap::new();
        assert!(f.is_match(&m).is_err());
    }
}
