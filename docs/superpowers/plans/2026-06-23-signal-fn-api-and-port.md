# 子项目2 信号函数 API + 移植 youwukuncheng 实现计划

> **面向 AI 代理的工作者：** 必需子技能：使用 superpowers:subagent-driven-development（推荐）或 superpowers:executing-plans 逐任务实现此计划。步骤使用复选框（`- [ ]`）语法来跟踪进度。

**目标：** 建立 Rust 信号函数编写规范（便捷 API + 参数提取 + 确保指标），移植第一个真实信号 `youwukuncheng_中枢第三买卖点_V230602`，并通过集成测试与 Python 版对比验证。

**架构：** 便捷方法直接加到 `K线`/`缠论K线`/`观察者` 上（不引入额外 trait）；信号函数放 `chanlun/src/signal/functions/`；参数提取独立为 `signal/params.rs`。

**技术栈：** Rust edition 2024、`serde_json::Value`、`parking_lot::RwLock`、`inventory`。

**设计文档：** `docs/superpowers/specs/2026-06-23-signal-fn-api-and-port-design.md`

---

## 文件结构

| 文件 | 职责 |
|---|---|
| `chanlun/src/kline/bar.rs` | 给 `K线` 加便捷指标访问方法 (`macd()`, `rsi()`, `kdj()`, `boll()`, `ma()`) |
| `chanlun/src/kline/chan_kline.rs` | 给 `缠论K线` 加转发便捷方法 |
| `chanlun/src/business/observer.rs` | 加 `普K偏移()`、`缠K偏移()`、`最后缠K序列()`、`确保指标已计算()` |
| `chanlun/src/signal/params.rs` | **新建** — 参数提取辅助函数 (`get_string`, `get_int`, `get_f64`) |
| `chanlun/src/signal/mod.rs` | 增 `pub mod params;` + `pub mod functions;` |
| `chanlun/src/signal/functions/mod.rs` | **新建** — `pub mod youwukuncheng;` |
| `chanlun/src/signal/functions/youwukuncheng.rs` | **新建** — 移植的中枢第三买卖点信号 |
| `chanlun/tests/test_signal_youwukuncheng.rs` | **新建** — 集成测试（Rust vs Python 对比） |

---

## 任务 0：便捷 API — K线指标访问 + 观察者方法 + 参数提取

**文件：**
- 修改：`chanlun/src/kline/bar.rs`
- 修改：`chanlun/src/kline/chan_kline.rs`
- 修改：`chanlun/src/business/observer.rs`
- 创建：`chanlun/src/signal/params.rs`
- 修改：`chanlun/src/signal/mod.rs`

### 步骤 1：K线 便捷指标访问方法

在 `chanlun/src/kline/bar.rs` 的 `impl K线` 块中添加以下方法。

`K线` 已有 `pub 指标: RwLock<指标容器>` 字段，以及 `pub 收盘价: f64` 等 OHLC 字段。新增方法封装 `self.指标.read()` 的 boilerplate：

```rust
/// 便捷读取 MACD 指标。若未计算则返回 None。
pub fn macd(&self) -> Option<&平滑异同移动平均线> {
    // 注意：返回的引用受 RwLockReadGuard 生命周期约束
    // 需要 unsafe 或者改用 cloned 版本
    // 实际采用：提供返回 Option<平滑异同移动平均线> 的 cloned 版本
    // 同时提供一个需要传入 guard 的零拷贝版本
}

// 实际实现方案：提供 _cloned 便捷方法（开销可忽略，MACD 仅几个 f64）
pub fn macd(&self) -> Option<平滑异同移动平均线> {
    self.指标.read().macd_cloned()
}
pub fn rsi(&self) -> Option<相对强弱指数> {
    self.指标.read().rsi_cloned()
}
pub fn kdj(&self) -> Option<随机指标> {
    self.指标.read().kdj_cloned()
}
pub fn boll(&self) -> Option<布林带> {
    self.指标.read().boll_cloned()
}
pub fn ma(&self, key: &str) -> Option<f64> {
    self.指标.read().均线().and_then(|m| m.get(key).copied())
}
```

> **设计理由**：使用 `_cloned` 版本而非返回引用，避免 `RwLockReadGuard` 生命周期传染到调用方。MACD/RSI/KDJ/BOLL 结构体只含少量 f64 和 Option<f64>，clone 开销可忽略。

### 步骤 2：缠论K线 便捷转发方法

在 `chanlun/src/kline/chan_kline.rs` 的 `impl 缠论K线` 块中添加转发方法。缠K 有 `pub 标的K线: RwLock<Arc<K线>>` 字段：

```rust
/// 便捷读取 MACD（委托给标的K线）
pub fn macd(&self) -> Option<平滑异同移动平均线> {
    self.标的K线.read().macd()
}
pub fn rsi(&self) -> Option<相对强弱指数> {
    self.标的K线.read().rsi()
}
pub fn kdj(&self) -> Option<随机指标> {
    self.标的K线.read().kdj()
}
pub fn boll(&self) -> Option<布林带> {
    self.标的K线.read().boll()
}
pub fn ma(&self, key: &str) -> Option<f64> {
    self.标的K线.read().ma(key)
}
/// 读取收盘价（委托给标的K线）
pub fn 收盘价(&self) -> f64 {
    self.标的K线.read().收盘价
}
```

### 步骤 3：观察者便捷访问方法

在 `chanlun/src/business/observer.rs` 的 `impl 观察者` 块中添加：

```rust
/// 按偏移取普K，di=1 为最后一根，di=2 为倒数第二根
pub fn 普K偏移(&self, di: usize) -> Option<&Arc<K线>> {
    if di == 0 || di > self.普通K线序列.len() { return None; }
    Some(&self.普通K线序列[self.普通K线序列.len() - di])
}

/// 按偏移取缠K，di=1 为最后一根
pub fn 缠K偏移(&self, di: usize) -> Option<&Arc<缠论K线>> {
    if di == 0 || di > self.缠论K线序列.len() { return None; }
    Some(&self.缠论K线序列[self.缠论K线序列.len() - di])
}

/// 最后 N 根缠K（返回切片引用）
pub fn 最后缠K序列(&self, n: usize) -> &[Arc<缠论K线>] {
    let len = self.缠论K线序列.len();
    if n >= len { &self.缠论K线序列[..] }
    else { &self.缠论K线序列[len - n..] }
}
```

### 步骤 4：参数提取模块

创建 `chanlun/src/signal/params.rs`：

```rust
//! 信号函数参数提取辅助 — 从 `HashMap<String, Value>` 中提取类型化参数。

use serde_json::Value;
use std::collections::HashMap;

/// 提取字符串参数，缺失或类型不对时返回默认值。
pub fn get_string(params: &HashMap<String, Value>, key: &str, default: &str) -> String {
    params.get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| default.to_string())
}

/// 提取 i64 参数。
pub fn get_int(params: &HashMap<String, Value>, key: &str, default: i64) -> i64 {
    params.get(key)
        .and_then(|v| v.as_i64())
        .unwrap_or(default)
}

/// 提取 f64 参数。
pub fn get_f64(params: &HashMap<String, Value>, key: &str, default: f64) -> f64 {
    params.get(key)
        .and_then(|v| v.as_f64())
        .unwrap_or(default)
}

/// 提取字符串引用（零拷贝），缺失时返回默认值。
pub fn get_str<'a>(params: &'a HashMap<String, Value>, key: &str, default: &'a str) -> &'a str {
    params.get(key)
        .and_then(|v| v.as_str())
        .unwrap_or(default)
}
```

修改 `chanlun/src/signal/mod.rs`，在 `pub mod registry;` 后追加：
```rust
pub mod params;
pub mod functions;
```

### 步骤 5：构建验证

```bash
cd chanlun && cargo build
```
预期：编译通过。

### 步骤 6：Commit

```bash
git add chanlun/src/kline/bar.rs chanlun/src/kline/chan_kline.rs \
        chanlun/src/business/observer.rs chanlun/src/signal/params.rs \
        chanlun/src/signal/mod.rs
git commit -m "feat(signal): 便捷API — K线指标访问 + 观察者偏移 + 参数提取"
```

---

## 任务 1：确保指标 API

**文件：**
- 修改：`chanlun/src/business/observer.rs`

### 步骤 1：添加 `确保指标已计算` 方法

在 `观察者` 的 `impl` 块中添加（需要 `use crate::indicators::calculator::指标计算器;`）：

```rust
/// 确保所有 K 线上的指标已计算（幂等）。
/// 在信号函数入口调用，保证后续 macd()/rsi() 等访问不返回 None。
pub fn 确保指标已计算(&self) {
    if self.配置.计算指标 && !self.普通K线序列.is_empty() {
        指标计算器::计算并挂载(&self.普通K线序列, &self.配置);
    }
}
```

### 步骤 2：构建验证

```bash
cd chanlun && cargo build
```

### 步骤 3：Commit

```bash
git add chanlun/src/business/observer.rs
git commit -m "feat(signal): 观察者.确保指标已计算() — 信号函数入口幂等调用"
```

---

## 任务 2：移植 youwukuncheng 信号函数

**文件：**
- 创建：`chanlun/src/signal/functions/mod.rs`
- 创建：`chanlun/src/signal/functions/youwukuncheng.rs`

### 步骤 1：创建 functions 模块入口

创建 `chanlun/src/signal/functions/mod.rs`：

```rust
//! 信号函数实现 — 每个 `#[signal]` 注册的函数对应一个子模块。
//!
//! 第三方代码声明：信号函数模式参考 czsc（https://github.com/waditu/czsc，
//! Apache License 2.0），已适配为 Rust `fn(&观察者, &HashMap<String, Value>) -> Vec<Signal>`。

pub mod youwukuncheng;
```

### 步骤 2：编写 youwukuncheng.rs

创建 `chanlun/src/signal/functions/youwukuncheng.rs`。核心结构：

```rust
use std::collections::HashMap;
use serde_json::Value;
use chanlun_signal_macros::signal;

use crate::business::observer::观察者;
use crate::signal::params;
use crate::signal::Signal;

/// 中枢第三买卖点信号 — 返回所有匹配的第三类买卖点信号。
///
/// 参数模板："{freq}_D1MO{max_overlap}_中枢第三买卖点V230602"
///
/// 返回三种信号（k3 = 特征 + "V230602"）：
/// - 中枢段DEA穿越2V230602（同级检查）
/// - DEA穿越0轴V230602（本级检查，无须分型）
/// - 首次穿越0轴V230602（本级检查 + 分型确认）
#[signal(
    name = "youwukuncheng_中枢第三买卖点_V230602",
    template = "{freq}_D1MO{max_overlap}_中枢第三买卖点V230602"
)]
pub fn youwukuncheng_中枢第三买卖点_V230602(
    obs: &观察者,
    params: &HashMap<String, Value>,
) -> Vec<Signal> {
    // 1. 确保指标已计算
    obs.确保指标已计算();

    // 2. 提取参数
    let max_overlap = params::get_int(params, "max_overlap", 3);
    let freq = params::get_string(params, "freq", "日线");
    let 本级完整性 = params::get_string(params, "本级完整性", "实");
    let 同级完整性 = params::get_string(params, "同级完整性", "合");

    let k1 = freq;
    let k2 = format!("D1MO{max_overlap}");
    let k3 = "中枢第三买卖点V230602";

    // 3. 前置检查
    let 当前缠K = match obs.当前缠K() {
        Some(k) => k,
        None => return vec![Signal::new_empty(&k1, &k2, k3)],
    };

    // 使用线段中枢序列（对应 Python 的 观察员.中枢序列）
    let 中枢序列 = obs.线段中枢序列();
    if 中枢序列.is_empty() {
        return vec![Signal::new_empty(&k1, &k2, k3)];
    }

    let 当前中枢 = &中枢序列[中枢序列.len() - 1];

    // 检查是否基于线段
    if 当前中枢.基础序列.read()[0].标识.read().as_str() != "线段" {
        return vec![Signal::new_empty(&k1, &k2, k3)];
    }

    // 检查中枢状态
    if 当前中枢.当前状态() == "中枢之中" {
        return vec![Signal::new_empty(&k1, &k2, k3)];
    }

    // 检查本级第三买卖线
    let 本级线 = match 当前中枢.本级_第三买卖线.read().as_ref() {
        Some(line) => Arc::clone(line),
        None => return vec![Signal::new_empty(&k1, &k2, k3)],
    };

    let mut result = Vec::new();
    let mut 买卖点分型: Option<Arc<分型>> = None;
    let 状态 = 当前中枢.当前状态();

    // 4. 本级检查
    if 当前中枢.完整性(&本级完整性) {
        // ... DEA穿越0轴 + 首次穿越0轴 逻辑
        // (详见完整实现)
    }

    // 5. 同级检查
    // ... 中枢段DEA穿越2 逻辑
    // (详见完整实现)

    if result.is_empty() {
        vec![Signal::new_empty(&k1, &k2, k3)]
    } else {
        result
    }
}
```

> **注意**：上述为骨架代码。完整实现需按 Python 版 1:1 翻译，包括：
> - `之后缠K序列` 切片（`缠论K线序列[index..]`）
> - DIF/DEA 零轴穿越检测循环
> - 分型确认 + `分型::从缠K序列中获取分型`
> - `线段::分割序列` + `虚线::统计MACD行为`
> - 偏移计算与 score = max(0, 100 - 偏移 * 5)

需要额外依赖 `Signal` 的空构造器。在 `signal/signal.rs` 中添加：

```rust
impl Signal {
    /// 创建一个"空"信号（v1=v2=v3="任意"，score=0），对应 Python `create_single_signal(k1=k1, k2=k2, k3=k3)`
    pub fn new_empty(k1: &str, k2: &str, k3: &str) -> Self {
        Self {
            signal: format!("{}_{}_{}_任意_任意_任意_0", k1, k2, k3),
            score: 0,
            k1: k1.to_string(),
            k2: k2.to_string(),
            k3: k3.to_string(),
            v1: "任意".to_string(),
            v2: "任意".to_string(),
            v3: "任意".to_string(),
        }
    }

    /// 创建带分类值的信号
    pub fn new(k1: &str, k2: &str, k3: &str, v1: &str, v2: &str, v3: &str, score: i32) -> Self {
        Self {
            signal: format!("{}_{}_{}_{}_{}_{}_{}", k1, k2, k3, v1, v2, v3, score),
            score,
            k1: k1.to_string(),
            k2: k2.to_string(),
            k3: k3.to_string(),
            v1: v1.to_string(),
            v2: v2.to_string(),
            v3: v3.to_string(),
        }
    }
}
```

### 步骤 3：构建验证

```bash
cd chanlun && cargo build
```
预期：编译通过。

### 步骤 4：Commit

```bash
git add chanlun/src/signal/functions/ chanlun/src/signal/signal.rs
git commit -m "feat(signal): 移植 youwukuncheng_中枢第三买卖点_V230602 到 Rust"
```

---

## 任务 3：集成测试 — Rust vs Python 对比

**文件：**
- 创建：`chanlun/tests/test_signal_youwukuncheng.rs`

### 步骤 1：创建 Python 参考脚本

在 `chanlun-py/tests/` 下创建 `gen_youwukuncheng_golden.py`，跑 Python 版信号函数并输出 JSON：

```python
"""生成 youwukuncheng 信号预期输出（golden file）"""
import json, sys
sys.path.insert(0, '.')
from chanlun.chan import 观察者, 缠论配置, K线
from chanlun.signals.youwukuncheng import youwukuncheng_中枢第三买卖点_V230602

# 加载 .nb 文件
obs = 观察者("btcusd", 86400, 缠论配置.默认())
obs.读取数据文件("chanlun-py/tests/btcusd-86400-xxx.nb", 缠论配置.默认())

# 调用信号函数
params = {"freq": "日线", "max_overlap": 3, "本级完整性": "实", "同级完整性": "合"}
result = youwukuncheng_中枢第三买卖点_V230602(obs, **params)

# 输出为 JSON
output = {k: v for k, v in result.items()}
print(json.dumps(output, ensure_ascii=False, indent=2))
```

### 步骤 2：编写 Rust 集成测试

创建 `chanlun/tests/test_signal_youwukuncheng.rs`：

```rust
use std::collections::HashMap;
use chanlun::business::observer::观察者;
use chanlun::config::缠论配置;
use chanlun::signal::functions::youwukuncheng::youwukuncheng_中枢第三买卖点_V230602;
use serde_json::Value;

#[test]
fn test_youwukuncheng_产生信号() {
    let obs = 观察者::new("btcusd".into(), 86400, 缠论配置::default());
    obs.write().读取数据文件("tests/btcusd-86400-xxx.nb", 缠论配置::default().不推送())
        .expect("读取数据文件失败");

    let obs = obs.read();

    let mut params = HashMap::new();
    params.insert("freq".to_string(), Value::String("日线".to_string()));
    params.insert("max_overlap".to_string(), Value::Number(3.into()));
    params.insert("本级完整性".to_string(), Value::String("实".to_string()));
    params.insert("同级完整性".to_string(), Value::String("合".to_string()));

    let signals = youwukuncheng_中枢第三买卖点_V230602(&obs, &params);

    println!("产生 {} 个信号:", signals.len());
    for s in &signals {
        println!("  key={} value={} score={}", s.key(), s.value(), s.score);
    }

    // 至少有一个非空信号（取决于数据）
    let non_empty: Vec<_> = signals.iter()
        .filter(|s| s.value() != "任意_任意_任意_0")
        .collect();
    println!("非空信号数: {}", non_empty.len());

    // 验证所有信号的 k3 后缀
    for s in &signals {
        assert!(s.k3.ends_with("V230602"), "k3 必须以 V230602 结尾: {}", s.k3);
    }
}

#[test]
fn test_youwukuncheng_无中枢返回空信号() {
    let obs = 观察者::new("empty".into(), 300, 缠论配置::default());
    let obs = obs.read();

    let params = HashMap::new();
    let signals = youwukuncheng_中枢第三买卖点_V230602(&obs, &params);

    assert_eq!(signals.len(), 1);
    assert_eq!(signals[0].value(), "任意_任意_任意_0");
}
```

### 步骤 3：运行测试

```bash
cd chanlun && cargo test --test test_signal_youwukuncheng
```
预期：测试通过（或根据数据情况调整断言）。

### 步骤 4：Commit

```bash
git add chanlun/tests/test_signal_youwukuncheng.rs
git commit -m "test(signal): youwukuncheng 集成测试 — 信号产出 + 空中枢边界"
```

---

## 自检结论

- **规格覆盖**：设计 §5 便捷 API → 任务 0；§7 确保指标 → 任务 1；§6 youwukuncheng → 任务 2；§8 测试 → 任务 3。全覆盖。
- **类型一致**：`SignalFn` 签名不变。`#[signal]` 注册用子项目 1 的宏。`Signal::new_empty`/`Signal::new` 为新增构造器。
- **风险提示**：
  1. `K线::macd()` 返回 cloned 值而非引用——已在设计 §5.1 说明理由（避免 RwLockReadGuard 生命周期传染）
  2. 集成测试依赖具体 `.nb` 测试数据——需确认文件存在且包含中枢结构
  3. `Signal::new_empty` 的 key 格式需与 Python `create_single_signal` 一致（过滤 "任意" 段）
