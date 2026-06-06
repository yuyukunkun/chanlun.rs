"""缠论技术分析库 — Rust 高性能实现"""

__all__ = [
    "K线",
    "K线合成器",
    "中枢",
    "买卖点",
    "买卖点类型",
    "分型",
    "分型结构",
    "基础买卖点",
    "平滑异同移动平均线",
    "指标",
    "指标容器",
    "指标计算器",
    "均线工具",
    "相对强弱指数",
    "相对方向",
    "立体分析器",
    "笔",
    "线段",
    "线段特征",
    "缠论K线",
    "缠论配置",
    "缺口",
    "背驰分析",
    "虚线",
    "观察者",
    "转化为时间戳",
    "转化为时间戳_数字",
    "随机指标",
    "布林带",
    "get_分型模式",
    "set_分型模式",
    "get_log_level",
    "set_log_level",
    "get_rs_log_level",
    "set_rs_log_level",
    "K线相等",
    "缠论K线相等",
    "分型相等",
    "缺口相等",
    "线段特征相等",
    "中枢相等",
    "虚线相等",
    "chan",
]
from ._chanlun import *
from ._chanlun import set_log_level as _rs_set_log_level, get_log_level as _rs_get_log_level
from . import chan
from .chan import set_log_level, get_log_level


def set_rs_log_level(level: str):
    """设置 Rust 侧日志级别 (trace / debug / info / warn / error / off)

    仅控制 Rust tracing 日志，不影响 Python loguru 日志。
    Python 侧日志通过 set_log_level() 独立控制。
    """
    _rs_set_log_level(level)


def get_rs_log_level() -> str:
    """获取 Rust 侧日志级别"""
    return _rs_get_log_level()
