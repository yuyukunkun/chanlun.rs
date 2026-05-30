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

# 开发模式（直接安装到当前 venv）
maturin develop

# 或构建 wheel
maturin build --release
pip install target/wheels/chanlun-*.whl
```

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

## 许可

MIT
