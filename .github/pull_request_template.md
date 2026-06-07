---
name: Pull Request
about: 提交代码变更
title: ""
labels: []
assignees: []
---

## 描述

请简洁清晰地描述这个 PR 做了什么。

## 变更类型

- [ ] Bug 修复
- [ ] 新功能
- [ ] 重构 / 代码清理
- [ ] 文档更新
- [ ] 测试
- [ ] 其他

## 变更范围

> 勾选涉及的文件或模块。

**核心层 (`chanlun/`)：**

- [ ] `types/` — 基础类型
- [ ] `kline/` — K线层
- [ ] `indicators/` — 技术指标
- [ ] `algorithm/bi` — 笔划分
- [ ] `algorithm/segment` — 线段划分
- [ ] `algorithm/hub` — 中枢识别
- [ ] `algorithm/divergence` — 背驰检测
- [ ] `structure/` — 结构体
- [ ] `business/observer` — 观察者
- [ ] `business/bsp` — 买卖点
- [ ] `business/synthesizer` — K线合成器
- [ ] `business/multi_frame` — 立体分析器
- [ ] `config` — 配置

**绑定层 (`chanlun-py/`)：**

- [ ] `src/lib.rs` — 模块注册
- [ ] `src/business_py.rs` — 业务绑定
- [ ] `src/config_py.rs` — 配置绑定
- [ ] `src/structure_py.rs` — 结构体绑定

**其他：**

- [ ] 测试 (`chanlun/src/*/tests` 或 `chanlun-py/tests/`)
- [ ] 文档 (`README.md` / `CLAUDE.md` / `.github/`)

## 测试

- [ ] 核心层测试通过 (`cargo test`)
- [ ] 绑定层测试通过 (`python3 -m pytest chanlun-py/tests/test_all.py -v`)
- [ ] `cargo clippy` 零警告
- [ ] 与 `chan.py` 输出一致 (双端对比)
- [ ] 新增了相关测试
- [ ] 无新增测试（请说明原因）：

## 破坏性变更

- [ ] 是（请在下文描述迁移步骤）
- [ ] 否

## 补充信息

任何有助于审查者理解此 PR 的截图、日志或对比数据。
