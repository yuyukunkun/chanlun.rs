# chanlun — 缠论技术分析 Python 绑定

[![PyPI](https://img.shields.io/pypi/v/chanlun)](https://pypi.org/project/chanlun/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

基于 [chanlun](./chanlun/) Rust 核心库的 PyO3 高性能 Python 绑定，API 参考 `chan.py` 设计，高度兼容。

## 安装

```bash
pip install chanlun
```

## 快速开始

```python
import chanlun

# 创建配置（全部默认值）
config = chanlun.缠论配置()

# 读取 K 线数据文件（文件名需遵循 `符号-周期-起始时间戳-结束时间戳.nb` 格式）
obs = chanlun.观察者.读取数据文件("path/to/btcusd-300-1631772074-1632222374.nb", config)

# 查看各层级序列
print(f"K线数量: {len(obs.普通K线序列)}")
print(f"笔数量: {len(obs.笔序列)}")
print(f"线段数量: {len(obs.线段序列)}")
print(f"中枢数量: {len(obs.中枢序列)}")

# 或使用立体分析器进行多周期分析
analyzer = chanlun.立体分析器("BTCUSD", [60, 60*5, 60*5*6], config)
# 逐根投喂 K 线...
```

## 信号计算 (Rust 核心)

信号框架（Signal/Factor/Event/Position/Operate）已全部迁移到 Rust 核心，通过 PyO3 暴露给 Python。

### 调用信号函数

```python
from chanlun._chanlun import 信号引擎, call_signal, list_signals, get_signal_template

# 准备数据
analyzer = chanlun.立体分析器("btcusd", [300, 900, 3600], chanlun.缠论配置())
for k in klines:
    analyzer.投喂K线(k)

# ── 方式 1: 信号引擎（批量） ──
engine = 信号引擎(信号配置=[
    {"name": "bar_zdt_V230331",   "freq": "300"},
    {"name": "macd_金叉_V260601", "freq": "300", "fast": "13", "slow": "31"},
])
engine.自动挂载指标(analyzer)
result = engine.更新(analyzer)            # → {key: value}
full   = engine.更新_完整(analyzer)        # → {"signals": {...}, "market": {...}}

# ── 方式 2: call_signal（单函数） ──
obs = analyzer._单体分析器[300]
signals = call_signal("macd_金叉_V260601", obs, {"freq": "5分钟", "di": "1"})
for s in signals:
    print(s.key, s.value)

# ── 方式 3: SignalOrchestrator（高级编排，含行情） ──
from chanlun.signal_orchestrator import SignalOrchestrator
orch = SignalOrchestrator(analyzer, 信号配置=[...])
orch.更新()
orch.信号字典  # → {信号..., "symbol": "btcusd", "close": 50050, ...}

# ── 方式 4: 注册表探索 ──
list_signals()                      # → ["bar_zdt_V230331", ...] (7个)
get_signal_template("bar_zdt_V230331")  # → "{freq}_D{di}_涨跌停V230331"
```

### 已注册信号函数（8个）

| 信号名 | 模板 | 说明 |
|--------|------|------|
| `bar_zdt_V230331` | `{freq}_D{di}_涨跌停V230331` | 涨跌停检测 |
| `macd_金叉_V260601` | `{freq}_D{di}#MACD#{fast}#{slow}#{signal}_MACD交叉V260601` | MACD 金叉/死叉 |
| `tas_macd_direct_V221106` | `{freq}_D{di}#MACD#{fast}#{slow}#{signal}_MACD方向V221106` | MACD DIF 方向 |
| `tas_ma_base_V230313` | `{freq}_D{di}#{ma_type}#{timeperiod}MO{max_overlap}_BS辅助V230313` | MA 均线多空 |
| `cxt_停顿分型_V230106` | `{freq}_D{di}停顿分型_BE辅助V230106` | 停顿分型检测 |
| `cxt_bi_end_V230222` | `{freq}_D1MO{max_overlap}_BE辅助V230222` | 笔结束辅助 |
| `youwukuncheng_中枢第三买卖点_V230602` | `{freq}_D1MO{max_overlap}_中枢第三买卖点V230602` | 中枢第三买卖点 |

### Python 信号函数混合调用

编排器支持 Rust + Python 信号混合执行：

```python
orch = SignalOrchestrator(analyzer, 信号配置=[
    {"name": "bar_zdt_V230331", "freq": 300},                    # → Rust 路径
    {"name": "chanlun.signals.demo.tas_ma_base_V230313", ...},   # → Python 回退
])
orch.更新()  # 自动分类，Rust 批量 + Python 逐个
```

## 编写信号函数

### Rust 信号函数（推荐）

```rust
// chanlun/src/signal/functions/my_signals.rs
use chanlun_signal_macros::signal;
use chanlun::business::observer::观察者;
use chanlun::signal::Signal;
use std::collections::HashMap;
use serde_json::Value;

#[signal(
    name = "my_signal_V000001",
    template = "{freq}_D{di}_模板V000001"
)]
pub fn my_signal_V000001(obs: &观察者, params: &HashMap<String, Value>) -> Vec<Signal> {
    obs.确保指标已计算();

    let di = params.get("di").and_then(|v| v.as_i64()).unwrap_or(1) as usize;
    let freq = params.get("freq").and_then(|v| v.as_str()).unwrap_or("日线");

    let k1 = freq.to_string();
    let k2 = format!("D{di}");
    let k3 = "模板V000001";

    let klines = &obs.普通K线序列;
    if klines.len() < di + 1 {
        return vec![Signal::new_empty(&k1, &k2, k3)];
    }

    let k线 = &klines[klines.len() - di];
    if k线.收盘价 > k线.开盘价 {
        vec![Signal::new(&k1, &k2, k3, "阳线", "任意", "任意", 0)]
    } else {
        vec![Signal::new_empty(&k1, &k2, k3)]
    }
}
```

然后在 `chanlun/src/signal/functions/mod.rs` 中添加 `pub mod my_signals;`，重新编译即可自动注册。

### 动态加载 .so 插件

信号函数可以编译为独立 `.so` 动态库，运行时加载。支持两种注册方式。

**方式 A：手动 C-ABI 注册**

```rust
// 独立 crate (cdylib)
fn my_plugin_signal(obs: &观察者, params: &HashMap<String, Value>) -> Vec<Signal> { ... }

unsafe extern "C" {
    fn chanlun_register_signal(name: *const c_char, template: *const c_char, func: SignalFn) -> i32;
    fn chanlun_unregister_signal(name: *const c_char) -> i32;
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn init_plugin() -> i32 {
    chanlun_register_signal(
        c"my_plugin_signal_V000001".as_ptr(),
        c"{freq}_D{di}_模板V000001".as_ptr(),
        my_plugin_signal,
    )
}
```

**方式 B：`#[signal]` 宏 + inventory 批量提交**

```rust
use chanlun_signal_macros::signal;

#[signal(name = "my_plugin_signal_V000001", template = "...", crate_path = "::chanlun")]
fn my_plugin_signal_V000001(obs: &观察者, params: &HashMap<String, Value>) -> Vec<Signal> { ... }

#[unsafe(no_mangle)]
pub unsafe extern "C" fn init_plugin() -> i32 {
    for desc in inventory::iter::<SignalDescriptor> {
        chanlun_register_signal(name_ptr, template_ptr, desc.func);
    }
    0
}
```

Python 加载：

```python
import ctypes, os, sys
sys.setdlopenflags(os.RTLD_LAZY | os.RTLD_GLOBAL)
import chanlun._chanlun  # 先加载宿主

plugin = ctypes.CDLL("./libmy_plugin.so")
plugin.init_plugin()

# 插件信号现在可通过 call_signal / 信号引擎 调用
from chanlun._chanlun import call_signal
call_signal("my_plugin_signal_V000001", obs, params)
```

完整示例见 [examples/plugin-demo/](./examples/plugin-demo/)。

## 从源码构建

前置依赖: [Rust](https://www.rust-lang.org) + [maturin](https://www.maturin.rs)

```bash
pip install maturin

# 开发模式（直接安装到当前 venv）
cd chanlun-py && maturin develop

# 或构建 wheel
cd chanlun-py && maturin build --release
pip install target/wheels/chanlun-*.whl
```

也可使用项目内的 `build.sh`:

```bash
cd chanlun-py
./build.sh develop   # 开发安装
./build.sh wheel     # 构建 wheel
./build.sh test      # 运行集成测试
```

## 导出类

| 类别 | 类名 | 说明 |
|------|------|------|
| 枚举 | `买卖点类型`, `相对方向`, `分型结构`, `Operate` | 缠论基础枚举 |
| 数据 | `缺口`, `K线`, `缠论K线` | K 线数据结构 |
| 结构 | `分型`, `虚线`, `线段特征`, `特征分型` | 分析层级结构 |
| 指标 | `平滑异同移动平均线`, `相对强弱指数`, `随机指标` | MACD/RSI/KDJ |
| 算法 | `笔`, `线段`, `中枢`, `背驰分析` | 识别算法 |
| 业务 | `缠论配置`, `观察者`, `K线合成器`, `立体分析器`, `买卖点` | 分析框架 |
| 信号 | `Signal`, `Factor`, `Event`, `Position`, `信号引擎` | 信号匹配+计算引擎 |
| 注册表 | `call_signal`, `list_signals`, `get_signal_template`, `register_signal`, `unregister_signal` | 信号发现+动态注册 |

## 兼容性

- Python 3.9+
- 类名 / 方法名 / 字段名与 `chan.py` 保持一致
- 支持 `.nb` 二进制文件格式（大端字节序）

## 许可

MIT
