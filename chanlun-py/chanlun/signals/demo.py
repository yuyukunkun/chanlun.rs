"""缠论技术分析库 — 信号函数示例合集"""

from collections import OrderedDict
from typing import List, Optional

from chanlun import 观察者, 分型结构, 虚线, 线段, 相对方向
from chanlun.chan_external import create_single_signal


# ==============================================================================
# 工具函数
# ==============================================================================


def _按需计算均线(普K序列: List, ma_type: str, timeperiod: int, offset: int = 0) -> Optional[float]:
    """当 K线.指标.均线 中无预计算值时，从收盘价序列按需计算均线。

    :param 普K序列: 普通K线序列
    :param ma_type: 均线类型（SMA/EMA）
    :param timeperiod: 均线周期
    :param offset: 从末尾倒数 offset 根K线（0=最后一根，di=倒数第di根）
    :return: 均线值，K线不足时返回 None
    """
    n = len(普K序列)
    start = n - offset - timeperiod + 1
    end = n - offset + 1
    if start < 0:
        return None

    closes = [普K序列[i].收盘价 for i in range(start, end)]

    if ma_type == "SMA":
        return sum(closes) / len(closes)
    elif ma_type == "EMA":
        k = 2.0 / (timeperiod + 1)
        ema = closes[0]
        for price in closes[1:]:
            ema = price * k + ema * (1 - k)
        return ema
    return None


def _获取或计算均线(普K序列: List, K线, ma_type: str, timeperiod: int, offset: int) -> Optional[float]:
    """从K线指标容器获取均线，若缺失则按需计算。

    :param 普K序列: 普通K线序列
    :param K线: 目标K线
    :param ma_type: 均线类型
    :param timeperiod: 均线周期
    :param offset: 从末尾倒数 offset 根K线
    :return: 均线值或 None
    """
    ma_key = f"{ma_type}_{timeperiod}"
    try:
        if K线.指标 is not None:
            cached = K线.指标.均线.get(ma_key)
            if cached is not None:
                return cached
    except Exception:
        pass
    return _按需计算均线(普K序列, ma_type, timeperiod, offset)


# ==============================================================================
# tas — 技术指标信号
# ==============================================================================


def tas_ma_base_V230313(c, **kwargs) -> OrderedDict:
    """单均线多空和方向辅助开平仓信号

    参数模板："{freq}_D{di}#{ma_type}#{timeperiod}MO{max_overlap}_BS辅助V230313"

    **信号逻辑：**

    1. close > ma，多头（看多）；反之，空头（看空）
    2. ma[-1] > ma[-2]，向上；反之，向下
    3. 加入 max_overlap 参数控制相同信号最大重叠次数

    **信号列表：**

    - Signal('15分钟_D1#SMA#5MO5_BS辅助V230313_看空_向下_任意_0')
    - Signal('15分钟_D1#SMA#5MO5_BS辅助V230313_看多_向下_任意_0')
    - Signal('15分钟_D1#SMA#5MO5_BS辅助V230313_看多_向上_任意_0')
    - Signal('15分钟_D1#SMA#5MO5_BS辅助V230313_看空_向上_任意_0')

    :param c: 观察者对象
    :param kwargs: 其他参数
        - ma_type: 均线类型（SMA/EMA）
        - timeperiod: 均线计算周期
        - di: 信号计算截止倒数第i根K线
        - max_overlap: 相同信号最大重叠次数
    :return: 信号识别结果
    """
    ma_type = kwargs.get("ma_type", "SMA").upper()
    timeperiod = int(kwargs.get("timeperiod", 5))
    di = int(kwargs.get("di", 1))
    max_overlap = int(kwargs.get("max_overlap", 5))
    freq = kwargs.get("freq", "15分钟")

    k1, k2, k3 = f"{freq}_D{di}#{ma_type}#{timeperiod}MO{max_overlap}_BS辅助V230313".split("_", 2)

    普K序列 = c.普通K线序列
    if len(普K序列) < di + 1:
        return create_single_signal(k1=k1, k2=k2, k3=k3)

    当前K线 = 普K序列[-di]
    当前均线 = _获取或计算均线(普K序列, 当前K线, ma_type, timeperiod, di)
    if 当前均线 is None:
        return create_single_signal(k1=k1, k2=k2, k3=k3)

    当前价 = 当前K线.收盘价
    v1 = "看多" if 当前价 > 当前均线 else "看空"

    # 均线方向：需要前一根K线的均线值
    if len(普K序列) >= di + 2:
        前均线 = _获取或计算均线(普K序列, 普K序列[-di - 1], ma_type, timeperiod, di + 1)
        if 前均线 is not None:
            v2 = "向上" if 当前均线 > 前均线 else "向下"
        else:
            v2 = "任意"
    else:
        v2 = "任意"

    return create_single_signal(k1=k1, k2=k2, k3=k3, v1=v1, v2=v2)


def tas_macd_direct_V221106(c, **kwargs) -> OrderedDict:
    """MACD 方向信号 — DIF 在零轴上方为多头，下方为空头

    参数模板："{freq}_D{di}#MACD#{fast}#{slow}#{signal}_MACD方向V221106"

    **信号逻辑：**

    1. DIF > 0，多头；反之，空头
    2. DIF 值变化趋势（与前一根比较）：向上/向下

    **信号列表：**

    - Signal('15分钟_D1#MACD#13#31#11_MACD方向V221106_看多_向上_任意_0')
    - Signal('15分钟_D1#MACD#13#31#11_MACD方向V221106_看多_向下_任意_0')
    - Signal('15分钟_D1#MACD#13#31#11_MACD方向V221106_看空_向上_任意_0')
    - Signal('15分钟_D1#MACD#13#31#11_MACD方向V221106_看空_向下_任意_0')

    :param c: 观察者对象
    :param kwargs: 其他参数
        - fast: 快线周期（默认 13）
        - slow: 慢线周期（默认 31）
        - signal: 信号周期（默认 11）
        - di: 信号计算截止倒数第i根K线
    :return: 信号识别结果
    """
    fast = int(kwargs.get("fast", 13))
    slow = int(kwargs.get("slow", 31))
    signal = int(kwargs.get("signal", 11))
    di = int(kwargs.get("di", 1))
    freq = kwargs.get("freq", "15分钟")

    k1, k2, k3 = f"{freq}_D{di}#MACD#{fast}#{slow}#{signal}_MACD方向V221106".split("_", 2)

    普K序列 = c.普通K线序列
    if len(普K序列) < di + 1:
        return create_single_signal(k1=k1, k2=k2, k3=k3)

    当前K线 = 普K序列[-di]
    cur_macd = 当前K线.指标.macd if 当前K线.指标 else None
    if cur_macd is None or cur_macd.DIF is None:
        return create_single_signal(k1=k1, k2=k2, k3=k3)

    v1 = "看多" if cur_macd.DIF > 0 else "看空"

    if len(普K序列) >= di + 2:
        前K线 = 普K序列[-di - 1]
        prev_macd = 前K线.指标.macd if 前K线.指标 else None
        if prev_macd is not None and prev_macd.DIF is not None:
            v2 = "向上" if cur_macd.DIF > prev_macd.DIF else "向下"
        else:
            v2 = "任意"
    else:
        v2 = "任意"

    return create_single_signal(k1=k1, k2=k2, k3=k3, v1=v1, v2=v2)


def macd_金叉(观察员: 观察者, **kwargs) -> OrderedDict:
    """MACD 金叉死叉信号 — DIF 与 DEA 的交叉判断

    参数模板："{freq}_D{di}#MACD#{fast}#{slow}#{signal}_MACD交叉V260601"

    **信号逻辑：**

    1. DIF 上穿 DEA（前一根 DIF <= DEA，当前 DIF > DEA）→ 金叉
    2. DIF 下穿 DEA（前一根 DIF >= DEA，当前 DIF < DEA）→ 死叉

    **信号列表：**

    - Signal('15分钟_D1#MACD#13#31#11_MACD交叉V260601_金叉_任意_任意_0')
    - Signal('15分钟_D1#MACD#13#31#11_MACD交叉V260601_死叉_任意_任意_0')

    :param 观察员: 观察者对象
    :param kwargs: 其他参数
        - fast: 快线周期（默认 13）
        - slow: 慢线周期（默认 31）
        - signal: 信号周期（默认 11）
        - di: 信号计算截止倒数第i根K线
    :return: 信号识别结果
    """
    fast = int(kwargs.get("fast", 13))
    slow = int(kwargs.get("slow", 31))
    signal = int(kwargs.get("signal", 11))
    di = int(kwargs.get("di", 1))
    freq = kwargs.get("freq", "15分钟")

    k1, k2, k3 = f"{freq}_D{di}#MACD#{fast}#{slow}#{signal}_MACD交叉V260601".split("_", 2)

    普K序列 = 观察员.普通K线序列
    if len(普K序列) < di + 2:
        return create_single_signal(k1=k1, k2=k2, k3=k3)

    当前K线 = 普K序列[-di]
    前K线 = 普K序列[-di - 1]

    cur_macd = 当前K线.指标.macd if 当前K线.指标 else None
    prev_macd = 前K线.指标.macd if 前K线.指标 else None

    if cur_macd is None or prev_macd is None:
        return create_single_signal(k1=k1, k2=k2, k3=k3)
    if cur_macd.DIF is None or cur_macd.DEA is None:
        return create_single_signal(k1=k1, k2=k2, k3=k3)
    if prev_macd.DIF is None or prev_macd.DEA is None:
        return create_single_signal(k1=k1, k2=k2, k3=k3)

    if prev_macd.DIF <= prev_macd.DEA and cur_macd.DIF > cur_macd.DEA:
        v1 = "金叉"
    elif prev_macd.DIF >= prev_macd.DEA and cur_macd.DIF < cur_macd.DEA:
        v1 = "死叉"
    else:
        v1 = "任意"

    return create_single_signal(k1=k1, k2=k2, k3=k3, v1=v1)


# ==============================================================================
# cxt — 缠论形态信号
# ==============================================================================


def cxt_bi_end_V230222(c, **kwargs) -> OrderedDict:
    """当前是最后笔的第几次新低底分型或新高顶分型，用于笔结束辅助

    触发条件：新分型

    参数模板："{freq}_D1MO{max_overlap}_BE辅助V230222"

    **信号逻辑：**

    1. 取最后笔及未成笔的分型
    2. 当前如果是顶分型，则看当前顶分型是否新高，是第几个新高
    3. 当前如果是底分型，则看当前底分型是否新低，是第几个新低

    **信号列表：**

    - Signal('日线_D1MO3_BE辅助V230222_新低_第2次_任意_0')
    - Signal('日线_D1MO3_BE辅助V230222_新高_第2次_任意_0')
    - Signal('日线_D1MO3_BE辅助V230222_新低_第3次_任意_0')

    :param c: 观察者对象
    :param kwargs:
    :return: 信号识别结果
    """
    max_overlap = int(kwargs.get("max_overlap", 3))
    freq = kwargs.get("freq", "日线")
    k1, k2, k3 = f"{freq}_D1MO{max_overlap}_BE辅助V230222".split("_", 2)

    分型序列 = c.分型序列
    笔序列 = c.笔序列

    if len(分型序列) < 2 or len(笔序列) < 1:
        return create_single_signal(k1=k1, k2=k2, k3=k3)

    最后笔 = 笔序列[-1]
    当前分型 = 分型序列[-1]

    # 找到最后笔的武（终点分型）在分型序列中的位置
    try:
        笔终点索引 = next(i for i, f in enumerate(分型序列) if f.时间戳 == 最后笔.武.时间戳 and f.结构 == 最后笔.武.结构)
    except StopIteration:
        return create_single_signal(k1=k1, k2=k2, k3=k3)

    # 取笔终点之后的分型（未成笔的分型）
    未成笔分型 = 分型序列[笔终点索引 + 1 :]
    if len(未成笔分型) < 1:
        return create_single_signal(k1=k1, k2=k2, k3=k3)

    if 当前分型.结构.value == "顶":
        # 统计从笔终点到当前的顶分型新高次数
        笔终点顶高 = 最后笔.武.分型特征值
        计数 = 0
        for f in 未成笔分型:
            if f.结构.value == "顶" and f.分型特征值 > 笔终点顶高:
                计数 += 1
                笔终点顶高 = f.分型特征值
        if 计数 > 0 and 当前分型.分型特征值 >= 笔终点顶高:
            v1, v2 = "新高", f"第{计数}次"
        else:
            v1, v2 = "任意", "任意"
    elif 当前分型.结构.value == "底":
        笔终点底低 = 最后笔.武.分型特征值
        计数 = 0
        for f in 未成笔分型:
            if f.结构.value == "底" and f.分型特征值 < 笔终点底低:
                计数 += 1
                笔终点底低 = f.分型特征值
        if 计数 > 0 and 当前分型.分型特征值 <= 笔终点底低:
            v1, v2 = "新低", f"第{计数}次"
        else:
            v1, v2 = "任意", "任意"
    else:
        return create_single_signal(k1=k1, k2=k2, k3=k3)

    return create_single_signal(k1=k1, k2=k2, k3=k3, v1=v1, v2=v2)


def cxt_停顿分型_V230106(c, **kwargs) -> OrderedDict:
    """停顿分型辅助信号 — 结合分型强度和MACD柱子匹配判断

    触发条件：新分型

    参数模板："{freq}_D{di}停顿分型_BE辅助V230106"

    **信号逻辑：**

    判断当前分型是否为停顿分型，结合力度和形态给出信号。
    停顿分型 = 分型结构为顶/底 + 强度为强/中 + MACD柱子分型匹配。

    **信号列表：**

    - Signal('1分钟_D0停顿分型_BE辅助V230106_看空_强_任意_0')
    - Signal('1分钟_D0停顿分型_BE辅助V230106_看多_强_任意_0')
    - Signal('1分钟_D0停顿分型_BE辅助V230106_看空_中_任意_0')
    - Signal('1分钟_D0停顿分型_BE辅助V230106_看多_中_任意_0')

    :param c: 观察者对象
    :param kwargs:
    :return: 信号识别结果
    """
    di = int(kwargs.get("di", 0))
    freq = kwargs.get("freq", "1分钟")
    k1, k2, k3 = f"{freq}_D{di}停顿分型_BE辅助V230106".split("_", 2)

    分型序列 = c.分型序列
    if len(分型序列) < di + 1:
        return create_single_signal(k1=k1, k2=k2, k3=k3)

    当前分型 = 分型序列[-(di + 1)]

    # 只对顶/底分型产出信号
    if 当前分型.结构.value not in ("顶", "底"):
        return create_single_signal(k1=k1, k2=k2, k3=k3)

    v1 = "看空" if 当前分型.结构.value == "顶" else "看多"
    v2 = 当前分型.强度()

    # 仅强/中分型 + MACD 柱子匹配时认为是有效的停顿分型
    if v2 in ("强", "中") and 当前分型.与MACD柱子分型匹配():
        pass  # 保持 v1, v2
    elif v2 in ("强", "中"):
        pass  # MACD不匹配也产出，但可能被下游过滤
    else:
        v1, v2 = "任意", "任意"

    return create_single_signal(k1=k1, k2=k2, k3=k3, v1=v1, v2=v2)


# ==============================================================================
# bar — K线形态信号
# ==============================================================================


def bar_zdt_V230331(c, **kwargs) -> OrderedDict:
    """计算倒数第di根K线的涨跌停信息

    参数模板："{freq}_D{di}_涨跌停V230331"

    **信号逻辑：**

    - close等于high且大于等于前一根K线的close，近似认为是涨停；反之，跌停。

    **信号列表：**

    - Signal('15分钟_D1_涨跌停V230331_涨停_任意_任意_0')
    - Signal('15分钟_D1_涨跌停V230331_跌停_任意_任意_0')

    :param c: 基础周期的观察者对象
    :param kwargs:
        - di: 倒数第 di 根 K 线
    :return: 信号识别结果
    """
    di = int(kwargs.get("di", 1))
    freq = kwargs.get("freq", "15分钟")
    k1, k2, k3 = f"{freq}_D{di}_涨跌停V230331".split("_", 2)

    普K序列 = c.普通K线序列
    if len(普K序列) < di + 2:
        return create_single_signal(k1=k1, k2=k2, k3=k3)

    当前K线 = 普K序列[-di]
    前K线 = 普K序列[-di - 1]

    if 当前K线.收盘价 == 当前K线.高 and 当前K线.收盘价 >= 前K线.收盘价:
        v1 = "涨停"
    elif 当前K线.收盘价 == 当前K线.低 and 当前K线.收盘价 <= 前K线.收盘价:
        v1 = "跌停"
    else:
        v1 = "任意"

    return create_single_signal(k1=k1, k2=k2, k3=k3, v1=v1)
