from __future__ import annotations
from datetime import datetime
from typing import Optional, List, Deque
from collections import deque
import math


# ======================= 辅助函数 =======================
def _ema_alpha(period: int) -> float:
    """EMA 平滑系数"""
    return 2.0 / (period + 1)


def _ultimate_smoother_coeffs(period: float):
    """终极平滑器系数"""
    a1 = math.exp(-1.414 * math.pi / period)
    b1 = 2.0 * a1 * math.cos(1.414 * math.pi / period)
    c2 = b1
    c3 = -a1 * a1
    c1 = (1.0 + c2 - c3) / 4.0
    return c1, c2, c3


# ======================= 1. SMA 简单移动平均 =======================
class SMA:
    """简单移动平均 — 窗口滚动均值"""

    def __init__(
        self,
        时间戳: datetime,
        收盘价: float,
        周期: int,
        sma值: Optional[float] = None,
        _窗口: Optional[Deque[float]] = None,
        _当前和: float = 0.0,
    ):
        if 周期 <= 0:
            raise ValueError(f"周期必须大于0，实际: {周期}")
        self.时间戳 = 时间戳
        self.收盘价 = 收盘价
        self.周期 = 周期
        self.sma值 = sma值
        self._窗口: Deque[float] = _窗口 if _窗口 is not None else deque()
        self._当前和: float = _当前和

    @classmethod
    def 首次计算(cls, 序列: List[float], 时间序列: List[datetime], 周期: int) -> "SMA":
        """用前周期个数据初始化"""
        if len(序列) < 周期:
            raise ValueError("数据长度不足")
        窗口 = deque(序列[:周期], maxlen=周期)
        当前和 = sum(窗口)
        return cls(时间戳=时间序列[周期 - 1], 收盘价=序列[周期 - 1], 周期=周期, sma值=当前和 / 周期, _窗口=窗口, _当前和=当前和)

    @classmethod
    def 增量计算(cls, 前一个: "SMA", 新收盘价: float, 新时间: datetime) -> "SMA":
        窗口 = 前一个._窗口
        旧值 = 窗口[0]
        窗口.append(新收盘价)
        当前和 = 前一个._当前和 + 新收盘价 - 旧值
        return cls(时间戳=新时间, 收盘价=新收盘价, 周期=前一个.周期, sma值=当前和 / 前一个.周期, _窗口=窗口, _当前和=当前和)


# ======================= 2. Ultimate Smoother =======================
class UltimateSmoother:
    """终极平滑器"""

    def __init__(
        self,
        时间戳: datetime,
        价格: float,
        周期: float,
        平滑值: Optional[float] = None,
        _历史价格: Optional[Deque[float]] = None,
        _平滑历史: Optional[Deque[float]] = None,
    ):
        self.时间戳 = 时间戳
        self.价格 = 价格
        self.周期 = 周期
        self.平滑值 = 平滑值
        self._历史价格: Deque[float] = _历史价格 if _历史价格 is not None else deque(maxlen=4)
        self._平滑历史: Deque[float] = _平滑历史 if _平滑历史 is not None else deque(maxlen=2)

    @classmethod
    def 首次计算(cls, 序列: List[float], 时间序列: List[datetime], 周期: float) -> "UltimateSmoother":
        if len(序列) < 4:
            raise ValueError("至少需要4个数据点")
        obj = cls(时间戳=时间序列[0], 价格=序列[0], 周期=周期, 平滑值=序列[0])
        obj._历史价格.extend(序列[:4])
        obj._平滑历史.append(序列[0])
        obj._平滑历史.append(序列[0])  # 初始化第二个平滑值，增量计算需要 [-1] 和 [-2]
        for i in range(1, 4):
            obj = cls.增量计算(obj, 序列[i], 时间序列[i])
        return obj

    @classmethod
    def 增量计算(cls, 前一个: "UltimateSmoother", 新价格: float, 新时间: datetime) -> "UltimateSmoother":
        c1, c2, c3 = _ultimate_smoother_coeffs(前一个.周期)
        价格历史 = 前一个._历史价格
        平滑历史 = 前一个._平滑历史
        if len(价格历史) < 4:
            价格历史.append(新价格)
            新平滑 = 新价格
        else:
            价格历史.append(新价格)
            p, p1, p2 = 新价格, 价格历史[-2], 价格历史[-3]
            us1, us2 = 平滑历史[-1], 平滑历史[-2]
            新平滑 = (1 - c1) * p + (2 * c1 - c2) * p1 - (c1 + c3) * p2 + c2 * us1 + c3 * us2
        平滑历史.append(新平滑)
        return cls(时间戳=新时间, 价格=新价格, 周期=前一个.周期, 平滑值=新平滑, _历史价格=价格历史, _平滑历史=平滑历史)


# ======================= 3. Rolling Rank =======================
class RollingRank:
    """滚动排名"""

    def __init__(
        self,
        时间戳: datetime,
        当前值: float,
        窗口大小: int,
        排名: Optional[int] = None,
        _窗口值: Optional[Deque[float]] = None,
    ):
        self.时间戳 = 时间戳
        self.当前值 = 当前值
        self.窗口大小 = 窗口大小
        self.排名 = 排名
        self._窗口值: Deque[float] = _窗口值 if _窗口值 is not None else deque()

    @classmethod
    def 首次计算(cls, 序列: List[float], 时间序列: List[datetime], 窗口: int) -> "RollingRank":
        if len(序列) < 窗口:
            raise ValueError
        窗口值 = deque(序列[:窗口], maxlen=窗口)
        排序 = sorted(窗口值)
        当前值 = 序列[窗口 - 1]
        排名 = 排序.index(当前值) + 1
        return cls(时间戳=时间序列[窗口 - 1], 当前值=当前值, 窗口大小=窗口, 排名=排名, _窗口值=窗口值)

    @classmethod
    def 增量计算(cls, 前一个: "RollingRank", 新值: float, 新时间: datetime) -> "RollingRank":
        窗口值 = 前一个._窗口值
        窗口值.append(新值)
        排序 = sorted(窗口值)
        排名 = 排序.index(新值) + 1
        return cls(时间戳=新时间, 当前值=新值, 窗口大小=前一个.窗口大小, 排名=排名, _窗口值=窗口值)


# ======================= 4. 单均线多空信号 =======================
class SingleSMAPositions:
    """单均线多空信号（双重平滑）"""

    def __init__(
        self,
        时间戳: datetime,
        收盘价: float,
        周期: int,
        持仓信号: float = 0.0,
        _ms窗口: Optional[Deque[float]] = None,
        _ms_sma窗口: Optional[Deque[float]] = None,
        _当前ms和: float = 0.0,
        _当前ms_sma和: float = 0.0,
    ):
        self.时间戳 = 时间戳
        self.收盘价 = 收盘价
        self.周期 = 周期
        self.持仓信号 = 持仓信号
        self._ms窗口: Deque[float] = _ms窗口 if _ms窗口 is not None else deque()
        self._ms_sma窗口: Deque[float] = _ms_sma窗口 if _ms_sma窗口 is not None else deque()
        self._当前ms和: float = _当前ms和
        self._当前ms_sma和: float = _当前ms_sma和

    @classmethod
    def 首次计算(cls, 序列: List[float], 时间序列: List[datetime], 周期: int) -> "SingleSMAPositions":
        n = 周期
        有效起点 = 2 * n - 2
        if len(序列) <= 有效起点:
            raise ValueError
        obj = None
        for i in range(有效起点, len(序列)):
            if obj is None:
                ms = [sum(序列[j - n + 1 : j + 1]) / n for j in range(n - 1, 有效起点 + 1)]
                ms_sma = [sum(ms[k - n + 1 : k + 1]) / n for k in range(n - 1, 有效起点 + 1)]
                信号 = (序列[有效起点] - ms_sma[-1]) / abs(序列[有效起点] - ms_sma[-1]) if ms_sma[-1] != 0 else 0.0
                obj = cls(时间戳=时间序列[有效起点], 收盘价=序列[有效起点], 周期=周期, 持仓信号=信号, _ms窗口=deque(ms[-n:], maxlen=n), _ms_sma窗口=deque(ms_sma[-n:], maxlen=n), _当前ms和=sum(ms[-n:]), _当前ms_sma和=sum(ms_sma[-n:]))
            else:
                obj = cls.增量计算(obj, 序列[i], 时间序列[i])
        return obj

    @classmethod
    def 增量计算(cls, 前一个: "SingleSMAPositions", 新收盘价: float, 新时间: datetime) -> "SingleSMAPositions":
        # 更新 ms
        ms窗口 = 前一个._ms窗口
        ms旧 = ms窗口[0]
        ms窗口.append(新收盘价)
        当前ms和 = 前一个._当前ms和 + 新收盘价 - ms旧
        ms新 = 当前ms和 / 前一个.周期
        # 更新 ms_sma
        ms_sma窗口 = 前一个._ms_sma窗口
        ms_sma旧 = ms_sma窗口[0]
        ms_sma窗口.append(ms新)
        当前ms_sma和 = 前一个._当前ms_sma和 + ms新 - ms_sma旧
        ms_sma新 = 当前ms_sma和 / 前一个.周期
        if ms_sma新 != 0:
            信号 = 1.0 if 新收盘价 > ms_sma新 else (-1.0 if 新收盘价 < ms_sma新 else 0.0)
        else:
            信号 = 0.0
        return cls(时间戳=新时间, 收盘价=新收盘价, 周期=前一个.周期, 持仓信号=信号, _ms窗口=ms窗口, _ms_sma窗口=ms_sma窗口, _当前ms和=当前ms和, _当前ms_sma和=当前ms_sma和)


# ======================= 5. 单EMA多空信号 =======================
class SingleEMAPositions:
    """单EMA多空信号"""

    def __init__(
        self,
        时间戳: datetime,
        收盘价: float,
        周期: int,
        持仓信号: float = 0.0,
        _ms窗口: Optional[Deque[float]] = None,
        _当前ms和: float = 0.0,
        _ema值: float = 0.0,
        _有效起始: int = 0,
    ):
        self.时间戳 = 时间戳
        self.收盘价 = 收盘价
        self.周期 = 周期
        self.持仓信号 = 持仓信号
        self._ms窗口: Deque[float] = _ms窗口 if _ms窗口 is not None else deque()
        self._当前ms和: float = _当前ms和
        self._ema值: float = _ema值
        self._有效起始: int = _有效起始

    @classmethod
    def 首次计算(cls, 序列: List[float], 时间序列: List[datetime], 周期: int) -> "SingleEMAPositions":
        n = 周期
        if len(序列) < n:
            raise ValueError
        ms = [sum(序列[i - n + 1 : i + 1]) / n for i in range(n - 1, len(序列))]
        alpha = _ema_alpha(n)
        有效起始 = n - 1 + n - 1
        if len(序列) <= 有效起始:
            raise ValueError
        ema = sum(ms[:n]) / n
        信号 = (序列[有效起始] - ema) / abs(序列[有效起始] - ema) if ema != 0 else 0.0
        obj = cls(时间戳=时间序列[有效起始], 收盘价=序列[有效起始], 周期=周期, 持仓信号=信号, _ms窗口=deque(序列[有效起始 - n + 1 : 有效起始 + 1], maxlen=n), _当前ms和=sum(序列[有效起始 - n + 1 : 有效起始 + 1]), _ema值=ema, _有效起始=有效起始)
        return obj

    @classmethod
    def 增量计算(cls, 前一个: "SingleEMAPositions", 新收盘价: float, 新时间: datetime) -> "SingleEMAPositions":
        窗口 = 前一个._ms窗口
        旧 = 窗口[0]
        窗口.append(新收盘价)
        ms和 = 前一个._当前ms和 + 新收盘价 - 旧
        ms = ms和 / 前一个.周期
        alpha = _ema_alpha(前一个.周期)
        ema = alpha * ms + (1 - alpha) * 前一个._ema值
        信号 = 1.0 if 新收盘价 > ema else (-1.0 if 新收盘价 < ema else 0.0)
        return cls(时间戳=新时间, 收盘价=新收盘价, 周期=前一个.周期, 持仓信号=信号, _ms窗口=窗口, _当前ms和=ms和, _ema值=ema, _有效起始=前一个._有效起始)


# ======================= 6. 中轴策略 =======================
class MidPositions:
    """中轴多空策略"""

    def __init__(
        self,
        时间戳: datetime,
        当前值: float,
        周期: int,
        持仓信号: float = 0.0,
        _ms窗口: Optional[Deque[float]] = None,
        _ms和: float = 0.0,
        _high_low窗口: Optional[Deque[float]] = None,
    ):
        self.时间戳 = 时间戳
        self.当前值 = 当前值
        self.周期 = 周期
        self.持仓信号 = 持仓信号
        self._ms窗口: Deque[float] = _ms窗口 if _ms窗口 is not None else deque()
        self._ms和: float = _ms和
        self._high_low窗口: Deque[float] = _high_low窗口 if _high_low窗口 is not None else deque()

    @classmethod
    def 首次计算(cls, 序列: List[float], 时间序列: List[datetime], 周期: int) -> "MidPositions":
        n = 周期
        有效起点 = 2 * n - 2
        if len(序列) <= 有效起点:
            raise ValueError
        ms列表 = [sum(序列[i - n + 1 : i + 1]) / n for i in range(n - 1, len(序列))]
        obj = None
        for idx in range(有效起点, len(序列)):
            i = idx
            if obj is None:
                high = max(ms列表[i - n + 1 : i + 1])
                low = min(ms列表[i - n + 1 : i + 1])
                mid = (high + low) / 2
                信号 = (ms列表[i] - mid) / abs(ms列表[i] - mid) if (high + low) != 0 else 0.0
                obj = cls(时间戳=时间序列[i], 当前值=序列[i], 周期=周期, 持仓信号=信号, _ms窗口=deque(序列[i - n + 1 : i + 1], maxlen=n), _ms和=sum(序列[i - n + 1 : i + 1]), _high_low窗口=deque(ms列表[i - n + 1 : i + 1], maxlen=n))
            else:
                obj = cls.增量计算(obj, 序列[i], 时间序列[i])
        return obj

    @classmethod
    def 增量计算(cls, 前一个: "MidPositions", 新值: float, 新时间: datetime) -> "MidPositions":
        ms窗口 = 前一个._ms窗口
        ms旧 = ms窗口[0]
        ms窗口.append(新值)
        ms和 = 前一个._ms和 + 新值 - ms旧
        ms = ms和 / 前一个.周期
        hl窗口 = 前一个._high_low窗口
        hl窗口.append(ms)
        high = max(hl窗口)
        low = min(hl窗口)
        mid = (high + low) / 2
        if high != 0 or low != 0:
            信号 = 1.0 if ms > mid else (-1.0 if ms < mid else 0.0)
        else:
            信号 = 0.0
        return cls(时间戳=新时间, 当前值=新值, 周期=前一个.周期, 持仓信号=信号, _ms窗口=ms窗口, _ms和=ms和, _high_low窗口=hl窗口)


# ======================= 7. 双均线多空信号 =======================
class DoubleSMAPositions:
    """双均线多空信号"""

    def __init__(
        self,
        时间戳: datetime,
        收盘价: float,
        短周期: int,
        长周期: int,
        持仓信号: float = 0.0,
        _短窗口: Optional[Deque[float]] = None,
        _长窗口: Optional[Deque[float]] = None,
        _短和: float = 0.0,
        _长和: float = 0.0,
    ):
        self.时间戳 = 时间戳
        self.收盘价 = 收盘价
        self.短周期 = 短周期
        self.长周期 = 长周期
        self.持仓信号 = 持仓信号
        self._短窗口: Deque[float] = _短窗口 if _短窗口 is not None else deque()
        self._长窗口: Deque[float] = _长窗口 if _长窗口 is not None else deque()
        self._短和: float = _短和
        self._长和: float = _长和

    @classmethod
    def 首次计算(cls, 序列: List[float], 时间序列: List[datetime], 短周期: int, 长周期: int) -> "DoubleSMAPositions":
        if 短周期 >= 长周期:
            raise ValueError
        有效起点 = 长周期 - 1
        if len(序列) <= 有效起点:
            raise ValueError
        obj = None
        for i in range(有效起点, len(序列)):
            if obj is None:
                短窗口 = deque(序列[i - 短周期 + 1 : i + 1], maxlen=短周期)
                长窗口 = deque(序列[i - 长周期 + 1 : i + 1], maxlen=长周期)
                短sma = sum(短窗口) / 短周期
                长sma = sum(长窗口) / 长周期
                信号 = 1.0 if 短sma > 长sma else (-1.0 if 短sma < 长sma else 0.0)
                obj = cls(时间戳=时间序列[i], 收盘价=序列[i], 短周期=短周期, 长周期=长周期, 持仓信号=信号, _短窗口=短窗口, _长窗口=长窗口, _短和=sum(短窗口), _长和=sum(长窗口))
            else:
                obj = cls.增量计算(obj, 序列[i], 时间序列[i])
        return obj

    @classmethod
    def 增量计算(cls, 前一个: "DoubleSMAPositions", 新收盘价: float, 新时间: datetime) -> "DoubleSMAPositions":
        短窗口 = 前一个._短窗口
        长窗口 = 前一个._长窗口
        短旧 = 短窗口[0]
        长旧 = 长窗口[0]
        短窗口.append(新收盘价)
        长窗口.append(新收盘价)
        短和 = 前一个._短和 + 新收盘价 - 短旧
        长和 = 前一个._长和 + 新收盘价 - 长旧
        短sma = 短和 / 前一个.短周期
        长sma = 长和 / 前一个.长周期
        信号 = 1.0 if 短sma > 长sma else (-1.0 if 短sma < 长sma else 0.0)
        return cls(时间戳=新时间, 收盘价=新收盘价, 短周期=前一个.短周期, 长周期=前一个.长周期, 持仓信号=信号, _短窗口=短窗口, _长窗口=长窗口, _短和=短和, _长和=长和)


# ======================= 8. 三均线系统 =======================
class TripleSMAPositions:
    """三均线系统持仓信号"""

    def __init__(
        self,
        时间戳: datetime,
        收盘价: float,
        m1: int,
        m2: int,
        m3: int,
        持仓信号: int = 0,
        _smoothed窗口: Optional[Deque[float]] = None,
        _smoothed和: float = 0.0,
        _ma1窗口: Optional[Deque[float]] = None,
        _ma1和: float = 0.0,
        _ma2窗口: Optional[Deque[float]] = None,
        _ma2和: float = 0.0,
        _ma3窗口: Optional[Deque[float]] = None,
        _ma3和: float = 0.0,
    ):
        self.时间戳 = 时间戳
        self.收盘价 = 收盘价
        self.m1 = m1
        self.m2 = m2
        self.m3 = m3
        self.持仓信号 = 持仓信号
        self._smoothed窗口: Deque[float] = _smoothed窗口 if _smoothed窗口 is not None else deque()
        self._smoothed和: float = _smoothed和
        self._ma1窗口: Deque[float] = _ma1窗口 if _ma1窗口 is not None else deque()
        self._ma1和: float = _ma1和
        self._ma2窗口: Deque[float] = _ma2窗口 if _ma2窗口 is not None else deque()
        self._ma2和: float = _ma2和
        self._ma3窗口: Deque[float] = _ma3窗口 if _ma3窗口 is not None else deque()
        self._ma3和: float = _ma3和

    @classmethod
    def 首次计算(cls, 序列: List[float], 时间序列: List[datetime], m1: int, m2: int, m3: int) -> "TripleSMAPositions":
        if not (m1 < m2 < m3):
            raise ValueError
        需要长度 = m3 + m3 - 1
        if len(序列) < 需要长度:
            raise ValueError
        smoothed = [None] * len(序列)
        for i in range(m1 - 1, len(序列)):
            smoothed[i] = sum(序列[i - m1 + 1 : i + 1]) / m1
        ma1 = [None] * len(序列)
        ma2 = [None] * len(序列)
        ma3 = [None] * len(序列)
        for i in range(m1 - 1, len(序列)):
            if i >= m1 - 1:
                窗口 = [smoothed[j] for j in range(i - m1 + 1, i + 1) if smoothed[j] is not None]
                if len(窗口) == m1:
                    ma1[i] = sum(窗口) / m1
        for i in range(m2 - 1, len(序列)):
            窗口 = [smoothed[j] for j in range(i - m2 + 1, i + 1) if smoothed[j] is not None]
            if len(窗口) == m2:
                ma2[i] = sum(窗口) / m2
        for i in range(m3 - 1, len(序列)):
            窗口 = [smoothed[j] for j in range(i - m3 + 1, i + 1) if smoothed[j] is not None]
            if len(窗口) == m3:
                ma3[i] = sum(窗口) / m3

        最后位置 = len(序列) - 1
        while 最后位置 >= 0 and (smoothed[最后位置] is None or ma1[最后位置] is None or ma2[最后位置] is None or ma3[最后位置] is None):
            最后位置 -= 1
        if 最后位置 < 0:
            raise ValueError
        i = 最后位置
        s, a1, a2, a3 = smoothed[i], ma1[i], ma2[i], ma3[i]
        if s > a3 and a1 > a2:
            sig = 1
        elif s < a3 and a1 < a2:
            sig = -1
        else:
            sig = 0
        obj = cls(
            时间戳=时间序列[i],
            收盘价=序列[i],
            m1=m1,
            m2=m2,
            m3=m3,
            持仓信号=sig,
            _smoothed窗口=deque(序列[i - m1 + 1 : i + 1], maxlen=m1),
            _smoothed和=sum(序列[i - m1 + 1 : i + 1]),
            _ma1窗口=deque([smoothed[j] for j in range(i - m1 + 1, i + 1) if smoothed[j] is not None], maxlen=m1),
            _ma1和=sum([smoothed[j] for j in range(i - m1 + 1, i + 1) if smoothed[j] is not None]),
            _ma2窗口=deque([smoothed[j] for j in range(i - m2 + 1, i + 1) if smoothed[j] is not None], maxlen=m2),
            _ma2和=sum([smoothed[j] for j in range(i - m2 + 1, i + 1) if smoothed[j] is not None]),
            _ma3窗口=deque([smoothed[j] for j in range(i - m3 + 1, i + 1) if smoothed[j] is not None], maxlen=m3),
            _ma3和=sum([smoothed[j] for j in range(i - m3 + 1, i + 1) if smoothed[j] is not None]),
        )
        return obj

    @classmethod
    def 增量计算(cls, 前一个: "TripleSMAPositions", 新收盘价: float, 新时间: datetime) -> "TripleSMAPositions":
        s窗口 = 前一个._smoothed窗口
        s旧 = s窗口[0]
        s窗口.append(新收盘价)
        s和 = 前一个._smoothed和 + 新收盘价 - s旧
        smoothed = s和 / 前一个.m1

        ma1窗口 = 前一个._ma1窗口
        ma1旧 = ma1窗口[0]
        ma1窗口.append(smoothed)
        ma1和 = 前一个._ma1和 + smoothed - ma1旧
        ma1 = ma1和 / 前一个.m1

        ma2窗口 = 前一个._ma2窗口
        ma2旧 = ma2窗口[0]
        ma2窗口.append(smoothed)
        ma2和 = 前一个._ma2和 + smoothed - ma2旧
        ma2 = ma2和 / 前一个.m2

        ma3窗口 = 前一个._ma3窗口
        ma3旧 = ma3窗口[0]
        ma3窗口.append(smoothed)
        ma3和 = 前一个._ma3和 + smoothed - ma3旧
        ma3 = ma3和 / 前一个.m3

        if smoothed > ma3 and ma1 > ma2:
            sig = 1
        elif smoothed < ma3 and ma1 < ma2:
            sig = -1
        else:
            sig = 0
        return cls(时间戳=新时间, 收盘价=新收盘价, m1=前一个.m1, m2=前一个.m2, m3=前一个.m3, 持仓信号=sig, _smoothed窗口=s窗口, _smoothed和=s和, _ma1窗口=ma1窗口, _ma1和=ma1和, _ma2窗口=ma2窗口, _ma2和=ma2和, _ma3窗口=ma3窗口, _ma3和=ma3和)


# ======================= 9. 布林线多空信号 =======================
class BollPositions:
    """布林线多空信号"""

    def __init__(
        self,
        时间戳: datetime,
        收盘价: float,
        周期: int,
        倍数: float,
        持仓信号: int = 0,
        _smoothed窗口: Optional[Deque[float]] = None,
        _smoothed和: float = 0.0,
        _sm窗口: Optional[Deque[float]] = None,
        _sm和: float = 0.0,
        _sd窗口: Optional[Deque[float]] = None,
    ):
        self.时间戳 = 时间戳
        self.收盘价 = 收盘价
        self.周期 = 周期
        self.倍数 = 倍数
        self.持仓信号 = 持仓信号
        self._smoothed窗口: Deque[float] = _smoothed窗口 if _smoothed窗口 is not None else deque()
        self._smoothed和: float = _smoothed和
        self._sm窗口: Deque[float] = _sm窗口 if _sm窗口 is not None else deque()
        self._sm和: float = _sm和
        self._sd窗口: Deque[float] = _sd窗口 if _sd窗口 is not None else deque()

    @classmethod
    def 首次计算(cls, 序列: List[float], 时间序列: List[datetime], 周期: int, 倍数: float) -> "BollPositions":
        n = 周期
        需要 = 2 * n - 1
        if len(序列) < 需要:
            raise ValueError
        smoothed_list = [sum(序列[i - n + 1 : i + 1]) / n for i in range(n - 1, len(序列))]
        sm_list = [None] * len(序列)
        sd_list = [None] * len(序列)
        for i in range(n - 1, len(序列)):
            sm_idx = i - n + 1
            win = smoothed_list[sm_idx - n + 1 : sm_idx + 1]
            if len(win) == n:
                mean = sum(win) / n
                var = sum((x - mean) ** 2 for x in win) / (n - 1)
                sm_list[i] = mean
                sd_list[i] = math.sqrt(var)
        最后 = len(序列) - 1
        while 最后 >= 0 and sm_list[最后] is None:
            最后 -= 1
        i = 最后
        sm_val = sm_list[i]
        sd_val = sd_list[i]
        平滑索引 = i - n + 1
        smoothed = smoothed_list[平滑索引]
        upper = sm_val + 倍数 * sd_val
        lower = sm_val - 倍数 * sd_val
        if smoothed > upper + 1e-10:
            sig = 1
        elif smoothed < lower - 1e-10:
            sig = -1
        else:
            sig = 0
        obj = cls(
            时间戳=时间序列[i],
            收盘价=序列[i],
            周期=周期,
            倍数=倍数,
            持仓信号=sig,
            _smoothed窗口=deque(序列[i - n + 1 : i + 1], maxlen=n),
            _smoothed和=sum(序列[i - n + 1 : i + 1]),
            _sm窗口=deque(smoothed_list[平滑索引 - n + 1 : 平滑索引 + 1], maxlen=n),
            _sm和=sum(smoothed_list[平滑索引 - n + 1 : 平滑索引 + 1]),
            _sd窗口=deque(sd_list[i - n + 1 : i + 1], maxlen=n),
        )
        return obj

    @classmethod
    def 增量计算(cls, 前一个: "BollPositions", 新收盘价: float, 新时间: datetime) -> "BollPositions":
        s窗口 = 前一个._smoothed窗口
        s旧 = s窗口[0]
        s窗口.append(新收盘价)
        s和 = 前一个._smoothed和 + 新收盘价 - s旧
        smoothed = s和 / 前一个.周期

        sm窗口 = 前一个._sm窗口
        sm窗口.append(smoothed)
        n = 前一个.周期
        mean = sum(sm窗口) / n
        var = sum((x - mean) ** 2 for x in sm窗口) / (n - 1)
        sd = math.sqrt(var)
        upper = mean + 前一个.倍数 * sd
        lower = mean - 前一个.倍数 * sd
        if smoothed > upper + 1e-10:
            sig = 1
        elif smoothed < lower - 1e-10:
            sig = -1
        else:
            sig = 0
        return cls(时间戳=新时间, 收盘价=新收盘价, 周期=前一个.周期, 倍数=前一个.倍数, 持仓信号=sig, _smoothed窗口=s窗口, _smoothed和=s和, _sm窗口=sm窗口, _sm和=mean * n, _sd窗口=前一个._sd窗口)


# ======================= 10. 布林反转策略 =======================
class BollReversePositions:
    """布林带反转策略信号"""

    def __init__(
        self,
        时间戳: datetime,
        收盘价: float,
        周期: int,
        倍数: float,
        持仓信号: int = 0,
        _smoothed窗口: Optional[Deque[float]] = None,
        _smoothed和: float = 0.0,
        _当前持仓: int = 0,
    ):
        self.时间戳 = 时间戳
        self.收盘价 = 收盘价
        self.周期 = 周期
        self.倍数 = 倍数
        self.持仓信号 = 持仓信号
        self._smoothed窗口: Deque[float] = _smoothed窗口 if _smoothed窗口 is not None else deque()
        self._smoothed和: float = _smoothed和
        self._当前持仓: int = _当前持仓

    @classmethod
    def 首次计算(cls, 序列: List[float], 时间序列: List[datetime], 周期: int, 倍数: float) -> "BollReversePositions":
        n = 周期
        if len(序列) < n:
            raise ValueError
        boll = BollPositions.首次计算(序列, 时间序列, 周期, 倍数)
        return cls._从布林计算(boll, 0)

    @classmethod
    def _从布林计算(cls, boll: "BollPositions", 当前持仓: int) -> "BollReversePositions":
        """从BollPositions状态构造BollReversePositions"""
        n = boll.周期
        # 计算当前的smoothed和布林带
        s窗口 = boll._smoothed窗口
        smoothed = boll._smoothed和 / n if boll._smoothed和 > 0 else sum(s窗口) / len(s窗口)
        sm窗口 = boll._sm窗口
        mean = sum(sm窗口) / len(sm窗口)
        var = sum((x - mean) ** 2 for x in sm窗口) / (len(sm窗口) - 1)
        sd = math.sqrt(var)
        upper = mean + boll.倍数 * sd
        lower = mean - boll.倍数 * sd
        if 当前持仓 == 0:
            if smoothed > upper:
                当前持仓 = -1
            elif smoothed < lower:
                当前持仓 = 1
        elif 当前持仓 == 1:
            if smoothed > mean:
                当前持仓 = 0
        elif 当前持仓 == -1:
            if smoothed < mean:
                当前持仓 = 0
        return cls(时间戳=boll.时间戳, 收盘价=boll.收盘价, 周期=boll.周期, 倍数=boll.倍数, 持仓信号=当前持仓, _smoothed窗口=s窗口, _smoothed和=boll._smoothed和, _当前持仓=当前持仓)

    @classmethod
    def 增量计算(cls, 前一个: "BollReversePositions", 新收盘价: float, 新时间: datetime) -> "BollReversePositions":
        s窗口 = 前一个._smoothed窗口
        s旧 = s窗口[0]
        s窗口.append(新收盘价)
        s和 = 前一个._smoothed和 + 新收盘价 - s旧
        n = 前一个.周期
        if len(s窗口) < n:
            return cls(时间戳=新时间, 收盘价=新收盘价, 周期=前一个.周期, 倍数=前一个.倍数, 持仓信号=0, _smoothed窗口=s窗口, _smoothed和=s和, _当前持仓=0)
        smoothed = s和 / n
        # 构建smoothed历史窗口
        sm_deque = deque(maxlen=n)
        for i in range(len(s窗口) - n + 1, len(s窗口) + 1):
            win = list(s窗口)[i - n : i] if i >= n else list(s窗口)[:i]
            if len(win) == n:
                sm_deque.append(sum(win) / n)
        if len(sm_deque) < n:
            return cls(时间戳=新时间, 收盘价=新收盘价, 周期=前一个.周期, 倍数=前一个.倍数, 持仓信号=前一个._当前持仓, _smoothed窗口=s窗口, _smoothed和=s和, _当前持仓=前一个._当前持仓)
        mean = sum(sm_deque) / n
        var = sum((x - mean) ** 2 for x in sm_deque) / (n - 1)
        sd = math.sqrt(var)
        upper = mean + 前一个.倍数 * sd
        lower = mean - 前一个.倍数 * sd
        当前持仓 = 前一个._当前持仓
        if 当前持仓 == 0:
            if smoothed > upper:
                当前持仓 = -1
            elif smoothed < lower:
                当前持仓 = 1
        elif 当前持仓 == 1:
            if smoothed > mean:
                当前持仓 = 0
        elif 当前持仓 == -1:
            if smoothed < mean:
                当前持仓 = 0
        return cls(时间戳=新时间, 收盘价=新收盘价, 周期=前一个.周期, 倍数=前一个.倍数, 持仓信号=当前持仓, _smoothed窗口=s窗口, _smoothed和=s和, _当前持仓=当前持仓)


# ======================= 11. MMS 归一化信号 =======================
class MMSPositions:
    """均线最大最小值归一化信号"""

    def __init__(
        self,
        时间戳: datetime,
        收盘价: float,
        均线周期: int,
        窗口: int,
        信号: float = 0.0,
        _sm窗口: Optional[Deque[float]] = None,
        _sm和: float = 0.0,
        _sm历史: Optional[Deque[float]] = None,
    ):
        self.时间戳 = 时间戳
        self.收盘价 = 收盘价
        self.均线周期 = 均线周期
        self.窗口 = 窗口
        self.信号 = 信号
        self._sm窗口: Deque[float] = _sm窗口 if _sm窗口 is not None else deque()
        self._sm和: float = _sm和
        self._sm历史: Deque[float] = _sm历史 if _sm历史 is not None else deque()

    @classmethod
    def 首次计算(cls, 序列: List[float], 时间序列: List[datetime], 均线周期: int, 窗口: int) -> "MMSPositions":
        tp, w = 均线周期, 窗口
        if len(序列) < tp + w - 1:
            raise ValueError
        sm_list = [sum(序列[i - tp + 1 : i + 1]) / tp for i in range(tp - 1, len(序列))]
        有效起点 = tp - 1 + w - 1
        if len(序列) <= 有效起点:
            raise ValueError
        i = 有效起点
        sm = sm_list[i]
        sm_min = min(sm_list[i - w + 1 : i + 1])
        sm_max = max(sm_list[i - w + 1 : i + 1])
        if sm_max - sm_min > 1e-10:
            norm = (sm - sm_min) / (sm_max - sm_min)
            信号 = norm * 2 - 1
        else:
            信号 = 0.0
        obj = cls(时间戳=时间序列[i], 收盘价=序列[i], 均线周期=tp, 窗口=w, 信号=信号, _sm窗口=deque(序列[i - tp + 1 : i + 1], maxlen=tp), _sm和=sum(序列[i - tp + 1 : i + 1]), _sm历史=deque(sm_list[i - w + 1 : i + 1], maxlen=w))
        return obj

    @classmethod
    def 增量计算(cls, 前一个: "MMSPositions", 新收盘价: float, 新时间: datetime) -> "MMSPositions":
        s窗口 = 前一个._sm窗口
        s旧 = s窗口[0]
        s窗口.append(新收盘价)
        s和 = 前一个._sm和 + 新收盘价 - s旧
        sm = s和 / 前一个.均线周期
        sm历史 = 前一个._sm历史
        sm历史.append(sm)
        sm_min = min(sm历史)
        sm_max = max(sm历史)
        if sm_max - sm_min > 1e-10:
            norm = (sm - sm_min) / (sm_max - sm_min)
            信号 = norm * 2 - 1
        else:
            信号 = 0.0
        return cls(时间戳=新时间, 收盘价=新收盘价, 均线周期=前一个.均线周期, 窗口=前一个.窗口, 信号=信号, _sm窗口=s窗口, _sm和=s和, _sm历史=sm历史)


# ======================= 12. RSI 反转策略 =======================
class RSIReversePositions:
    """RSI 反转策略信号"""

    def __init__(
        self,
        时间戳: datetime,
        收盘价: float,
        周期: int,
        rsi_upper: float,
        rsi_lower: float,
        rsi_exit: float,
        持仓信号: int = 0,
        _smoothed窗口: Optional[Deque[float]] = None,
        _smoothed和: float = 0.0,
        _变化序列: Optional[Deque[float]] = None,
        _当前持仓: int = 0,
    ):
        self.时间戳 = 时间戳
        self.收盘价 = 收盘价
        self.周期 = 周期
        self.rsi_upper = rsi_upper
        self.rsi_lower = rsi_lower
        self.rsi_exit = rsi_exit
        self.持仓信号 = 持仓信号
        self._smoothed窗口: Deque[float] = _smoothed窗口 if _smoothed窗口 is not None else deque()
        self._smoothed和: float = _smoothed和
        self._变化序列: Deque[float] = _变化序列 if _变化序列 is not None else deque()
        self._当前持仓: int = _当前持仓

    @classmethod
    def 首次计算(cls, 序列: List[float], 时间序列: List[datetime], 周期: int, rsi_upper: float, rsi_lower: float, rsi_exit: float) -> "RSIReversePositions":
        n = 周期
        if len(序列) < n:
            raise ValueError
        smoothed_list = [sum(序列[i - n + 1 : i + 1]) / n for i in range(n - 1, len(序列))]
        rsi_vals = [None] * len(序列)
        for i in range(1, len(序列)):
            if i < n - 1:
                continue
            gains = 0.0
            losses = 0.0
            valid = 0
            start = max(0, i - n)
            for j in range(start, i):
                平滑j = j - n + 1
                if 平滑j < 1 or 平滑j >= len(smoothed_list):
                    continue
                if smoothed_list[平滑j - 1] is not None and smoothed_list[平滑j] is not None:
                    change = smoothed_list[平滑j] - smoothed_list[平滑j - 1]
                    if change > 0:
                        gains += change
                    else:
                        losses += abs(change)
                    valid += 1
            if valid >= n - 1:
                if losses == 0:
                    rsi_vals[i] = 100.0
                else:
                    avg_gain = gains / valid
                    avg_loss = losses / valid
                    rs = avg_gain / avg_loss
                    rsi_vals[i] = 100.0 - (100.0 / (1 + rs))
        最后 = len(序列) - 1
        while 最后 >= 0 and rsi_vals[最后] is None:
            最后 -= 1
        i = 最后
        rsi = rsi_vals[i]
        pos = 0
        if rsi < rsi_lower:
            pos = 1
        elif rsi > rsi_upper:
            pos = -1
        obj = cls(
            时间戳=时间序列[i],
            收盘价=序列[i],
            周期=周期,
            rsi_upper=rsi_upper,
            rsi_lower=rsi_lower,
            rsi_exit=rsi_exit,
            持仓信号=pos,
            _smoothed窗口=deque(序列[i - n + 1 : i + 1], maxlen=n),
            _smoothed和=sum(序列[i - n + 1 : i + 1]),
            _变化序列=deque([smoothed_list[r] for r in range(max(0, i - 2 * n + 1), len(smoothed_list)) if smoothed_list[r] is not None], maxlen=n + 1),
            _当前持仓=pos,
        )
        return obj

    @classmethod
    def 增量计算(cls, 前一个: "RSIReversePositions", 新收盘价: float, 新时间: datetime) -> "RSIReversePositions":
        s窗口 = 前一个._smoothed窗口
        s旧 = s窗口[0]
        s窗口.append(新收盘价)
        s和 = 前一个._smoothed和 + 新收盘价 - s旧
        smoothed = s和 / 前一个.周期

        变化序列 = 前一个._变化序列
        变化序列.append(smoothed)

        gains = losses = 0.0
        n = 前一个.周期
        vals = list(变化序列)[-n - 1 :]
        valid = 0
        for j in range(1, len(vals)):
            if vals[j] is not None and vals[j - 1] is not None:
                ch = vals[j] - vals[j - 1]
                if ch > 0:
                    gains += ch
                else:
                    losses += abs(ch)
                valid += 1
        if valid >= n - 1 and losses != 0:
            rs = (gains / valid) / (losses / valid)
            rsi = 100 - 100 / (1 + rs)
        elif losses == 0:
            rsi = 100.0
        else:
            rsi = None

        pos = 前一个._当前持仓
        if rsi is not None:
            if pos == 0:
                if rsi < 前一个.rsi_lower:
                    pos = 1
                elif rsi > 前一个.rsi_upper:
                    pos = -1
            elif pos == 1:
                if rsi > 前一个.rsi_exit:
                    pos = 0
            elif pos == -1:
                if rsi < 前一个.rsi_exit:
                    pos = 0
        return cls(时间戳=新时间, 收盘价=新收盘价, 周期=前一个.周期, rsi_upper=前一个.rsi_upper, rsi_lower=前一个.rsi_lower, rsi_exit=前一个.rsi_exit, 持仓信号=pos, _smoothed窗口=s窗口, _smoothed和=s和, _变化序列=变化序列, _当前持仓=pos)


# ======================= 13. tanh 多空策略 =======================
class TanhPositions:
    """tanh 多空策略信号"""

    def __init__(
        self,
        时间戳: datetime,
        收盘价: float,
        周期: int,
        信号: float = 0.0,
        _ms窗口: Optional[Deque[float]] = None,
        _ms和: float = 0.0,
        _ms历史: Optional[Deque[float]] = None,
    ):
        self.时间戳 = 时间戳
        self.收盘价 = 收盘价
        self.周期 = 周期
        self.信号 = 信号
        self._ms窗口: Deque[float] = _ms窗口 if _ms窗口 is not None else deque()
        self._ms和: float = _ms和
        self._ms历史: Deque[float] = _ms历史 if _ms历史 is not None else deque()

    @classmethod
    def 首次计算(cls, 序列: List[float], 时间序列: List[datetime], 周期: int) -> "TanhPositions":
        n = 周期
        需要 = 2 * n - 1
        if len(序列) < 需要:
            raise ValueError
        ms_list = [sum(序列[i - n + 1 : i + 1]) / n for i in range(n - 1, len(序列))]
        有效 = n - 1 + n - 1
        i = 有效
        ms_win = ms_list[i - n + 1 : i + 1]
        mean = sum(ms_win) / n
        var = sum((x - mean) ** 2 for x in ms_win) / (n - 1)
        std = math.sqrt(var)
        z = (ms_list[i] - mean) / std if std > 0 else 0.0
        sig = math.tanh(z)
        obj = cls(时间戳=时间序列[i], 收盘价=序列[i], 周期=周期, 信号=round(sig, 2), _ms窗口=deque(序列[i - n + 1 : i + 1], maxlen=n), _ms和=sum(序列[i - n + 1 : i + 1]), _ms历史=deque(ms_win, maxlen=n))
        return obj

    @classmethod
    def 增量计算(cls, 前一个: "TanhPositions", 新收盘价: float, 新时间: datetime) -> "TanhPositions":
        ms窗口 = 前一个._ms窗口
        旧 = ms窗口[0]
        ms窗口.append(新收盘价)
        ms和 = 前一个._ms和 + 新收盘价 - 旧
        ms = ms和 / 前一个.周期
        ms历史 = 前一个._ms历史
        ms历史.append(ms)
        n = 前一个.周期
        mean = sum(ms历史) / n
        var = sum((x - mean) ** 2 for x in ms历史) / (n - 1)
        std = math.sqrt(var)
        z = (ms - mean) / std if std > 0 else 0.0
        sig = math.tanh(z)
        return cls(时间戳=新时间, 收盘价=新收盘价, 周期=前一个.周期, 信号=round(sig, 2), _ms窗口=ms窗口, _ms和=ms和, _ms历史=ms历史)


# ======================= 14. rank 多空策略 =======================
class RankPositions:
    """Rank 多空策略"""

    def __init__(
        self,
        时间戳: datetime,
        收盘价: float,
        周期: int,
        信号: float = 0.0,
        _ms窗口: Optional[Deque[float]] = None,
        _ms和: float = 0.0,
        _ms历史: Optional[Deque[float]] = None,
    ):
        self.时间戳 = 时间戳
        self.收盘价 = 收盘价
        self.周期 = 周期
        self.信号 = 信号
        self._ms窗口: Deque[float] = _ms窗口 if _ms窗口 is not None else deque()
        self._ms和: float = _ms和
        self._ms历史: Deque[float] = _ms历史 if _ms历史 is not None else deque()

    @classmethod
    def 首次计算(cls, 序列: List[float], 时间序列: List[datetime], 周期: int) -> "RankPositions":
        n = 周期
        if len(序列) < 2 * n - 1:
            raise ValueError
        ms_list = [sum(序列[i - n + 1 : i + 1]) / n for i in range(n - 1, len(序列))]
        i = 2 * n - 2
        win = ms_list[i - n + 1 : i + 1]
        cur = ms_list[i]
        rank = sum(1 for x in win if x < cur) + 1
        norm_rank = (rank - 1) / (n - 1)
        sig = (norm_rank - 0.5) * 2
        obj = cls(时间戳=时间序列[i], 收盘价=序列[i], 周期=周期, 信号=round(sig, 2), _ms窗口=deque(序列[i - n + 1 : i + 1], maxlen=n), _ms和=sum(序列[i - n + 1 : i + 1]), _ms历史=deque(win, maxlen=n))
        return obj

    @classmethod
    def 增量计算(cls, 前一个: "RankPositions", 新收盘价: float, 新时间: datetime) -> "RankPositions":
        ms窗口 = 前一个._ms窗口
        旧 = ms窗口[0]
        ms窗口.append(新收盘价)
        ms和 = 前一个._ms和 + 新收盘价 - 旧
        ms = ms和 / 前一个.周期
        ms历史 = 前一个._ms历史
        ms历史.append(ms)
        cur = ms
        rank = sum(1 for x in ms历史 if x < cur) + 1
        n = 前一个.周期
        norm = (rank - 1) / (n - 1)
        sig = (norm - 0.5) * 2
        return cls(时间戳=新时间, 收盘价=新收盘价, 周期=前一个.周期, 信号=round(sig, 2), _ms窗口=ms窗口, _ms和=ms和, _ms历史=ms历史)


# ======================= 15. EMA =======================
class EMA:
    """指数移动平均"""

    def __init__(
        self,
        时间戳: datetime,
        收盘价: float,
        周期: int,
        ema值: float = 0.0,
        _种子和: float = 0.0,
        _计数: int = 0,
        _上一个ema: float = 0.0,
    ):
        self.时间戳 = 时间戳
        self.收盘价 = 收盘价
        self.周期 = 周期
        self.ema值 = ema值
        self._种子和: float = _种子和
        self._计数: int = _计数
        self._上一个ema: float = _上一个ema

    @classmethod
    def 首次计算(cls, 序列: List[float], 时间序列: List[datetime], 周期: int) -> "EMA":
        if len(序列) < 周期:
            raise ValueError
        种子 = sum(序列[:周期]) / 周期
        obj = cls(时间戳=时间序列[周期 - 1], 收盘价=序列[周期 - 1], 周期=周期, ema值=种子, _上一个ema=种子, _计数=周期)
        return obj

    @classmethod
    def 增量计算(cls, 前一个: "EMA", 新收盘价: float, 新时间: datetime) -> "EMA":
        if 前一个._计数 < 前一个.周期:
            和 = 前一个._种子和 + 新收盘价
            计数 = 前一个._计数 + 1
            if 计数 == 前一个.周期:
                ema = 和 / 前一个.周期
            else:
                return cls(时间戳=新时间, 收盘价=新收盘价, 周期=前一个.周期, ema值=0.0, _种子和=和, _计数=计数, _上一个ema=0.0)
        else:
            alpha = _ema_alpha(前一个.周期)
            ema = alpha * 新收盘价 + (1 - alpha) * 前一个._上一个ema
        return cls(时间戳=新时间, 收盘价=新收盘价, 周期=前一个.周期, ema值=ema, _种子和=前一个._种子和, _计数=前一个.周期, _上一个ema=ema)


# ======================= 16. True Range =======================
class TrueRange:
    """真实波幅"""

    def __init__(
        self,
        时间戳: datetime,
        最高: float,
        最低: float,
        前收: float,
        TR: float = 0.0,
    ):
        self.时间戳 = 时间戳
        self.最高 = 最高
        self.最低 = 最低
        self.前收 = 前收
        self.TR = TR

    @classmethod
    def 首次计算(cls, high: List[float], low: List[float], close: List[float], 时间序列: List[datetime]) -> "TrueRange":
        if not (len(high) == len(low) == len(close)):
            raise ValueError
        tr = high[0] - low[0]
        return cls(时间戳=时间序列[0], 最高=high[0], 最低=low[0], 前收=close[0], TR=tr)

    @classmethod
    def 增量计算(cls, 前一个: "TrueRange", 新高: float, 新低: float, 新收: float, 新时间: datetime) -> "TrueRange":
        前收 = 前一个.前收
        tr = max(新高 - 新低, abs(新高 - 前收), abs(新低 - 前收))
        return cls(时间戳=新时间, 最高=新高, 最低=新低, 前收=新收, TR=tr)


# ======================= 17. RSX-SS2 =======================
class RSXSS2:
    """RSX-SS2 平滑 RSI"""

    def __init__(
        self,
        时间戳: datetime,
        收盘价: float,
        周期: int,
        平滑周期: int,
        rsx: float = 0.0,
        _上一个收: float = 0.0,
        _avg_gain_ema: float = 0.0,
        _avg_loss_ema: float = 0.0,
        _rsi历史: Optional[Deque[float]] = None,
        _us: Optional[UltimateSmoother] = None,
    ):
        self.时间戳 = 时间戳
        self.收盘价 = 收盘价
        self.周期 = 周期
        self.平滑周期 = 平滑周期
        self.rsx = rsx
        self._上一个收: float = _上一个收
        self._avg_gain_ema: float = _avg_gain_ema
        self._avg_loss_ema: float = _avg_loss_ema
        self._rsi历史: Deque[float] = _rsi历史 if _rsi历史 is not None else deque()
        self._us: Optional[UltimateSmoother] = _us

    @classmethod
    def 首次计算(cls, 序列: List[float], 时间序列: List[datetime], 周期: int, 平滑周期: int) -> "RSXSS2":
        if len(序列) < 周期:
            raise ValueError
        gains = []
        losses = []
        for i in range(1, len(序列)):
            delta = 序列[i] - 序列[i - 1]
            if delta > 0:
                gains.append(delta)
                losses.append(0.0)
            else:
                gains.append(0.0)
                losses.append(-delta)
        alpha = 1.0 / 周期
        avg_gain = gains[0]
        avg_loss = losses[0]
        rsi_vals = [0.0] * len(序列)
        rsi_vals[0] = 0.0
        for i in range(1, len(序列)):
            if i < len(gains):
                avg_gain = alpha * gains[i] + (1 - alpha) * avg_gain
                avg_loss = alpha * losses[i] + (1 - alpha) * avg_loss
            if avg_loss == 0:
                rsi_vals[i] = 100.0
            else:
                rs = avg_gain / avg_loss
                rsi_vals[i] = 100.0 - 100.0 / (1 + rs)
        us_obj = UltimateSmoother.首次计算(rsi_vals, 时间序列, 平滑周期)
        last_val = us_obj.平滑值
        obj = cls(时间戳=时间序列[-1], 收盘价=序列[-1], 周期=周期, 平滑周期=平滑周期, rsx=last_val, _上一个收=序列[-1], _avg_gain_ema=avg_gain, _avg_loss_ema=avg_loss, _rsi历史=deque(rsi_vals[-平滑周期:], maxlen=平滑周期), _us=us_obj)
        return obj

    @classmethod
    def 增量计算(cls, 前一个: "RSXSS2", 新收盘价: float, 新时间: datetime) -> "RSXSS2":
        delta = 新收盘价 - 前一个._上一个收
        gain = delta if delta > 0 else 0.0
        loss = -delta if delta < 0 else 0.0
        alpha = 1.0 / 前一个.周期
        avg_gain = alpha * gain + (1 - alpha) * 前一个._avg_gain_ema
        avg_loss = alpha * loss + (1 - alpha) * 前一个._avg_loss_ema
        if avg_loss == 0:
            rsi = 100.0
        else:
            rs = avg_gain / avg_loss
            rsi = 100.0 - 100.0 / (1 + rs)
        新us = UltimateSmoother.增量计算(前一个._us, rsi, 新时间)
        return cls(时间戳=新时间, 收盘价=新收盘价, 周期=前一个.周期, 平滑周期=前一个.平滑周期, rsx=新us.平滑值, _上一个收=新收盘价, _avg_gain_ema=avg_gain, _avg_loss_ema=avg_loss, _rsi历史=前一个._rsi历史, _us=新us)


# ======================= 18. Jurik Volty =======================
class JurikVolty:
    """Jurik波动平滑器 — 低噪声波动指标"""

    def __init__(
        self,
        时间戳: datetime,
        收盘价: float,
        周期: int,
        强度: float,
        波动率: float = 0.0,
        _上一个收盘: float = 0.0,
        _smooth1_ema: float = 0.0,
        _smooth2_ema: float = 0.0,
        _jurik_val: float = 0.0,
        _result_ema: float = 0.0,
        _初始化完成: bool = False,
    ):
        self.时间戳 = 时间戳
        self.收盘价 = 收盘价
        self.周期 = 周期
        self.强度 = 强度
        self.波动率 = 波动率
        self._上一个收盘: float = _上一个收盘
        self._smooth1_ema: float = _smooth1_ema
        self._smooth2_ema: float = _smooth2_ema
        self._jurik_val: float = _jurik_val
        self._result_ema: float = _result_ema
        self._初始化完成: bool = _初始化完成

    @classmethod
    def 首次计算(cls, 序列: List[float], 时间序列: List[datetime], 周期: int, 强度: float) -> "JurikVolty":
        if len(序列) < 周期:
            raise ValueError("数据长度不足")
        changes = [abs(序列[i] - 序列[i - 1]) for i in range(1, len(序列))]
        if not changes:
            raise ValueError("数据过短")
        span1 = 周期 // 2
        alpha1 = 2.0 / (span1 + 1)
        smooth1 = [changes[0]]
        for c in changes[1:]:
            smooth1.append(alpha1 * c + (1 - alpha1) * smooth1[-1])
        smooth2 = [smooth1[0]]
        for s in smooth1[1:]:
            smooth2.append(alpha1 * s + (1 - alpha1) * smooth2[-1])
        jv = [0.0] * len(序列)
        for i in range(2, len(序列)):
            idx = i - 1
            if idx >= len(smooth2):
                break
            jv[i] = (smooth2[idx] + 0.5 * (smooth2[idx] - smooth2[idx - 1])) * 强度
        span3 = 周期 // 3
        alpha3 = 2.0 / (span3 + 1)
        result = [0.0] * len(序列)
        first_valid = next((i for i, v in enumerate(jv) if v != 0.0), None)
        if first_valid is None:
            raise ValueError("无法初始化")
        result[first_valid] = jv[first_valid]
        for i in range(first_valid + 1, len(序列)):
            result[i] = alpha3 * jv[i] + (1 - alpha3) * result[i - 1]
        last_val = result[-1]
        obj = cls(时间戳=时间序列[-1], 收盘价=序列[-1], 周期=周期, 强度=强度, 波动率=last_val, _上一个收盘=序列[-1], _smooth1_ema=smooth1[-1], _smooth2_ema=smooth2[-1], _jurik_val=jv[-1], _result_ema=last_val, _初始化完成=True)
        return obj

    @classmethod
    def 增量计算(cls, 前一个: "JurikVolty", 新收盘价: float, 新时间: datetime) -> "JurikVolty":
        if not 前一个._初始化完成:
            raise RuntimeError("请先使用首次计算进行初始化")
        change = abs(新收盘价 - 前一个._上一个收盘)
        span1 = 前一个.周期 // 2
        alpha1 = 2.0 / (span1 + 1)
        smooth1 = alpha1 * change + (1 - alpha1) * 前一个._smooth1_ema
        smooth2 = alpha1 * smooth1 + (1 - alpha1) * 前一个._smooth2_ema
        jurik = (smooth2 + 0.5 * (smooth2 - 前一个._smooth2_ema)) * 前一个.强度
        span3 = 前一个.周期 // 3
        alpha3 = 2.0 / (span3 + 1)
        result = alpha3 * jurik + (1 - alpha3) * 前一个._result_ema
        return cls(时间戳=新时间, 收盘价=新收盘价, 周期=前一个.周期, 强度=前一个.强度, 波动率=result, _上一个收盘=新收盘价, _smooth1_ema=smooth1, _smooth2_ema=smooth2, _jurik_val=jurik, _result_ema=result, _初始化完成=True)


# ======================= 19. Ultimate Channel =======================
class UltimateChannel:
    """终极通道 — 基于终极平滑器的通道指标"""

    def __init__(
        self,
        时间戳: datetime,
        最高: float,
        最低: float,
        收盘价: float,
        周期: int,
        乘数: float,
        中线: float = 0.0,
        上轨: float = 0.0,
        下轨: float = 0.0,
        _us: Optional[UltimateSmoother] = None,
        _atr_ema: float = 0.0,
        _atr_us: Optional[UltimateSmoother] = None,
        _prev_close: float = 0.0,
    ):
        self.时间戳 = 时间戳
        self.最高 = 最高
        self.最低 = 最低
        self.收盘价 = 收盘价
        self.周期 = 周期
        self.乘数 = 乘数
        self.中线 = 中线
        self.上轨 = 上轨
        self.下轨 = 下轨
        self._us: Optional[UltimateSmoother] = _us
        self._atr_ema: float = _atr_ema
        self._atr_us: Optional[UltimateSmoother] = _atr_us
        self._prev_close: float = _prev_close

    @classmethod
    def 首次计算(cls, high: List[float], low: List[float], close: List[float], 时间序列: List[datetime], 周期: int, 乘数: float) -> "UltimateChannel":
        n = 周期
        if len(close) < n:
            raise ValueError("数据长度不足")
        us_mid = UltimateSmoother.首次计算(close, 时间序列, float(n))
        tr_vals = []
        prev_close = close[0]
        for i in range(len(high)):
            if i == 0:
                tr = high[i] - low[i]
            else:
                tr = max(high[i] - low[i], abs(high[i] - prev_close), abs(low[i] - prev_close))
            tr_vals.append(tr)
            prev_close = close[i]
        alpha = 1.0 / n
        atr_ema = sum(tr_vals[:n]) / n
        for i in range(n, len(tr_vals)):
            atr_ema = alpha * tr_vals[i] + (1 - alpha) * atr_ema
        atr_seq = [0.0] * len(close)
        atr_seq[n - 1] = sum(tr_vals[:n]) / n
        for i in range(n, len(close)):
            atr_seq[i] = alpha * tr_vals[i] + (1 - alpha) * atr_seq[i - 1]
        us_atr = UltimateSmoother.首次计算(atr_seq, 时间序列, float(n // 2))
        last = len(close) - 1
        mid = us_mid.平滑值
        str_val = us_atr.平滑值
        upper = mid + 乘数 * str_val
        lower = mid - 乘数 * str_val
        obj = cls(时间戳=时间序列[last], 最高=high[last], 最低=low[last], 收盘价=close[last], 周期=周期, 乘数=乘数, 中线=mid, 上轨=upper, 下轨=lower, _us=us_mid, _atr_ema=atr_ema, _atr_us=us_atr, _prev_close=close[last])
        return obj

    @classmethod
    def 增量计算(cls, 前一个: "UltimateChannel", 新高: float, 新低: float, 新收: float, 新时间: datetime) -> "UltimateChannel":
        新us = UltimateSmoother.增量计算(前一个._us, 新收, 新时间)
        tr = max(新高 - 新低, abs(新高 - 前一个._prev_close), abs(新低 - 前一个._prev_close))
        alpha = 1.0 / 前一个.周期
        atr_ema = alpha * tr + (1 - alpha) * 前一个._atr_ema
        新us_atr = UltimateSmoother.增量计算(前一个._atr_us, atr_ema, 新时间)
        mid = 新us.平滑值
        str_val = 新us_atr.平滑值
        upper = mid + 前一个.乘数 * str_val
        lower = mid - 前一个.乘数 * str_val
        return cls(时间戳=新时间, 最高=新高, 最低=新低, 收盘价=新收, 周期=前一个.周期, 乘数=前一个.乘数, 中线=mid, 上轨=upper, 下轨=lower, _us=新us, _atr_ema=atr_ema, _atr_us=新us_atr, _prev_close=新收)


# ======================= 20. Ultimate Bands =======================
class UltimateBands:
    """终极带 — 基于终极平滑器的布林带变体"""

    def __init__(
        self,
        时间戳: datetime,
        收盘价: float,
        周期: int,
        标准差乘数: float,
        平滑周期: int,
        中线: float = 0.0,
        上轨: float = 0.0,
        下轨: float = 0.0,
        _us_mid: Optional[UltimateSmoother] = None,
        _close窗口: Optional[Deque[float]] = None,
        _std_us: Optional[UltimateSmoother] = None,
    ):
        self.时间戳 = 时间戳
        self.收盘价 = 收盘价
        self.周期 = 周期
        self.标准差乘数 = 标准差乘数
        self.平滑周期 = 平滑周期
        self.中线 = 中线
        self.上轨 = 上轨
        self.下轨 = 下轨
        self._us_mid: Optional[UltimateSmoother] = _us_mid
        self._close窗口: Deque[float] = _close窗口 if _close窗口 is not None else deque()
        self._std_us: Optional[UltimateSmoother] = _std_us

    @classmethod
    def 首次计算(cls, 序列: List[float], 时间序列: List[datetime], 周期: int, 标准差乘数: float, 平滑周期: int) -> "UltimateBands":
        n = 周期
        if len(序列) < n:
            raise ValueError("数据长度不足")
        us_mid = UltimateSmoother.首次计算(序列, 时间序列, float(n))
        std_seq = [0.0] * len(序列)
        for i in range(n - 1, len(序列)):
            win = 序列[i - n + 1 : i + 1]
            mean = sum(win) / n
            var = sum((x - mean) ** 2 for x in win) / (n - 1)
            std_seq[i] = math.sqrt(var)
        for i in range(n - 1):
            std_seq[i] = float("nan")
        us_std = UltimateSmoother.首次计算(std_seq, 时间序列, float(平滑周期))
        last = len(序列) - 1
        mid = us_mid.平滑值
        smooth_std = us_std.平滑值
        upper = mid + 标准差乘数 * smooth_std
        lower = mid - 标准差乘数 * smooth_std
        win_deque = deque(序列[last - n + 1 : last + 1], maxlen=n)
        return cls(时间戳=时间序列[last], 收盘价=序列[last], 周期=周期, 标准差乘数=标准差乘数, 平滑周期=平滑周期, 中线=mid, 上轨=upper, 下轨=lower, _us_mid=us_mid, _close窗口=win_deque, _std_us=us_std)

    @classmethod
    def 增量计算(cls, 前一个: "UltimateBands", 新收盘价: float, 新时间: datetime) -> "UltimateBands":
        新us = UltimateSmoother.增量计算(前一个._us_mid, 新收盘价, 新时间)
        win = 前一个._close窗口
        win.append(新收盘价)
        mean = sum(win) / len(win)
        var = sum((x - mean) ** 2 for x in win) / (len(win) - 1)
        std = math.sqrt(var)
        新std_us = UltimateSmoother.增量计算(前一个._std_us, std, 新时间)
        mid = 新us.平滑值
        smooth_std = 新std_us.平滑值
        upper = mid + 前一个.标准差乘数 * smooth_std
        lower = mid - 前一个.标准差乘数 * smooth_std
        return cls(时间戳=新时间, 收盘价=新收盘价, 周期=前一个.周期, 标准差乘数=前一个.标准差乘数, 平滑周期=前一个.平滑周期, 中线=mid, 上轨=upper, 下轨=lower, _us_mid=新us, _close窗口=win, _std_us=新std_us)


# ======================= 21. Ultimate Oscillator =======================
class UltimateOscillator:
    """终极波动指标 (UOS) — 多周期融合"""

    def __init__(
        self,
        时间戳: datetime,
        最高: float,
        最低: float,
        收盘价: float,
        短周期: int,
        中周期: int,
        长周期: int,
        UOS: float = 0.0,
        _bp_short: Optional[Deque[float]] = None,
        _tr_short: Optional[Deque[float]] = None,
        _bp_med: Optional[Deque[float]] = None,
        _tr_med: Optional[Deque[float]] = None,
        _bp_long: Optional[Deque[float]] = None,
        _tr_long: Optional[Deque[float]] = None,
        _prev_close: float = 0.0,
    ):
        self.时间戳 = 时间戳
        self.最高 = 最高
        self.最低 = 最低
        self.收盘价 = 收盘价
        self.短周期 = 短周期
        self.中周期 = 中周期
        self.长周期 = 长周期
        self.UOS = UOS
        self._bp_short: Deque[float] = _bp_short if _bp_short is not None else deque()
        self._tr_short: Deque[float] = _tr_short if _tr_short is not None else deque()
        self._bp_med: Deque[float] = _bp_med if _bp_med is not None else deque()
        self._tr_med: Deque[float] = _tr_med if _tr_med is not None else deque()
        self._bp_long: Deque[float] = _bp_long if _bp_long is not None else deque()
        self._tr_long: Deque[float] = _tr_long if _tr_long is not None else deque()
        self._prev_close: float = _prev_close

    @classmethod
    def 首次计算(cls, high: List[float], low: List[float], close: List[float], 时间序列: List[datetime], 短: int = 7, 中: int = 14, 长: int = 28) -> "UltimateOscillator":
        if len(close) < 长:
            raise ValueError("数据长度至少等于长周期")
        prev_close = close[0]
        bp_short = deque(maxlen=短)
        tr_short = deque(maxlen=短)
        bp_med = deque(maxlen=中)
        tr_med = deque(maxlen=中)
        bp_long = deque(maxlen=长)
        tr_long = deque(maxlen=长)
        uos_vals = []
        for i in range(1, len(close)):
            true_low = min(low[i], prev_close)
            bp = close[i] - true_low
            tr = max(high[i] - low[i], abs(high[i] - prev_close), abs(low[i] - prev_close))
            bp_short.append(bp)
            tr_short.append(tr)
            bp_med.append(bp)
            tr_med.append(tr)
            bp_long.append(bp)
            tr_long.append(tr)
            if len(bp_long) == 长:
                avg7 = sum(bp_short) / sum(tr_short) if sum(tr_short) != 0 else 0.0
                avg14 = sum(bp_med) / sum(tr_med) if sum(tr_med) != 0 else 0.0
                avg28 = sum(bp_long) / sum(tr_long) if sum(tr_long) != 0 else 0.0
                uos = 100.0 * (4.0 * avg7 + 2.0 * avg14 + avg28) / 7.0
                uos_vals.append(uos)
            else:
                uos_vals.append(float("nan"))
            prev_close = close[i]
        last = len(uos_vals) - 1
        while last >= 0 and math.isnan(uos_vals[last]):
            last -= 1
        if last < 0:
            raise ValueError("无法计算有效UOS")
        obj = cls(时间戳=时间序列[last], 最高=high[last], 最低=low[last], 收盘价=close[last], 短周期=短, 中周期=中, 长周期=长, UOS=uos_vals[last], _bp_short=bp_short, _tr_short=tr_short, _bp_med=bp_med, _tr_med=tr_med, _bp_long=bp_long, _tr_long=tr_long, _prev_close=close[last])
        return obj

    @classmethod
    def 增量计算(cls, 前一个: "UltimateOscillator", 新高: float, 新低: float, 新收: float, 新时间: datetime) -> "UltimateOscillator":
        true_low = min(新低, 前一个._prev_close)
        bp = 新收 - true_low
        tr = max(新高 - 新低, abs(新高 - 前一个._prev_close), abs(新低 - 前一个._prev_close))
        bp_short = 前一个._bp_short
        tr_short = 前一个._tr_short
        bp_med = 前一个._bp_med
        tr_med = 前一个._tr_med
        bp_long = 前一个._bp_long
        tr_long = 前一个._tr_long
        bp_short.append(bp)
        tr_short.append(tr)
        bp_med.append(bp)
        tr_med.append(tr)
        bp_long.append(bp)
        tr_long.append(tr)
        if len(bp_long) < 前一个.长周期:
            uos = float("nan")
        else:
            avg7 = sum(bp_short) / sum(tr_short) if sum(tr_short) != 0 else 0.0
            avg14 = sum(bp_med) / sum(tr_med) if sum(tr_med) != 0 else 0.0
            avg28 = sum(bp_long) / sum(tr_long) if sum(tr_long) != 0 else 0.0
            uos = 100.0 * (4.0 * avg7 + 2.0 * avg14 + avg28) / 7.0
        return cls(时间戳=新时间, 最高=新高, 最低=新低, 收盘价=新收, 短周期=前一个.短周期, 中周期=前一个.中周期, 长周期=前一个.长周期, UOS=uos, _bp_short=bp_short, _tr_short=tr_short, _bp_med=bp_med, _tr_med=tr_med, _bp_long=bp_long, _tr_long=tr_long, _prev_close=新收)


# ======================= 22. Exponential Smoothing =======================
class ExponentialSmoothing:
    """指数平滑"""

    def __init__(
        self,
        时间戳: datetime,
        值: float,
        alpha: float,
        平滑值: float = 0.0,
        _上一个平滑: Optional[float] = None,
    ):
        self.时间戳 = 时间戳
        self.值 = 值
        self.alpha = alpha
        self.平滑值 = 平滑值
        self._上一个平滑: float = _上一个平滑 if _上一个平滑 is not None else 平滑值

    @classmethod
    def 首次计算(cls, 序列: List[float], 时间序列: List[datetime], alpha: float) -> "ExponentialSmoothing":
        obj = cls(时间戳=时间序列[0], 值=序列[0], alpha=alpha, 平滑值=序列[0], _上一个平滑=序列[0])
        for i in range(1, len(序列)):
            obj = cls.增量计算(obj, 序列[i], 时间序列[i])
        return obj

    @classmethod
    def 增量计算(cls, 前一个: "ExponentialSmoothing", 新值: float, 新时间: datetime) -> "ExponentialSmoothing":
        平滑 = 前一个.alpha * 新值 + (1 - 前一个.alpha) * 前一个._上一个平滑
        return cls(时间戳=新时间, 值=新值, alpha=前一个.alpha, 平滑值=平滑, _上一个平滑=平滑)


# ======================= 23. Holt-Winters =======================
class HoltWinters:
    """Holt-Winters 三参数平滑"""

    def __init__(
        self,
        时间戳: datetime,
        值: float,
        季节长度: int,
        alpha: float,
        beta: float,
        gamma: float,
        拟合值: float = 0.0,
        _level: Optional[float] = None,
        _trend: float = 0.0,
        _季节: Optional[Deque[float]] = None,
    ):
        self.时间戳 = 时间戳
        self.值 = 值
        self.季节长度 = 季节长度
        self.alpha = alpha
        self.beta = beta
        self.gamma = gamma
        self.拟合值 = 拟合值
        self._level: float = _level if _level is not None else 值
        self._trend: float = _trend
        self._季节: Deque[float] = _季节 if _季节 is not None else deque()

    @classmethod
    def 首次计算(cls, 序列: List[float], 时间序列: List[datetime], 季节长度: int, alpha: float, beta: float, gamma: float) -> "HoltWinters":
        if len(序列) < 季节长度:
            raise ValueError
        initial_level = sum(序列[:季节长度]) / 季节长度
        level = initial_level
        trend = 0.0
        季节 = deque([序列[i] - initial_level for i in range(季节长度)], maxlen=季节长度)
        fit = 序列[季节长度 - 1]
        obj = cls(时间戳=时间序列[季节长度 - 1], 值=序列[季节长度 - 1], 季节长度=季节长度, alpha=alpha, beta=beta, gamma=gamma, 拟合值=fit, _level=level, _trend=trend, _季节=季节)
        for i in range(季节长度, len(序列)):
            obj = cls.增量计算(obj, 序列[i], 时间序列[i])
        return obj

    @classmethod
    def 增量计算(cls, 前一个: "HoltWinters", 新值: float, 新时间: datetime) -> "HoltWinters":
        L = 前一个.季节长度
        season_old = 前一个._季节[0]
        level = 前一个.alpha * (新值 - season_old) + (1 - 前一个.alpha) * (前一个._level + 前一个._trend)
        trend = 前一个.beta * (level - 前一个._level) + (1 - 前一个.beta) * 前一个._trend
        season_new = 前一个.gamma * (新值 - level) + (1 - 前一个.gamma) * season_old
        季节 = 前一个._季节
        季节.append(season_new)
        fit = level + trend + season_new
        return cls(时间戳=新时间, 值=新值, 季节长度=L, alpha=前一个.alpha, beta=前一个.beta, gamma=前一个.gamma, 拟合值=fit, _level=level, _trend=trend, _季节=季节)
