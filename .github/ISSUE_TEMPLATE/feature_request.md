---
name: 功能请求
about: 提出一个新的功能或增强建议
title: "[Feature] "
labels: enhancement
assignees: []
---

## 动机

请描述这个功能要解决什么问题，或者满足什么使用场景。

## 提案

请描述您期望的功能或 API。

**Rust 核心层:**

```rust
// 期望的 API 或行为
```

**Python 绑定层 (如适用):**

```python
# 期望的 API 或行为
```

## 替代方案

是否有其他替代方案或现有机制可以满足需求？如果有，请描述。

## 与 chan.py 的关系

- [ ] `chan.py` (Python 参考实现) 中已有此功能
  - 相关代码位置: `chan.py` 行号或方法名
- [ ] 这是绑定层 (`chanlun-py`) 的功能需求
- [ ] 这是核心层 (`chanlun`) 的算法需求
- [ ] 这是全新的功能提案

## 影响范围

> 请勾选可能受影响的模块。

- [ ] 类型定义 (`types/`)
- [ ] K线层 (`kline/`)
- [ ] 技术指标 (`indicators/`)
- [ ] 笔划分 (`algorithm/bi`)
- [ ] 线段划分 (`algorithm/segment`)
- [ ] 中枢识别 (`algorithm/hub`)
- [ ] 背驰检测 (`algorithm/divergence`)
- [ ] 结构体 (`structure/`)
- [ ] 观察者 (`business/observer`)
- [ ] 买卖点 (`business/bsp`)
- [ ] K线合成器 (`business/synthesizer`)
- [ ] 立体分析器 (`business/multi_frame`)
- [ ] 配置 (`config`)
- [ ] Python 绑定 (`chanlun-py`)

## 补充信息

任何参考链接、图表、伪代码或其他有助于说明该功能的内容。
