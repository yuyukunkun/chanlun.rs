# chanlun — 缠论技术分析 Rust 核心库

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org)
[![edition](https://img.shields.io/badge/edition-2024-9cf.svg)](https://doc.rust-lang.org/edition-guide/rust-2024/)
[![tests](https://img.shields.io/badge/tests-199%20passed-green.svg)](.)

[缠中说禅](https://zh.wikipedia.org/wiki/%E7%BC%A0%E4%B8%AD%E8%AF%B4%E7%A6%85) 理论的 Rust 高性能实现——**约 14,000 行 Rust（2024 edition）**，覆盖缠论完整算法体系：K线包含处理、分型识别、笔/线段划分（含特征序列与缺口修正）、中枢识别（延伸/扩展/多级）、背驰检测（MACD/斜率/测度）、买卖点生成（18 种类型）、技术指标计算（MACD/RSI/KDJ/BOLL/均线）、信号匹配框架、编译期信号注册表与 .so 动态插件系统。

Python 参考实现 `chan.py`（~4,200 行）已完整移植，API **1:1 兼容**——所有类型、方法、字段名均使用中文标识符。Python 绑定通过 [`chanlun-py`](../chanlun-py/) crate（PyO3）发布为 `chanlun` PyPI 包。

---

## 目录

- [核心概念](#核心概念)
- [架构总览](#架构总览)
- [项目结构](#项目结构)
- [快速开始](#快速开始)
- [数据管线](#数据管线)
- [核心类型详解](#核心类型详解)
- [配置体系](#配置体系)
- [算法模块](#算法模块)
- [技术指标](#技术指标)
- [信号框架](#信号框架)
- [插件系统](#插件系统)
- [线程安全与并发](#线程安全与并发)
- [Python 绑定](#python-绑定)
- [数据序列化](#数据序列化)
- [测试](#测试)
- [许可](#许可)

---

## 核心概念

缠论将价格走势分解为层级结构，从最底层K线向上逐级构造：

```
原始K线 → 包含处理 → 缠论K线 → 顶底分型 → 笔 → 线段 → 中枢 → 买卖点
```

理论核心：
- **走势终完美**：任何级别的走势类型终要完成，不可长期存续
- **自同构性**：不同级别的走势呈现相同的形态结构，可通过递归分析进行多级别联立
- **完全分类**：三段式完全分类（上涨/下跌/盘整），每个节点都可通过买卖点找到操作依据

---

## 架构总览

```
┌──────────────────────────────────────────────────────────────┐
│  Python 层                                                   │
│  ┌─────────┐  ┌──────────────┐  ┌─────────────────────────┐ │
│  │ main.py │  │ strategies.py│  │ signal_orchestrator.py   │ │
│  │ (Web UI)│  │ (回测引擎)    │  │ (混合信号编排: Rust+Py)   │ │
│  └─────────┘  └──────────────┘  └─────────────────────────┘ │
│                        │ PyO3 FFI                             │
├────────────────────────┼─────────────────────────────────────┤
│  PyO3 绑定层 (chanlun-py)                                     │
│  ┌────────────────────────────────────────────────────────┐  │
│  │ config_py │ kline_py │ structure_py │ algorithm_py     │  │
│  │ business_py │ signal_py │ signal_engine_py │ cache     │  │
│  └────────────────────────────────────────────────────────┘  │
│                        │ Rust API                             │
├────────────────────────┼─────────────────────────────────────┤
│  Rust 核心 (chanlun)                                          │
│  ┌────────────────────────────────────────────────────────┐  │
│  │  config  │ types │ kline │ indicators │ algorithm      │  │
│  │  structure │ business │ signal (primitives + engine    │  │
│  │  + registry + ffi + functions) │ utils                 │  │
│  └────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
```

**关键设计决策**：

| 决策 | 选择 | 原因 |
|------|------|------|
| 内部可变性 | `AtomicI64` / `SyncF64` / `RwLock<T>` | 多 Arc 共享下原地修改（如动态替换笔的武） |
| 增量计算 | 只重新分析末尾元素 | O(1) 均摊，适合实时交易 |
| 浮点原子 | `SyncF64`（`AtomicU64` 位转换） | 跨线程原子读写价格字段 |
| 按需计算 | 缠K/分型/笔/线段/中枢全部惰性 | 避免不必要的计算开销 |
| 3 级线段递归 | 线段 → 线段<线段> → 线段<线段<线段>> | 多级自同构性分析 |
| 中文标识符 | `#![allow(non_snake_case)]` | 与 Python `chan.py` API 完全对应 |

---

## 项目结构

```
chanlun/
├── Cargo.toml                       # 依赖: serde, byteorder, chrono, sha2, inventory
├── README.md                        # 本文档
└── src/
    ├── lib.rs                       # 模块注册 + 日志开关 + tracing 初始化
    ├── main.rs                      # CLI 工具 (read/synth 子命令)
    ├── config.rs                    # 缠论配置 (44 字段, serde, model_fields/对比/往返)
    │
    ├── types/                       # 基础类型 (5 文件, ~514 行)
    │   ├── mod.rs                   # 重导出
    │   ├── direction.rs             # 相对方向 (9 变体): 向上/向下/缺口/衔接/顺/逆/同
    │   ├── bsp_type.rs              # 买卖点类型 (18 变体)
    │   ├── fractal.rs               # 分型结构 (5 变体) + 有高低 trait
    │   ├── gap.rs                   # 缺口: 价格区间 + 居中截取
    │   └── sync_f64.rs              # SyncF64: AtomicU64 线程安全 f64
    │
    ├── kline/                       # K线层 (2 文件, ~1,113 行)
    │   ├── bar.rs                   # 原始K线: OHLCV + RwLock<指标容器>, 48字节大端序列化
    │   └── chan_kline.rs            # 缠论K线: 包含处理后, 分型标记, 内部可变性字段
    │
    ├── indicators/                  # 技术指标 (7 文件, ~1,589 行)
    │   ├── macd.rs                  # MACD (EMA快慢线, DIF/DEA/MACD柱, 首次+增量)
    │   ├── rsi.rs                   # RSI (Wilder SMA 平滑, 超买超卖)
    │   ├── kdj.rs                   # KDJ (RSV→K→D→J, 超买超卖阈值)
    │   ├── boll.rs                  # BOLL (中轨/上轨/下轨, 带宽)
    │   ├── calculator.rs            # 指标计算器: 增量计算 + 回填 + 均线
    │   └── container.rs             # 指标容器: 注册表模式, 动态指标存取
    │
    ├── algorithm/                   # 核心算法 (4 文件, ~4,987 行)
    │   ├── bi.rs                    # 笔划分: 递归分析, 弱化/次级/实际高低点
    │   ├── segment.rs               # 线段划分: 特征序列, 缺口处理, 四种修正, 扩展线段
    │   ├── hub.rs                   # 中枢识别: 重叠检测, 延伸/扩展, 第三买卖点
    │   └── divergence.rs            # 背驰检测: MACD/斜率/测度 + 四种组合模式
    │
    ├── structure/                   # 结构体 (4 文件, ~2,089 行)
    │   ├── dash_line.rs             # 虚线: 笔/线段的通用抽象, MACD行为统计, 买卖意义
    │   ├── fractal_obj.rs           # 分型: 左中右构型, 强度判定, MACD匹配
    │   ├── segment_feat.rs          # 线段特征: 文/武取极值, 静态分析, 分型序列
    │   └── feat_fractal.rs          # 特征分型: 三线段特征元素构成的分型
    │
    ├── business/                    # 业务层 (4 文件, ~1,648 行)
    │   ├── observer.rs              # 观察者: 单周期流式分析器, 3级递归管线
    │   ├── bsp.rs                   # 买卖点工厂: 18种类型, 偏移/失效/终结
    │   ├── synthesizer.rs           # K线合成器: 小周期→大周期合成
    │   └── multi_frame.rs           # 立体分析器: 多周期联立, 合成器+观察者协调
    │
    ├── signal/                      # 信号框架 (14 文件, ~4,300 行)
    │   ├── mod.rs                   # 重导出 + 匹配值/信号字典/缺键错误/sha256前4
    │   ├── signal.rs                # Signal: czsc 风格七段key匹配 (k1~k3/v1~v3/score)
    │   ├── factor.rs                # Factor: 多信号all/not匹配 → name含哈希后缀
    │   ├── event.rs                 # Event: 因子匹配 → 分配操作(开多/平多/开空/平空)
    │   ├── operate.rs               # Operate: 5 种操作枚举 (开多/平多/开空/平空/持币)
    │   ├── position.rs              # Position: 完整仓位状态机 (~1,039行)
    │   │                            #   update() 事件匹配 + LO/SO/LE/SE 状态转换
    │   │                            #   + 止损/超时/间隔检查 + pairs() 盈亏计算
    │   ├── params.rs                # 参数提取: get_int/get_float/get_string/get_bool
    │   ├── registry.rs              # 双注册表: 编译时(LazyLock) + 动态(RwLock)
    │   ├── engine.rs                # 信号引擎: 批量执行信号函数 + 自动挂载指标
    │   ├── ffi.rs                   # C-ABI 导出: 供 .so 插件动态注册
    │   ├── functions/               # 信号函数实现
    │   │   ├── mod.rs               #
    │   │   ├── youwukuncheng.rs     # 中枢第三买卖点 (~280行, 3种信号变体)
    │   │   └── demo.rs              # 示例信号 (~440行, 涨停/MACD金叉/MA/停顿分型/笔结束)
    │   └── registry_macro_test.rs   # #[signal] 宏端到端测试
    │
    └── utils/                       # 工具 (2 文件, ~145 行)
        ├── datetime.rs              # 时间戳转换: 字符串↔Unix时间戳
        └── format.rs                # 格式化: f64→最小字符串 (format_f64_g)
```

### 依赖项

| 依赖 | 用途 |
|------|------|
| `serde` + `serde_json` | 配置序列化、信号参数解析 |
| `byteorder` | .nb 文件的大端字节序读写 |
| `chrono` | 时间戳<->日期转换（Position 交易日判断） |
| `sha2` | 信号 Factor/Event 名称确定性的 sha256 前4 |
| `inventory` | `#[signal]` 宏编译期注册表收集 |
| `parking_lot` | RwLock/Mutex（性能优于 std 实现） |
| `cached` | LRU 缓存（买卖意义/ MACD 行为） |
| `tracing` | 结构化日志（warn/info/error!） |

---

## 快速开始

### 作为 Rust 库使用

```toml
[dependencies]
chanlun = "26.6"
```

### 单周期分析

```rust
use chanlun::config::缠论配置;
use chanlun::kline::bar::K线;
use chanlun::business::observer::观察者;

// 创建观察者（返回 Arc<RwLock<观察者>>，支持多线程共享）
let 观察员 = 观察者::new("BTCUSD".into(), 3600, 缠论配置::default());

// 逐根投喂 K 线（流式增量分析）
for k线 in 数据流 {
    观察员.write().增加原始K线(k线);
}

// 读取分析结果
let obs = 观察员.read();
println!("笔: {}, 线段: {}, 中枢: {}",
    obs.笔序列.len(), obs.线段序列().len(), obs.中枢序列().len());
```

### 便捷投喂（无需构造 K 线对象）

```rust
let 观察员 = 观察者::new("ETHUSD".into(), 300, Default::default());
观察员.write().投喂原始数据(
    1736640000,  // Unix 时间戳
    3500.0,      // 开盘
    3550.0,      // 最高
    3480.0,      // 最低
    3520.0,      // 收盘
    1200.0,      // 成交量
);
```

### 从 .nb 文件批量加载

```rust
观察员.write().读取数据文件("btcusd-300-1000000-1100000.nb", Default::default())?;
```

### 多周期联立分析

```rust
use chanlun::business::multi_frame::立体分析器;

// 周期组: [60s, 300s, 1800s, 7200s]
// 只投喂最小周期 K 线，大周期自动合成
let mut 分析器 = 立体分析器::new(
    "BTCUSD".into(),
    vec![60, 300, 1800, 7200],
    None,  // 默认配置
    None,  // 无周期特定配置
);

for k线 in 小周期K线流 {
    分析器.投喂K线(k线);
}

// 获取各周期分析结果
if let Some(日线) = 分析器.获取观察者(86400) {
    println!("日线笔数: {}", 日线.read().笔序列.len());
}
```

### 信号计算

```rust
use chanlun::signal::engine::{SignalEngine, SignalConfig};

// 构建信号引擎
let 引擎 = SignalEngine::new(vec![
    SignalConfig {
        signal_name: "youwukuncheng_中枢第三买卖点_V230602".into(),
        freq: 86400,
        params: [
            ("freq".into(), "日线".into()),
            ("max_overlap".into(), 3.into()),
            ("本级完整性".into(), "实".into()),
            ("同级完整性".into(), "合".into()),
        ].into_iter().collect(),
    },
]);

// 自动挂载所需指标
引擎.自动挂载指标(&分析器);

// 计算信号
let 信号 = 引擎.更新_完整(&分析器);
// → { signals: { "日线_D1MO3_中枢段DEA穿越2V230602": "三买" }, market: { ... } }
```

### CLI 工具

```bash
# 单周期分析 + 保存结果
cargo run -- read btcusd-3600-1000000-1100000.nb

# 多周期合成 (从文件名的周期推断周期组: N, N×5, N×30)
cargo run -- synth btcusd-14400-1753142400-1781928000.nb
```

---

## 数据管线

每收到一根新 K 线，**增量更新**所有层级（只重算末尾）。这是该库适合实时交易系统的关键设计。

### 管线流程

```
原始K线 (K线)
  │
  ├─ 1. 指标计算 (indicators/calculator)
  │     ├─ MACD 组  (遍历 MACD_参数列表, 首次/增量)
  │     ├─ RSI 组   (遍历 RSI_周期列表)
  │     ├─ KDJ 组   (遍历 KDJ_参数列表)
  │     ├─ BOLL 组  (遍历 BOLL_参数列表)
  │     ├─ 均线组    (遍历 均线参数列表, SMA/EMA)
  │     └─ 回填新指标 (回溯填充前序K线遗漏的指标)
  │
  ├─ 2. 包含处理 → 缠论K线 (缠论K线)
  │     ├─ 顺序包含合并: 顺方向取极值
  │     ├─ 逆序包含合并: 更新时间/标的K线
  │     ├─ 方向判定: 9 种相对方向
  │     └─ 合并替换模式: 原地修改 / 产出新缠K
  │
  ├─ 3. 分型识别 → 分型 (分型)
  │     ├─ 左中右三根缠K构成
  │     ├─ 类型: 顶/底/上/下/散
  │     └─ 分型模式开关控制缓存/实时读取
  │
  ├─ 4. 笔划分 → 笔 (虚线, 级别=1)
  │     ├─ 递归分析: 顶底分型交替验证
  │     ├─ 弱化模式: ≥3 根原始K线即可成笔
  │     ├─ 次级成笔: 次高/次低作为候选
  │     └─ 笔内验证: 分型包含整笔 / 原始K线包含整笔
  │
  ├─ 5. 笔中枢识别 → 笔中枢
  │     └─ 虚线重叠检测 → 延伸/扩展/第三买卖点
  │
  ├─ 6. 线段划分 → 线段 (虚线, 级别≥2), 3级递归
  │     ├─ 特征序列提取 (同向笔序列)
  │     ├─ 缺口处理: 有缺口(老阴老阳) / 无缺口(小阳少阴)
  │     ├─ 四种修正: 缺口突破 / 非缺口下穿刺 / 缺口后紧急修正 / 短路修正
  │     └─ 递归: 线段 → 线段<线段> → 线段<线段<线段>>
  │
  ├─ 7. 扩展线段 + 混合扩展线段 (各3级递归)
  │
  ├─ 8. 线段中枢识别
  │     └─ 多级中枢: 线段中枢 / 扩展线段中枢 / 混合扩展中枢 / ...
  │
  └─ 9. 买卖点生成 → 基础买卖点 (18 种类型)
        ├─ 6 经典: 一买/一卖/二买/二卖/三买/三卖
        ├─ 12 扩展: T1~T3B 各含买卖
        └─ 指标匹配: MACD/KDJ/RSI 确认
```

### 流式增量处理

`观察者.__处理数据` 每步只处理末尾几个元素，时间复杂度 **O(1) 均摊**。

**推送模式**: 调用方可通过 `add_bar_listener()` 等回调在每级结果产出时收到通知。

---

## 核心类型详解

### 枚举类型

#### 相对方向 (`types::direction`) — 9 变体

| 变体 | 判定条件 | 场景 |
|------|---------|------|
| `向上` | 后高>前高 且 后低>前低, 无缺口 | 正常上涨 |
| `向下` | 后高<前高 且 后低<前低, 无缺口 | 正常下跌 |
| `向上缺口` | 后低 > 前高 | 大幅高开 |
| `向下缺口` | 后高 < 前低 | 大幅低开 |
| `衔接向上` | 后低 ≈ 前高 | 精准衔接 |
| `衔接向下` | 后高 ≈ 前低 | 精准衔接 |
| `顺` | 前包含后 (前高≥后高 且 前低≤后低) | 顺序包含 |
| `逆` | 后包含前 (前高≤后高 且 前低≥后低) | 逆序包含 |
| `同` | 完全相同 | 重复数据 |

方法: `翻转()`, `是否向上()`, `是否向下()`, `是否包含()`, `是否缺口()`, `分析(前高,前低,后高,后低)`

#### 买卖点类型 (`types::bsp_type`) — 18 变体

| 类别 | 买点变体 | 卖点变体 | 说明 |
|------|---------|---------|------|
| 一类 | `一买` | `一卖` | 中枢背驰后第一类 |
| 二类 | `二买` | `二卖` | 回调到中枢内第二类 |
| 三类 | `三买` | `三卖` | 离开中枢不回第三类 |
| T1 | `T1买` | `T1卖` | 事后确认型 |
| T1P | `T1P买` | `T1P卖` | 事后确认型+ |
| T2 | `T2买` | `T2卖` | 中枢回调型，破位值判定 |
| T2S | `T2S买` | `T2S卖` | 中枢回调次级 |
| T3A | `T3A买` | `T3A卖` | 第三类扩展A |
| T3B | `T3B买` | `T3B卖` | 第三类扩展B |

方法: `是买点()`, `是卖点()`

#### 分型结构 (`types::fractal`) — 5 变体

| 变体 | 左中右关系 | 图示 |
|------|-----------|------|
| `上` | 向上 + 向上 | ↗↗ |
| `下` | 向下 + 向下 | ↘↘ |
| `顶` | 向上 + 向下 | ↗↘ (Λ) |
| `底` | 向下 + 向上 | ↘↗ (V) |
| `散` | 逆包含 + 逆包含 | 扩散 |

### 数据结构详解

#### K线 (`kline::bar`)
```
K线 {
    标识, 序号, 周期, 时间戳,
    高, 低, 开盘价, 收盘价, 成交量,
    指标: RwLock<指标容器>,   // MACD/RSI/KDJ/BOLL/均线
}
```

- `创建普K()` 工厂方法, `相等()` 结构化校验
- `to_bytes()` → 48 字节大端序列化, `读取大端字节数组()` 反序列化
- `获取MACD()` — 两K线间 MACD 柱面积 (阳/阴/合/总)
- 实现 `Clone` (深拷贝, 包括指标容器)

#### 缠论K线 (`kline::chan_kline`)
```
缠论K线 {
    序号: AtomicI64,          时间戳: AtomicI64,
    高: SyncF64,              低: SyncF64,          // 包含处理可能拉高/压低
    方向: RwLock<相对方向>,    分型: RwLock<Option<分型结构>>,
    分型特征值: SyncF64,       // 历史极值
    周期, 标识,               原始起始序号, 原始结束序号: AtomicI64,
    标的K线: RwLock<Arc<K线>>,  买卖点信息: RwLock<HashSet<String>>,
}
```

- `_兼并()` — 顺序/逆序包含合并, 重复提交检测, 原地修改
- `分析()` — 完整普K→缠K+分型管线
- 指标匹配: `与MACD柱子匹配()` / `与RSI匹配()` / `与KDJ匹配()`
- `相等()` — 20+ 字段递归校验

#### 虚线 (`structure::dash_line`) — 笔和线段的通用数据结构
```
虚线 {
    标识: RwLock<String>,        // "笔"/"线段"/"扩展线段"/"线段<线段>" 等
    级别: AtomicI64,             // 笔=1, 线段=2, 递归递增
    文: Arc<分型>,               // 起点（不可变）
    武: RwLock<Arc<分型>>,       // 终点（可变，动态更新）
    基础序列: RwLock<Vec<Arc<虚线>>>, // 子级虚线序列
    实/虚/合_中枢序列,           // 三类中枢序列
    确认K线, 模式, 缺口处理, 短路修正,
    ...
}
```

- 静态工厂: `创建笔()`(级别=1), `创建线段()`(级别递增)
- 属性: `方向()`, `高()`, `低()`, `之前是()`, `之后是()`
- 数据遍历: `获取普K序列()`, `获取缠K序列()`, `获取_武()`(递归到底层笔)
- MACD 分析: `计算MACD柱子均值()`, `统计MACD行为()`, `计算K线序列MACD趋向背驰()`
- 核心判断: `买卖意义()` — LRU 缓存 128 条目

#### 中枢 (`algorithm::hub`)
```
中枢 {
    基础序列: RwLock<Vec<Arc<虚线>>>, // ≥3根, 延伸可达 9+
    第三买卖线: RwLock<Option<Arc<虚线>>>,
    本级_第三买卖线: RwLock<Option<Arc<虚线>>>,
}
```

- `高()` / `低()` — 前三根虚线的最大/最小重叠区域
- `文()` → `武()` → `方向()`
- 中枢延伸 → 扩展 (≥9段) → 第三买卖点确立 → 新中枢开始

#### 基础买卖点 (`business::bsp`)
```
基础买卖点 {
    类型: 买卖点类型,       买卖点分型: Arc<分型>,
    买卖点K线, 当前K线,    失效K线, 终结K线,
    破位值, 结构, 偏移量,
}
```

- `偏移()` — 当前缠K序号与买卖点K线序号的差
- `有效性()` — 失效K线存在则无效
- 18 种类型由 `买卖点工厂` 统一生成, 含背驰确认+指标匹配

### 服务类型

| 类型 | 职责 |
|------|------|
| `观察者` | 单周期流式分析器, 维护全层级序列, 3级递归线段分析 |
| `K线合成器` | 小周期→大周期合成, 高取max/低取min/量求和 |
| `立体分析器` | 多周期联立, 内含合成器+每周期一个观察者 |

### 静态算法类

| 类型 | 职责 |
|------|------|
| `笔` | 分型→笔 (递归+弱化+次级+实际高低点) |
| `线段` | 笔→线段 (特征序列+缺口+四种修正+扩展分析) |
| `中枢` | 虚线→中枢 (重叠+延伸/扩展+第三买卖点) |
| `背驰分析` | MACD/斜率/测度 + 全量/任意/配置/相对组合 |
| `指标计算器` | 增量计算 MACD/RSI/KDJ/BOLL + 回填 + 均线 |

---

## 配置体系

`缠论配置` 是一个 serde 驱动的结构体，**44 个字段**全部带默认值，支持 JSON 往返和部分反序列化容错。

### 完整配置表

#### 基础设置

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `标识` | `String` | `"bar"` | 品种标识 |

#### 缠K设置

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `缠K合并替换` | `bool` | `false` | false=原地修改, true=产出新缠K |

#### 笔设置

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `笔内元素数量` | `i64` | `5` | 成笔最低缠K数(含端点) |
| `笔内相同终点取舍` | `bool` | `false` | false=取first, true=取last |
| `笔内起始分型包含整笔` | `bool` | `false` | 起点分型区间必须包含整笔 |
| `笔内起始分型包含整笔_包括右` | `bool` | `false` | 同上+包含右端点 |
| `笔内原始K线包含整笔` | `bool` | `false` | 原始K线区间必须包含整笔 |
| `笔次级成笔` | `bool` | `false` | 允许在非分型处成笔 |
| `笔弱化` | `bool` | `false` | 放宽成笔条件 |
| `笔弱化_原始数量` | `i64` | `3` | 弱化模式最小原始K线数 |

#### 线段设置

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `线段_非缺口下穿刺` | `bool` | `false` | 非缺口下的穿刺处理 |
| `线段_特征序列忽视老阴老阳` | `bool` | `false` | 缺口时不严格处理包含 |
| `线段_缺口后紧急修正` | `bool` | `true` | 缺口后自动修正 |
| `线段_修正` | `bool` | `false` | 短路修正(≥9笔快速完成) |
| `线段内部中枢图显` | `bool` | `true` | 线段内部中枢图表显示 |
| `扩展线段_当下分析` | `bool` | `false` | 扩展线段实时分析模式 |

#### 分析开关

| 字段 | 类型 | 默认值 |
|------|------|--------|
| `分析笔` | `bool` | `true` |
| `分析线段` | `bool` | `true` |
| `分析扩展线段` | `bool` | `true` |
| `分析笔中枢` | `bool` | `true` |
| `分析线段中枢` | `bool` | `true` |

#### 终止

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `手动终止` | `String` | `""` | 手动终止时间字符串，非空时生效 |

#### 指标设置

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `计算指标` | `bool` | `true` | 是否计算技术指标 |
| `指标计算方式` | `String` | `"收"` | 开/高/低/收/高低均值/高低收均值/开高低收均值 |

#### 指标参数列表（多参数变体）

| 字段 | 类型 | 默认值 |
|------|------|--------|
| `MACD_参数列表` | `Vec<(String, String, i64, i64, i64)>` | `[("macd","收",13,31,11)]` |
| `RSI_周期列表` | `Vec<(String, String, i64, i64, f64, f64)>` | `[("rsi","收",14,13,75,25)]` |
| `KDJ_参数列表` | `Vec<(String, String, i64, i64, i64, f64, f64)>` | `[("kdj","收",13,5,5,80,20)]` |
| `BOLL_参数列表` | `Vec<(String, String, i64, f64)>` | `[("boll","收",20,2.0)]` |
| `均线参数列表` | `Vec<(String, String, String, i64)>` | `[]` |

#### 推送/图表显示

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `图表展示` | `bool` | `true` | 全局图表显示开关 |
| `图表展示标签` | `Option<Vec<String>>` | `None` | None=全部展示, `[]`=不展示 |

#### 买卖点设置

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `买卖点偏移` | `i64` | `1` | 买卖点与当前K线的最大允许偏移 |
| `买卖点激进识别` | `bool` | `false` | 是否激进识别买卖点 |
| `买卖点与MACD柱强相关` | `bool` | `false` | 买卖点须MACD柱确认 |
| `买卖点错过误差值` | `f64` | `0.01` | 价格误差容忍度 |
| `买卖点_指标模式` | `String` | `"配置"` | 任意/配置/全量/相对 |
| `买卖点_指标匹配_MACD` | `bool` | `true` | MACD柱子匹配确认 |
| `买卖点_指标匹配_KDJ` | `bool` | `true` | KDJ匹配确认 |
| `买卖点_指标匹配_RSI` | `bool` | `true` | RSI匹配确认 |

#### 背驰设置

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `线段内部背驰_MACD` | `bool` | `true` | 使用 MACD 背驰 |
| `线段内部背驰_斜率` | `bool` | `true` | 使用斜率背驰 |
| `线段内部背驰_测度` | `bool` | `true` | 使用测度背驰 |
| `线段内部背驰_模式` | `String` | `"相对"` | 任意/配置/全量/相对 |

#### 文件

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `加载文件路径` | `String` | `""` | 数据文件路径 |

### 配置方法

```rust
// JSON 往返
let json = config.to_json();                              // 序列化 → String
let config2 = 缠论配置::from_json(&json)?;                // 反序列化

// to_dict / from_dict（仅 model_fields 中的字段）
let dict = config.to_dict();                              // → serde_json::Value (Object)
let restored = 缠论配置::from_dict(&dict)?;              // 过滤非模型字段，缺失回退默认值

// 文件 I/O
config.保存配置("config.json")?;
let loaded = 缠论配置::加载配置("config.json")?;

// 静默模式
let silent = config.不推送();  // 图表展示 + 线段内部中枢图显 = false

// 差异对比（仅比较 model_fields）
let diffs = config_a.对比(&config_b);  // → HashMap<&str, Value> 仅差异字段

// 部分更新
let updated = config.model_copy(&[("标识", "new_id")].into());  // 深拷贝 + 更新

// 旧版兼容：按序号前缀重组配置字典 ("1_open" → group 1)
let configs = 缠论配置::按序号重组字典(&default_config, &raw_json);
// → Vec<(i64, 缠论配置)>
```

### 非法值容错

三个字符串枚举字段带自定义反序列化器，非法值自动回退默认值并发出 `warn!` 日志：

| 字段 | 合法值 | 回退默认值 |
|------|--------|-----------|
| `指标计算方式` | 开/高/低/收/高低均值/高低收均值/开高低收均值 | `"收"` |
| `买卖点_指标模式` | 任意/配置/全量/相对 | `"配置"` |
| `线段内部背驰_模式` | 任意/配置/全量/相对 | `"相对"` |

---

## 算法模块

### 笔划分 (`algorithm::bi`, ~1,009 行)

| 方法 | 说明 |
|------|------|
| `分析()` | 主算法：从分型序列递归分析生成笔 |
| `_获取缠K数量()` | 计算潜在成笔区域的缠K数（含弱化模式放宽） |
| `_实际高点()` / `_实际低点()` | 潜在成笔区域的最高/最低缠K |
| `_获取成笔核心分型()` | 弱化模式下获取次高/次低成笔的候选分型 |
| `根据缠K找笔()` | 在笔序列中查找包含指定缠K的笔 |
| `是否背驰过()` | 检查笔范围内是否发生过MACD背驰 |

**关键设计**:
- 笔弱化: ≥`笔弱化_原始数量`(默认3)根原始K线即可成笔
- 次级成笔: 次高/次低作为候选终点分型
- 相同终点取舍: false=取first, true=取last (对应 Python `list.index` 行为)
- 笔内验证: 起始分型包含整笔 / 原始K线包含整笔

### 线段划分 (`algorithm::segment`, ~2,078 行)

**最复杂的算法模块**。核心方法：

| 方法 | 说明 |
|------|------|
| `分析()` | 主算法：从笔序列计算线段 |
| `扩展分析()` | 扩展线段分析（不同的缺口处理策略） |
| `分割序列()` | 将特征序列分割为前/后/三/贯穿伤四个子序列 |
| `四象()` | 判断特征序列类型: 老阳/老阴/小阳/少阴 |
| `获取缺口()` | 获取特征序列第一二元素间的缺口 |
| `判断线段内部是否背驰()` | 进入段 vs 离开段背驰判断 |
| `是否背驰过()` | 线段范围内是否发生过背驰 |

**四种修正机制**（按优先级）：
1. **`_缺口突破`** — 老阳/老阴状态 + 价格突破前线段极值 → 线段完成
2. **`_非缺口下穿刺`** — 贯穿伤存在 + 方向相同的3笔 → 线段完成
3. **`_缺口后紧急修正`** — 前一线段有缺口 + 价格反向突破 → 修正结束位置
4. **`_修正`** — 短线路径 (≥9笔) → 快速完成

**3 级递归**: `线段` → `线段<线段>` → `线段<线段<线段>>`，每一级将前级结果作为基础序列。

### 中枢识别 (`algorithm::hub`, ~1,041 行)

| 方法 | 说明 |
|------|------|
| `分析()` | 主算法：虚线→中枢，同时识别第三买卖点 |
| `线段扩展分析()` | 中枢扩展：≥9段时生成扩展中枢 |
| `高()` / `低()` | 前三根虚线重叠区域 |
| `获取离开虚线()` | 找到离开中枢的虚线（确立第三买卖点） |
| `获取第N买卖线()` | 获取第N个买卖点相关的虚线 |

**识别流程**:
1. 基础检查: 三条连续虚线重叠且方向关系正确
2. 中枢延伸: 后继虚线未离开则加入基础序列
3. 第三买卖点: 虚线离开中枢且不回 → 记录 `第三买卖线`
4. 中枢完成: `第三买卖线` 确立后在上方开始新中枢
5. 多级中枢: 笔中枢(级别=1) / 线段中枢(级别=2) / 扩展线段中枢 等

### 背驰检测 (`algorithm::divergence`, ~227 行)

三种检测方式：

| 方式 | 原理 |
|------|------|
| MACD 背驰 | 进入段 vs 离开段的 MACD 柱面积对比 (阳 + |阴| 绝对值和) |
| 斜率背驰 | 价格变化速率 (dy/dx) 减弱 |
| 测度背驰 | 价格-时间向量长度 (√(dx²+dy²)) 缩减 |

四种组合模式: `全量`(全满足) / `任意`(任意满足) / `配置`(按开关) / `相对`(多数投票)

---

## 技术指标

### 统一容器 (`indicators::container`)

注册表模式，通过 `指标值` 枚举统一管理：

```rust
pub enum 指标值 {
    MACD(平滑异同移动平均线),
    RSI(相对强弱指数),
    KDJ(随机指标),
    BOLL(布林带),
    均线(HashMap<String, f64>),
    单值(HashMap<String, f64>),
}
```

- 预注册槽位: `macd`, `rsi`, `kdj`, `boll`, `均线`, `单值`
- 支持多参数变体: `MACD_13_31_11`, `RSI_14`, `KDJ_9_3_3` 等
- 动态注册: `注册(name, default)` / `设置(name, value)` / `获取(name)`
- K线取值 7 种方式: 开/高/低/收/高低均值/高低收均值/开高低收均值

### 指标计算器 (`indicators::calculator`, ~476 行)

`计算并挂载(全序列, 配置)` 单次周期：

1. MACD组 → RSI组 → KDJ组 → BOLL组 → 均线组
2. 首次计算 vs 增量计算自动判定（基于前序K线是否有值）
3. 回填新指标：回溯填充前序K线遗漏的指标

### 各指标结构

**MACD** — EMA(12/26/9) 默认
```
DIF: Option<f64>,  DEA: Option<f64>,  MACD柱: f64
```

**RSI** — Wilder SMA 平滑
```
RSI: Option<f64>,  RSI_SMA: Option<f64>,  超买/超卖阈值
```

**KDJ** — RSV → K → D → J
```
K: Option<f64>,  D: Option<f64>,  J: Option<f64>
```

**BOLL** — SMA ± k·σ
```
中轨: Option<f64>,  上轨: Option<f64>,  下轨: Option<f64>
```

---

## 信号框架

信号框架是缠论分析的上层应用——在已有笔/线段/中枢的基础上，通过**声明式规则匹配**产生交易信号，驱动**仓位状态机**进行多空操作。

### 信号原语

信号框架的 5 种原语（源自 czsc，Apache-2.0，已做中文命名适配与 Rust 重写）：

```
Signal → Factor → Event → Position (含 Operate)
```

#### Signal — 七段键信号

```rust
pub struct Signal {
    pub k1: String,   // 级别 (freq)       e.g. "日线"
    pub k2: String,   // 信号分组           e.g. "D1MO3"
    pub k3: String,   // 信号名             e.g. "中枢第三买卖点V230602"
    pub v1: String,   // 品种              e.g. "三买"
    pub v2: String,   // 标记              e.g. "中枢段DEA穿越2"
    pub v3: String,   // 描述              e.g. "偏移3"
    pub score: i32,   // 0~100
}
```

- `key()` → `"k1_k2_k3"` (三段键，用于去重/查询)
- `value()` → `"v1_v2_v3"` (三段值)
- `is_match(signals: &信号字典)` → 三态: 命中→`Ok(true)`, 不匹配→`Ok(false)`, 缺键→`Err(缺键错误)`
- `空信号` → value=`"任意_任意_任意_0"`, score=0

#### Factor — 因子 = 多信号 all/not 组合

```rust
pub struct Factor {
    pub name: String,           // SHA256前4 + 自定义后缀，确定性
    pub signals_all: Vec<Signal>,   // 必须全部命中
    pub signals_not: Vec<Signal>,   // 必须全部不命中
}
```

`is_match(signals)` → 先检查全部 `signals_all` 命中, 再检查全部 `signals_not` 不命中。

`name` 使用 sha256 前 4 位十六进制，确保 Rust 与 Python 计算**确定性**相同（注意：不保证跨语言字节兼容，但同语言内一致）。

#### Event — 事件 = 因子组 + 操作

```rust
pub struct Event {
    pub name: String,
    pub operate: Operate,           // 匹配时执行的操作
    pub factors: Vec<Factor>,
}
```

`is_match(signals)` → 任一因子命中则为 true（OR 逻辑）。

#### Operate — 操作枚举

```rust
pub enum Operate {
    开多,  // LO — Long Open
    平多,  // LE — Long Exit
    开空,  // SO — Short Open
    平空,  // SE — Short Exit
    持币,  // HO — Hold (无操作)
}
```

实现 `Ord` (LO < SO < LE < SE < HO), `Display` 中文输出。

#### Position — 仓位状态机

Position 是信号框架的最终消费者——将 Event 匹配结果转化为实际的仓位操作：

```rust
pub struct Position {
    // 配置字段
    pub symbol: String,
    pub opens: Vec<Event>,     // 开仓事件
    pub exits: Vec<Event>,     // 平仓事件
    pub events: Vec<Event>,    // 通用事件（先于 opens/exits 匹配）
    pub interval: i64,         // 开仓间隔（秒），0=不限制
    pub timeout: i64,          // 持仓超时（K线数）
    pub stop_loss: i64,        // 止损阈值（BP，1/10000）
    pub T0: bool,              // T+0 模式

    // 状态字段
    pub pos: i32,              // 1=多, -1=空, 0=空仓
    pub pos_changed: bool,     // 本轮是否发生仓位变化
    pub operates: Vec<操作记录>,  // 操作历史
    pub holds: Vec<持仓记录>,     // 持仓快照序列
}
```

**`update(dt, price, bid, signals)` 状态机**（~200 行 Rust，与 Python 1:1 对应）：

1. 时间校验: `dt <= end_dt` → 跳过（信号时间倒退）
2. 事件匹配: 遍历 `events` → `is_match(signals)` → 首个命中即break
3. **开多** (LO): pos≠1 且间隔检查通过 → pos=1; 若 pos=-1 且允许平仓 → pos=0 先平空
4. **平多** (LE): pos=1 且允许操作 → pos=0
5. **开空** (SO): 对称
6. **平空** (SE): 对称
7. **止损检查**: 多头→`price/last_price-1 < -stop_loss/10000`; 空头→`1-price/last_price < -stop_loss/10000`
8. **超时检查**: `bid - last_bid > timeout`
9. 追加持仓快照 `{dt, pos, price}`

**`pairs()` 盈亏计算** — 从 `operates` 中配对开平操作，计算每对的 BP（基点）盈亏：

```
多头盈亏 = (平仓价/开仓价 - 1) * 10000
空头盈亏 = (1 - 平仓价/开仓价) * 10000
```

**辅助判断**:
- `同一交易日(a, b)` — chrono 日期比较
- `间隔检查(last_dt, dt, interval)` — 距上次开仓是否超过 interval 秒
- `允许操作(T0, dt, last_dt)` — T+0 或不同交易日

### 信号注册表 (`signal::registry`)

双注册表架构，支持**编译时注册**和**运行时动态注册**：

```
┌──────────────────────────────────────┐
│              获取信号                  │
│         get_signal(name)              │
│         ├─ 1. 检查 SIGNAL_REGISTRY   │  ← LazyLock<HashMap> (编译时, #[signal] 宏)
│         └─ 2. 检查 DYNAMIC_REGISTRY  │  ← RwLock<HashMap> (运行时, 动态注册)
└──────────────────────────────────────┘
```

**编译时注册** — `#[signal]` 属性宏：

```rust
use chanlun_signal_macros::signal;

#[signal(
    name = "my_custom_signal_V230101",
    template = "{freq}_D1MO{max_overlap}_my_custom_signalV230101"
)]
fn my_custom_signal(obs: &观察者, params: &HashMap<String, Value>) -> Vec<Signal> {
    // 信号逻辑...
    vec![Signal::new("日线", "D1", "my_signalV230101", "三买", "", "", 80)]
}
```

宏在编译期生成 `SignalDescriptor { name, template, func }` 并通过 `inventory::submit!` 注册。

**动态注册** — 运行时 API：

```rust
use chanlun::signal::registry::{register_signal, unregister_signal, list_signal_names};

// 注册
register_signal(
    "my_signal_V230101",
    "{freq}_templateV230101",
    my_signal_fn,  // SignalFn
)?;

// 注销
unregister_signal("my_signal_V230101")?;

// 列出所有已注册信号（编译时 + 动态）
let names: Vec<String> = list_signal_names();
```

**约束**: 动态注册不能覆盖编译时信号（返回 Err）；同名动态信号重复注册也报错。

### 信号引擎 (`signal::engine`)

`SignalEngine` 批量执行已注册的信号函数：

```rust
pub struct SignalEngine {
    configs: Vec<SignalConfig>,  // [{signal_name, freq, params}]
}
```

- `自动挂载指标(分析器)` — 扫描所有 config 中的 MACD/均线关键词，为各周期 observer 自动添加所需指标参数
- `更新(分析器)` → `HashMap<String, String>` — 遍历配置，按 freq 聚合，调用注册表函数，过滤空信号
- `更新_完整(分析器)` → `完整更新结果 { signals, market }` — 附加基础周期的 OHLCV 行情数据

### 已实现的信号函数

| 信号名 | 来源 | 功能 |
|--------|------|------|
| `youwukuncheng_中枢第三买卖点_V230602` | `functions/youwukuncheng.rs` | 中枢第三买卖点 (3种变体: DEA穿越/首次穿越0轴+分型确认) |
| `bar_zdt_V230331` | `functions/demo.rs` | 涨跌停检测 |
| `macd_金叉_V260601` | `functions/demo.rs` | MACD 金叉/死叉 |
| `tas_macd_direct_V221106` | `functions/demo.rs` | MACD DIF 方向 |
| `tas_ma_base_V230313` | `functions/demo.rs` | MA 均线多空 (SMA/EMA) |
| `cxt_停顿分型_V230106` | `functions/demo.rs` | 停顿分型检测 |
| `cxt_bi_end_V230222` | `functions/demo.rs` | 笔结束辅助 (~90行) |

---

## 插件系统

支持通过 `.so` 动态库在运行时加载第三方 Rust 信号函数。

### C-ABI 导出 (`signal::ffi`)

宿主进程导出三个 C-ABI 函数供 `.so` 插件调用：

```rust
// 注册信号 → 0=成功, 非0=失败
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chanlun_register_signal(
    name: *const c_char, template: *const c_char, func: SignalFn,
) -> i32;

// 注销信号
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chanlun_unregister_signal(name: *const c_char) -> i32;

// 查询信号总数
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chanlun_list_signal_count() -> i32;
```

### 插件编写

**方式 A: 手动注册** — 在插件的构造函数中直接调用 `chanlun_register_signal`：

```rust
#[unsafe(no_mangle)]
pub unsafe extern "C" fn init_plugin() -> i32 {
    chanlun_register_signal(
        "my_signal\0".as_ptr() as *const c_char,
        "template\0".as_ptr() as *const c_char,
        my_signal_fn,
    )
}
```

**方式 B: `#[signal]` 宏** — 使用 `crate_path = "::chanlun"` 参数在外部 crate 中使用宏注册：

```rust
use chanlun_signal_macros::signal;

#[signal(
    name = "plugin_signal_V230101",
    template = "{freq}_pluginV230101",
    crate_path = "::chanlun"  // 关键：外部 crate 需指定路径
)]
fn plugin_signal(obs: &观察者, params: &HashMap<String, Value>) -> Vec<Signal> {
    // ...
}
```

然后在 `init_plugin()` 中遍历 `inventory::iter::<SignalDescriptor>` 批量提交。

### Python 端加载

```python
import ctypes, os

# 设置全局符号可见性
sys.setdlopenflags(os.RTLD_LAZY | os.RTLD_GLOBAL)

# 加载插件
plugin = ctypes.CDLL("./libmy_plugin.so")
plugin.init_plugin()  # 内部调用 chanlun_register_signal

# 此后可通过 call_signal 或 SignalEngine 使用插件信号
from chanlun._chanlun import call_signal, list_signals
print(list_signals())  # 包含插件注册的信号
```

**约束**: 插件和宿主必须用**相同 Rust 编译器版本**编译，因为 `SignalFn` 使用 Rust 调用约定。

---

## 线程安全与并发

### 内部可变性策略

| 机制 | 底层 | 适用场景 |
|------|------|---------|
| `SyncF64` | `AtomicU64` 位转换 | 价格字段 (高/低/分型特征值) |
| `AtomicI64` | `AtomicI64` | 序号/时间戳 |
| `AtomicBool` | `AtomicBool` | 有效性/短路修正/开关 |
| `RwLock<T>` | `parking_lot::RwLock` | 复杂数据 (Arc<分型>/Vec/中枢序列) |

### Send + Sync

所有核心数据结构（`缠论K线`, `虚线`, `中枢`, `观察者`）均实现 `Send + Sync`，可安全跨线程：

```rust
// 编译期断言
fn _要求_Send_Sync<T: Send + Sync>() {}
_要求_Send_Sync::<缠论K线>();
_要求_Send_Sync::<虚线>();
_要求_Send_Sync::<中枢>();
_要求_Send_Sync::<观察者>();
```

`观察者` 使用 `Arc<RwLock<观察者>>` 模式——多线程并发读取同一观察者。

### 全局缓存

- **买卖意义缓存**: `LazyLock<Mutex<LruCache<(usize, usize), (bool, String)>>>` — 128 条目
- Python 端使用 `@lru_cache` 装饰器，Rust 使用 `LazyLock<DashMap>`（替代 `thread_local!` 解决跨线程不可见问题）

### RwLock 死锁预防

Rust `RwLock` 是**非递归锁**——在持读锁时尝试获取写锁会死锁。解决：将读锁作用域化，在所有写锁调用前释放：

```rust
{ let prev_guard = 前序K线.指标.read(); /* 读取 */ }
// ↑ 作用域结束 → 释放读锁
Self::_回填新指标(...);  // 安全：此时无读锁
```

---

## Python 绑定

通过 [`chanlun-py`](../chanlun-py/) crate（PyO3）将 Rust 核心导出为 Python `chanlun` 模块。

```bash
cd chanlun-py
maturin develop --release
```

```python
import chanlun

# 配置
config = chanlun.缠论配置()
config.笔内元素数量 = 7
config.MACD_参数列表 = [("my_macd", "收", 12, 26, 9)]

# 观察者
obs = chanlun.观察者("BTCUSD", 3600, config)
obs.增加原始K线(k)

# 多周期分析器
analyzer = chanlun.立体分析器("BTCUSD", [60, 300, 1800], config, None)

# 信号计算
from chanlun._chanlun import 信号引擎, call_signal, list_signals

引擎 = 信号引擎([
    {"name": "youwukuncheng_中枢第三买卖点_V230602",
     "freq": 86400, "max_overlap": 3,
     "本级完整性": "实", "同级完整性": "合"},
])
引擎.自动挂载指标(analyzer)
result = 引擎.更新_完整(analyzer)  # → {"signals": {...}, "market": {...}}

# 单独调用信号
signals = call_signal("macd_金叉_V260601", obs, {"freq": "日线"})

# 查询注册表
print(list_signals())  # → ["youwukuncheng_...", "macd_金叉_...", ...]
```

### 导出类映射

| Rust 类型 | Python 类 | 说明 |
|-----------|----------|------|
| `缠论配置` | `chanlun.缠论配置` | 配置管理 |
| `K线` | `chanlun.K线` | 原始 K 线 |
| `缠论K线` | `chanlun.缠论K线` | 缠论 K 线 |
| `分型` | `chanlun.分型` | 顶底分型 |
| `虚线` | `chanlun.虚线` | 笔/线段 |
| `线段特征` | `chanlun.线段特征` | 特征序列元素 |
| `中枢` | `chanlun.中枢` | 中枢 |
| `基础买卖点` | `chanlun.基础买卖点` | 买卖点 |
| `观察者` | `chanlun.观察者` | 单周期分析器 |
| `立体分析器` | `chanlun.立体分析器` | 多周期分析器 |
| `Signal` | `chanlun._chanlun.Signal` | 信号 |
| `Factor` | `chanlun._chanlun.Factor` | 因子 |
| `Event` | `chanlun._chanlun.Event` | 事件 |
| `Position` | `chanlun._chanlun.Position` | 仓位状态机 |
| `Operate` | `chanlun._chanlun.Operate` | 操作枚举 |
| `信号引擎` | `chanlun._chanlun.信号引擎` | 信号计算引擎 |

---

## 数据序列化

### K线二进制格式 (.nb 文件)

与 Python `struct.pack(">6d")` 完全兼容，每根 K 线 **48 字节**：

| 偏移 | 大小 | 字段 | 字节序 |
|------|------|------|--------|
| 0 | 8 B | 时间戳 (Unix秒 as f64) | Big Endian |
| 8 | 8 B | 开盘价 | Big Endian |
| 16 | 8 B | 最高价 | Big Endian |
| 24 | 8 B | 最低价 | Big Endian |
| 32 | 8 B | 收盘价 | Big Endian |
| 40 | 8 B | 成交量 | Big Endian |

```rust
let bytes: [u8; 48] = k线.to_bytes();
let k线 = K线::读取大端字节数组(&bytes, 周期, 标识)?;
K线::保存到DAT文件("output.dat", &[&k1, &k2])?;
```

### 配置 JSON

```json
{
  "标识": "BTCUSD",
  "笔内元素数量": 5,
  "MACD_参数列表": [["macd", "收", 13, 31, 11]],
  "买卖点_指标模式": "配置",
  "线段内部背驰_模式": "相对"
}
```

- `#[serde(default)]` → 前向/后向兼容，缺失字段自动回退默认值
- `from_dict()` 自动过滤非 `model_fields` 字段

### 结构化相等校验

所有核心结构体实现 `相等(&self, other, 浮点容差) → (bool, String)`:
- 浮点字段容差比较（非直接 `==`）
- `Arc` 共享结构深度校验值语义
- 嵌套容器 (Vec/Option/HashMap) 逐一比对
- 返回详细差异描述

---

## 测试

### Rust 核心测试

```bash
cd chanlun
cargo test                        # 199 项测试
cargo test -- test_50线程         # 匹配模式的测试
cargo clippy                      # 零警告
```

**测试覆盖**:
- **配置**: 默认值 / JSON 往返 / 部分反序列化 / 非法值回退 / 不推送 / 差异对比
- **K线**: 方向判定 / 大端序列化往返
- **缠K**: 创建/包含合并/分型识别
- **笔**: 基本分析 / 弱化 / 次级成笔 / 实际高低点
- **线段**: 四种象 / 缺口 / 分割序列 / 特征序列状态
- **中枢**: 字段读写 / Clone 后指针一致 / 延伸/扩展 / 第三买卖点
- **买卖点**: 18 种类型生成 / 偏移/失效/有效性
- **指标**: MACD/RSI/KDJ/BOLL 首次+增量计算
- **观察者**: 指针一致性 / 重复计算确定性 / 重置后一致 / RefCell 安全 / 跨线程 Send/Sync
- **Position**: 28 项单元测试 — LO/SO/LE/SE 转换 / 止损/超时/间隔 / 盈亏计算
- **Registry**: 编译时/动态注册 / 归并/注销 / 冲突报错 / #[signal] 宏端到端
- **并发**: 50 线程 10,000 K线压测

### Python 集成测试

```bash
cd chanlun-py
python -m pytest tests/test_all.py -v       # 107+ 项
python -m pytest tests/test_signal_primitives.py -v
python -m pytest tests/test_position_update.py -v  # 24 项
```

---

## 许可

MIT License. 详见 [LICENSE](LICENSE).

---

## 相关项目

- [`chan.py`](../chan.py) — Python 参考实现（~4,200 行），作为跨校验基准
- [`chanlun-py`](../chanlun-py/) — PyO3 绑定，发布为 `chanlun` PyPI 包
- [`chanlun-signal-macros`](../chanlun-signal-macros/) — `#[signal]` 属性宏 proc-macro crate
- [`main.py`](../main.py) — FastAPI Web 应用（WebSocket 实时图表 + 回测）
- [`strategies.py`](../strategies.py) — 回测策略定义（Backtrader 集成）
- [`examples/plugin-demo`](../examples/plugin-demo/) — .so 动态插件示例（两种注册方式）
