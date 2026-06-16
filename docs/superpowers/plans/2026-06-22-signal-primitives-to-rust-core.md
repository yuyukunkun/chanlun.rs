# 信号原语层移植到 Rust 核心层 实现计划

> **面向 AI 代理的工作者：** 必需子技能：使用 superpowers:subagent-driven-development（推荐）或 superpowers:executing-plans 逐任务实现此计划。步骤使用复选框（`- [ ]`）语法来跟踪进度。

**目标：** 把信号匹配原语（`Operate` / `Signal` / `Factor` / `Event` / `Position` 配置与匹配部分）从 Python `chan_external.py` 移植到 Rust 核心层，并通过 PyO3 暴露为 drop-in 兼容的 `chanlun._chanlun.{Signal,Factor,Event,Operate,Position}`。

**架构：** 纯 Rust 类型放 `chanlun/src/signal/`（零依赖 business/algorithm，只跟字符串和 `HashMap<String, 匹配值>` 打交道，全可 `cargo test`）；PyO3 包装放 `chanlun-py/src/signal_py.rs`（`XxxPy` 包装核心类型，注册为原名）；`Position` 由 Python 子类继承 Rust 基类补 `update()` 状态机。

**技术栈：** Rust + PyO3 0.28 + sha2（确定性命名）；Python 3.14 + pytest；maturin 构建。

**设计文档：** `docs/superpowers/specs/2026-06-22-signal-primitives-to-rust-core-design.md`

---

## 文件结构

| 文件 | 职责 |
|---|---|
| `chanlun/src/signal/mod.rs` | 模块声明 + re-export + 第三方声明 + `匹配值` 枚举 + sha256 helper |
| `chanlun/src/signal/operate.rs` | `Operate` 枚举 + `value()` |
| `chanlun/src/signal/signal.rs` | `Signal`：`from_str`/`key`/`value`/`is_match_value`/`is_match` |
| `chanlun/src/signal/factor.rs` | `Factor`：`is_match`/`unique_signals`/`dump`/`load`/确定性 name |
| `chanlun/src/signal/event.rs` | `Event`：`is_match`/`unique_signals`/`dump`/`load`/确定性 name |
| `chanlun/src/signal/position.rs` | `Position`：config + 校验 + `unique_signals` |
| `chanlun/src/lib.rs` | 增加 `pub mod signal;` |
| `chanlun/Cargo.toml` | 增加 `sha2` 依赖 |
| `chanlun-py/src/signal_py.rs` | PyO3 包装 + `register(m)` |
| `chanlun-py/src/lib.rs` | 注册 `signal_py`（types 之后、config 之前） |
| `chanlun-py/chanlun/chan_external.py` | 删除 Python 原语类，import Rust 版；`Position` 改子类 |
| `chanlun-py/tests/test_signal_primitives.py` | 跨语言一致性测试 |

**核心匹配设计**：核心层 `is_match` 接收 `&HashMap<String, 匹配值>`，`匹配值` 区分「字符串」与「非字符串」，从而在纯 Rust 内完整表达「缺键 raise / 非 str 返回 False / str 匹配」三态，全部可 cargo test。PyO3 层只做一次 `PyDict → HashMap<String, 匹配值>` 转换。

---

## 任务 0：脚手架与依赖

**文件：**
- 创建：`chanlun/src/signal/mod.rs`、`operate.rs`、`signal.rs`、`factor.rs`、`event.rs`、`position.rs`
- 修改：`chanlun/src/lib.rs:36`、`chanlun/Cargo.toml`

- [ ] **步骤 1：加 sha2 依赖**

修改 `chanlun/Cargo.toml` 的 `[dependencies]`，追加：

```toml
sha2 = "0.10"
```

- [ ] **步骤 2：创建模块骨架**

创建 `chanlun/src/signal/mod.rs`：

```rust
//! 信号匹配原语层。
//!
//! 第三方代码声明：本模块的 Signal/Factor/Event/Position/Operate 匹配框架
//! 摘录自 czsc 项目（https://github.com/waditu/czsc），Apache License 2.0 授权，
//! 已做中文命名适配与 Rust 重写。

use std::collections::HashMap;

pub mod event;
pub mod factor;
pub mod operate;
pub mod position;
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
```

创建 `operate.rs`、`signal.rs`、`factor.rs`、`event.rs`、`position.rs` 五个空文件（内容 `// placeholder`，后续任务填充）。

- [ ] **步骤 3：在 lib.rs 注册模块**

修改 `chanlun/src/lib.rs`，在 `pub mod kline;`（第 32 行）之后加入：

```rust
pub mod signal;
```

- [ ] **步骤 4：验证构建**

运行：`cd /home/moscow/chanlun.rs/chanlun && cargo build`
预期：编译通过（仅 placeholder 文件 + 未使用警告可接受）。

- [ ] **步骤 5：Commit**

```bash
cd /home/moscow/chanlun.rs
git add chanlun/src/signal chanlun/src/lib.rs chanlun/Cargo.toml
git commit -m "feat(signal): 脚手架 — 信号原语模块 + sha2 依赖"
```

---

## 任务 1：Operate 枚举

**文件：**
- 修改：`chanlun/src/signal/operate.rs`

- [ ] **步骤 1：编写失败的测试**

在 `operate.rs` 写入：

```rust
//! 缠论买卖操作类型。

/// 持仓/操作类型。值对应中文，与 Python `chan_external.Operate` 一致。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operate {
    持多, // HL
    持空, // HS
    持币, // HO
    开多, // LO
    平多, // LE
    开空, // SO
    平空, // SE
}

impl Operate {
    /// 中文值，对应 Python Enum 的 `.value`。
    pub fn value(&self) -> &'static str {
        match self {
            Operate::持多 => "持多",
            Operate::持空 => "持空",
            Operate::持币 => "持币",
            Operate::开多 => "开多",
            Operate::平多 => "平多",
            Operate::开空 => "开空",
            Operate::平空 => "平空",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operate_value() {
        assert_eq!(Operate::开多.value(), "开多");
        assert_eq!(Operate::平空.value(), "平空");
        assert_eq!(Operate::持币.value(), "持币");
    }
}
```

- [ ] **步骤 2：运行测试验证通过**

运行：`cd /home/moscow/chanlun.rs/chanlun && cargo test signal::operate`
预期：PASS（`test_operate_value`）。

- [ ] **步骤 3：Commit**

```bash
cd /home/moscow/chanlun.rs
git add chanlun/src/signal/operate.rs
git commit -m "feat(signal): Operate 枚举"
```

---

## 任务 2：Signal 核心

**文件：**
- 修改：`chanlun/src/signal/signal.rs`

- [ ] **步骤 1：编写测试 + 实现**

在 `signal.rs` 写入：

```rust
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
    /// 从字段构造。任一 kx/vx 缺省时由调用方传 "任意"。
    pub fn 从字段(
        k1: String, k2: String, k3: String,
        v1: String, v2: String, v3: String, score: i32,
    ) -> Result<Self, String> {
        if !(0..=100).contains(&score) {
            return Err("score 必须在0~100之间".to_string());
        }
        let signal = format!("{k1}_{k2}_{k3}_{v1}_{v2}_{v3}_{score}");
        Ok(Self { signal, score, k1, k2, k3, v1, v2, v3 })
    }

    /// 从完整信号串解析（七段，六个下划线）。
    pub fn 从字符串(s: &str) -> Result<Self, String> {
        let parts: Vec<&str> = s.split('_').collect();
        if parts.len() != 7 {
            return Err(format!(
                "Signal 格式无效：应为 k1_k2_k3_v1_v2_v3_score（7段），收到 {}",
                s
            ));
        }
        let score: i32 = parts[6]
            .parse()
            .map_err(|_| format!("无法解析 score: {}", parts[6]))?;
        if !(0..=100).contains(&score) {
            return Err("score 必须在0~100之间".to_string());
        }
        Ok(Self {
            signal: s.to_string(),
            score,
            k1: parts[0].to_string(),
            k2: parts[1].to_string(),
            k3: parts[2].to_string(),
            v1: parts[3].to_string(),
            v2: parts[4].to_string(),
            v3: parts[5].to_string(),
        })
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
    fn test_从字符串_七段() {
        let s = Signal::从字符串("14400_D1MO3_中枢段DEA穿越2V230602_中枢段DEA穿越2_三卖_偏移0_100").unwrap();
        assert_eq!(s.k1, "14400");
        assert_eq!(s.k3, "中枢段DEA穿越2V230602");
        assert_eq!(s.v2, "三卖");
        assert_eq!(s.score, 100);
    }

    #[test]
    fn test_从字符串_非七段_报错() {
        assert!(Signal::从字符串("a_b_c").is_err());
    }

    #[test]
    fn test_score_越界_报错() {
        assert!(Signal::从字段("a".into(),"b".into(),"c".into(),"d".into(),"e".into(),"f".into(),101).is_err());
    }

    #[test]
    fn test_key_过滤任意() {
        let s = Signal::从字段("14400".into(),"D1MO3".into(),"中枢".into(),任意.into(),"三买".into(),任意.into(),0).unwrap();
        assert_eq!(s.key(), "14400_D1MO3_中枢");
    }

    #[test]
    fn test_value() {
        let s = Signal::从字段("k1".into(),"k2".into(),"k3".into(),"v1".into(),"v2".into(),"v3".into(),88).unwrap();
        assert_eq!(s.value(), "v1_v2_v3_88");
    }

    #[test]
    fn test_is_match_缺键_报错() {
        let s = Signal::从字段("14400".into(),"D1MO3".into(),"中枢".into(),任意.into(),"三买".into(),任意.into(),0).unwrap();
        let 字典: HashMap<String, 匹配值> = HashMap::new();
        assert!(s.is_match(&字典).is_err());
    }

    #[test]
    fn test_is_match_非字符串_false() {
        let s = Signal::从字段("14400".into(),"D1MO3".into(),"中枢".into(),任意.into(),"三买".into(),任意.into(),0).unwrap();
        let mut 字典 = HashMap::new();
        字典.insert("14400_D1MO3_中枢".to_string(), 匹配值::非字符串);
        assert_eq!(s.is_match(&字典).unwrap(), false);
    }

    #[test]
    fn test_is_match_命中() {
        let s = Signal::从字段("14400".into(),"D1MO3".into(),"中枢".into(),任意.into(),"三买".into(),任意.into(),0).unwrap();
        let mut 字典 = HashMap::new();
        字典.insert("14400_D1MO3_中枢".to_string(), 匹配值::字符串("中枢段DEA穿越2_三买_偏移0_100".into()));
        assert_eq!(s.is_match(&字典).unwrap(), true);
    }

    #[test]
    fn test_is_match_v2不符_未命中() {
        let s = Signal::从字段("14400".into(),"D1MO3".into(),"中枢".into(),任意.into(),"三买".into(),任意.into(),0).unwrap();
        let mut 字典 = HashMap::new();
        字典.insert("14400_D1MO3_中枢".to_string(), 匹配值::字符串("中枢段DEA穿越2_三卖_偏移0_100".into()));
        assert_eq!(s.is_match(&字典).unwrap(), false);
    }
}
```

- [ ] **步骤 2：运行测试验证通过**

运行：`cd /home/moscow/chanlun.rs/chanlun && cargo test signal::signal`
预期：8 个测试全 PASS。

- [ ] **步骤 3：Commit**

```bash
cd /home/moscow/chanlun.rs
git add chanlun/src/signal/signal.rs
git commit -m "feat(signal): Signal 核心 — 解析/key/value/is_match 三态"
```

---

## 任务 3：Factor 核心

**文件：**
- 修改：`chanlun/src/signal/factor.rs`

- [ ] **步骤 1：编写测试 + 实现**

在 `factor.rs` 写入：

```rust
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
        Ok(Self { signals_all, signals_any, signals_not, name })
    }

    /// 确定性哈希 — 拼接三组 signals 串后算 sha256 前4。
    fn 计算哈希(all: &[Signal], any: &[Signal], not: &[Signal]) -> String {
        let 取串 = |v: &[Signal]| v.iter().map(|s| s.signal.clone()).collect::<Vec<_>>().join(",");
        let 规范 = format!("all=[{}]|any=[{}]|not=[{}]", 取串(all), 取串(any), 取串(not));
        sha256前4(&规范)
    }

    pub fn unique_signals(&self) -> Vec<String> {
        let mut 集合 = std::collections::BTreeSet::new();
        for s in self.signals_all.iter().chain(&self.signals_any).chain(&self.signals_not) {
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
        Signal::从字段("14400".into(),"D1MO3".into(),k3.into(),"任意".into(),v2.into(),"任意".into(),0).unwrap()
    }
    fn 字典(k3: &str, v2: &str) -> HashMap<String, 匹配值> {
        let mut m = HashMap::new();
        m.insert(format!("14400_D1MO3_{k3}"), 匹配值::字符串(format!("x_{v2}_y_100")));
        m
    }

    #[test]
    fn test_signals_all_为空_报错() {
        assert!(Factor::新建(vec![], vec![], vec![], "".into()).is_err());
    }

    #[test]
    fn test_name_含哈希后缀() {
        let f = Factor::新建(vec![信号("中枢","三买")], vec![], vec![], "测试".into()).unwrap();
        assert!(f.name.starts_with("测试#"));
        assert_eq!(f.name.len(), "测试#".len() + 4);
    }

    #[test]
    fn test_name_确定性() {
        let f1 = Factor::新建(vec![信号("中枢","三买")], vec![], vec![], "".into()).unwrap();
        let f2 = Factor::新建(vec![信号("中枢","三买")], vec![], vec![], "".into()).unwrap();
        assert_eq!(f1.name, f2.name);
    }

    #[test]
    fn test_all_命中() {
        let f = Factor::新建(vec![信号("中枢","三买")], vec![], vec![], "".into()).unwrap();
        assert_eq!(f.is_match(&字典("中枢","三买")).unwrap(), true);
    }

    #[test]
    fn test_not_命中则false() {
        let f = Factor::新建(vec![信号("中枢","三买")], vec![], vec![信号("中枢","三买")], "".into()).unwrap();
        assert_eq!(f.is_match(&字典("中枢","三买")).unwrap(), false);
    }

    #[test]
    fn test_缺键传播错误() {
        let f = Factor::新建(vec![信号("中枢","三买")], vec![], vec![], "".into()).unwrap();
        let m: HashMap<String, 匹配值> = HashMap::new();
        assert!(f.is_match(&m).is_err());
    }
}
```

- [ ] **步骤 2：运行测试验证通过**

运行：`cd /home/moscow/chanlun.rs/chanlun && cargo test signal::factor`
预期：6 个测试全 PASS。

- [ ] **步骤 3：Commit**

```bash
cd /home/moscow/chanlun.rs
git add chanlun/src/signal/factor.rs
git commit -m "feat(signal): Factor 核心 — and/or/not 匹配 + 确定性命名"
```

---

## 任务 4：Event 核心

**文件：**
- 修改：`chanlun/src/signal/event.rs`

- [ ] **步骤 1：编写测试 + 实现**

在 `event.rs` 写入：

```rust
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
        Ok(Self { operate, factors, signals_all, signals_any, signals_not, name, sha256: hash })
    }

    fn 计算哈希(factors: &[Factor], all: &[Signal], any: &[Signal], not: &[Signal]) -> String {
        let 取串 = |v: &[Signal]| v.iter().map(|s| s.signal.clone()).collect::<Vec<_>>().join(",");
        let 因子串 = factors.iter().map(|f| f.name.clone()).collect::<Vec<_>>().join(";");
        let 规范 = format!(
            "factors=[{}]|all=[{}]|any=[{}]|not=[{}]",
            因子串, 取串(all), 取串(any), 取串(not)
        );
        sha256前4(&规范)
    }

    pub fn unique_signals(&self) -> Vec<String> {
        let mut 集合 = std::collections::BTreeSet::new();
        for s in self.signals_all.iter().chain(&self.signals_any).chain(&self.signals_not) {
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
        Signal::从字段("14400".into(),"D1MO3".into(),k3.into(),"任意".into(),v2.into(),"任意".into(),0).unwrap()
    }
    fn 因子(k3: &str, v2: &str) -> Factor {
        Factor::新建(vec![信号(k3, v2)], vec![], vec![], "".into()).unwrap()
    }
    fn 字典(k3: &str, v2: &str) -> HashMap<String, 匹配值> {
        let mut m = HashMap::new();
        m.insert(format!("14400_D1MO3_{k3}"), 匹配值::字符串(format!("x_{v2}_y_100")));
        m
    }

    #[test]
    fn test_factors_为空_报错() {
        assert!(Event::新建(Operate::开多, vec![], vec![], vec![], vec![], "".into()).is_err());
    }

    #[test]
    fn test_name_默认用operate值() {
        let e = Event::新建(Operate::开多, vec![因子("中枢","三买")], vec![], vec![], vec![], "".into()).unwrap();
        assert!(e.name.starts_with("开多#"));
    }

    #[test]
    fn test_任一因子命中() {
        let e = Event::新建(
            Operate::开多,
            vec![因子("中枢A","三买"), 因子("中枢B","三买")],
            vec![], vec![], vec![], "".into(),
        ).unwrap();
        // 只满足第二个因子
        let (命中, 名) = e.is_match(&字典("中枢B","三买")).unwrap();
        assert!(命中);
        assert!(名.is_some());
    }

    #[test]
    fn test_无因子命中_false() {
        let e = Event::新建(Operate::开多, vec![因子("中枢","三买")], vec![], vec![], vec![], "".into()).unwrap();
        let (命中, 名) = e.is_match(&字典("中枢","三卖")).unwrap();
        assert!(!命中);
        assert!(名.is_none());
    }
}
```

- [ ] **步骤 2：运行测试验证通过**

运行：`cd /home/moscow/chanlun.rs/chanlun && cargo test signal::event`
预期：4 个测试全 PASS。

- [ ] **步骤 3：Commit**

```bash
cd /home/moscow/chanlun.rs
git add chanlun/src/signal/event.rs
git commit -m "feat(signal): Event 核心 — 因子 OR 匹配 + 命中返回因子名"
```

---

## 任务 5：Position 核心（配置 + 校验）

**文件：**
- 修改：`chanlun/src/signal/position.rs`

- [ ] **步骤 1：编写测试 + 实现**

在 `position.rs` 写入：

```rust
//! 仓位配置基类 — 只含配置与事件匹配；update 状态机在 Python 子类。

use crate::signal::event::Event;
use crate::signal::operate::Operate;

#[derive(Clone, Debug)]
pub struct Position {
    pub symbol: String,
    pub opens: Vec<Event>,
    pub exits: Vec<Event>,
    pub events: Vec<Event>,
    pub name: String,
    pub interval: i64,
    pub timeout: i64,
    pub stop_loss: i64,
    pub T0: bool,
}

impl Position {
    /// 构造。name 必填；每个 event.operate 必须 ∈ {开多,平多,开空,平空}。
    pub fn 新建(
        symbol: String,
        opens: Vec<Event>,
        exits: Vec<Event>,
        interval: i64,
        timeout: i64,
        stop_loss: i64,
        T0: bool,
        name: String,
    ) -> Result<Self, String> {
        if name.is_empty() {
            return Err("name 是必须的参数".to_string());
        }
        let mut events = opens.clone();
        events.extend(exits.clone());
        for e in &events {
            if !matches!(
                e.operate,
                Operate::开多 | Operate::平多 | Operate::开空 | Operate::平空
            ) {
                return Err(format!("非法 operate: {}", e.operate.value()));
            }
        }
        Ok(Self { symbol, opens, exits, events, name, interval, timeout, stop_loss, T0 })
    }

    pub fn unique_signals(&self) -> Vec<String> {
        let mut 集合 = std::collections::BTreeSet::new();
        for e in &self.events {
            for s in e.unique_signals() {
                集合.insert(s);
            }
        }
        集合.into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signal::factor::Factor;
    use crate::signal::signal::Signal;

    fn 开多事件() -> Event {
        let s = Signal::从字段("14400".into(),"D1MO3".into(),"中枢".into(),"任意".into(),"三买".into(),"任意".into(),0).unwrap();
        let f = Factor::新建(vec![s], vec![], vec![], "".into()).unwrap();
        Event::新建(Operate::开多, vec![f], vec![], vec![], vec![], "".into()).unwrap()
    }

    #[test]
    fn test_name_缺失_报错() {
        assert!(Position::新建("btc".into(), vec![开多事件()], vec![], 0, 1000, 1000, false, "".into()).is_err());
    }

    #[test]
    fn test_构造成功() {
        let p = Position::新建("btc".into(), vec![开多事件()], vec![], 0, 1000, 1000, false, "中枢".into()).unwrap();
        assert_eq!(p.name, "中枢");
        assert_eq!(p.events.len(), 1);
    }

    #[test]
    fn test_unique_signals_去重() {
        let p = Position::新建("btc".into(), vec![开多事件(), 开多事件()], vec![], 0, 1000, 1000, false, "中枢".into()).unwrap();
        assert_eq!(p.unique_signals().len(), 1);
    }
}
```

- [ ] **步骤 2：运行测试验证通过**

运行：`cd /home/moscow/chanlun.rs/chanlun && cargo test signal::position`
预期：3 个测试全 PASS。运行 `cargo test signal` 确认整模块通过。

- [ ] **步骤 3：Commit**

```bash
cd /home/moscow/chanlun.rs
git add chanlun/src/signal/position.rs
git commit -m "feat(signal): Position 核心 — 配置校验 + unique_signals"
```

---

## 任务 6：PyO3 绑定 — Operate + Signal + 注册接线

**文件：**
- 创建：`chanlun-py/src/signal_py.rs`
- 修改：`chanlun-py/src/lib.rs:111`（mod 声明）、`:238`（注册顺序）

- [ ] **步骤 1：创建绑定文件（Operate + Signal）**

创建 `chanlun-py/src/signal_py.rs`（MIT 头略，照搬 types_py.rs 头）：

```rust
use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::collections::HashMap;

use chanlun::signal::operate::Operate as 核心Operate;
use chanlun::signal::signal::Signal as 核心Signal;
use chanlun::signal::{匹配值, 信号字典};

/// Operate 枚举绑定。
#[pyclass(name = "Operate", module = "chanlun._chanlun", eq, eq_int)]
#[derive(Clone, Copy, PartialEq)]
pub enum OperatePy { HL, HS, HO, LO, LE, SO, SE }

impl OperatePy {
    pub(crate) fn 转核心(self) -> 核心Operate {
        match self {
            OperatePy::HL => 核心Operate::持多,
            OperatePy::HS => 核心Operate::持空,
            OperatePy::HO => 核心Operate::持币,
            OperatePy::LO => 核心Operate::开多,
            OperatePy::LE => 核心Operate::平多,
            OperatePy::SO => 核心Operate::开空,
            OperatePy::SE => 核心Operate::平空,
        }
    }
    pub(crate) fn 从核心(o: 核心Operate) -> Self {
        match o {
            核心Operate::持多 => OperatePy::HL,
            核心Operate::持空 => OperatePy::HS,
            核心Operate::持币 => OperatePy::HO,
            核心Operate::开多 => OperatePy::LO,
            核心Operate::平多 => OperatePy::LE,
            核心Operate::开空 => OperatePy::SO,
            核心Operate::平空 => OperatePy::SE,
        }
    }
}

#[pymethods]
impl OperatePy {
    #[getter]
    fn value(&self) -> &'static str { self.转核心().value() }
    fn __str__(&self) -> &'static str { self.转核心().value() }
    fn __repr__(&self) -> String { format!("Operate.{:?}", self) }
}

/// 把 PyDict 转成核心层信号字典：str 值 → 字符串，其余 → 非字符串。
pub(crate) fn 字典转核心(s: &Bound<'_, PyDict>) -> PyResult<信号字典> {
    let mut out: 信号字典 = HashMap::new();
    for (k, v) in s.iter() {
        let key: String = k.extract()?;
        let 值 = match v.extract::<String>() {
            Ok(文本) if !文本.is_empty() => 匹配值::字符串(文本),
            _ => 匹配值::非字符串,
        };
        out.insert(key, 值);
    }
    Ok(out)
}

/// Signal 绑定。
#[pyclass(name = "Signal", module = "chanlun._chanlun")]
#[derive(Clone)]
pub struct SignalPy {
    pub(crate) inner: 核心Signal,
}

#[pymethods]
impl SignalPy {
    #[new]
    #[pyo3(signature = (signal=String::new(), score=0, k1="任意".to_string(), k2="任意".to_string(), k3="任意".to_string(), v1="任意".to_string(), v2="任意".to_string(), v3="任意".to_string()))]
    fn new(signal: String, score: i32, k1: String, k2: String, k3: String, v1: String, v2: String, v3: String) -> PyResult<Self> {
        let inner = if signal.is_empty() {
            核心Signal::从字段(k1, k2, k3, v1, v2, v3, score)
        } else {
            核心Signal::从字符串(&signal)
        }
        .map_err(PyValueError::new_err)?;
        Ok(Self { inner })
    }

    #[getter] fn signal(&self) -> String { self.inner.signal.clone() }
    #[getter] fn score(&self) -> i32 { self.inner.score }
    #[getter] fn k1(&self) -> String { self.inner.k1.clone() }
    #[getter] fn k2(&self) -> String { self.inner.k2.clone() }
    #[getter] fn k3(&self) -> String { self.inner.k3.clone() }
    #[getter] fn v1(&self) -> String { self.inner.v1.clone() }
    #[getter] fn v2(&self) -> String { self.inner.v2.clone() }
    #[getter] fn v3(&self) -> String { self.inner.v3.clone() }

    #[getter] fn key(&self) -> String { self.inner.key() }
    #[getter] fn value(&self) -> String { self.inner.value() }

    fn is_match(&self, s: &Bound<'_, PyDict>) -> PyResult<bool> {
        let 字典 = 字典转核心(s)?;
        self.inner
            .is_match(&字典)
            .map_err(|e| PyValueError::new_err(format!("{} 不在信号列表中", e.0)))
    }

    fn __repr__(&self) -> String { format!("Signal('{}')", self.inner.signal) }
}

pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<OperatePy>()?;
    m.add_class::<SignalPy>()?;
    Ok(())
}
```

- [ ] **步骤 2：在 lib.rs 接线**

修改 `chanlun-py/src/lib.rs`：第 111 行 `mod types_py;` 之后加 `mod signal_py;`；在第 238 行 `types_py::register(m)?;` 之后加：

```rust
    // 阶段 1.5: 信号原语
    signal_py::register(m)?;
```

- [ ] **步骤 3：构建并冒烟测试导入**

运行：
```bash
cd /home/moscow/chanlun.rs/chanlun-py && maturin develop 2>&1 | tail -3
cd /tmp && python -c "
from chanlun._chanlun import Signal, Operate
s = Signal('14400_D1MO3_中枢_中枢段DEA穿越2_三买_偏移0_100')
assert s.k3 == '中枢', s.k3
assert s.key == '14400_D1MO3_中枢'
assert s.value == '中枢段DEA穿越2_三买_偏移0_100'
assert s.is_match({'14400_D1MO3_中枢': '中枢段DEA穿越2_三买_偏移0_100'}) is True
try:
    s.is_match({})
    raise SystemExit('应抛 ValueError')
except ValueError:
    pass
assert s.is_match({'14400_D1MO3_中枢': 123}) is False
print('Signal/Operate 绑定 OK')
"
```
预期：打印 `Signal/Operate 绑定 OK`。

- [ ] **步骤 4：Commit**

```bash
cd /home/moscow/chanlun.rs
git add chanlun-py/src/signal_py.rs chanlun-py/src/lib.rs
git commit -m "feat(signal-py): Operate/Signal PyO3 绑定 + 注册接线"
```

---

## 任务 7：PyO3 绑定 — Factor + Event

**文件：**
- 修改：`chanlun-py/src/signal_py.rs`

- [ ] **步骤 1：追加 Factor + Event 绑定**

在 `signal_py.rs` 的 `register` 函数之前追加：

```rust
use chanlun::signal::event::Event as 核心Event;
use chanlun::signal::factor::Factor as 核心Factor;

#[pyclass(name = "Factor", module = "chanlun._chanlun")]
#[derive(Clone)]
pub struct FactorPy {
    pub(crate) inner: 核心Factor,
}

#[pymethods]
impl FactorPy {
    #[new]
    #[pyo3(signature = (signals_all, signals_any=Vec::new(), signals_not=Vec::new(), name=String::new()))]
    fn new(signals_all: Vec<SignalPy>, signals_any: Vec<SignalPy>, signals_not: Vec<SignalPy>, name: String) -> PyResult<Self> {
        let 取 = |v: Vec<SignalPy>| v.into_iter().map(|s| s.inner).collect::<Vec<_>>();
        let inner = 核心Factor::新建(取(signals_all), 取(signals_any), 取(signals_not), name)
            .map_err(PyValueError::new_err)?;
        Ok(Self { inner })
    }

    #[getter] fn name(&self) -> String { self.inner.name.clone() }
    #[getter] fn signals_all(&self) -> Vec<SignalPy> { self.inner.signals_all.iter().cloned().map(|inner| SignalPy { inner }).collect() }
    #[getter] fn signals_any(&self) -> Vec<SignalPy> { self.inner.signals_any.iter().cloned().map(|inner| SignalPy { inner }).collect() }
    #[getter] fn signals_not(&self) -> Vec<SignalPy> { self.inner.signals_not.iter().cloned().map(|inner| SignalPy { inner }).collect() }

    #[getter] fn unique_signals(&self) -> Vec<String> { self.inner.unique_signals() }

    fn is_match(&self, s: &Bound<'_, PyDict>) -> PyResult<bool> {
        let 字典 = 字典转核心(s)?;
        self.inner.is_match(&字典).map_err(|e| PyValueError::new_err(format!("{} 不在信号列表中", e.0)))
    }
}

#[pyclass(name = "Event", module = "chanlun._chanlun")]
#[derive(Clone)]
pub struct EventPy {
    pub(crate) inner: 核心Event,
}

#[pymethods]
impl EventPy {
    #[new]
    #[pyo3(signature = (operate, factors, signals_all=Vec::new(), signals_any=Vec::new(), signals_not=Vec::new(), name=String::new()))]
    fn new(operate: OperatePy, factors: Vec<FactorPy>, signals_all: Vec<SignalPy>, signals_any: Vec<SignalPy>, signals_not: Vec<SignalPy>, name: String) -> PyResult<Self> {
        let 取s = |v: Vec<SignalPy>| v.into_iter().map(|s| s.inner).collect::<Vec<_>>();
        let 取f = |v: Vec<FactorPy>| v.into_iter().map(|f| f.inner).collect::<Vec<_>>();
        let inner = 核心Event::新建(operate.转核心(), 取f(factors), 取s(signals_all), 取s(signals_any), 取s(signals_not), name)
            .map_err(PyValueError::new_err)?;
        Ok(Self { inner })
    }

    #[getter] fn name(&self) -> String { self.inner.name.clone() }
    #[getter] fn sha256(&self) -> String { self.inner.sha256.clone() }
    #[getter] fn operate(&self) -> OperatePy { OperatePy::从核心(self.inner.operate) }
    #[getter] fn factors(&self) -> Vec<FactorPy> { self.inner.factors.iter().cloned().map(|inner| FactorPy { inner }).collect() }
    #[getter] fn unique_signals(&self) -> Vec<String> { self.inner.unique_signals() }

    fn is_match(&self, s: &Bound<'_, PyDict>) -> PyResult<(bool, Option<String>)> {
        let 字典 = 字典转核心(s)?;
        self.inner.is_match(&字典).map_err(|e| PyValueError::new_err(format!("{} 不在信号列表中", e.0)))
    }
}
```

在 `register` 中追加：

```rust
    m.add_class::<FactorPy>()?;
    m.add_class::<EventPy>()?;
```

- [ ] **步骤 2：构建并冒烟测试**

运行：
```bash
cd /home/moscow/chanlun.rs/chanlun-py && maturin develop 2>&1 | tail -3
cd /tmp && python -c "
from chanlun._chanlun import Signal, Factor, Event, Operate
s = Signal(k1='14400', k2='D1MO3', k3='中枢', v2='三买')
f = Factor(signals_all=[s])
assert f.name.startswith('#'), f.name
e = Event(Operate.LO, [f])
assert e.name.startswith('开多#'), e.name
d = {'14400_D1MO3_中枢': '中枢段DEA穿越2_三买_偏移0_100'}
assert f.is_match(d) is True
ok, fname = e.is_match(d)
assert ok and fname
print('Factor/Event 绑定 OK')
"
```
预期：打印 `Factor/Event 绑定 OK`。

- [ ] **步骤 3：Commit**

```bash
cd /home/moscow/chanlun.rs
git add chanlun-py/src/signal_py.rs
git commit -m "feat(signal-py): Factor/Event PyO3 绑定"
```

---

## 任务 8：PyO3 绑定 — Position（可子类化）

**文件：**
- 修改：`chanlun-py/src/signal_py.rs`

- [ ] **步骤 1：追加 Position 绑定**

在 `register` 之前追加：

```rust
use chanlun::signal::position::Position as 核心Position;

#[pyclass(name = "Position", module = "chanlun._chanlun", subclass)]
#[derive(Clone)]
pub struct PositionPy {
    pub(crate) inner: 核心Position,
}

#[pymethods]
impl PositionPy {
    #[new]
    #[pyo3(signature = (symbol, opens, exits=Vec::new(), interval=0, timeout=1000, stop_loss=1000, T0=false, name=String::new()))]
    fn new(symbol: String, opens: Vec<EventPy>, exits: Vec<EventPy>, interval: i64, timeout: i64, stop_loss: i64, T0: bool, name: String) -> PyResult<Self> {
        let 取 = |v: Vec<EventPy>| v.into_iter().map(|e| e.inner).collect::<Vec<_>>();
        let inner = 核心Position::新建(symbol, 取(opens), 取(exits), interval, timeout, stop_loss, T0, name)
            .map_err(PyValueError::new_err)?;
        Ok(Self { inner })
    }

    #[getter] fn symbol(&self) -> String { self.inner.symbol.clone() }
    #[getter] fn name(&self) -> String { self.inner.name.clone() }
    #[getter] fn opens(&self) -> Vec<EventPy> { self.inner.opens.iter().cloned().map(|inner| EventPy { inner }).collect() }
    #[getter] fn exits(&self) -> Vec<EventPy> { self.inner.exits.iter().cloned().map(|inner| EventPy { inner }).collect() }
    #[getter] fn events(&self) -> Vec<EventPy> { self.inner.events.iter().cloned().map(|inner| EventPy { inner }).collect() }
    #[getter] fn interval(&self) -> i64 { self.inner.interval }
    #[getter] fn timeout(&self) -> i64 { self.inner.timeout }
    #[getter] fn stop_loss(&self) -> i64 { self.inner.stop_loss }
    #[getter] fn T0(&self) -> bool { self.inner.T0 }
    #[getter] fn unique_signals(&self) -> Vec<String> { self.inner.unique_signals() }

    fn __repr__(&self) -> String {
        format!(
            "Position(name={}, symbol={}, timeout={}, stop_loss={}BP, T0={}, interval={}s)",
            self.inner.name, self.inner.symbol, self.inner.timeout, self.inner.stop_loss, self.inner.T0, self.inner.interval
        )
    }
}
```

在 `register` 追加 `m.add_class::<PositionPy>()?;`。

- [ ] **步骤 2：构建并验证可被 Python 子类化 + 加状态**

运行：
```bash
cd /home/moscow/chanlun.rs/chanlun-py && maturin develop 2>&1 | tail -3
cd /tmp && python -c "
from chanlun._chanlun import Signal, Factor, Event, Operate, Position as Base
class Pos(Base):
    def __init__(self, **kw):
        super().__init__(**kw)
        self.pos = 0          # 子类加状态字段
        self.operates = []
    def update(self, s): self.pos = 1
s = Signal(k1='14400', k2='D1MO3', k3='中枢', v2='三买')
e = Event(Operate.LO, [Factor(signals_all=[s])])
p = Pos(symbol='btc', opens=[e], name='中枢')
assert p.name == '中枢'
assert p.unique_signals == ['14400_D1MO3_中枢_任意_三买_任意_0']
p.update({})
assert p.pos == 1
assert p.operates == []
print('Position 子类化 + 状态字段 OK')
"
```
预期：打印 `Position 子类化 + 状态字段 OK`。若 `super().__init__(**kw)` 报错（PyO3 子类构造限制），改为 `super().__init__(symbol, opens, exits, interval, timeout, stop_loss, T0, name)` 位置传参并在计划任务 9 的子类里同步。

- [ ] **步骤 3：Commit**

```bash
cd /home/moscow/chanlun.rs
git add chanlun-py/src/signal_py.rs
git commit -m "feat(signal-py): Position PyO3 绑定（可子类化）"
```

---

## 任务 9：Python 集成 — chan_external.py 切换

**文件：**
- 修改：`chanlun-py/chanlun/chan_external.py`

- [ ] **步骤 1：替换原语类为 Rust 导入**

在 `chan_external.py` 顶部 import 区加入（紧跟现有 import）：

```python
from chanlun._chanlun import (
    Signal,
    Factor,
    Event,
    Operate,
    Position as _PositionBase,
)
```

删除文件中原有的 `class Operate(Enum)`、`@dataclass class Signal`、`@dataclass class Factor`、`@dataclass class Event` 四处定义（即任务 0 设计文档 §7 列出的类）。`Signal.load`/`Factor.load`/`Event.load` 等 classmethod 若调用方有用到，保留为模块级 helper（见步骤 2）。

- [ ] **步骤 2：Position 改为子类**

把原 `class Position:` 改为继承 Rust 基类，仅保留状态字段 + `update`/`pairs`/`get_signals_config`/`with_data` 版 `dump`：

```python
class Position(_PositionBase):
    def __init__(self, symbol, opens, exits=[], interval=0, timeout=1000,
                 stop_loss=1000, T0=False, name=None):
        if not name:
            raise ValueError("name 是必须的参数")
        super().__init__(symbol, opens, exits, interval, timeout, stop_loss, T0, name)
        self.pos_changed = False
        self.operates = []
        self.holds = []
        self.pos = 0
        self.last_event = {"dt": None, "bid": None, "price": None, "op": None, "op_desc": None}
        self.last_lo_dt = None
        self.last_so_dt = None
        self.end_dt = None

    # 以下方法在当前 Position 类中已存在，转为子类时原样保留其方法体不变：
    #   - pairs (property)
    #   - update(self, s)
    #   - get_signals_config(self, signals_module="")
    #   - dump(self, with_data=False)
    #   - load (classmethod)
    # 即：把 `class Position:` 改成 `class Position(_PositionBase):`，
    #     __init__ 换成上面的版本（super().__init__ + 状态字段），
    #     其余方法定义整段保留。
```

> 注：原 `Position.dump()` 的 config 部分由 Rust 基类不提供 dump 方法，故 Python 子类继续实现完整 `dump(with_data=False)`（config 字段从 `self.symbol`/`self.opens` 等 getter 读，state 字段从子类读）。`Position.load` classmethod 保留在子类。

- [ ] **步骤 3：同步根目录 chan.py**

对根目录 `chan.py` 中合并进来的 `Operate`/`Signal`/`Factor`/`Event`/`Position` 做同样替换（import Rust 版 + Position 子类），保持与包版本一致。

- [ ] **步骤 4：冒烟回归**

运行：
```bash
cd /home/moscow/chanlun.rs/chanlun-py && maturin develop 2>&1 | tail -2
cd /home/moscow/chanlun.rs && python -c "
import chanlun.chan_external as cet
s = cet.Signal(k1='14400', k2='D1MO3', k3='中枢段DEA穿越2V230602', v2='三买')
e = cet.Event(cet.Operate.LO, [cet.Factor(signals_all=[s])])
p = cet.Position(symbol='btc', opens=[e], name='中枢')
assert p.pos == 0
print('chan_external 集成 OK')
"
```
预期：打印 `chan_external 集成 OK`。

- [ ] **步骤 5：Commit**

```bash
cd /home/moscow/chanlun.rs
git add chanlun-py/chanlun/chan_external.py chan.py
git commit -m "refactor(signal): chan_external 切换 Rust 原语 + Position 子类化"
```

---

## 任务 10：跨语言一致性测试 + 回归

**文件：**
- 创建：`chanlun-py/tests/test_signal_primitives.py`

- [ ] **步骤 1：编写一致性测试**

创建 `chanlun-py/tests/test_signal_primitives.py`：

```python
"""信号原语 Rust 移植后的跨语言一致性与边界行为测试。"""
import pytest
from chanlun._chanlun import Signal, Factor, Event, Operate, Position


def test_signal_parse_and_props():
    s = Signal("14400_D1MO3_中枢_中枢段DEA穿越2_三买_偏移0_100")
    assert s.k1 == "14400" and s.k3 == "中枢" and s.v2 == "三买" and s.score == 100
    assert s.key == "14400_D1MO3_中枢"
    assert s.value == "中枢段DEA穿越2_三买_偏移0_100"
    assert repr(s) == "Signal('14400_D1MO3_中枢_中枢段DEA穿越2_三买_偏移0_100')"


def test_signal_score_out_of_range():
    with pytest.raises(ValueError):
        Signal(k1="a", k2="b", k3="c", score=101)


def test_signal_is_match_missing_key_raises():
    s = Signal(k1="14400", k2="D1MO3", k3="中枢", v2="三买")
    with pytest.raises(ValueError):
        s.is_match({})


def test_signal_is_match_non_str_value_false():
    s = Signal(k1="14400", k2="D1MO3", k3="中枢", v2="三买")
    assert s.is_match({"14400_D1MO3_中枢": 123}) is False


def test_signal_is_match_hit():
    s = Signal(k1="14400", k2="D1MO3", k3="中枢", v2="三买")
    assert s.is_match({"14400_D1MO3_中枢": "x_三买_y_100"}) is True


def test_factor_empty_all_raises():
    with pytest.raises(ValueError):
        Factor(signals_all=[])


def test_factor_name_deterministic():
    s = Signal(k1="14400", k2="D1MO3", k3="中枢", v2="三买")
    f1 = Factor(signals_all=[s])
    f2 = Factor(signals_all=[Signal(k1="14400", k2="D1MO3", k3="中枢", v2="三买")])
    assert f1.name == f2.name


def test_event_empty_factors_raises():
    with pytest.raises(ValueError):
        Event(Operate.LO, [])


def test_event_match_returns_factor_name():
    s = Signal(k1="14400", k2="D1MO3", k3="中枢", v2="三买")
    e = Event(Operate.LO, [Factor(signals_all=[s])])
    ok, name = e.is_match({"14400_D1MO3_中枢": "x_三买_y_100"})
    assert ok and name


def test_position_requires_name():
    s = Signal(k1="14400", k2="D1MO3", k3="中枢", v2="三买")
    e = Event(Operate.LO, [Factor(signals_all=[s])])
    with pytest.raises((ValueError, TypeError)):
        Position(symbol="btc", opens=[e])


def test_position_multi_factor_event_or():
    """复刻策略里的多 Factor OR 用法。"""
    base = "14400"
    买 = [
        Factor(signals_all=[Signal(k1=base, k2="D1MO3", k3=k3, v2="三买")])
        for k3 in ["中枢段DEA穿越2V230602", "DEA穿越0轴V230602", "首次穿越0轴V230602"]
    ]
    e = Event(Operate.LO, 买)
    # 只命中第三个 k3
    d = {"14400_D1MO3_首次穿越0轴V230602": "首次穿越0轴_三买_偏移0_100"}
    # 另两个 k3 缺键 → is_match 抛 ValueError（与 Python 行为一致，由策略 try/except 兜底）
    with pytest.raises(ValueError):
        e.is_match(d)
```

- [ ] **步骤 2：运行测试**

运行：`cd /home/moscow/chanlun.rs/chanlun-py && python -m pytest tests/test_signal_primitives.py -v`
预期：全部 PASS。

> 注：`test_position_multi_factor_event_or` 验证「缺键 raise」契约——策略中 `pos.update` 外层有 `try/except ValueError`，这里直接断言 raise 行为，确认 Rust 与 Python 语义一致。

- [ ] **步骤 3：回归 — 信号识别 + sync 回测**

运行：
```bash
cd /home/moscow/chanlun.rs && python -c "
import chan as root_chan
from chanlun.chan import 缠论配置
配置 = 缠论配置(买卖点偏移=5)
配置.加载文件路径 = 'templates/btcusd-14400-1753171200-1781956800.nb'
魔法 = root_chan.测试_信号识别(配置)
魔法()
" 2>&1 | grep -c "📡"
```
预期：信号行数 > 0，且包含多种信号类型（与移植前一致）。

- [ ] **步骤 4：Commit**

```bash
cd /home/moscow/chanlun.rs
git add chanlun-py/tests/test_signal_primitives.py
git commit -m "test(signal): 跨语言一致性 + 缺键契约 + 回归"
```

---

## 自检结论

- **规格覆盖**：§5 五个组件 → 任务 1-5（核心）+ 6-8（绑定）；§6 三关键点 → 任务 2（is_match 三态）/任务 6（字典转核心 + 缺键 raise）/任务 3-4（确定性命名）；§7 drop-in → 任务 9；§8 测试 → 任务 1-5 cargo 单测 + 任务 10 pytest + 回归。全覆盖。
- **类型一致**：核心 `Signal`/`Factor`/`Event`/`Position`/`Operate` 与绑定 `SignalPy`/`FactorPy`/`EventPy`/`PositionPy`/`OperatePy` 命名贯穿；`匹配值`/`信号字典`/`缺键错误`/`sha256前4` 在 mod.rs 定义，各处引用一致。
- **占位符**：无 TODO/待定；每个代码步骤含完整可编译代码。
- **已知风险**：任务 8 步骤 2 标注了 PyO3 子类构造的 fallback（位置传参）。
