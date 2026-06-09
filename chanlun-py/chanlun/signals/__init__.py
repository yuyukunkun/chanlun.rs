"""缠论技术分析库 — 信号函数模块

每个信号函数接收 观察者 对象 + 关键字参数，返回 OrderedDict。
信号 key 格式：k1_k2_k3，value 格式：v1_v2_v3_score。

数据访问路径：
- K线指标：k线.指标.macd.DIF / k线.指标.rsi.RSI / k线.指标.kdj.K / k线.指标.均线["SMA_5"]
- 笔序列：观察员.笔序列（List[虚线]）
- 分型序列：观察员.分型序列（List[分型]）
"""

from collections import OrderedDict

from chanlun.chan import 观察者, 分型结构, 虚线, 相对方向
from chanlun.chan_external import create_single_signal


# 信号函数模板
def 模板_V日期(观察员: 观察者, **kwargs) -> OrderedDict:
    """##信号名称介绍##

    触发条件：## 触发条件 ##

    参数模板：## 具体模板 如: "{freq}_D{di}#{ma_type}#{timeperiod}MO{max_overlap}_BS辅助V230313" ##

    **信号逻辑：**

    ## 详细信号逻辑 ##

    **信号列表：**

    ## 具体信号 如下:
    - Signal('15分钟_D1#SMA#5MO5_BS辅助V230313_看空_向下_任意_0')
    - Signal('15分钟_D1#SMA#5MO5_BS辅助V230313_看多_向下_任意_0')
    - Signal('15分钟_D1#SMA#5MO5_BS辅助V230313_看多_向上_任意_0')
    - Signal('15分钟_D1#SMA#5MO5_BS辅助V230313_看空_向上_任意_0')
    ##

    :param 观察员: 观察者对象
    :param kwargs: 其他参数
        - ## 具体参数介绍 ##
    :return: 信号识别结果
    """
    ## 具体代码过程 ##

    return  ## create_single_signal(k1=k1, k2=k2, k3=k3, v1=v1, v2=v2) ##


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
    if 当前K线.指标 is None:
        return create_single_signal(k1=k1, k2=k2, k3=k3)

    ma_key = f"{ma_type}_{timeperiod}"
    当前均线 = 当前K线.指标.均线.get(ma_key)
    if 当前均线 is None:
        return create_single_signal(k1=k1, k2=k2, k3=k3)

    当前价 = 当前K线.收盘价
    v1 = "看多" if 当前价 > 当前均线 else "看空"

    # 均线方向：需要前一根K线的均线值
    if len(普K序列) >= di + 2:
        前K线 = 普K序列[-di - 1]
        if 前K线.指标 is not None:
            前均线 = 前K线.指标.均线.get(ma_key)
            if 前均线 is not None:
                v2 = "向上" if 当前均线 > 前均线 else "向下"
            else:
                v2 = "任意"
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


def cxt_中枢第三买卖点_V230602(c, **kwargs) -> OrderedDict:
    """中枢第三买卖点信号——线段中枢的第三类买卖点识别

    触发条件：新中枢

    参数模板："{freq}_D1MO{max_overlap}_中枢第三买卖点V230602"

    **信号逻辑：**

    1. 取最后一个中枢，仅处理线段中枢（标识="中枢<线段>"）
    2. 判断中枢状态（中枢之上→三买，中枢之下→三卖）
    3. 首次穿越0轴：中枢本级第三买卖点后，DIF首次反向穿越0轴并出现对应底/顶分型
    4. 中枢段DEA穿越2：第三买卖线段内部DEA双向穿越0轴（上穿+下穿均发生）

    **信号列表：**

    - Signal('日线_D1MO3_中枢第三买卖点V230602_首次穿越0轴_三买_任意_0')
    - Signal('日线_D1MO3_中枢第三买卖点V230602_首次穿越0轴_三卖_任意_0')
    - Signal('日线_D1MO3_中枢第三买卖点V230602_中枢段DEA穿越2_三买_任意_0')
    - Signal('日线_D1MO3_中枢第三买卖点V230602_中枢段DEA穿越2_三卖_任意_0')

    :param c: 观察者对象
    :param kwargs:
        - max_overlap: 相同信号最大重叠次数
    :return: 信号识别结果
    """
    max_overlap = int(kwargs.get("max_overlap", 3))
    freq = kwargs.get("freq", "日线")
    k1, k2, k3 = f"{freq}_D1MO{max_overlap}_中枢第三买卖点V230602".split("_", 2)

    中枢序列 = c.中枢序列
    if not 中枢序列:
        return create_single_signal(k1=k1, k2=k2, k3=k3)

    当前中枢 = 中枢序列[-1]
    if 当前中枢.标识 != "中枢<线段>":
        return create_single_signal(k1=k1, k2=k2, k3=k3)

    状态 = 当前中枢.当前状态()
    if 状态 == "中枢之中":
        return create_single_signal(k1=k1, k2=k2, k3=k3)

    if 状态 == "中枢之上":
        v2 = "三买"
    elif 状态 == "中枢之下":
        v2 = "三卖"
    else:
        return create_single_signal(k1=k1, k2=k2, k3=k3)

    v1 = None

    # 1. 首次穿越0轴：本级第三买卖点后，DIF反向穿越0轴并出现对应分型
    if 当前中枢.本级_第三买卖线 is not None and 当前中枢.完整性("合"):
        第三买卖虚线 = 当前中枢.本级_第三买卖线
        中K线 = 第三买卖虚线.武.中
        缠K序列 = c.缠论K线序列
        try:
            起点索引 = 缠K序列.index(中K线)
        except ValueError:
            起点索引 = 0
        之后缠K序列 = 缠K序列[起点索引:]

        之后缠K = None
        if 状态 == "中枢之上" and 中K线.标的K线.macd.DIF > 0:
            for k in 之后缠K序列:
                if k.标的K线.macd.DIF < 0 and 之后缠K is None:
                    之后缠K = k
                if 之后缠K is not None:
                    if k.分型 is 分型结构.底 and k.标的K线.macd.DIF < 0:
                        v1 = "首次穿越0轴"
                        break

        elif 状态 == "中枢之下" and 中K线.标的K线.macd.DIF < 0:
            for k in 之后缠K序列:
                if k.标的K线.macd.DIF > 0 and 之后缠K is None:
                    之后缠K = k
                if 之后缠K is not None:
                    if k.分型 is 分型结构.顶 and k.标的K线.macd.DIF > 0:
                        v1 = "首次穿越0轴"
                        break

    # 2. 中枢段DEA穿越2：第三买卖线段内部DEA双向穿越0轴
    if v1 is None and 当前中枢.第三买卖线 is not None and 当前中枢.完整性("合"):
        第三线 = 当前中枢.第三买卖线
        if 相对方向.分析(当前中枢.高, 当前中枢.低, 第三线.高, 第三线.低).是否缺口():
            普K序列 = 第三线.获取普K序列(c.观察员)
            MACD特性 = 虚线.统计MACD行为(普K序列, 8, 3)
            if MACD特性["DEA上穿0"] > 0 and MACD特性["DEA下穿0"] > 0:
                v1 = "中枢段DEA穿越2"

    if v1 is None:
        return create_single_signal(k1=k1, k2=k2, k3=k3)

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
