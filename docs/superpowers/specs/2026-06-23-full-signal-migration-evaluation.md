# 信号计算器完全移植评估

- 日期：2026-06-23
- 前置：混合迁移（SignalEngine + SignalOrchestrator）已完成

## 1. 当前差距

### 1.1 未移植的 Python 信号函数（7/8）

| 函数 | 文件 | 行数 | 复杂度 | 移植工时 |
|------|------|------|--------|----------|
| `bar_zdt_V230331` | demo.py | 38 | 极低 | ~1h |
| `macd_金叉` | demo.py | 56 | 低 | ~1-2h |
| `tas_macd_direct_V221106` | demo.py | 55 | 低 | ~1-2h |
| `tas_ma_base_V230313` | demo.py | 57 | 低-中 | ~2-3h |
| `cxt_停顿分型_V230106` | demo.py | 49 | 低-中 | ~3-5h |
| `cxt_bi_end_V230222` | demo.py | 74 | 中 | ~4-8h |
| `模板_V日期` | _template.py | 28 | 模板 | 不需要 |

**总计：约 12-21 小时**

> 已移植的只有 `youwukuncheng_中枢第三买卖点_V230602`（1/8）。

### 1.2 可移除的 Python 组件

| 组件 | 文件 | 替换方案 |
|------|------|----------|
| `SignalsParser` 类 | chan_external.py:102-293 | 不再需要——配置由 Rust 注册表直接生成 |
| `get_signals_config()` | chan_external.py:296-310 | `list_signals()` + 直接构造配置 |
| `从信号列表提取配置()` | chan_external.py:522-530 | `list_signals()` + `get_signal_template()` |
| `create_single_signal()` | chan_external.py:312-319 | 不再需要（Rust 信号函数使用 `Signal::new_empty`） |
| `chanlun.signals` 包 | signals/*.py | 所有函数已移植到 Rust |
| `chanlun.parse` | parse.py | 仅被 `SignalsParser` 使用 |
| `chan.py` 中的副本 | chan.py:7394+ | 内部副本，可单独处理 |

## 2. 关键依赖链

```
strategies.py
  └→ get_signals_config(position.unique_signals, signals_module)
       └→ SignalsParser(signals_module).parse(signal_strings)
            └→ 遍历 chanlun.signals 模块的所有函数
            └→ 读取文档字符串 → 正则提取参数模板
            └→ parse 库反向格式化 → 配置字典
```

完全移植后，这个链简化为：
```
strategies.py
  └→ 直接构造 config = [{name, freq, params}] 从 Rust list_signals()
```

## 3. 建议：分两阶段执行

### 阶段 1：移植剩余信号函数（~12-21h）

按复杂度递增顺序：

| 子任务 | 内容 |
|--------|------|
| 1.1 | 移植 `bar_zdt_V230331` → `chanlun/src/signal/functions/demo.rs` |
| 1.2 | 移植 `macd_金叉` → `demo.rs` |
| 1.3 | 移植 `tas_macd_direct_V221106` → `demo.rs` |
| 1.4 | 移植 `tas_ma_base_V230313` → `demo.rs`（需要均线计算辅助） |
| 1.5 | 移植 `cxt_停顿分型_V230106` → `demo.rs` |
| 1.6 | 移植 `cxt_bi_end_V230222` → `demo.rs` |

每个子任务：
- 编写 Rust 函数 + `#[signal]` 注册
- 编写 Rust 单元测试
- 编写 Python 对比测试（Rust vs Python 输出）

### 阶段 2：移除 Python 回退路径（~4-6h）

| 子任务 | 内容 |
|--------|------|
| 2.1 | 简化 `SignalOrchestrator` → 仅使用 `SignalEngine` |
| 2.2 | 移除 `SignalsParser`、`get_signals_config`、`从信号列表提取配置` |
| 2.3 | 移除 `chanlun.signals` 包（demo.py/youwukuncheng.py/_template.py） |
| 2.4 | 移除 `chanlun.parse`（vendored parse 库） |
| 2.5 | 更新 `strategies.py` 使用直接配置构造 |
| 2.6 | 更新测试文件 |

## 4. 收益

| 收益 | 说明 |
|------|------|
| 代码量减少 | 移除 ~1,200 行 Python（SignalsParser + signals 包 + parse.py + chan.py 副本） |
| 统一执行路径 | 不再有 Rust/Python 双路径，消除维护成本 |
| 编译时安全 | 所有信号函数编译时注册，不会运行时 `import_by_name` 失败 |
| 性能提升 | 批量 Rust 执行 vs 逐个 Python 调用 |
| 依赖精简 | 移除 vendored `parse` 库和 `chanlun.signals` 包 |

## 5. 风险

| 风险 | 缓解 |
|------|------|
| `cxt_bi_end_V230222` 依赖笔/分型序列指针比较 | Rust 已有 `分型`/`笔` 结构，使用 `Arc` 指针 |
| `cxt_停顿分型_V230106` 依赖 `与MACD柱子分型匹配` | 需要确认 Rust 侧是否有该方法或等效逻辑 |
| `tas_ma_base_V230313` 依赖均线按需计算 | Rust 已有 `指标计算器::计算并挂载` 和 `k.ma(key)` |
| `strategies.py` 默认信号配置为空时依赖 `get_signals_config` | 切换到 `list_signals()` + 直接构造 |

## 6. 结论

**完全移植可行，建议执行。** 总工作量约 16-27 小时。7 个未移植信号函数按复杂度递增顺序逐个移植（阶段 1），然后移除 Python 回退路径（阶段 2）。完成后信号框架为纯 Rust 核心 + Python 薄绑定，不再有 Python 动态导入路径。
