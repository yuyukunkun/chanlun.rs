# chanlun — 缠论技术分析 Python 绑定

[![PyPI](https://img.shields.io/pypi/v/chanlun)](https://pypi.org/project/chanlun/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

基于 [chanlun](../chanlun/) Rust 核心库的 PyO3 高性能 Python 绑定，API 参考 `chan.py` 设计，高度兼容。

## 安装

```bash
pip install chanlun
```

## 快速开始

```python
import chanlun

# 创建配置（全部默认值）
config = chanlun.缠论配置()

# 读取 K 线数据文件（文件名需遵循 `符号-周期-起始时间戳-结束时间戳.nb` 格式，如 `btcusd-300-1631772074-1632222374.nb`）
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

## 从源码构建

前置依赖: [Rust](https://www.rust-lang.org) + [maturin](https://www.maturin.rs)

```bash
pip install maturin

# 推荐：一键清理缓存 + 构建 + 安装
./clean_install.sh

# 或手动：
# 开发模式（直接安装到当前 venv）
maturin develop

# 或构建 wheel
maturin build --release
pip install target/wheels/chanlun-*.whl
```

> **注意**：若修改了 `chan.py`，安装前需清除 `__pycache__`，否则旧 `.pyc` 会被打包进 wheel 导致修改不生效：
> ```bash
> find . -type d -name __pycache__ -exec rm -rf {} + 2>/dev/null
> find . -type f -name "*.pyc" -delete 2>/dev/null
> ```

也可使用项目内的 `build.sh`:

```bash
./build.sh develop   # 开发安装
./build.sh wheel     # 构建 wheel
```

## 导出类

| 类别 | 类名 | 说明 |
|------|------|------|
| 枚举 | `买卖点类型`, `相对方向`, `分型结构` | 缠论基础枚举 |
| 数据 | `缺口`, `K线`, `缠论K线` | K 线数据结构 |
| 结构 | `分型`, `虚线`, `线段特征`, `特征分型` | 分析层级结构 |
| 指标 | `平滑异同移动平均线`, `相对强弱指数`, `随机指标` | MACD/RSI/KDJ |
| 算法 | `笔`, `线段`, `中枢`, `背驰分析` | 识别算法 |
| 业务 | `缠论配置`, `基础买卖点`, `买卖点`, `观察者`, `K线合成器`, `立体分析器` | 分析框架 |

## 兼容性

- Python 3.9+
- 类名 / 方法名 / 字段名与 `chan.py` 保持一致
- 支持 `.nb` 二进制文件格式（大端字节序）

## 性能配置

### 缓存模式

Python 对象缓存有两种模式，通过环境变量 `CHANLUN_CACHE_MODE` 或函数调用切换：

```python
from chanlun._chanlun import set_cache_mode, get_cache_mode

# 默认：thread_local，每线程独立缓存，零锁，多线程场景最佳
print(get_cache_mode())  # "thread_local"

# 全局：dashmap 分片哈希表，跨线程 Python `is` 身份一致
set_cache_mode("global")  # 必须在创建任何观察者之前调用
```

```bash
# 环境变量方式
CHANLUN_CACHE_MODE=global python main.py   # 全局缓存
python main.py                              # 默认：线程局部缓存
```

| 模式 | 性能 | Python `is` 跨线程 | 适用场景 |
|------|------|---------------------|----------|
| `thread_local`（默认） | 零锁，最快 | 否 | 批量回测、多线程独立分析 |
| `global` | dashmap 分片锁 | 是 | 测试验证、跨线程对象共享 |

### 日志模式

日志输出有三种模式，通过环境变量 `CHANLUN_LOG_MODE` 或函数调用切换：

```python
from chanlun._chanlun import set_log_mode, set_log_level, get_log_mode

# 默认：off，不输出，零开销
print(get_log_mode())  # "off"

# 简单模式：直接 eprintln/println
set_log_mode("simple")
set_log_level("debug")  # 必需：设置日志级别启用输出

# Tracing 模式：带时间戳和文件位置格式化输出
set_log_mode("tracing")
set_log_level("debug")
```

```bash
# 环境变量方式
CHANLUN_LOG_MODE=simple python main.py     # 简单输出
CHANLUN_LOG_MODE=tracing python main.py    # 格式化输出
python main.py                              # 默认：静默
```

| 模式 | 输出方式 | 性能 | 格式 |
|------|---------|------|------|
| `off`（默认） | 无 | 零开销 | — |
| `simple` | `eprintln!` / `println!` | 极轻 | 纯文本 |
| `tracing` | tracing-subscriber | 稍重 | `2026-06-12 01:57:59.942 WARN file.rs:line` |

### 观察者直传（避免 Python list 转换）

背驰分析新增 `_OBS` 后缀方法，直接接受观察者引用，跳过 `list[K线]` ↔ `Vec<Arc<K线>>` 转换：

```python
# 旧方式：构建 Python 列表
result = 背驰分析.MACD背驰(进入段, 离开段, obs.普通K线序列, "总")

# 新方式：直接传观察者
result = 背驰分析.MACD背驰_OBS(进入段, 离开段, obs, "总")
```

## 许可

本项目主体采用 MIT 许可。包含以下第三方开源代码：czsc（Apache 2.0）、parse（MIT）、termcolor（MIT）。

详见 [NOTICE](../NOTICE) 和 [LICENSES/](../LICENSES/) 目录。
