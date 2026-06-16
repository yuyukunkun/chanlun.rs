# 信号计算器 Rust 迁移 — 设计决策 + 实现计划

> **面向 AI 代理的工作者：** 必需子技能：使用 superpowers:subagent-driven-development（推荐）或 superpowers:executing-plans 逐任务实现此计划。步骤使用复选框（`- [ ]`）语法来跟踪进度。

**目标：** 将 `信号计算器`（Python 信号编排器）替换为混合架构：Rust `SignalEngine` 为主，Python fallback 为辅，逐步完成最终迁移。

**架构：** 增强 Rust `SignalEngine` 使其返回完整的 `信号字典`（信号 + OHLCV 行情）；创建 `SignalOrchestrator` 支持 Rust 注册表优先 + Python `import_by_name` 回退；`SignalsParser` 暂留 Python。

**技术栈：** Rust edition 2024、PyO3 0.28、`serde_json::Value`、`parking_lot::RwLock`、`inventory`。

**设计文档：** `docs/superpowers/specs/2026-06-23-signal-calculator-migration-design.md`

---

## 0. 决策分析

### 现状

| 组件 | 语言 | 职责 |
|------|------|------|
| `SignalEngine` | ✅ Rust | 按名查找已注册信号函数 → 执行 → 合并结果 |
| `信号计算器` | Python | 同上 + OHLCV 行情提取 + `SignalsParser` 集成 |
| `SignalsParser` | Python | 解析信号函数文档字符串 → 生成配置字典 |
| `get_signals_config` | Python | 将信号字符串列表 → 配置字典列表（用 `SignalsParser`） |

两个计算引擎**并行存在**，完全独立。`strategies.py` 使用 Python `信号计算器`。Rust `SignalEngine` 没有被任何生产代码使用。

### 关键差异

| 能力 | Python `信号计算器` | Rust `SignalEngine` |
|------|---------------------|---------------------|
| 信号函数解析 | 运行时 `import_by_name()` | 编译时 `#[signal]` + `inventory` |
| OHLCV 行情 | 提取到 `self.行情` | ❌ 不处理 |
| 观察者访问 | 预提取 `{freq: Observer}` 字典 | 每次调用时通过 `&立体分析器` 查找 |
| 错误处理 | 每个信号函数的 `except Exception` | `tracing::warn!`，继续 |
| freq 验证 | 检查是否在分析器周期组中 | ❌ 不验证 |
| 信号字符串→配置 | `从信号列表提取配置()` | ❌ 不存在（Python `SignalsParser` 处理） |

### 建议：混合迁移（3 阶段）

**阶段 A：增强 Rust SignalEngine。** 添加 OHLCV 行情提取 + freq 验证 + Python `call_signal` 集成。

**阶段 B：创建混合编排器 `SignalOrchestrator`。** 替代 Python `信号计算器`；Rust 注册表优先，Python `import_by_name` 回退。

**阶段 C：废弃 Python 并行路径。** 所有信号函数移植到 Rust 后，移除 `import_by_name` 回退和 `SignalsParser`。

| 阶段 | 交付物 | 向后兼容 |
|------|--------|----------|
| A | `SignalEngine::更新_完整()` → `{signals, market_data}` | ✅ 不影响现有路径 |
| B | `SignalOrchestrator`（Rust 优先 + Python fallback） | ✅ `strategies.py` 切换到新类 |
| C | 移除 Python `信号计算器` 和 `SignalsParser` | ⚠️ 需所有信号函数先移植到 Rust |

---

## 文件结构

```
chanlun/src/signal/engine.rs           ← 增强：更新_完整() 返回 {signals, market}
chanlun-py/src/signal_engine_py.rs      ← 增强：SignalEnginePy 暴露 更新_完整()
chanlun-py/chanlun/signal_orchestrator.py  ← 新建：混合编排器
chanlun-py/chanlun/chan_external.py     ← 废弃：信号计算器（最终移除）
strategies.py                           ← 切换：使用 SignalOrchestrator
main.py                                  ← 修复：损坏的 信号计算器 调用点
chanlun-py/tests/test_signal_orchestrator.py  ← 新建：编排器测试
```

---

## 阶段 A：增强 Rust SignalEngine（信号 + 行情）

### 任务 A1：SignalEngine 增加 `更新_完整()` 方法

**文件：** `chanlun/src/signal/engine.rs`

- [ ] **步骤 1：添加返回类型**

在 `SignalEngine` 的 `更新_含分数()` 之后添加新结构体：

```rust
/// 完整更新结果：信号字典 + 基础周期行情数据。
#[derive(Debug, Clone)]
pub struct 完整更新结果 {
    /// 信号 key → value 映射
    pub signals: HashMap<String, String>,
    /// 基础周期最后一根 K 线的 OHLCV 数据
    pub market: Option<MarketData>,
}

#[derive(Debug, Clone)]
pub struct MarketData {
    pub symbol: String,
    pub dt: i64,       // Unix 秒
    pub id: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub vol: f64,
}
```

- [ ] **步骤 2：实现 `更新_完整()`**

```rust
/// 运行信号计算并附带基础周期行情。
/// `base_freq` 为分析器的第一个周期（最小周期）。
pub fn 更新_完整(&self, analyzer: &立体分析器) -> 完整更新结果 {
    let signals = self.更新(analyzer);
    let base_freq = analyzer.周期组.first().copied().unwrap_or(0);
    let market = analyzer._单体分析器.get(&base_freq).and_then(|obs| {
        let obs = obs.read();
        obs.普通K线序列.last().map(|k| {
            MarketData {
                symbol: obs.符号.clone(),
                dt: k.时间戳,
                id: k.序号.load(std::sync::atomic::Ordering::Relaxed),
                open: k.开盘价,
                high: k.最高价,
                low: k.最低价,
                close: k.收盘价,
                vol: k.成交量,
            }
        })
    });
    完整更新结果 { signals, market }
}
```

- [ ] **步骤 3：构建验证**

```bash
cd chanlun && cargo build
```
预期：编译通过。

- [ ] **步骤 4：Commit**

```bash
git add chanlun/src/signal/engine.rs
git commit -m "feat(signal): SignalEngine.更新_完整() — 信号 + 基础周期行情

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### 任务 A2：PyO3 绑定增强

**文件：** `chanlun-py/src/signal_engine_py.rs`

- [ ] **步骤 1：暴露 `更新_完整()`**

在 `SignalEnginePy` 的 `#[pymethods]` 块中添加：

```rust
/// 更新信号并返回完整结果（信号 + 行情）。
/// 返回 dict: {"signals": {...}, "market": {...}}  
fn 更新_完整<'py>(&self, py: Python<'py>, analyzer: &立体分析器Py) -> PyResult<Bound<'py, PyDict>> {
    let result = self.inner.更新_完整(&analyzer.inner);
    let d = PyDict::new(py);
    // signals
    let signals_dict = PyDict::new(py);
    for (k, v) in &result.signals {
        signals_dict.set_item(k, v)?;
    }
    d.set_item("signals", signals_dict)?;
    // market
    if let Some(m) = &result.market {
        let md = PyDict::new(py);
        md.set_item("symbol", &m.symbol)?;
        // Convert i64 to Python datetime
        let dt = 时间戳转datetime(py, m.dt)?;
        md.set_item("dt", dt)?;
        md.set_item("id", m.id)?;
        md.set_item("open", m.open)?;
        md.set_item("high", m.high)?;
        md.set_item("low", m.low)?;
        md.set_item("close", m.close)?;
        md.set_item("vol", m.vol)?;
        d.set_item("market", md)?;
    } else {
        d.set_item("market", py.None())?;
    }
    Ok(d)
}
```

> 注意：`时间戳转datetime` 已在 `signal_py.rs` 中定义。需要将其改为 `pub(crate)` 可见性，或在 `signal_engine_py.rs` 中重复定义。

- [ ] **步骤 2：将 `时间戳转datetime` 改为 `pub(crate)`**

在 `signal_py.rs` 中：
```rust
// 将 fn 改为 pub(crate)
pub(crate) fn 时间戳转datetime(py: Python<'_>, ts: i64) -> PyResult<Py<PyAny>> {
```

- [ ] **步骤 3：添加 `freq 验证` 辅助函数**

在 `signal_engine_py.rs` 的 `SignalEnginePy::new()` 中添加 freq 验证（匹配 Python `信号计算器` setter 的行为）：

```rust
// 在 new() 中，转换配置后：
// 验证所有 freq 已由调用方提供（不在构造时验证——没有分析器引用）
// 频率验证推迟到 更新() 调用时（与 Rust 核心行为一致）
```

不改变构造函数——保持最小侵入。频率验证由调用方负责（`SignalOrchestrator`）。

- [ ] **步骤 4：构建验证**

```bash
cd chanlun-py && cargo build
```
预期：编译通过。

- [ ] **步骤 5：Commit**

```bash
git add chanlun-py/src/signal_engine_py.rs chanlun-py/src/signal_py.rs
git commit -m "feat(signal-py): SignalEnginePy.更新_完整() + 时间戳转datetime 公开

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## 阶段 B：混合编排器 SignalOrchestrator

### 任务 B1：创建 `signal_orchestrator.py`

**文件：** 创建 `chanlun-py/chanlun/signal_orchestrator.py`

这是核心新文件。编排器：
1. 构造时接受 `立体分析器` + 信号配置 + 信号模块
2. 对每个配置，先尝试 Rust `call_signal()` 查找（通过 `list_signals()`）
3. 如果信号名在 Rust 注册表中：使用 `SignalEngine` 批量执行
4. 如果不在：使用 Python `import_by_name` 回退
5. 合并所有结果，附加 OHLCV 行情

- [ ] **步骤 1：创建文件框架**

```python
"""信号编排器 — Rust 优先 + Python 回退的混合信号计算。

替代 chan_external.信号计算器，逐步迁移到全 Rust 路径。

使用方式::

    分析器 = 立体分析器("btcusd", [300, 900, 3600], 配置)
    编排器 = SignalOrchestrator(分析器, 信号配置=[...], 信号模块="chanlun.signals")

    for k in k线列表:
        分析器.投喂K线(k)
        编排器.更新()
        print(编排器.信号字典)
"""

import sys
from collections import OrderedDict
from typing import Any, Callable, Dict, List, Optional

from loguru import logger

from chanlun.chan import 观察者, 立体分析器
from chanlun._chanlun import (
    SignalEngine as _RustSignalEngine,
    call_signal as _rust_call_signal,
    list_signals as _rust_list_signals,
)


class SignalOrchestrator:
    """混合信号编排器：Rust 注册表优先，Python import_by_name 回退。"""

    def __init__(
        self,
        分析器: 立体分析器,
        信号配置: Optional[List[Dict]] = None,
        信号模块: str = "chanlun.signals",
    ):
        self._分析器 = 分析器
        self._观察者字典 = {p: 分析器._单体分析器[p] for p in 分析器.周期组}
        self._基础周期 = 分析器.周期组[0]
        self._信号模块 = 信号模块

        # 初始化 Rust 引擎（用于已注册的 Rust 信号）
        self._rust_engine = _RustSignalEngine(信号配置=信号配置 or [])
        self._rust_engine.自动挂载指标(分析器)

        # 分类配置：Rust 注册 vs Python 回退
        self._rust_configs: List[Dict] = []
        self._python_configs: List[Dict] = []
        self._python_func_cache: Dict[str, Callable] = {}

        # 结果容器
        self.信号: Dict[str, str] = {}
        self.行情: Dict[str, Any] = {}

        # 初始设置
        self.信号配置 = 信号配置 or []

    # ... 其余方法见下面步骤
```

- [ ] **步骤 2：实现配置分类**

```python
    @property
    def 信号配置(self) -> List[Dict]:
        return self._信号配置

    @信号配置.setter
    def 信号配置(self, value: List[Dict]):
        可用周期 = set(self._分析器.周期组)
        rust_names = set(_rust_list_signals())

        self._rust_configs = []
        self._python_configs = []

        for c in self._去重配置(value):
            freq = c.get("freq")
            if freq is not None:
                周期秒 = int(freq)
                if 周期秒 not in 可用周期:
                    raise ValueError(
                        f"信号配置 freq={freq}({周期秒}s) 不在分析器周期组 {sorted(可用周期)} 中"
                    )

            name = c.get("name", "")
            if name in rust_names:
                self._rust_configs.append(c)
            else:
                self._python_configs.append(c)

        self._信号配置 = value
        self._预加载Python信号函数()
```

- [ ] **步骤 3：实现更新循环**

```python
    def 更新(self):
        """执行所有信号计算。Rust 优先（批量），Python 回退（逐个）。"""
        self.信号.clear()
        self.行情.clear()

        # 1. Rust 批量执行
        if self._rust_configs:
            result = self._rust_engine.更新_完整(self._分析器)
            if result.get("signals"):
                for k, v in result["signals"].items():
                    if v != "任意_任意_任意_0":
                        self.信号[k] = v
            if result.get("market"):
                self.行情.update(result["market"])

        # 2. Python 回退（逐个执行）
        for config in self._python_configs:
            try:
                result = self._执行Python信号函数(config)
                if result:
                    for k, v in result.items():
                        if v != "任意_任意_任意_0":
                            self.信号[k] = v
            except Exception:
                logger.exception(f"Python 信号函数执行失败: {config.get('name')}")

        # 3. 补充基础周期行情（如果 Rust 引擎未提供）
        if not self.行情:
            self._提取行情()
```

- [ ] **步骤 4：实现 Python 信号函数执行（移植自 chan_external.py）**

```python
    def _执行Python信号函数(self, config: Dict) -> Optional[OrderedDict]:
        """执行单个 Python 信号函数（移植自 信号计算器._执行信号函数）。"""
        import traceback
        param = dict(config)
        sig_name = param.pop("name")
        sig_func = self._python_func_cache.get(sig_name) or self._解析信号函数(sig_name)
        if sig_func is None:
            logger.warning(f"信号函数未找到: {sig_name}")
            return None

        freq = param.pop("freq", None)
        if freq is not None:
            周期秒 = int(freq)
            obs = self._观察者字典.get(周期秒)
            if obs is None:
                logger.warning(f"未找到周期 {freq} 的观察者")
                return None
            try:
                return sig_func(obs, **param)
            except Exception:
                logger.exception(f"信号函数执行异常: {sig_name}")
                return None
        else:
            try:
                return sig_func(self, **param)
            except Exception:
                logger.exception(f"信号函数执行异常: {sig_name}")
                return None
```

- [ ] **步骤 5：移植辅助方法**

```python
    def _去重配置(self, configs: List[Dict]) -> List[Dict]:
        seen = set()
        unique = []
        for c in configs:
            key = (c.get("name"), frozenset(
                (k, str(v)) for k, v in c.items() if k != "name"
            ))
            if key not in seen:
                seen.add(key)
                unique.append(c)
        return unique

    def _预加载Python信号函数(self):
        for config in self._python_configs:
            name = config.get("name", "")
            if name and name not in self._python_func_cache:
                self._python_func_cache[name] = None  # placeholder
        for name in list(self._python_func_cache.keys()):
            try:
                self._python_func_cache[name] = self._解析信号函数(name)
            except Exception:
                logger.warning(f"预加载信号函数失败: {name}")

    @staticmethod
    def _解析信号函数(name: str) -> Optional[Callable]:
        """动态导入信号函数（移植自 信号计算器._解析信号函数）。"""
        import os
        if "." not in name:
            return __import__(name)

        module_name, func_name = name.rsplit(".", 1)
        # 检查 __main__ 缓存
        main_mod = sys.modules.get("__main__")
        if main_mod is not None and hasattr(main_mod, func_name):
            return getattr(main_mod, func_name)

        module = __import__(module_name, fromlist=[func_name])
        return getattr(module, func_name)

    def _提取行情(self):
        """从基础周期观察者提取 OHLCV 行情（Python 回退路径）。"""
        obs = self._观察者字典.get(self._基础周期)
        if obs is None:
            return
        klines = obs.普通K线序列
        if not klines:
            return
        k = klines[-1]
        self.行情 = {
            "symbol": obs.符号,
            "dt": k.时间戳,  # 需要从 i64 转 datetime
            "id": k.序号,
            "open": k.开盘价,
            "high": k.最高价,
            "low": k.最低价,
            "close": k.收盘价,
            "vol": k.成交量,
        }

    @property
    def 信号字典(self) -> dict:
        """合并信号 + 行情（与 Position.update() 兼容）。"""
        return {**self.信号, **self.行情}

    def 获取周期观察者(self, freq: str) -> Optional[观察者]:
        """按频率获取观察者。"""
        return self._观察者字典.get(int(freq))

    def 从信号列表提取配置(self, 信号序列: List[str]):
        """从信号字符串列表解析配置（委托给 SignalsParser）。"""
        from chanlun.chan_external import get_signals_config
        from chanlun.chan_external import SignalsParser

        if not 信号序列:
            return
        sp = SignalsParser(signals_module=self._信号模块)
        conf = sp.parse(信号序列)
        self.信号配置 = conf
```

- [ ] **步骤 6：Commit**

```bash
git add chanlun-py/chanlun/signal_orchestrator.py
git commit -m "feat(signal): SignalOrchestrator — Rust 优先 + Python 回退混合编排器

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### 任务 B2：切换到 strategies.py

**文件：** `strategies.py`

- [ ] **步骤 1：更新导入**

将第 28 行的导入从：
```python
from chanlun.chan_external import 信号计算器 as _信号计算器, get_signals_config
```
改为：
```python
from chanlun.chan_external import get_signals_config
from chanlun.signal_orchestrator import SignalOrchestrator as _信号计算器
```

> 使用别名 `_信号计算器` 保持类名不变——策略内部代码零改动。

- [ ] **步骤 2：运行策略验证测试**

```bash
python test_策略验证.py
```
预期：所有 V1-V7 测试通过，无回归。

- [ ] **步骤 3：Commit**

```bash
git add strategies.py
git commit -m "refactor(strategies): 切换到 SignalOrchestrator 混合编排器

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### 任务 B3：修复 main.py 中损坏的调用点

**文件：** `main.py:2220`

- [ ] **步骤 1：修复构造函数调用**

当前损坏的代码：
```python
计算器 = cet.信号计算器(观察者字典, 基础周期=周期组[0], 信号模块="chanlun.signals")
计算器.从信号序列设置配置([...])  # 方法不存在
```

修复为：
```python
计算器 = cet.SignalOrchestrator(分析器, 信号模块="chanlun.signals")
计算器.从信号列表提取配置([...])
```

> 注意：此处 `分析器` 变量需要在该作用域内可用。需要先检查 main.py 上下文。

- [ ] **步骤 2：Commit**

```bash
git add main.py
git commit -m "fix(main): 修复损坏的 信号计算器 调用点 → SignalOrchestrator

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## 阶段 C：测试

### 任务 C1：编排器单元测试

**文件：** 创建 `chanlun-py/tests/test_signal_orchestrator.py`

- [ ] **步骤 1：编写框架测试**

```python
"""SignalOrchestrator 集成测试 — 混合 Rust + Python 信号执行。"""
import pytest
from datetime import datetime, timezone
from chanlun.signal_orchestrator import SignalOrchestrator


def test_构造_空配置():
    """空配置构造不崩溃。"""
    from chanlun import 立体分析器, 缠论配置
    analyzer = 立体分析器("test", [300, 900], 缠论配置())
    orch = SignalOrchestrator(analyzer)
    assert orch.信号字典 == {}
    assert orch._rust_configs == []
    assert orch._python_configs == []


def test_Rust信号已注册():
    """youwukuncheng 信号名在 Rust 注册表中（应分类到 rust_configs）。"""
    from chanlun import 立体分析器, 缠论配置
    analyzer = 立体分析器("test", [86400], 缠论配置())
    config = [{
        "name": "youwukuncheng_中枢第三买卖点_V230602",
        "freq": 86400,
        "max_overlap": 3,
        "本级完整性": "实",
        "同级完整性": "合",
    }]
    orch = SignalOrchestrator(analyzer, 信号配置=config)
    assert len(orch._rust_configs) == 1
    assert len(orch._python_configs) == 0


def test_Python信号回退():
    """未知信号名分类到 python_configs。"""
    from chanlun import 立体分析器, 缠论配置
    analyzer = 立体分析器("test", [300], 缠论配置())
    config = [{
        "name": "chanlun.signals.demo.tas_ma_base_V230313",
        "freq": 300,
        "ma_type": "SMA",
        "timeperiod": 5,
    }]
    orch = SignalOrchestrator(analyzer, 信号配置=config)
    assert len(orch._rust_configs) == 0
    assert len(orch._python_configs) == 1


def test_freq验证_不在周期组():
    """freq 不在分析器周期组中时抛出 ValueError。"""
    from chanlun import 立体分析器, 缠论配置
    analyzer = 立体分析器("test", [300], 缠论配置())
    with pytest.raises(ValueError, match="不在分析器周期组"):
        SignalOrchestrator(analyzer, 信号配置=[{
            "name": "some_signal",
            "freq": 99999,
        }])
```

- [ ] **步骤 2：运行测试**

```bash
python -m pytest chanlun-py/tests/test_signal_orchestrator.py -v
```
预期：全部通过。

- [ ] **步骤 3：Commit**

```bash
git add chanlun-py/tests/test_signal_orchestrator.py
git commit -m "test(signal): SignalOrchestrator 单元测试

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### 任务 C2：端到端回归测试

- [ ] **步骤 1：运行所有 tests**

```bash
cd chanlun && cargo test
cd chanlun-py && cargo test
python -m pytest chanlun-py/tests/ -v
python test_策略验证.py
```

- [ ] **步骤 2：验证零回归**

预期：所有已有测试通过。新编排器测试通过。

---

## 自检结论

- **规格覆盖**：阶段 A 覆盖 SignalEngine 增强 → 完整信号字典；阶段 B 覆盖混合编排器 → 替代 Python `信号计算器`；阶段 C 覆盖测试 → 零回归
- **类型一致**：`完整更新结果` 的 `MarketData` 字段与 Python `self.行情` 键名一致
- **风险提示**：
  1. `main.py:2220` 调用点需要确认其所在函数的上下文（分析器变量是否在作用域内）
  2. `SignalOrchestrator` 的 `_提取行情()` 中 `k.时间戳` 是 i64，需用 `datetime.fromtimestamp` 转换
  3. Python 信号函数需要 `chanlun.signals` 可导入——需确认安装包含 signals 子包
- **向后兼容**：`strategies.py` 使用别名导入——内部代码零改动
