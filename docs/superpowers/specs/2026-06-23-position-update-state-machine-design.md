# 子项目 4：Position.update 状态机迁移到 Rust — 设计文档

- 日期：2026-06-23
- 所属：「全 Rust 信号计算迁移」第 4 个子项目（共 4 个）
- 前置：子项目 1-3 已完成（注册表、信号函数、计算引擎）

## 1. 背景与目标

子项目 1-3 交付了完整的信号计算链路：`#[signal]` 注册表 → 信号函数 → 计算引擎。Position.update 状态机是信号框架中最后一个仍留在 Python 中的核心逻辑（~135 行），将其迁移到 Rust 后，信号框架的纯 Rust 核心部分全部就位。

子项目 4 交付：
1. Position 状态字段（pos, operates, holds, last_event...）→ Rust 核心
2. update() 状态机算法 → Rust 核心（与 Python 版 1:1 对应）
3. pairs() 开平配对计算 → Rust 核心
4. PyO3 绑定：update(), 状态 getter, dump/load 带状态

## 2. 设计决策

| 决策 | 选择 | 理由 |
|------|------|------|
| 状态字段位置 | 直接加在 Position 结构体 | backtrader 在 GIL 下单线程访问；不需要额外锁 |
| update 签名（Rust） | `fn update(&mut self, dt: i64, price: f64, bid: i64, signals: &信号字典)` | 核心不依赖 Python 类型；OHLCV 由 PyO3 层提取 |
| PyDict → 信号字典 | 排除 OHLCV 键后调用 字典转核心 | 复用已有转换逻辑 |
| dt 类型兼容 | 支持 datetime/i64/f64 → 统一转为 i64 Unix 秒 | 兼容三种常见输入格式 |
| 时间戳 → Python datetime | `datetime.datetime.fromtimestamp(ts, UTC)` | 保持 operates/holds 元素类型与旧版一致 |
| Python 向后兼容 | 保留 Python 子类，__init__ 简化为空；update/pairs/dump 由 Rust 提供 | 不破坏 strategies.py 等下游代码 |
| Operate 枚举映射 | `核心Operate → OperatePy` 一对一转换函数 | 类型安全，无运行时开销 |

## 3. 新增 Rust 类型

```rust
pub struct 操作记录 { symbol, dt, bid, price, op: Operate, op_desc, pos }
pub struct 持仓记录 { dt, pos, price }
pub struct 开平配对 { 标的代码, 策略标记, 交易方向, 开仓时间, 平仓时间, 开仓价格, 平仓价格, 持仓K线数, 事件序列, 持仓天数, 盈亏比例 }
pub struct 最近事件 { dt, bid, price, op, op_desc }
```

Position 新增 7 个状态字段：`pos, pos_changed, operates, holds, last_event, last_lo_dt, last_so_dt, end_dt`

## 4. update() 状态机

与 Python `Position.update(s)` 1:1 对应：

1. 时间校验：`dt <= end_dt` → 日志警告，跳过
2. 事件匹配：遍历 events，调用 `event.is_match(signals)`
3. 开仓处理：LO → 间隔检查 → 开多/平空；SO → 间隔检查 → 开空/平多
4. 多头出场：LE 信号 / 止损（price/last_price - 1 < -stop_loss/10000）/ 超时（bid - last_bid > timeout）
5. 空头出场：SE 信号 / 止损（方向反转）/ 超时
6. 记录持仓快照 holds

## 5. 文件结构

```
chanlun/src/signal/position.rs     ← 操作记录/持仓记录/开平配对/最近事件 类型 + 状态字段 + update/pairs
chanlun-py/src/signal_py.rs        ← PositionPy: update(PyDict), 状态 getter, dump(with_data), load, 时间戳转datetime
chanlun-py/chanlun/chan_external.py ← Python Position 子类简化（__init__ → pass）
chanlun-py/tests/test_position_update.py ← 集成测试（24 用例）
```
