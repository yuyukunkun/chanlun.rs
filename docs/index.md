# chanlun — 缠论技术分析库

基于 Rust + PyO3 的高性能缠论技术分析 Python 绑定，API 参考 `chan.py` 设计，高度兼容。

## 特性

- **高性能**：核心算法用 Rust 实现，通过 PyO3 暴露为 Python 原生模块
- **完全兼容**：类名、方法名、字段名与 `chan.py` 保持一致
- **流式计算**：逐 K 线增量更新，无需重复计算
- **多周期分析**：内置 K 线合成器和立体分析器，支持跨周期联动

## 安装

```bash
pip install chanlun
```

## 快速开始

```python
import chanlun

config = chanlun.缠论配置()
obs = chanlun.观察者.读取数据文件("BTCUSD-300.nb", config)

print(f"K线: {len(obs.普通K线序列)}")
print(f"笔:   {len(obs.笔序列)}")
print(f"线段: {len(obs.线段序列)}")
print(f"中枢: {len(obs.中枢序列)}")
```

```{toctree}
:hidden:
:caption: 文档导航

api
```

```{toctree}
:hidden:
:caption: 外部链接

GitHub <https://github.com/yuwukuncheng/chanlun.rs>
PyPI <https://pypi.org/project/chanlun/>
```
