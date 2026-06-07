---
name: 自定义问题
about: 其他问题（问题咨询、文档改进、重构建议等）
title: "[Question] "
labels: question
assignees: []
---

## 问题概述

请描述您的需求。

## 问题类型

- [ ] 问题咨询 — 对 API 或算法的使用存在疑问
- [ ] 文档 — 文档错误、缺失或改进建议
- [ ] 重构 — 代码结构或设计调整建议
- [ ] 兼容性 — Python 绑定层与 `chan.py` 的行为差异
- [ ] 性能 — 运行效率或内存占用问题
- [ ] 其他

## 涉及范围

> 可选择一项或多项。

| 层次 | 模块 |
|------|------|
| 核心层 | `types` / `kline` / `indicators` / `algorithm` / `structure` / `business` / `config` |
| 绑定层 | `chanlun-py` (`src/business_py.rs` / `src/config_py.rs` / `src/structure_py.rs`) |
| 测试 | `chanlun/src/*/tests` / `chanlun-py/tests/test_all.py` |
| 文档 | `chanlun/README.md` / `CLAUDE.md` / 其他 |
| 其他 | |

## 补充信息

任何有助于更好理解或解决此问题的信息。
