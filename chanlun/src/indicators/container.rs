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

use super::{布林带, 平滑异同移动平均线, 相对强弱指数, 随机指标};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 统一指标值 — 支持所有指标类型的动态注册
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum 指标值 {
    /// MACD 指标
    MACD(平滑异同移动平均线),
    /// RSI 指标
    RSI(相对强弱指数),
    /// KDJ 指标
    KDJ(随机指标),
    /// 布林带指标
    BOLL(布林带),
    /// 均线组 (key → 值)
    均线(HashMap<String, f64>),
    /// 单值指标组 (key → 值)
    单值(HashMap<String, f64>),
}

/// 指标容器 — 挂载在每根 K线上，基于注册表模式持有该时刻所有指标快照
///
/// 与 Python `指标容器` 保持一致：
///   - 复杂指标：MACD/RSI/KDJ/BOLL，通过默认 key（"macd"/"rsi"/"kdj"/"boll"）访问
///   - 多参数变体：key 格式 "MACD_{快}_{慢}_{信号}" / "RSI_{周期}" 等
///   - 均线组：通过 `均线` 子映射访问，key 格式 "{类型}_{周期}"
///   - 单值指标：通过 `单值` 子映射访问，key 格式 "{名称}_{周期}"
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct 指标容器 {
    pub _数据: HashMap<String, Option<指标值>>,
}

impl 指标容器 {
    /// 创建指标容器，预注册 macd/rsi/kdj/boll/均线/单值 默认槽位
    pub fn new() -> Self {
        let mut _数据 = HashMap::new();
        _数据.insert("macd".into(), None);
        _数据.insert("rsi".into(), None);
        _数据.insert("kdj".into(), None);
        _数据.insert("boll".into(), None);
        _数据.insert("均线".into(), Some(指标值::均线(HashMap::new())));
        _数据.insert("单值".into(), Some(指标值::单值(HashMap::new())));
        Self { _数据 }
    }

    /// 预注册指标（不覆盖已有值）
    pub fn 注册(&mut self, 名称: &str, 默认值: Option<指标值>) {
        self._数据.entry(名称.to_string()).or_insert(默认值);
    }

    /// 按名称获取指标值
    pub fn 获取(&self, 名称: &str) -> Option<&指标值> {
        self._数据.get(名称).and_then(|v| v.as_ref())
    }

    /// 按名称设置指标值
    pub fn 设置(&mut self, 名称: &str, 值: 指标值) {
        self._数据.insert(名称.to_string(), Some(值));
    }

    /// 检查是否包含指定名称的指标
    pub fn 包含(&self, 名称: &str) -> bool {
        self._数据.contains_key(名称)
    }

    // ---- 默认指标便捷访问 ----

    /// 获取默认 MACD 指标
    pub fn macd(&self) -> Option<&平滑异同移动平均线> {
        match self._数据.get("macd")?.as_ref()? {
            指标值::MACD(m) => Some(m),
            _ => None,
        }
    }

    /// 克隆获取默认 MACD 指标
    pub fn macd_cloned(&self) -> Option<平滑异同移动平均线> {
        self.macd().cloned()
    }

    /// 设置默认 MACD 指标
    pub fn set_macd(&mut self, m: 平滑异同移动平均线) {
        self._数据.insert("macd".into(), Some(指标值::MACD(m)));
    }

    /// 获取默认 RSI 指标
    pub fn rsi(&self) -> Option<&相对强弱指数> {
        match self._数据.get("rsi")?.as_ref()? {
            指标值::RSI(r) => Some(r),
            _ => None,
        }
    }

    /// 克隆获取默认 RSI 指标
    pub fn rsi_cloned(&self) -> Option<相对强弱指数> {
        self.rsi().cloned()
    }

    /// 设置默认 RSI 指标
    pub fn set_rsi(&mut self, r: 相对强弱指数) {
        self._数据.insert("rsi".into(), Some(指标值::RSI(r)));
    }

    /// 获取默认 KDJ 指标
    pub fn kdj(&self) -> Option<&随机指标> {
        match self._数据.get("kdj")?.as_ref()? {
            指标值::KDJ(k) => Some(k),
            _ => None,
        }
    }

    /// 克隆获取默认 KDJ 指标
    pub fn kdj_cloned(&self) -> Option<随机指标> {
        self.kdj().cloned()
    }

    /// 设置默认 KDJ 指标
    pub fn set_kdj(&mut self, k: 随机指标) {
        self._数据.insert("kdj".into(), Some(指标值::KDJ(k)));
    }

    /// 获取默认布林带指标
    pub fn boll(&self) -> Option<&布林带> {
        match self._数据.get("boll")?.as_ref()? {
            指标值::BOLL(b) => Some(b),
            _ => None,
        }
    }

    /// 克隆获取默认布林带指标
    pub fn boll_cloned(&self) -> Option<布林带> {
        self.boll().cloned()
    }

    /// 设置默认布林带指标
    pub fn set_boll(&mut self, b: 布林带) {
        self._数据.insert("boll".into(), Some(指标值::BOLL(b)));
    }

    /// 获取均线组
    pub fn 均线(&self) -> Option<&HashMap<String, f64>> {
        match self._数据.get("均线")?.as_ref()? {
            指标值::均线(m) => Some(m),
            _ => None,
        }
    }

    /// 获取均线组可变引用
    pub fn 均线_mut(&mut self) -> Option<&mut HashMap<String, f64>> {
        match self._数据.get_mut("均线")?.as_mut()? {
            指标值::均线(m) => Some(m),
            _ => None,
        }
    }

    /// 获取单值指标组
    pub fn 单值(&self) -> Option<&HashMap<String, f64>> {
        match self._数据.get("单值")?.as_ref()? {
            指标值::单值(s) => Some(s),
            _ => None,
        }
    }
}

impl std::fmt::Display for 指标容器 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let keys: Vec<&str> = self
            ._数据
            .iter()
            .filter(|(_, v)| v.is_some())
            .map(|(k, _)| k.as_str())
            .collect();
        write!(f, "指标容器({})", keys.join(", "))
    }
}
