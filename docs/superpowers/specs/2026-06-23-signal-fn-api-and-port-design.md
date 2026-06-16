# 子项目 2：信号函数 API + 移植第一个真实信号 — 设计文档

- 日期：2026-06-23
- 所属：「全 Rust 信号计算迁移」第 2 个子项目（共 4 个）
- 前置：子项目 1 已完成（`#[signal]` 宏 + `inventory` 注册表）
- 参考：`chanlun-py/chanlun/signals/youwukuncheng.py`、czsc

## 1. 背景与目标

子项目 1 交付了编译期信号注册机制（`#[signal]` + `inventory` + `SIGNAL_REGISTRY`），探针信号已验证注册→查表链路。现在是时候移植第一个真实信号函数，并在过程中建立 Rust 信号函数的**编写规范**和**辅助 API**。

子项目 2 交付：
1. **信号函数便捷 API** — 扩展 trait，让 Rust 信号函数代码读起来接近 Python 版本
2. **确保指标按需增量计算** — 信号函数可确保所需指标已计算
3. **移植 youwukuncheng_中枢第三买卖点_V230602** — 第一个真实信号（3 种信号变体）
4. **集成测试** — Rust vs Python 输出对比

## 2. 范围

### 纳入
- `chanlun/src/signal/functions/` 模块（信号函数目录）
- `chanlun/src/signal/functions/youwukuncheng.rs` — 移植的中枢第三买卖点信号
- 便捷扩展 trait：`IndicatorAccess`（K线指标读取）、`ObserverAccess`（观察者便捷访问）
- 参数提取辅助函数（`params_ext.rs`）
- 确保指标 API：`观察者::确保指标已计算(&self)`
- 集成测试：喂入 `.nb` 数据，Rust 信号输出 vs Python 信号输出
- `#[signal]` 注册 youwukuncheng

### 不纳入（后续子项目）
- 信号计算引擎 + `call_signal` PyO3 分发器（子项目 3）
- Position.update 状态机（子项目 4）
- 其他信号函数（demo.py 中的 macd_金叉、cxt_bi_end 等）
- Python 侧可直接调用的 PyO3 信号函数分发器

## 3. 关键设计决策

| 决策 | 选择 | 理由 |
|---|---|---|
| 便捷 API 形式 | **直接给 K线 / 观察者 加方法** | 简洁，不需要 import 额外 trait。已有前例（观察者.当前缠K()） |
| 指标访问封装 | **方法返回 Option，隐藏 RwLock** | 信号函数不应关心锁细节；`kline.macd()` 返回 `Option<&MACD>` |
| 确保指标机制 | **观察者.确保指标已计算() 重跑计算器** | 简单，复用现有 `指标计算器::计算并挂载`。后续子项目 3 由计算引擎在调用前统一 ensure |
| 参数提取 | **独立 `params` 子模块，纯函数** | `HashMap<String, Value>` 的字符串/数字提取到处都需要，集中处理 |
| 信号函数位置 | `chanlun/src/signal/functions/` | 与 registry 同 crate，`#[signal]` emit 的 `crate::` 路径可直接解析 |
| 测试策略 | **Rust 集成测试 + Python 对比** | 加载 .nb → 跑 Rust 信号 → 序列化输出；Python 侧同样跑 → diff |

## 4. 文件结构

```
chanlun/src/signal/
├── mod.rs                          ← pub mod functions; pub mod params;
├── functions/
│   ├── mod.rs                      ← pub mod youwukuncheng;
│   └── youwukuncheng.rs            ← #[signal] fn youwukuncheng_中枢第三买卖点_V230602
├── params.rs                       ← 参数提取辅助函数
├── ... (已有: signal, factor, event, position, operate, registry)
│
chanlun/src/kline/
├── bar.rs                          ← 给 K线 加便捷指标访问方法
│
chanlun/src/business/
├── observer.rs                     ← 给 观察者 加便捷方法 + 确保指标

chanlun/tests/
├── test_signal_youwukuncheng.rs    ← 集成测试（Rust vs Python 对比）
```

## 5. 便捷 API 设计

### 5.1 K线 便捷指标访问（`bar.rs` 新增方法）

将现有的 `k线.指标.read().macd()` 封装为直接的 `k线.macd()`：

```rust
impl K线 {
    /// 读取 MACD 指标（已计算则返回引用，否则 None）
    pub fn macd(&self) -> Option<&平滑异同移动平均线> { ... }
    pub fn rsi(&self) -> Option<&相对强弱指数> { ... }
    pub fn kdj(&self) -> Option<&随机指标> { ... }
    pub fn boll(&self) -> Option<&布林带> { ... }
    /// 读取均线值，如 ma("SMA_5") → Option<f64>
    pub fn ma(&self, key: &str) -> Option<f64> { ... }
}
```

同样给 `缠论K线` 加转发方法（委托给 `self.标的K线`）。

### 5.2 观察者便捷访问（`observer.rs` 新增方法）

```rust
impl 观察者 {
    /// 按偏移取普K（di=1 为最后一根）
    pub fn 普K偏移(&self, di: usize) -> Option<&Arc<K线>> { ... }
    /// 按偏移取缠K
    pub fn 缠K偏移(&self, di: usize) -> Option<&Arc<缠论K线>> { ... }
    /// 最后 N 根缠K
    pub fn 最后缠K序列(&self, n: usize) -> &[Arc<缠论K线>] { ... }
    /// 线段级中枢序列（= 中枢序列组[1]）
    pub fn 线段中枢序列(&self) -> &Vec<Arc<中枢>> { ... }
    /// 确保所有 K 线上的指标已计算（调用 指标计算器::计算并挂载）
    pub fn 确保指标已计算(&self) { ... }
}
```

### 5.3 参数提取（`signal/params.rs`）

```rust
/// 从 params HashMap 提取字符串参数
pub fn get_string(params: &HashMap<String, Value>, key: &str, default: &str) -> String;
/// 从 params HashMap 提取整数参数
pub fn get_int(params: &HashMap<String, Value>, key: &str, default: i64) -> i64;
/// 从 params HashMap 提取浮点参数
pub fn get_f64(params: &HashMap<String, Value>, key: &str, default: f64) -> f64;
```

这些是纯辅助函数，不做任何复杂逻辑。

## 6. youwukuncheng 移植要点

### 6.1 信号逻辑

Python 版 143 行 → Rust 预计 ~200 行（含类型标注和 RwLock 读取）。

三种产出信号（k3 后缀均为 `V230602`）：

| k3 | 触发条件 | v1 | v2 | score |
|---|---|---|---|---|
| `中枢段DEA穿越2V230602` | 同级第三买卖线段内 DEA 穿越 0 轴 | 中枢段DEA穿越2 | 三买/三卖 | max(0, 100-偏移×5) |
| `DEA穿越0轴V230602` | 本级第三买卖线处 DEA 在 0 轴同侧 | DEA穿越0轴 | 三买/三卖 | max(0, 100-偏移×5) |
| `首次穿越0轴V230602` | DIF 首次反穿 0 轴 + 分型确认 | 首次穿越0轴 | 三买/三卖 | max(0, 100-偏移×5) |

### 6.2 关键 Rust 对应

| Python | Rust |
|---|---|
| `观察员.当前缠K` | `obs.当前缠K()` |
| `观察员.中枢序列` | `obs.中枢序列()` (笔中枢) 或 `obs.线段中枢序列()` (线段中枢) |
| `当前中枢.基础序列[0].标识` | `当前中枢.基础序列.read()[0].标识.read().as_str()` |
| `当前中枢.当前状态()` | `当前中枢.当前状态()` |
| `当前中枢.本级_第三买卖线` | `当前中枢.本级_第三买卖线.read().as_ref()` |
| `当前中枢.完整性("实")` | `当前中枢.完整性("实")` |
| `k.标的K线.macd.DEA` | `k.标的K线.read().macd().map(\|m\| m.DEA)` |
| `k.分型 is 分型结构.底` | `*k.分型.read() == Some(分型结构::底)` |
| `分型.从缠K序列中获取分型(序列, k)` | `分型::从缠K序列中获取分型(序列, k)` |
| `虚线.统计MACD行为(普K序列, 8, 3)` | `虚线::统计MACD行为(&普K序列, 8, 3)` |
| `段.获取普K序列(观察员.观察员)` | `段.获取普K序列(&obs.普通K线序列)` |

### 6.3 注意事项

1. **lock 顺序**：读取 `基础序列`、`武`、`标的K线`、`指标` 时注意 RwLock 不可重入。同一作用域内避免同时持有多个写锁。本函数只有读操作，安全。
2. **AtomicI64**：`序号` 用 `.load(Ordering::Relaxed)` 读取
3. **Option 链**：Python 的 `x.y.z` 在 Rust 中是 `x.y.read().z`，需要处理 `Option`
4. **空信号返回**：Python 返回 `create_single_signal(k1, k2, k3)`（v1=v2=v3="任意"）；Rust 返回 `vec![Signal::new_empty(k1, k2, k3)]`

## 7. 确保指标 API

```rust
impl 观察者 {
    /// 确保所有 K线上的指标已计算。
    /// 如果 配置.计算指标 为 true 且序列非空，则调用 指标计算器::计算并挂载。
    pub fn 确保指标已计算(&self) {
        if self.配置.计算指标 && !self.普通K线序列.is_empty() {
            指标计算器::计算并挂载(&self.普通K线序列, &self.配置);
        }
    }
}
```

信号函数在入口调用一次 `obs.确保指标已计算()`（幂等——计算器检测已计算的值会跳过）。

注：后续子项目 3 的信号计算引擎会在调用任何信号前统一 ensure，信号函数内部的 ensure 调用届时可移除。

## 8. 测试设计

### 8.1 集成测试（`chanlun/tests/test_signal_youwukuncheng.rs`）

1. 加载测试 `.nb` 文件（选择已有中枢结构的 btcusd 数据）
2. 创建观察者，喂入 K 线，触发分析
3. 调用 `youwukuncheng_中枢第三买卖点_V230602(&obs, &params)`
4. 验证返回的 `Vec<Signal>` 非空，信号 key/value 格式正确
5. 与 Python 版输出对比（golden 方式：运行 Python 脚本生成预期输出文件，Rust 测试读取对比）

### 8.2 测试数据

使用已有测试 `.nb` 文件（如 `btcusd-86400-...`，日线数据有丰富的中枢结构）。

## 9. 错误处理

- 信号函数内部所有 `Option` 缺值 → 返回空信号（与 Python 行为一致）
- `确保指标已计算` 失败 → 静默跳过（指标不存在时信号函数内部 `macd().is_none()` 自然会返回空）
- `#[signal]` 注册失败（重名）→ 子项目 1 已处理（编译期 panic）

## 10. 已知取舍

- **便捷方法只加常用读路径**：`macd()/rsi()/kdj()/boll()/ma()` + 偏移访问。复杂查询（如遍历所有 K 线做自定义分析）直接用底层 API。
- **确保指标基于现有管线**：不做 czsc 式的 TaCache（已决策，见子项目 1 §3）。SignalFn 签名保持 `&观察者` 单参数。
- **信号函数在 lib 内**：不暴露为独立的 `chanlun-signals` crate。与子项目 1 决策一致——信号函数同 crate，可直接访问 observer 内部。

## 11. 许可证

新增文件沿用项目 MIT 头。youwukuncheng 移植自项目自有 Python 代码，不涉及第三方许可证。
