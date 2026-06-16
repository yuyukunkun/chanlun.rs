# 信号原语层移植到 Rust 核心层 — 设计文档

- 日期：2026-06-22
- 范围：原语层（Operate / Signal / Factor / Event / Position 配置与匹配部分）
- 参考：czsc（`/home/moscow/czsc`）的 Rust workspace 分层

## 1. 目标与背景

当前信号匹配框架（`Signal` / `Factor` / `Event` / `Position` / `Operate`）以纯 Python 实现于 `chanlun-py/chanlun/chan_external.py`（已合并进根目录 `chan.py`）。这套框架抄录自 czsc（Apache 2.0）。

把这层**纯结构 + 匹配逻辑**移植到 Rust 核心层（`chanlun/src/signal/`），目的：

- **消除跨模块枚举/类型不一致问题**：信号原语只跟字符串和信号字典打交道，不持有 Rust 分析对象，天然规避「同值枚举跨模块 `is` 不相等」「动态导入找不到模块」这类坑。
- **统一原语来源**：Rust 端策略/回测可直接用同一套 `Signal`/`Event`，无需经过 Python。
- **性能**：匹配逻辑是热路径（每根 K 线、每个 Position 都跑），Rust 实现去掉 Python 解释开销。
- **为后续分层铺路**：原语层稳定后，未来可按 czsc 的路线增量推进注册表、信号串解析、交易引擎。

## 2. 范围

### 纳入（Rust + PyO3）

- `Operate` 枚举
- `Signal`：`key()` / `value()` / `is_match()`
- `Factor`：`is_match()` / `unique_signals()` / `dump()` / `load()`
- `Event`：`is_match()` / `unique_signals()` / `dump()` / `load()`
- `Position` 基类：配置字段 + 校验 + `unique_signals` + `__repr__` + config 部分的 `dump`/`load`

### 不纳入（保持 Python）

- `Position.update()` 状态机（持仓推进、止损、超时、`pairs`、操作决策）
- `信号计算器`（信号计算引擎、配置管理、`_自动挂载指标`）
- `SignalsParser`（docstring 解析）
- `import_by_name`（动态导入）
- 全部信号函数（`chanlun.signals.*`）

## 3. czsc 参考映射

czsc 把信号体系拆成分层 crate。本次只对应其最底层「信号原语」：

| czsc | 本次对应 |
|---|---|
| `czsc-core/objects/{signal,event,position,operate}.rs` | `chanlun/src/signal/{signal,factor,event,position,operate}.rs` |
| `czsc-core` 内 `#[cfg(feature="python")]` 内联 PyO3 包装 | `chanlun-py/src/signal_py.rs`（本项目沿用独立绑定 crate 的既有约定，不内联） |

czsc 的 `inventory` 编译期注册表、`#[signal]` 宏、`sig_parse`、`engine_v2` 交易引擎、`signals_dispatcher` **本次均不涉及**（属后续分层）。

## 4. 架构与模块布局

```
chanlun/src/signal/
├── mod.rs          # pub mod 声明 + re-export
├── operate.rs      # Operate 枚举（HL/HS/HO/LO/LE/SO/SE）
├── signal.rs       # Signal
├── factor.rs       # Factor
├── event.rs        # Event
└── position.rs     # Position 基类（config + matching，不含 update）
```

- `chanlun/src/lib.rs` 增加 `pub mod signal;`。
- PyO3 绑定新增 `chanlun-py/src/signal_py.rs`，在 `lib.rs` 注册顺序：types → **signal** → config → indicators → kline → structure → algorithm → business → equality。

### 依赖边界

信号原语层**零依赖** `business` / `algorithm` / `structure` 层。它只操作：

- `String`（信号各字段）
- 信号字典：匹配时通过 PyO3 接收 `&Bound<PyDict>`，逐键取值判类型

这是它能独立 `cargo test`、规避跨模块类型问题的根本原因。

## 5. 逐组件设计

### 5.1 Operate

```rust
#[pyclass(eq, eq_int)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Operate { HL, HS, HO, LO, LE, SO, SE }
```

- 值映射中文：`HL="持多" HS="持空" HO="持币" LO="开多" LE="平多" SO="开空" SE="平空"`，通过 `value()` 方法 / `__str__` 暴露。
- Python 端 `cet.Operate.LO` 直接用该枚举。

### 5.2 Signal

```rust
#[pyclass(module = "chanlun._chanlun")]
pub struct Signal {
    signal: String,
    score: i32,
    k1: String, k2: String, k3: String,
    v1: String, v2: String, v3: String,
}
```

> 注：仅 `Position` 需要 `#[pyclass(subclass)]`（Python 子类补 `update()`）。`Signal`/`Factor`/`Event` 不被子类化，用普通 `#[pyclass]`。

- 构造签名：`Signal(signal="", score=0, k1="任意", k2="任意", k3="任意", v1="任意", v2="任意", v3="任意")`。
  - `signal` 非空 → 按 `_` 拆 7 段（非 7 段 raise）；为空 → 由各字段拼。
  - `signal` 非字符串 → `TypeError`（对齐 Python `__post_init__`）。
  - `score` 越界 [0,100] → `ValueError`。
- `key` property：拼接 k1/k2/k3 中非「任意」的部分，`_` 连接。
- `value` property：`v1_v2_v3_score`。
- `is_match(s) -> bool`：见 §6。
- `__repr__` → `Signal('<signal>')`。

### 5.3 Factor

```rust
#[pyclass(module = "chanlun._chanlun")]
pub struct Factor {
    signals_all: Vec<Signal>,
    signals_any: Vec<Signal>,
    signals_not: Vec<Signal>,
    name: String,
}
```

- 构造：`Factor(signals_all, signals_any=[], signals_not=[], name="")`。`signals_all` 空 → `ValueError`。
- 构造时计算 `name`：见 §6 ③（确定性哈希）。
- `unique_signals` property：所有 signals 的 `signal` 字符串去重列表。
- `is_match`：`signals_not` 任一命中 → False；`signals_all` 必须全中；`signals_any` 非空时至少一中。
- `dump() -> dict`、`load(raw) classmethod`。

### 5.4 Event

```rust
#[pyclass(module = "chanlun._chanlun")]
pub struct Event {
    operate: Operate,
    factors: Vec<Factor>,
    signals_all: Vec<Signal>,
    signals_any: Vec<Signal>,
    signals_not: Vec<Signal>,
    name: String,
    sha256: String,
}
```

- 构造：`Event(operate, factors, signals_all=[], signals_any=[], signals_not=[], name="")`。`factors` 空 → `ValueError`。
- `name`：有传名 → `<name>#<hash>`，否则 `<operate中文值>#<hash>`；同时存 `sha256` 字段。
- `unique_signals`、`is_match(s) -> (bool, Option<String>)`（命中返回 `(True, factor_name)`）、`dump`、`load`。
- `get_signals_config` **不在 Rust 实现**（依赖 Python 的 `SignalsParser`），保留在调用方 Python。

### 5.5 Position 基类

```rust
#[pyclass(subclass, module = "chanlun._chanlun")]
pub struct Position {
    symbol: String,
    opens: Vec<Event>,
    exits: Vec<Event>,
    events: Vec<Event>,   // opens + exits
    name: String,
    interval: i64,
    timeout: i64,
    stop_loss: i64,
    T0: bool,
}
```

- 构造：`Position(symbol, opens, exits=[], interval=0, timeout=1000, stop_loss=1000, T0=False, name)`。
  - `name` 缺失 → `ValueError`（对齐 Python `assert name`）。
  - 每个 event 的 `operate` ∈ {LO,LE,SO,SE}，否则 raise。
- `unique_signals` property、`__repr__`、config 部分的 `dump`/`load`。
- **状态字段、`update()`、`pairs`、`with_data` 版 dump、`get_signals_config` 全部留 Python 子类。**

## 6. 三个兼容性关键点

### ① `Signal.is_match` 缺键时 raise `ValueError`

Python 现状：键不在信号字典 → `raise ValueError`。`strategies.py` 靠 `try: pos.update(...) except ValueError: pass` 兜底。

**决策**：Rust `is_match` 缺键 → `PyValueError`，**不静默返回 False**。这是行为契约。

### ② 信号字典值可能非字符串

`信号计算器.信号字典` 合并了 OHLCV 行情（值为 datetime/float）。Python 有 `isinstance(v, str)` 守卫：非 str → `logger.warning` + 返回 False。

**决策**：`is_match` 接收 `&Bound<PyDict>`。取到 key 对应值后：

- 值不存在 → `PyValueError`（关键点 ①）。
- 值非字符串 → 返回 False（对齐 Python 守卫）。**不打 warning**：匹配是每根 K 线的热路径，省去日志噪音；非 str 值来自 OHLCV 行情注入，是预期情况而非异常。
- 值是字符串 → 按 `_` 拆 4 段（`v1_v2_v3_score`）做匹配。

### ③ Factor/Event 的 sha256 命名

Python：`hashlib.sha256(str(dump_dict_minus_name).encode()).hexdigest().upper()[:4]`，依赖 Python `str(dict)` 的逐字节格式。

**决策**：用 Rust 确定性哈希——对 `signals_all`/`signals_any`/`signals_not`（Factor）或加上 factors 的 dump（Event）拼成稳定字符串后算 sha256，取大写前 4。

- 自洽：同输入恒等同名，`dump`/`load` 来回一致。
- **取舍（已知不兼容）**：生成的 hash 与 Python 旧版不同。依赖旧 `name` 的持久化仓位（保存的 .json）不再 roundtrip。本项目 Position 基本每次运行新建，可接受。

## 7. Drop-in 兼容策略

- `chan_external.py` 顶部：`from chanlun._chanlun import Signal, Factor, Event, Operate, Position as _PositionBase`，删除原 Python 类定义。
- `Position` 改为子类：

```python
class Position(_PositionBase):
    def __init__(self, symbol, opens, exits=[], interval=0, timeout=1000,
                 stop_loss=1000, T0=False, name=None):
        super().__init__(symbol, opens, exits, interval, timeout, stop_loss, T0, name)
        # Python 侧状态
        self.pos_changed = False
        self.operates = []
        self.holds = []
        self.pos = 0
        self.last_event = {...}
        self.last_lo_dt = None
        self.last_so_dt = None
        self.end_dt = None
    # update() / pairs / get_signals_config / with_data dump 保留
```

- `main.py` / `strategies.py` 中 `cet.Signal(...)`、`cet.Factor(...)`、`cet.Event(...)`、`cet.Position(...)`、`cet.Operate.LO` **无需改动**——构造签名与方法名一致。
- 根目录 `chan.py` 的对应类同样替换为 import Rust 版本（保持与包版本一致）。

## 8. 测试策略

1. **Rust 单测**（`cargo test`，`chanlun/src/signal/` 内 `#[cfg(test)]`）：
   - Signal：7 段解析、非 7 段 raise、score 越界 raise、key 过滤「任意」、value 拼接。
   - Factor/Event：`signals_all/any/not` 真值表全覆盖、空 signals_all/factors raise、确定性哈希同输入同名。
   - Position：name 缺失 raise、非法 operate raise、unique_signals 去重。
2. **跨语言一致性**（pytest，复用 `tests/helpers/api_consistency.py`）：
   - 构造相同 Signal/Factor/Event/Position，断言 `is_match`、`unique_signals`、`dump` 结构与移植前**逐字段一致**（name hash 除外）。
   - `is_match` 缺键 raise `ValueError`、值非 str 返回 False 两条边界。
3. **回归**：跑 `测试_信号识别` + sync 回测，确认信号匹配与开关仓行为不变。

## 9. 已知取舍

- **name hash 不兼容旧 Python 版本**（§6 ③）：依赖旧 name 的持久化仓位会对不上。可接受，因 Position 多为运行时新建。
- **`get_signals_config` 留 Python**：它依赖 `SignalsParser` 动态解析，本次不移植；Rust `Event`/`Position` 不提供该方法，由 Python 调用方补。
- **`Position.update` 留 Python**：状态机本次不移植，Position 被一分为二（Rust 基类配置 + Python 子类状态）。

## 10. 许可证

新增 Rust 文件沿用项目 MIT 头。信号原语逻辑摘录/参考自 czsc（Apache 2.0），在 `signal/mod.rs` 顶部加第三方代码声明（与根 `chan.py` 已有声明一致）。
