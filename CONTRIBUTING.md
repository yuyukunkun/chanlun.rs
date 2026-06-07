# 贡献指南

感谢你对 `chanlun` 项目的关注！

本项目将 Python 版缠论技术分析库 (`chan.py`) 完整移植为 Rust，同时通过 PyO3 绑定层保持 Python API 兼容。以下指南旨在帮助平滑贡献流程。

---

## 目录

- [角色与分工](#角色与分工)
- [开发环境](#开发环境)
- [项目结构](#项目结构)
- [开发流程](#开发流程)
- [代码规范](#代码规范)
- [测试指南](#测试指南)
- [提交信息](#提交信息)
- [双端对齐](#双端对齐)

---

## 角色与分工

| 角色 | 范围 | 联系 |
|------|------|------|
| 维护者 | 架构决策、代码审查、发布 | @YuWuKunCheng |
| 贡献者 | 提交 PR、报告 Bug、改进文档 | 任何人 |

---

## 开发环境

### 必需工具

| 工具 | 最低版本 | 用途 |
|------|---------|------|
| Rust | 1.85+ | 核心层编译 |
| Python | 3.10+ | 绑定层测试、对比验证 |
| maturin | 1.x | PyO3 绑定开发与安装 |

### 初始化

```bash
# 克隆仓库
git clone https://github.com/YuYuKunKun/chanlun.rs.git
cd chanlun.rs

# 核心层
cd chanlun
cargo build
cargo test

# 绑定层
cd ../chanlun-py
maturin develop
python3 -m pytest tests/test_all.py -v

# 确保 clippy 零警告
cd ../chanlun
cargo clippy
```

---

## 项目结构

```
chanlun.rs/
├── chan.py                          # Python 参考实现 (~4200 行)
├── chanlun/                         # Rust 核心层
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs                   # 模块注册
│       ├── config.rs                # 缠论配置 (62 字段, serde)
│       ├── types/                   # 基础类型
│       ├── kline/                   # K线层
│       ├── indicators/              # 技术指标
│       ├── algorithm/               # 核心算法 (笔/线段/中枢/背驰)
│       ├── structure/               # 结构体 (虚线/分型/特征)
│       ├── business/                # 业务层 (观察者/合成器/立体分析)
│       └── utils/                   # 工具
├── chanlun-py/                      # PyO3 Python 绑定
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs                   # 模块注册与导出
│   │   ├── business_py.rs           # 业务层 Python 封装
│   │   ├── config_py.rs             # 配置 Python 封装
│   │   └── structure_py.rs          # 结构体 Python 封装
│   ├── chanlun/                     # Python 存根模块
│   │   └── __init__.py
│   └── tests/
│       └── test_all.py              # 完整测试套件
├── CLAUDE.md                        # AI 辅助开发指令
├── .github/                         # GitHub 模板
│   ├── pull_request_template.md
│   └── ISSUE_TEMPLATE/
│       ├── bug_report.md
│       ├── feature_request.md
│       └── custom.md
├── README.md
├── SECURITY.md
└── CODE_OF_CONDUCT.md
```

---

## 开发流程

### 从 Issue 开始

1. 查找或创建相关 Issue
2. 在 Issue 中讨论方案，达成共识后再开始编码
3. 避免在没有 Issue 的情况下提交大型 PR

### 分支策略

```bash
# 从 develop 分支创建功能分支
git checkout develop
git pull origin develop
git checkout -b feature/your-feature-name

# 或从 develop 分支创建修复分支
git checkout -b fix/your-bug-fix
```

### 提交 PR

1. 确保所有测试通过
2. 确保 `cargo clippy` 零警告
3. 推送到你的分支并发起 PR 到 `develop`
4. 填写 PR 模板中的所有内容
5. 等待审查并响应反馈

---

## 代码规范

### 中文标识符

所有类型名、方法名、字段名必须使用中文，与 `chan.py` 保持 1:1 对应：

```rust
// ✓ 正确
pub struct 缠论K线 { pub 高: SyncF64, pub 低: SyncF64 }
pub fn 方向(&self) -> 相对方向 { ... }

// ✗ 错误 — 不允许英文
pub struct ChanKline { pub high: f64 }
```

### 许可证头部

每个 `.rs` 文件必须以 MIT 许可证头部开始：

```rust
/*
 * MIT License
 *
 * Copyright (c) 2026 YuYuKunKun
 * ...
 */
```

### 代码风格

- 使用 `cargo fmt` 自动格式化
- 遵循 `cargo clippy` 建议（零警告）
- 仅写必要注释 — 解释"为什么"而非"做什么"
- 不对仅使用一次的代码做抽象
- 不添加方案之外的特性和错误处理

### Rust 相关约定

- `#![allow(non_snake_case)]` 和 `#![allow(non_camel_case_types)]` 已在 `lib.rs` 中声明
- 内部可变性优先用 `AtomicI64`/`AtomicBool`/`SyncF64`，复杂字段用 `RwLock`
- `Arc<分型>` 通过 `Arc::as_ptr` 比较身份（而非值比较）
- 全局缓存使用 `LazyLock<Mutex<>>`，不使用 `thread_local!`
- 读写锁作用域化，防止死锁

---

## 测试指南

### 核心层测试

```bash
cd chanlun
cargo test             # 运行所有测试
cargo test -- <name>   # 运行匹配名称的测试
```

测试应覆盖：
- 类型构造/字段读写/Clone 后指针一致性
- 算法函数的边界情况（空序列、单元素、极端价格）
- 流式增量结果与静态重新分析的一致性
- `Send + Sync` 编译期断言
- 跨线程读写不 panic

### 绑定层测试

```bash
cd chanlun-py
maturin develop
python3 -m pytest tests/test_all.py -v
```

测试应覆盖：
- Python API 与 `chan.py` 的接口兼容性
- 跨线程 `is` 身份一致性
- 双端（Rust 绑定 vs `chan.py`）关键算法输出对比

### 双端对比

当我们修改算法层代码时，必须验证 Rust 输出与 Python 版一致：

```python
# 典型双端对比模式
from chanlun import 观察者 as 观察者Rust
from chanlun.chan import 观察者 as 观察者Py

# 加载同样的数据
obs_rust = 观察者Rust("btcusd", 300, config)
obs_py = 观察者Py("btcusd", 300, config)

# 对比结果
assert len(obs_rust.笔序列) == len(obs_py.笔序列)
assert len(obs_rust.线段序列) == len(obs_py.线段序列)
```

---

## 提交信息

使用简洁的中文，格式为：

```
<类型>: <简要描述>

<详细说明（可选）>

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
```

类型示例：
- `fix:` — Bug 修复
- `feat:` — 新功能
- `refactor:` — 重构（行为不变）
- `test:` — 添加或修改测试
- `docs:` — 文档更新
- `chore:` — 构建/工具

所有提交必须以 `Co-Authored-By:` 行结尾，这是本项目对 AI 辅助开发的惯例。

---

## 双端对齐

本项目最核心的质量要求是 Rust 实现与 `chan.py` 行为完全一致。对齐时遵循：

1. **以 `chan.py` 为准** — Python 实现是 golden source
2. **增量对齐** — 优先修复数量差异（笔数、线段数），再深入字段级对齐
3. **算法差异分类**：
   - 核心公式错误：如 MACD 面积计算 `阳+阴` vs `阳+|阴|`
   - 边界条件遗漏：如 `计算MACD柱子分段` 末尾段未追加
   - 指针身份 vs 值索引：`position(|k| Arc::as_ptr(k) == ...)` vs `position(|k| k.序号 == ...)`
4. **使用测试驱动** — 先写双端对比测试，确认差异存在，再改 Rust 代码对齐
