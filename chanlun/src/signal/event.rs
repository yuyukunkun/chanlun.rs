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

//! 事件 — operate + 因子列表（任一因子满足则事件为真）。

use crate::signal::factor::Factor;
use crate::signal::operate::Operate;
use crate::signal::signal::Signal;
use crate::signal::{sha256前4, 信号字典, 缺键错误};

#[derive(Clone, Debug)]
pub struct Event {
    pub operate: Operate,
    pub factors: Vec<Factor>,
    pub signals_all: Vec<Signal>,
    pub signals_any: Vec<Signal>,
    pub signals_not: Vec<Signal>,
    pub name: String,
    pub sha256: String,
}

impl Event {
    /// 构造。factors 为空 → Err。name 自动补哈希。
    pub fn 新建(
        operate: Operate,
        factors: Vec<Factor>,
        signals_all: Vec<Signal>,
        signals_any: Vec<Signal>,
        signals_not: Vec<Signal>,
        name: String,
    ) -> Result<Self, String> {
        if factors.is_empty() {
            return Err("factors 不能为空".to_string());
        }
        let hash = Self::计算哈希(&factors, &signals_all, &signals_any, &signals_not);
        let name = if name.is_empty() {
            format!("{}#{hash}", operate.value())
        } else {
            format!("{}#{hash}", name.split('#').next().unwrap_or(""))
        };
        Ok(Self {
            operate,
            factors,
            signals_all,
            signals_any,
            signals_not,
            name,
            sha256: hash,
        })
    }

    fn 计算哈希(factors: &[Factor], all: &[Signal], any: &[Signal], not: &[Signal]) -> String {
        let 取串 = |v: &[Signal]| {
            v.iter()
                .map(|s| s.signal.clone())
                .collect::<Vec<_>>()
                .join(",")
        };
        let 因子串 = factors
            .iter()
            .map(|f| f.name.clone())
            .collect::<Vec<_>>()
            .join(";");
        let 规范 = format!(
            "factors=[{}]|all=[{}]|any=[{}]|not=[{}]",
            因子串,
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
        for f in &self.factors {
            for s in f.unique_signals() {
                集合.insert(s);
            }
        }
        集合.into_iter().collect()
    }

    /// 事件匹配。命中返回 (true, 因子名)，否则 (false, None)。
    pub fn is_match(&self, 字典: &信号字典) -> Result<(bool, Option<String>), 缺键错误> {
        for s in &self.signals_not {
            if s.is_match(字典)? {
                return Ok((false, None));
            }
        }
        for s in &self.signals_all {
            if !s.is_match(字典)? {
                return Ok((false, None));
            }
        }
        if !self.signals_any.is_empty() {
            let mut 任一命中 = false;
            for s in &self.signals_any {
                if s.is_match(字典)? {
                    任一命中 = true;
                    break;
                }
            }
            if !任一命中 {
                return Ok((false, None));
            }
        }
        for f in &self.factors {
            if f.is_match(字典)? {
                return Ok((true, Some(f.name.clone())));
            }
        }
        Ok((false, None))
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
    fn 因子(k3: &str, v2: &str) -> Factor {
        Factor::新建(vec![信号(k3, v2)], vec![], vec![], "".into()).unwrap()
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
    fn test_factors_为空_报错() {
        assert!(Event::新建(Operate::开多, vec![], vec![], vec![], vec![], "".into()).is_err());
    }

    #[test]
    fn test_name_默认用operate值() {
        let e = Event::新建(
            Operate::开多,
            vec![因子("中枢", "三买")],
            vec![],
            vec![],
            vec![],
            "".into(),
        )
        .unwrap();
        assert!(e.name.starts_with("开多#"));
    }

    #[test]
    fn test_任一因子命中() {
        let e = Event::新建(
            Operate::开多,
            vec![因子("中枢A", "三买"), 因子("中枢B", "三买")],
            vec![],
            vec![],
            vec![],
            "".into(),
        )
        .unwrap();
        // 两个因子的 key 都在字典：中枢A 在场但 v2=三卖 不匹配，中枢B v2=三买 匹配
        let mut d = HashMap::new();
        d.insert(
            "14400_D1MO3_中枢A".to_string(),
            匹配值::字符串("x_三卖_y_100".to_string()),
        );
        d.insert(
            "14400_D1MO3_中枢B".to_string(),
            匹配值::字符串("x_三买_y_100".to_string()),
        );
        let (命中, 名) = e.is_match(&d).unwrap();
        assert!(命中);
        assert!(名.is_some());
    }

    #[test]
    fn test_无因子命中_false() {
        let e = Event::新建(
            Operate::开多,
            vec![因子("中枢", "三买")],
            vec![],
            vec![],
            vec![],
            "".into(),
        )
        .unwrap();
        let (命中, 名) = e.is_match(&字典("中枢", "三卖")).unwrap();
        assert!(!命中);
        assert!(名.is_none());
    }
}
