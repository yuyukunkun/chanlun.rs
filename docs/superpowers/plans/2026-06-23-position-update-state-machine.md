# 子项目 4 Position.update 状态机迁移到 Rust 实现计划

> 目标：将 Position.update 状态机（~135 行 Python）从 Python 子类迁移到 Rust 核心。

**设计文档：** `docs/superpowers/specs/2026-06-23-position-update-state-machine-design.md`

---

## 任务 0：扩展 Rust 核心 Position

**文件：** `chanlun/src/signal/position.rs`

- [x] 新增类型：`操作记录`、`持仓记录`、`开平配对`、`最近事件`
- [x] Position 结构体新增 7 个状态字段（pos, pos_changed, operates, holds, last_event, last_lo_dt, last_so_dt, end_dt）
- [x] `新建()` 构造函数适配（状态字段初始化为默认值）
- [x] 实现 `push_operate()` 内部辅助方法
- [x] 实现 `update(&mut self, dt, price, bid, signals) -> Result<(), 缺键错误>` — 核心状态机
- [x] 实现 `pairs() -> Vec<开平配对>` — 开平配对计算
- [x] 实现 `dump_config()` / `load_config()` — 序列化辅助
- [x] 内部辅助函数：`同一交易日`、`间隔检查`、`允许操作`
- [x] Rust 单元测试（28 用例）

## 任务 1：更新 PyO3 绑定

**文件：** `chanlun-py/src/signal_py.rs`

- [x] 新增 helper：`核心op转pyop()`、`时间戳转datetime()`
- [x] 新增状态 getter：`pos`, `pos_changed`, `operates`, `holds`, `pairs`
- [x] 实现 `update(PyDict)` — 提取 dt/close/bid + 转换 信号字典 + 调用核心
- [x] dt 类型兼容：支持 datetime / int / float
- [x] dump(with_data) — 支持附带 pairs/holds
- [x] load() 静态方法
- [x] 新增 `取事件列表` 辅助函数
- [x] 更新 `__repr__` 包含 pos

## 任务 2：更新 Python 子类

**文件：** `chanlun-py/chanlun/chan_external.py`

- [x] `__init__` 简化为 `pass`（状态由 Rust 初始化）
- [x] 删除 `update()`（Rust 提供）
- [x] 删除 `pairs` property（Rust 提供）
- [x] `dump()` 委托给 Rust `super().dump(with_data=...)`
- [x] `load()` 使用 `cls(...)` 构造（保持子类类型）
- [x] 保留 `get_signals_config()`

## 任务 3：测试

**文件：**
- `chanlun/src/signal/position.rs` — Rust 单元测试（28 用例）
- `chanlun-py/tests/test_position_update.py` — Python 集成测试（24 用例）
- `chanlun-py/tests/test_signal_primitives.py` — 已有测试更新（4 position 用例）

- [x] 基础开多/开空/平多/平空
- [x] 间隔限制
- [x] 止损（多头/空头）
- [x] 超时
- [x] 时间倒退容错
- [x] 空事件列表容错
- [x] 无匹配事件容错
- [x] 缺键错误
- [x] T0 模式
- [x] pairs 盈亏计算（多头/空头）
- [x] pairs 持仓天数
- [x] dump/load with/without data
- [x] dt 类型兼容（datetime / int / float）

## 任务 4：文档

- [x] 创建设计文档 `docs/superpowers/specs/2026-06-23-position-update-state-machine-design.md`
- [x] 创建实现计划 `docs/superpowers/plans/2026-06-23-position-update-state-machine.md`
- [x] 更新 `CLAUDE.md` 子项目表

## 自检结论

- **规格覆盖**：设计 §3 新增类型 → 任务 0；§4 update 算法 → 任务 0；§5 文件结构 → 任务 0-3
- **类型一致**：`update()` 参数使用已有 `信号字典` 类型；`Operate` 枚举已有 Rust 版
- **向后兼容**：Python 子类保留；update/pairs/operates/holds API 不变；dt 支持三种输入格式
- **测试覆盖**：Rust 28 用例 + Python 24 用例 + 已有 4 用例更新
