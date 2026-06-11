/*
 * MIT License
 *
 * Copyright (c) 2026 YuYuKunKun
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

use crate::warn;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

/// 缠论配置 —— 控制所有分析阶段的行为
///
/// 50+ 参数集中控制缠K合并、笔/线段划分、中枢识别、买卖点生成等所有阶段。
/// 所有字段带默认值，使用 `#[serde(default)]` 实现缺失字段容错。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct 缠论配置 {
    // ---- 基础 ----
    /// 品种标识（如 "btcusd"）
    pub 标识: String,

    // ---- 缠K ----
    /// 包含处理时使用合并替换模式（而非添加模式）
    pub 缠K合并替换: bool,

    // ---- 笔 ----
    /// 笔内最少缠K数量（含端点）
    pub 笔内元素数量: i64,
    /// 笔内相同终点取舍开关
    pub 笔内相同终点取舍: bool,
    /// 笔内起始分型包含整笔
    pub 笔内起始分型包含整笔: bool,
    /// 笔内起始分型包含整笔（含右端点）
    pub 笔内起始分型包含整笔_包括右: bool,
    /// 笔内原始K线包含整笔
    pub 笔内原始K线包含整笔: bool,
    /// 笔次级成笔（允许在非分型处成笔）
    pub 笔次级成笔: bool,
    /// 笔弱化开关（允许更少元素成笔）
    pub 笔弱化: bool,
    /// 笔弱化模式下的最小原始K线数
    pub 笔弱化_原始数量: i64,

    // ---- 线段 ----
    /// 线段非缺口下的穿刺处理
    pub 线段_非缺口下穿刺: bool,
    /// 线段特征序列忽略老阴老阳
    pub 线段_特征序列忽视老阴老阳: bool,
    /// 线段缺口后紧急修正
    pub 线段_缺口后紧急修正: bool,
    /// 线段修正开关
    pub 线段_修正: bool,
    /// 线段内部中枢图显示
    pub 线段内部中枢图显: bool,
    /// 扩展线段当下分析模式
    pub 扩展线段_当下分析: bool,

    // ---- 分析开关 ----
    /// 是否分析笔
    pub 分析笔: bool,
    /// 是否分析线段
    pub 分析线段: bool,
    /// 是否分析扩展线段
    pub 分析扩展线段: bool,
    /// 是否分析笔中枢
    pub 分析笔中枢: bool,
    /// 是否分析线段中枢
    pub 分析线段中枢: bool,

    // ---- 终止 ----
    /// 手动终止时间（时间字符串，非空时生效）
    pub 手动终止: String,

    // ---- 指标 ----
    /// 是否计算技术指标
    pub 计算指标: bool,
    /// 是否计算布林带
    pub 计算BOLL: bool,
    /// 指标计算方式（开/高/低/收/高低均值/高低收均值/开高低收均值）
    #[serde(deserialize_with = "deserialize_指标计算方式")]
    pub 指标计算方式: String,

    // ---- MACD ----
    /// MACD 快线 EMA 周期
    pub 平滑异同移动平均线_快线周期: i64,
    /// MACD 慢线 EMA 周期
    pub 平滑异同移动平均线_慢线周期: i64,
    /// MACD 信号线周期
    pub 平滑异同移动平均线_信号周期: i64,
    /// MACD 多参数列表: Vec<(key, 快线, 慢线, 信号)>
    #[serde(default)]
    pub MACD_参数列表: Vec<(String, i64, i64, i64)>,

    // ---- RSI ----
    /// RSI 计算周期
    pub 相对强弱指数_周期: i64,
    /// RSI SMA 平滑周期
    pub 相对强弱指数_移动平均线周期: i64,
    /// RSI 超买阈值
    pub 相对强弱指数_超买阈值: f64,
    /// RSI 超卖阈值
    pub 相对强弱指数_超卖阈值: f64,
    /// RSI 多周期列表: Vec<(key, 周期)>
    #[serde(default)]
    pub RSI_周期列表: Vec<(String, i64)>,

    // ---- KDJ ----
    /// KDJ RSV 周期
    pub 随机指标_RSV周期: i64,
    /// KDJ K 值平滑周期
    pub 随机指标_K值平滑周期: i64,
    /// KDJ D 值平滑周期
    pub 随机指标_D值平滑周期: i64,
    /// KDJ 超买阈值
    pub 随机指标_超买阈值: f64,
    /// KDJ 超卖阈值
    pub 随机指标_超卖阈值: f64,
    /// KDJ 多参数列表: Vec<(key, RSV周期, K平滑, D平滑)>
    #[serde(default)]
    pub KDJ_参数列表: Vec<(String, i64, i64, i64)>,

    // ---- BOLL ----
    /// 布林带周期
    pub 布林带_周期: i64,
    /// 布林带标准差倍数
    pub 布林带_标准差倍数: f64,
    /// BOLL 多参数列表: Vec<(key, 周期, 标准差倍数)>
    #[serde(default)]
    pub BOLL_参数列表: Vec<(String, i64, f64)>,

    // ---- 均线 ----
    /// 均线类型列表: ["SMA", "EMA", ...]
    #[serde(default)]
    pub 均线_类型列表: Vec<String>,
    /// 均线周期列表: [5, 10, 20, ...]
    #[serde(default)]
    pub 均线_周期列表: Vec<i64>,

    // ---- 推送/显示 ----
    /// 是否启用图表展示
    pub 图表展示: bool,
    /// 是否推送K线
    pub 推送K线: bool,
    /// 是否推送笔
    pub 推送笔: bool,
    /// 是否推送线段
    pub 推送线段: bool,
    /// 是否推送中枢
    pub 推送中枢: bool,

    // ---- 图表展示细分 ----
    /// 图表展示笔
    pub 图表展示_笔: bool,
    /// 图表展示线段
    pub 图表展示_线段: bool,
    /// 图表展示扩展线段
    pub 图表展示_扩展线段: bool,
    /// 图表展示扩展线段（线段级）
    pub 图表展示_扩展线段_线段: bool,
    /// 图表展示线段之线段
    pub 图表展示_线段_线段: bool,
    /// 图表展示笔中枢
    pub 图表展示_中枢_笔: bool,
    /// 图表展示线段中枢
    pub 图表展示_中枢_线段: bool,
    /// 图表展示扩展中枢
    pub 图表展示_中枢_扩展线段: bool,
    /// 图表展示扩展中枢（线段级）
    pub 图表展示_中枢_扩展线段_线段: bool,
    /// 图表展示线段之中枢
    pub 图表展示_中枢_线段_线段: bool,
    /// 图表展示线段内部中枢
    pub 图表展示_中枢_线段内部: bool,

    // ---- 买卖点 ----
    /// 买卖点偏移量
    pub 买卖点偏移: i64,
    /// 买卖点激进识别模式
    pub 买卖点激进识别: bool,
    /// 买卖点与MACD柱强相关
    pub 买卖点与MACD柱强相关: bool,
    /// 买卖点错过误差值
    pub 买卖点错过误差值: f64,
    /// 买卖点指标模式（任意/配置/全量/相对）
    #[serde(deserialize_with = "deserialize_买卖点_指标模式")]
    pub 买卖点_指标模式: String,
    /// 买卖点指标匹配 MACD
    pub 买卖点_指标匹配_MACD: bool,
    /// 买卖点指标匹配 KDJ
    pub 买卖点_指标匹配_KDJ: bool,
    /// 买卖点指标匹配 RSI
    pub 买卖点_指标匹配_RSI: bool,

    // ---- 背驰 ----
    /// 线段内部背驰使用 MACD
    pub 线段内部背驰_MACD: bool,
    /// 线段内部背驰使用斜率
    pub 线段内部背驰_斜率: bool,
    /// 线段内部背驰使用测度
    pub 线段内部背驰_测度: bool,
    /// 线段内部背驰模式（任意/配置/全量/相对）
    #[serde(deserialize_with = "deserialize_线段内部背驰_模式")]
    pub 线段内部背驰_模式: String,

    // ---- 文件 ----
    /// 加载数据文件路径
    pub 加载文件路径: String,
}

fn deserialize_指标计算方式<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    const VALID: &[&str] = &[
        "开",
        "高",
        "低",
        "收",
        "高低均值",
        "高低收均值",
        "开高低收均值",
    ];
    const DEFAULT: &str = "收";
    if VALID.contains(&s.as_str()) {
        Ok(s)
    } else {
        warn!(
            "[配置警告] 指标计算方式: \"{s}\" 不在有效值 {VALID:?} 内，已使用默认值 \"{DEFAULT}\""
        );
        Ok(DEFAULT.to_string())
    }
}

fn deserialize_买卖点_指标模式<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    const VALID: &[&str] = &["任意", "配置", "全量", "相对"];
    const DEFAULT: &str = "配置";
    if VALID.contains(&s.as_str()) {
        Ok(s)
    } else {
        warn!(
            "[配置警告] 买卖点_指标模式: \"{s}\" 不在有效值 {VALID:?} 内，已使用默认值 \"{DEFAULT}\""
        );
        Ok(DEFAULT.to_string())
    }
}

fn deserialize_线段内部背驰_模式<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    const VALID: &[&str] = &["任意", "配置", "全量", "相对"];
    const DEFAULT: &str = "相对";
    if VALID.contains(&s.as_str()) {
        Ok(s)
    } else {
        warn!(
            "[配置警告] 线段内部背驰_模式: \"{s}\" 不在有效值 {VALID:?} 内，已使用默认值 \"{DEFAULT}\""
        );
        Ok(DEFAULT.to_string())
    }
}

impl Default for 缠论配置 {
    fn default() -> Self {
        Self {
            标识: "bar".into(),
            缠K合并替换: false,
            笔内元素数量: 5,
            笔内相同终点取舍: false,
            笔内起始分型包含整笔: false,
            笔内起始分型包含整笔_包括右: false,
            笔内原始K线包含整笔: false,
            笔次级成笔: false,
            笔弱化: false,
            笔弱化_原始数量: 3,
            线段_非缺口下穿刺: false,
            线段_特征序列忽视老阴老阳: false,
            线段_缺口后紧急修正: true,
            线段_修正: false,
            线段内部中枢图显: true,
            扩展线段_当下分析: false,
            分析笔: true,
            分析线段: true,
            分析扩展线段: true,
            分析笔中枢: true,
            分析线段中枢: true,
            手动终止: String::new(),
            计算指标: true,
            计算BOLL: false,
            指标计算方式: "收".into(),
            平滑异同移动平均线_快线周期: 13,
            平滑异同移动平均线_慢线周期: 31,
            平滑异同移动平均线_信号周期: 11,
            相对强弱指数_周期: 13,
            相对强弱指数_移动平均线周期: 13,
            相对强弱指数_超买阈值: 75.0,
            相对强弱指数_超卖阈值: 25.0,
            随机指标_RSV周期: 13,
            随机指标_K值平滑周期: 5,
            随机指标_D值平滑周期: 5,
            随机指标_超买阈值: 80.0,
            随机指标_超卖阈值: 20.0,
            MACD_参数列表: Vec::new(),
            RSI_周期列表: Vec::new(),
            KDJ_参数列表: Vec::new(),
            布林带_周期: 20,
            布林带_标准差倍数: 2.0,
            BOLL_参数列表: Vec::new(),
            均线_类型列表: Vec::new(),
            均线_周期列表: Vec::new(),
            图表展示: true,
            推送K线: true,
            推送笔: true,
            推送线段: true,
            推送中枢: true,
            图表展示_笔: true,
            图表展示_线段: true,
            图表展示_扩展线段: true,
            图表展示_扩展线段_线段: true,
            图表展示_线段_线段: true,
            图表展示_中枢_笔: true,
            图表展示_中枢_线段: true,
            图表展示_中枢_扩展线段: true,
            图表展示_中枢_扩展线段_线段: true,
            图表展示_中枢_线段_线段: true,
            图表展示_中枢_线段内部: true,
            买卖点偏移: 1,
            买卖点激进识别: false,
            买卖点与MACD柱强相关: false,
            买卖点错过误差值: 0.01,
            买卖点_指标模式: "配置".into(),
            买卖点_指标匹配_MACD: true,
            买卖点_指标匹配_KDJ: true,
            买卖点_指标匹配_RSI: true,
            线段内部背驰_MACD: true,
            线段内部背驰_斜率: true,
            线段内部背驰_测度: true,
            线段内部背驰_模式: "相对".into(),
            加载文件路径: String::new(),
        }
    }
}

impl 缠论配置 {
    /// 解析MACD参数列表 — 如果列表非空则使用列表，否则返回默认单组
    pub fn _解析MACD参数列表(&self) -> Vec<(String, i64, i64, i64)> {
        if !self.MACD_参数列表.is_empty() {
            return self.MACD_参数列表.clone();
        }
        vec![(
            "macd".into(),
            self.平滑异同移动平均线_快线周期,
            self.平滑异同移动平均线_慢线周期,
            self.平滑异同移动平均线_信号周期,
        )]
    }

    /// 解析RSI周期列表 — 如果列表非空则使用列表，否则返回默认单组
    pub fn _解析RSI周期列表(&self) -> Vec<(String, i64)> {
        if !self.RSI_周期列表.is_empty() {
            return self.RSI_周期列表.clone();
        }
        vec![("rsi".into(), self.相对强弱指数_周期)]
    }

    /// 解析KDJ参数列表 — 如果列表非空则使用列表，否则返回默认单组
    pub fn _解析KDJ参数列表(&self) -> Vec<(String, i64, i64, i64)> {
        if !self.KDJ_参数列表.is_empty() {
            return self.KDJ_参数列表.clone();
        }
        vec![(
            "kdj".into(),
            self.随机指标_RSV周期,
            self.随机指标_K值平滑周期,
            self.随机指标_D值平滑周期,
        )]
    }

    /// 解析BOLL参数列表 — 如果列表非空则使用列表，否则返回默认单组
    pub fn _解析BOLL参数列表(&self) -> Vec<(String, i64, f64)> {
        if !self.BOLL_参数列表.is_empty() {
            return self.BOLL_参数列表.clone();
        }
        vec![("boll".into(), self.布林带_周期, self.布林带_标准差倍数)]
    }

    /// 序列化为 JSON 字典（对应 Python to_dict，仅返回 model_fields 中的字段）
    pub fn to_dict(&self) -> serde_json::Value {
        let full = serde_json::to_value(self).unwrap_or_default();
        let valid = Self::model_fields();
        if let serde_json::Value::Object(map) = full {
            let filtered: serde_json::Map<_, _> = map
                .into_iter()
                .filter(|(k, _)| valid.contains(&k.as_str()))
                .collect();
            serde_json::Value::Object(filtered)
        } else {
            full
        }
    }

    /// 从 JSON 字典反序列化（对应 Python from_dict / 兼容旧版本配置）
    pub fn from_dict(value: &serde_json::Value) -> Result<Self, serde_json::Error> {
        if let serde_json::Value::Object(map) = value {
            let valid_fields = Self::model_fields();
            let cleaned: serde_json::Map<_, _> = map
                .iter()
                .filter(|(k, _)| valid_fields.contains(&k.as_str()))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            serde_json::from_value(serde_json::Value::Object(cleaned))
        } else {
            serde_json::from_value(value.clone())
        }
    }

    /// 验证并修正字段值（对应 Python _validate_all_fields）
    pub fn _validate_all_fields(&mut self) {
        const 允许: &[&str] = &[
            "开",
            "高",
            "低",
            "收",
            "高低均值",
            "高低收均值",
            "开高低收均值",
        ];
        if !允许.contains(&self.指标计算方式.as_str()) {
            warn!(
                "[指标计算方式] = {} 值不在允许范围内，使用默认值：收",
                self.指标计算方式
            );
            self.指标计算方式 = "收".into();
        }
    }

    /// 返回字段名列表（对应 Python model_fields().keys()）
    pub fn model_fields() -> &'static [&'static str] {
        &[
            "标识",
            "缠K合并替换",
            "笔内元素数量",
            "笔内相同终点取舍",
            "笔内起始分型包含整笔",
            "笔内起始分型包含整笔_包括右",
            "笔内原始K线包含整笔",
            "笔次级成笔",
            "笔弱化",
            "笔弱化_原始数量",
            "线段_非缺口下穿刺",
            "线段_特征序列忽视老阴老阳",
            "线段_缺口后紧急修正",
            "线段_修正",
            "线段内部中枢图显",
            "扩展线段_当下分析",
            "分析笔",
            "分析线段",
            "分析扩展线段",
            "分析笔中枢",
            "分析线段中枢",
            "手动终止",
            "计算指标",
            "计算BOLL",
            "指标计算方式",
            "平滑异同移动平均线_快线周期",
            "平滑异同移动平均线_慢线周期",
            "平滑异同移动平均线_信号周期",
            "相对强弱指数_周期",
            "相对强弱指数_移动平均线周期",
            "相对强弱指数_超买阈值",
            "相对强弱指数_超卖阈值",
            "随机指标_RSV周期",
            "随机指标_K值平滑周期",
            "随机指标_D值平滑周期",
            "随机指标_超买阈值",
            "随机指标_超卖阈值",
            "布林带_周期",
            "布林带_标准差倍数",
            "MACD_参数列表",
            "RSI_周期列表",
            "KDJ_参数列表",
            "BOLL_参数列表",
            "均线_类型列表",
            "均线_周期列表",
            "图表展示",
            "推送K线",
            "推送笔",
            "推送线段",
            "推送中枢",
            "图表展示_笔",
            "图表展示_线段",
            "图表展示_扩展线段",
            "图表展示_扩展线段_线段",
            "图表展示_线段_线段",
            "图表展示_中枢_笔",
            "图表展示_中枢_线段",
            "图表展示_中枢_扩展线段",
            "图表展示_中枢_扩展线段_线段",
            "图表展示_中枢_线段_线段",
            "图表展示_中枢_线段内部",
            "买卖点偏移",
            "买卖点激进识别",
            "买卖点与MACD柱强相关",
            "买卖点错过误差值",
            "买卖点_指标模式",
            "买卖点_指标匹配_MACD",
            "买卖点_指标匹配_KDJ",
            "买卖点_指标匹配_RSI",
            "线段内部背驰_MACD",
            "线段内部背驰_斜率",
            "线段内部背驰_测度",
            "线段内部背驰_模式",
            "加载文件路径",
        ]
    }

    /// 深拷贝并更新指定字段（对应 Python model_copy(update={...}, deep=True)）
    pub fn model_copy(&self, update: &HashMap<String, serde_json::Value>) -> Self {
        let mut value = serde_json::to_value(self).unwrap_or_default();
        if let serde_json::Value::Object(ref mut map) = value {
            for (k, v) in update {
                map.insert(k.clone(), v.clone());
            }
        }
        serde_json::from_value(value).unwrap_or_else(|_| self.clone())
    }

    /// 序列化为 JSON 字符串
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }

    /// 从 JSON 字符串反序列化
    pub fn from_json(json_str: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json_str)
    }

    /// 保存配置到 JSON 文件
    pub fn 保存配置(&self, path: &str) -> std::io::Result<()> {
        std::fs::write(path, self.to_json())
    }

    /// 从 JSON 文件加载配置
    pub fn 加载配置(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config = Self::from_json(&content)?;
        Ok(config)
    }

    /// 返回一个关闭所有推送/显示的新配置（对应 Python 不推送）
    pub fn 不推送(&self) -> Self {
        Self {
            线段内部中枢图显: false,
            图表展示: false,
            推送K线: false,
            推送笔: false,
            推送线段: false,
            推送中枢: false,
            图表展示_笔: false,
            图表展示_线段: false,
            图表展示_扩展线段: false,
            图表展示_扩展线段_线段: false,
            图表展示_线段_线段: false,
            图表展示_中枢_笔: false,
            图表展示_中枢_线段: false,
            图表展示_中枢_扩展线段: false,
            图表展示_中枢_扩展线段_线段: false,
            图表展示_中枢_线段_线段: false,
            图表展示_中枢_线段内部: false,
            ..self.clone()
        }
    }

    /// 按序号重组字典 — 兼容旧版配置的复合key格式
    pub fn 按序号重组字典(
        默认配置: &Self,
        原始字典: &serde_json::Value,
    ) -> Vec<(i64, Self)> {
        let mut result = Vec::new();
        if let serde_json::Value::Object(map) = 原始字典 {
            // 按数字前缀分组: "1_open" → group 1 key "open"
            let mut groups: std::collections::BTreeMap<
                i64,
                serde_json::Map<String, serde_json::Value>,
            > = std::collections::BTreeMap::new();
            for (key, value) in map {
                if let Some(pos) = key.find('_')
                    && let Ok(num) = key[..pos].parse::<i64>()
                {
                    let field = key[pos + 1..].to_string();
                    groups.entry(num).or_default().insert(field, value.clone());
                }
            }
            for (num, fields) in groups {
                let mut config = 默认配置.clone();
                if let Ok(partial) =
                    serde_json::from_value::<缠论配置>(serde_json::Value::Object(fields))
                {
                    // merge partial into config (override matching fields)
                    config = partial;
                }
                result.push((num, config));
            }
        }
        result
    }

    /// 对比两个配置，返回差异字段及新值（对应 Python 对比 → dict[字段名, 新值]）
    pub fn 对比(&self, other: &Self) -> HashMap<String, serde_json::Value> {
        let mut diffs = HashMap::new();
        let self_dict = self.to_dict();
        let other_dict = other.to_dict();
        if let (serde_json::Value::Object(self_map), serde_json::Value::Object(other_map)) =
            (&self_dict, &other_dict)
        {
            for key in Self::model_fields() {
                let self_val = self_map.get(*key);
                let other_val = other_map.get(*key);
                if self_val != other_val
                    && let Some(v) = other_val
                {
                    diffs.insert(key.to_string(), v.clone());
                }
            }
        }
        diffs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_roundtrip() {
        let config = 缠论配置::default();
        let json = config.to_json();
        let parsed: 缠论配置 = 缠论配置::from_json(&json).unwrap();
        let json2 = parsed.to_json();
        assert_eq!(json, json2);
    }

    #[test]
    fn test_default_values() {
        let config = 缠论配置::default();
        assert_eq!(config.标识, "bar");
        assert_eq!(config.笔内元素数量, 5);
        assert_eq!(config.指标计算方式, "收");
    }

    #[test]
    fn test_partial_deserialize() {
        let json = r#"{"标识": "custom", "笔内元素数量": 7}"#;
        let config: 缠论配置 = serde_json::from_str(json).unwrap();
        assert_eq!(config.标识, "custom");
        assert_eq!(config.笔内元素数量, 7);
        // 未指定字段使用默认值
        assert_eq!(config.买卖点偏移, 1);
    }

    #[test]
    fn test_invalid_enum_field_fallback() {
        // 无效的 指标计算方式 → 回退默认值 "收"
        let json = r#"{"指标计算方式": "胡写"}"#;
        let config: 缠论配置 = serde_json::from_str(json).unwrap();
        assert_eq!(config.指标计算方式, "收");

        // 有效的 指标计算方式 → 正常通过
        let json = r#"{"指标计算方式": "开"}"#;
        let config: 缠论配置 = serde_json::from_str(json).unwrap();
        assert_eq!(config.指标计算方式, "开");

        // 无效的 买卖点_指标模式 → 回退默认值 "配置"
        let json = r#"{"买卖点_指标模式": "瞎搞"}"#;
        let config: 缠论配置 = serde_json::from_str(json).unwrap();
        assert_eq!(config.买卖点_指标模式, "配置");

        // 有效的 买卖点_指标模式 → 正常通过
        let json = r#"{"买卖点_指标模式": "任意"}"#;
        let config: 缠论配置 = serde_json::from_str(json).unwrap();
        assert_eq!(config.买卖点_指标模式, "任意");

        // 无效的 线段内部背驰_模式 → 回退默认值 "相对"
        let json = r#"{"线段内部背驰_模式": "乱来"}"#;
        let config: 缠论配置 = serde_json::from_str(json).unwrap();
        assert_eq!(config.线段内部背驰_模式, "相对");

        // 有效的 线段内部背驰_模式 → 正常通过
        let json = r#"{"线段内部背驰_模式": "全量"}"#;
        let config: 缠论配置 = serde_json::from_str(json).unwrap();
        assert_eq!(config.线段内部背驰_模式, "全量");
    }

    #[test]
    fn test_to_dict_roundtrip() {
        let config = 缠论配置::default();
        let dict = config.to_dict();
        let restored = 缠论配置::from_dict(&dict).unwrap();
        assert_eq!(config.to_json(), restored.to_json());
    }

    #[test]
    fn test_from_dict_filters_unknown_fields() {
        // 兼容旧版本配置 — unknown fields are silently dropped
        let json = serde_json::json!({
            "标识": "test",
            "不存在的字段": 42,
            "另一个废弃字段": "xxx",
            "笔内元素数量": 8,
        });
        let config = 缠论配置::from_dict(&json).unwrap();
        assert_eq!(config.标识, "test");
        assert_eq!(config.笔内元素数量, 8);
        // 未指定字段使用默认值
        assert_eq!(config.买卖点偏移, 1);
    }

    #[test]
    fn test_model_fields_contains_all() {
        let fields = 缠论配置::model_fields();
        assert!(fields.contains(&"标识"));
        assert!(fields.contains(&"笔内元素数量"));
        assert!(fields.contains(&"买卖点偏移"));
        assert!(fields.contains(&"线段内部背驰_MACD"));
    }

    #[test]
    fn test_model_copy() {
        let mut update = std::collections::HashMap::new();
        update.insert("标识".into(), serde_json::json!("custom"));
        update.insert("推送K线".into(), serde_json::json!(false));
        update.insert("笔内元素数量".into(), serde_json::json!(10));

        let config = 缠论配置::default();
        let copied = config.model_copy(&update);

        assert_eq!(copied.标识, "custom");
        assert!(!copied.推送K线);
        assert_eq!(copied.笔内元素数量, 10);
        // 未指定字段保持不变
        assert_eq!(copied.买卖点偏移, 1);
        assert!(copied.推送笔);
    }

    #[test]
    fn test_to_dict_to_json_consistency() {
        let config = 缠论配置::default();
        let dict = config.to_dict();
        // to_dict → from_dict → to_json should equal original to_json
        let restored = 缠论配置::from_dict(&dict).unwrap();
        assert_eq!(config.to_json(), restored.to_json());
    }

    #[test]
    fn test_不推送() {
        let config = 缠论配置::default();
        let muted = config.不推送();
        assert!(!muted.推送K线);
        assert!(!muted.推送笔);
        assert!(!muted.图表展示);
        assert_eq!(muted.笔内元素数量, 5);
    }

    #[test]
    fn test_对比_无差异() {
        let a = 缠论配置::default();
        let b = 缠论配置::default();
        let diff = a.对比(&b);
        assert!(diff.is_empty(), "identical configs should have empty diff");
    }

    #[test]
    fn test_对比_有差异() {
        let a = 缠论配置::default();
        let mut b = 缠论配置::default();
        b.标识 = "changed".into();
        b.笔内元素数量 = 99;

        let diff = a.对比(&b);
        assert_eq!(diff.len(), 2);
        assert_eq!(diff.get("标识").unwrap().as_str().unwrap(), "changed");
        assert_eq!(diff.get("笔内元素数量").unwrap().as_i64().unwrap(), 99);
    }

    #[test]
    fn test_对比_仅比较model_fields() {
        // 仅比较 model_fields 中的字段（Python 一致行为）
        let a = 缠论配置::default();
        let b = 缠论配置::default();
        let diff = a.对比(&b);
        // 验证不包含废弃字段（如已删除的 "买卖点_背离率" 等）
        assert!(!diff.contains_key("买卖点_背离率"));
        assert!(diff.is_empty(), "default configs should have no diff");
    }

    #[test]
    fn test_to_dict_excludes_non_model_fields() {
        let config = 缠论配置::default();
        let dict = config.to_dict();
        let valid = 缠论配置::model_fields();
        if let serde_json::Value::Object(map) = &dict {
            for key in map.keys() {
                assert!(
                    valid.contains(&key.as_str()),
                    "{key} should not be in to_dict output"
                );
            }
        }
        assert_eq!(
            valid.len(),
            dict.as_object().map(|m| m.len()).unwrap_or(0),
            "to_dict should have exactly model_fields count"
        );
    }

    #[test]
    fn test_model_copy_then_对比() {
        let config = 缠论配置::default();
        let mut update = HashMap::new();
        update.insert("标识".into(), serde_json::json!("copied"));
        update.insert("笔内元素数量".into(), serde_json::json!(10));

        let copied = config.model_copy(&update);
        let diff = config.对比(&copied);

        assert_eq!(diff.len(), 2);
        assert_eq!(diff["标识"].as_str().unwrap(), "copied");
        assert_eq!(diff["笔内元素数量"].as_i64().unwrap(), 10);
    }

    #[test]
    fn test_对比_boolean_difference() {
        let a = 缠论配置::default();
        let mut b = 缠论配置::default();
        b.推送K线 = false;
        b.图表展示 = false;

        let diff = a.对比(&b);
        assert_eq!(diff.len(), 2);
        assert_eq!(diff["推送K线"], serde_json::json!(false));
        assert_eq!(diff["图表展示"], serde_json::json!(false));
    }

    #[test]
    fn test_to_dict_from_dict_对比_roundtrip() {
        let config = 缠论配置::default();
        let dict = config.to_dict();
        let restored = 缠论配置::from_dict(&dict).unwrap();
        let diff = config.对比(&restored);
        assert!(
            diff.is_empty(),
            "to_dict→from_dict roundtrip should produce no diff"
        );
    }
}
