# `信号计算器` Rust 迁移 — 设计文档

- 日期：2026-06-23
- 所属：全 Rust 信号计算迁移 — 子项目 1-4 完成后的延续
- 前置：子项目 1-4 全部完成（注册表 + 信号函数 + 引擎 + Position 状态机）

## 1. 背景

「全 Rust 信号计算迁移」4 个子项目完成后，信号框架的 Rust 核心已就位：

- `#[signal]` 注册表 → 编译时信号函数发现
- `SignalEngine` → 按名查找 + 批量执行
- `Position.update()` → 状态机

但 **`信号计算器`（Python 信号编排器）仍然在使用 Python 动态导入**（`import_by_name`）来发现和执行信号函数。它与 Rust `SignalEngine` **并行存在**，形成两条独立的执行路径。

## 2. 设计目标

1. **统一信号执行路径**：Rust `SignalEngine` 作为主路径，Python 动态导入作为回退
2. **保持向后兼容**：`strategies.py` 无需改动内部逻辑
3. **渐进式迁移**：新增 Rust 信号函数自动通过引擎执行，无需修改编排器代码
4. **最终目标**：所有信号函数移植到 Rust 后，Python 回退路径可移除

## 3. 关键设计决策

| 决策 | 选择 | 理由 |
|------|------|------|
| 编排器架构 | 新建 `SignalOrchestrator` 类，不修改 `信号计算器` | 零风险切换；旧类保留用于对比验证 |
| 信号函数分类 | 构造时按 `list_signals()` 将配置分为 Rust/Python 两组 | 避免每次 `更新()` 都查注册表 |
| Rust 路径 | 使用 `SignalEngine.更新_完整()`（批量） | 性能优于逐个 `call_signal()` |
| Python 路径 | 保留 `import_by_name` + `_解析信号函数` | 非侵入式；已有信号函数无需任何修改 |
| OHLCV 行情 | Rust 引擎直接返回基础周期行情 | 消除 Python 侧的独立行情提取步骤 |
| freq 验证 | 在编排器 setter 中验证 | 与旧 `信号计算器` 行为一致 |

## 4. 架构图

```
┌─────────────────────────────────────────────────┐
│              SignalOrchestrator                  │
│                                                  │
│  信号配置 ──→ 分类（list_signals() 查表）          │
│              │                                   │
│     ┌────────┴────────┐                         │
│     │ Rust 已注册      │ Python 未注册             │
│     │ SignalEngine    │ import_by_name           │
│     │ .更新_完整()     │ ._执行Python信号函数()     │
│     └────────┬────────┘                         │
│              │                                   │
│         合并结果 → self.信号 + self.行情            │
│                                                  │
│         self.信号字典 → Position.update()          │
└─────────────────────────────────────────────────┘
```

## 5. SignalEngine 增强

### 5.1 新增 `更新_完整()` 方法

```rust
pub struct 完整更新结果 {
    pub signals: HashMap<String, String>,
    pub market: Option<MarketData>,
}

pub struct MarketData {
    pub symbol: String,
    pub dt: i64,       // Unix 秒
    pub id: i64,
    pub open: f64, pub high: f64, pub low: f64,
    pub close: f64, pub vol: f64,
}
```

`更新_完整(&self, analyzer: &立体分析器) -> 完整更新结果`:
1. 调用 `self.更新(analyzer)` 获取信号
2. 从 `analyzer.周期组[0]` 获取基础周期观察者
3. 提取最后一根普K的 OHLCV 数据
4. 返回组合结果

## 6. SignalOrchestrator 设计

### 6.1 类签名

```python
class SignalOrchestrator:
    def __init__(
        self,
        分析器: 立体分析器,
        信号配置: Optional[List[Dict]] = None,
        信号模块: str = "chanlun.signals",
    ):
```

### 6.2 方法

| 方法 | 来源 | 说明 |
|------|------|------|
| `更新()` | 新写 | 先 Rust 批量，再 Python 逐个 |
| `信号配置` (property) | 移植 | setter 中添加 Rust/Python 分类 |
| `信号字典` (property) | 移植 | `{**self.信号, **self.行情}` |
| `获取周期观察者(freq)` | 移植 | 委托给 `_观察者字典` |
| `从信号列表提取配置(信号序列)` | 移植 | 委托给 `SignalsParser` |
| `_去重配置(configs)` | 移植 | 与旧版一致 |
| `_预加载Python信号函数()` | 移植 | 缓存 Python 函数引用 |
| `_解析信号函数(name)` | 移植 | `import_by_name` 逻辑 |
| `_执行Python信号函数(config)` | 移植 | Python 函数调用 |
| `_提取行情()` | 移植 | 仅 Python-only 回退路径使用 |

## 7. 迁移路径

### 阶段 A：增强 SyncSignalEngine（1-2 commits）
- `更新_完整()` + PyO3 绑定
- 不改变现有行为

### 阶段 B：引入 SignalOrchestrator（2-3 commits）
- 新文件 `signal_orchestrator.py`
- `strategies.py` 切换到新类（别名导入）
- 修复 `main.py` 损坏的调用点

### 阶段 C：废弃 Python 并行路径（未来）
- 所有信号函数移植到 Rust 后
- 移除 `信号计算器`、`SignalsParser`、`import_by_name`
- 移除 `signals/` 目录中的 Python 信号函数

## 8. 向后兼容

| 组件 | 兼容策略 |
|------|---------|
| `strategies.py` | 别名导入 `SignalOrchestrator as _信号计算器`——零代码改动 |
| `main.py` | 修复损坏的调用点（原本就 broken） |
| `test_策略验证.py` | 零改动——`信号计算器` 类名不变 |
| Python 信号函数 | 零改动——`import_by_name` 路径不变 |
| `Position.update()` | 零改动——Rust 状态机不变 |

## 9. 风险

| 风险 | 缓解 |
|------|------|
| `_提取行情()` 中 `k.时间戳` 是 i64（Rust K线），不是 Python datetime | 已由 Rust `PositionPy::时间戳转datetime` 处理 |
| `SignalEngine.更新_完整()` 的基础周期可能与 `_基础周期` 不一致 | 统一从 `分析器.周期组[0]` 获取 |
| Python 信号函数的 `**kwargs` 中 `freq` 是字符串（来自 SignalsParser） | `_执行Python信号函数` 中 `int(freq)` 转换 |
| `list_signals()` 返回的是 Rust 注册名，不含模块路径 | 按短名匹配（`youwukuncheng_中枢第三买卖点_V230602` 不含 `chanlun.signals.` 前缀） |
