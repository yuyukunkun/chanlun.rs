# Generated from chan.py — type stubs for chanlun Rust bindings
# Auto-generated, do not edit manually

from datetime import datetime
from functools import lru_cache
from typing import Any, Callable, ClassVar, Dict, Iterator, List, Optional, Self, Sequence, Tuple, Union

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
    "测试_读取数据",
    "特征分型",
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
]

class 买卖点类型(str, Enum):
    """
    买卖点类型 — 缠论的三类买卖点及扩展类型。

        :ivar 是买点: 是否为买入类型
        :ivar 是卖点: 是否为卖出类型
    """
    def __str__(self) -> str:
        """返回买卖点类型名称"""
        ...
    def __repr__(self) -> str:
        """返回买卖点类型名称"""
        ...
    def 是买点(self) -> bool:
        """
        判断是否为买入类型（名称中含"买"字）

                :return: 是否为买入类型
        """
        ...
    def 是卖点(self) -> bool:
        """
        判断是否为卖出类型（名称中含"卖"字）

                :return: 是否为卖出类型
        """
        ...

class 基础买卖点:
    """
    基础买卖点 — 描述偏离买入/卖出位置的程度。

        :ivar 备注: 描述文本
        :ivar 偏移: 当前K线相对于买卖点K线的偏移量
        :ivar 失效偏移: 失效K线相对于买卖点K线的偏移量（-1表示未失效）
        :ivar 有效性: 是否已失效（存在失效K线）
        :ivar 破位值: 中枢破位价格
        :ivar 与MACD柱子匹配: MACD柱子是否匹配买卖点方向
        :ivar 与MACD柱子分型匹配: MACD柱子分型是否匹配
    """
    @property
    def 类型(self) -> 买卖点类型: ...
    @类型.setter
    def 类型(self, value: 买卖点类型) -> None: ...
    @property
    def 买卖点分型(self) -> "分型": ...
    @买卖点分型.setter
    def 买卖点分型(self, value: "分型") -> None: ...
    @property
    def 备注(self) -> str: ...
    @备注.setter
    def 备注(self, value: str) -> None: ...
    @property
    def 中枢破位值(self) -> float: ...
    @中枢破位值.setter
    def 中枢破位值(self, value: float) -> None: ...
    def __init__(self, 类型: 买卖点类型, 当前K线: "K线", 买卖点分型: "分型", 备注: str, 中枢破位值: float) -> Any:
        """
        :param 类型: 买卖点类型
                :param 当前K线: 触发买卖点的K线
                :param 买卖点分型: 买卖点对应的分型
                :param 备注: 描述文本（如"笔_1买"）
                :param 中枢破位值: 中枢破位价格
        """
        ...
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def 当前K线(self) -> Any:
        """当前K线"""
        ...
    def 破位值(self) -> float:
        """
        破位值

                :return: float
        """
        ...
    def 偏移(self) -> int:
        """
        偏移

                :return: int
        """
        ...
    def 失效偏移(self) -> int:
        """
        失效偏移

                :return: int
        """
        ...
    def 有效性(self) -> bool:
        """
        有效性

                :return: bool
        """
        ...
    def 与MACD柱子匹配(self) -> bool:
        """
        与MACD柱子匹配

                :return: bool
        """
        ...
    def 与MACD柱子分型匹配(self) -> bool:
        """
        与MACD柱子分型匹配

                :return: bool
        """
        ...

class 买卖点(基础买卖点):
    """
    买卖点 — 一二三类买卖点构造器。

        类方法:
           一卖点 / 一买点 / 二卖点 / 二买点 / 三卖点 / 三买点 — 创建对应买卖点实例
           生成买卖点(特征, 序号, 级别, 分型, 当前缠K) — 根据特征路由到对应构造函数
    """
    @classmethod
    def 一卖点(cls, 买卖点分型: "分型", 当前K线: "K线", 标识: str, 备注: str, 中枢破位值: float) -> "买卖点":
        """
        :param 买卖点分型: 买卖点对应的分型
                :param 当前K线: 当前K线
                :param 标识: 标识（未使用，仅保持接口一致）
                :param 备注: 描述文本
                :param 中枢破位值: 中枢破位价格
                :return: '买卖点'
        """
        ...
    @classmethod
    def 一买点(cls, 买卖点分型: "分型", 当前K线: "K线", 标识: str, 备注: str, 中枢破位值: float) -> "买卖点":
        """
        :param 买卖点分型: 买卖点对应的分型
                :param 当前K线: 当前K线
                :param 标识: 标识（未使用，仅保持接口一致）
                :param 备注: 描述文本
                :param 中枢破位值: 中枢破位价格
                :return: '买卖点'
        """
        ...
    @classmethod
    def 二卖点(cls, 买卖点分型: "分型", 当前K线: "K线", 标识: str, 备注: str, 中枢破位值: float) -> "买卖点":
        """
        :param 买卖点分型: 买卖点对应的分型
                :param 当前K线: 当前K线
                :param 标识: 标识（未使用，仅保持接口一致）
                :param 备注: 描述文本
                :param 中枢破位值: 中枢破位价格
                :return: '买卖点'
        """
        ...
    @classmethod
    def 二买点(cls, 买卖点分型: "分型", 当前K线: "K线", 标识: str, 备注: str, 中枢破位值: float) -> "买卖点":
        """
        :param 买卖点分型: 买卖点对应的分型
                :param 当前K线: 当前K线
                :param 标识: 标识（未使用，仅保持接口一致）
                :param 备注: 描述文本
                :param 中枢破位值: 中枢破位价格
                :return: '买卖点'
        """
        ...
    @classmethod
    def 三卖点(cls, 买卖点分型: "分型", 当前K线: "K线", 标识: str, 备注: str, 中枢破位值: float) -> "买卖点":
        """
        :param 买卖点分型: 买卖点对应的分型
                :param 当前K线: 当前K线
                :param 标识: 标识（未使用，仅保持接口一致）
                :param 备注: 描述文本
                :param 中枢破位值: 中枢破位价格
                :return: '买卖点'
        """
        ...
    @classmethod
    def 三买点(cls, 买卖点分型: "分型", 当前K线: "K线", 标识: str, 备注: str, 中枢破位值: float) -> "买卖点":
        """
        :param 买卖点分型: 买卖点对应的分型
                :param 当前K线: 当前K线
                :param 标识: 标识（未使用，仅保持接口一致）
                :param 备注: 描述文本
                :param 中枢破位值: 中枢破位价格
                :return: '买卖点'
        """
        ...
    @classmethod
    def 生成买卖点(cls, 特征: str, 序号: str, 级别: str, 买卖点分型: "分型", 当前缠K: "缠论K线") -> Any:
        """
        :param 特征: 特征字符串
                :param 序号: 序号（如"一"、"二"、"三"）
                :param 级别: 级别字符串
                :param 买卖点分型: 买卖点对应的分型
                :param 当前缠K: 当前缠论K线
        """
        ...

class 缠论配置:
    """
    缠论配置 — 控制所有分析阶段行为的参数集（共 60+ 字段，均有默认值）。

        字段分组:
          [基础] 标识
          [缠K] 缠K合并替换
          [笔] 笔内元素数量, 笔弱化, 笔次成笔, 笔内相同终点取舍 等
          [线段] 线段_特征序列忽视老阴老阳, 线段_缺口后紧急修正, 线段内部中枢图显 等
          [分析开关] 分析笔, 分析线段, 分析扩展线段, 分析笔中枢, 分析线段中枢
          [指标] 计算指标, 指标计算方式, MACD/RSI/KDJ 参数
          [推送/显示] 图表展示, 推送K线/笔/线段/中枢, 图表展示_笔 等细分开关
          [买卖点] 买卖点偏移, 买卖点激进识别, 买卖点_背离率, 买卖点_计算方式 等
          [背驰] 线段内部背驰_MACD, 线段内部背驰_斜率 等
          [其他] 手动终止, 加载文件路径

        方法:
           to_dict() / to_json() / 保存配置(path?) / 加载配置(path?) (classmethod)
           from_dict(data) (classmethod) / from_json(json_str) (classmethod)
           不推送() (classmethod) / 对比(other) -> dict
    """
    @model_validator(mode="before")
    def 兼容旧版本配置(self, values: Dict[str, Any]) -> Dict[str, Any]:
        """
        自动兼容：
                1. 旧版本少字段 → 使用默认值
                2. 新版本多字段 → 自动忽略多余字段
                3. 字段改名/删除 → 不报错

                :param values: 原始配置字典
                :return: 兼容后的字典
        """
        ...
    @field_validator("*", mode="wrap")
    def bool_parse_fallback_default(self, value: Any, handler: Any, info: Any) -> Any:
        """
        :param value: 待验证的值
                :param handler: 默认验证器
                :param info: 字段信息
                :return: 验证后的值
        """
        ...
    def to_dict(self) -> dict:
        """对象 → 字典"""
        ...
    def to_json(self) -> str:
        """对象 → JSON字符串"""
        ...
    def 保存配置(self, path: Any = "缠论配置.json") -> Any:
        """
        将配置保存为JSON文件
                :param path: 保存路径，默认"缠论配置.json"
        """
        ...
    @staticmethod
    def 加载配置(path: Any = "缠论配置.json") -> "缠论配置":
        """
        从JSON文件加载配置
                :param path: 配置文件路径
                :return: 缠论配置实例
        """
        ...
    @classmethod
    def from_dict(cls, data: dict) -> "缠论配置":
        """
        :param data: 字典数据
                :return: 缠论配置实例
        """
        ...
    @classmethod
    def from_json(cls, json_str: str) -> "缠论配置":
        """
        :param json_str: JSON字符串
                :return: 缠论配置实例
        """
        ...
    @classmethod
    def 不推送(cls) -> Any:
        """
        创建不推送任何图表的静默配置（用于纯计算场景）

                :return: 新缠论配置实例
        """
        ...
    @classmethod
    def 按序号重组字典(cls, 默认配置: Any, 原始字典: dict) -> dict:
        """
        将形如 "1_open", "1_close", "2_open", "name" 的字典重组为嵌套结构
                {
                    "1_open": 10,
                    "1_close": 11,
                    "2_open": 20,
                    "name": "BTC",    # 无法拆分
                    "time": 123456    # 无法拆分
                }
                转化为
                {
                    1: {"open": 10, "close": 11},
                    2: {"open": 20},
                    "无法拆分": {
                        "name": "BTC",
                        "time": 123456
                    }
                }

                :param 默认配置: 默认配置实例
                :param 原始字典: 待重组的原始字典
                :return: 重组后的字典
        """
        ...
    def 对比(self, other: "缠论配置") -> dict:
        """
        比较当前配置与另一个配置的差异

                :param other: 另一个配置实例
                :return: {
                    "字段名": {
                        "旧值": 当前配置的值,
                        "新值": 另一个配置的值
                    },
                    ...
                }
                仅当值不同时才包含该字段
        """
        ...

class 相对方向(Enum):
    """
    相对方向 — 描述两个K线/分型之间相对位置关系的枚举。

        类属性: 向上, 向下, 向上缺口, 向下缺口, 衔接向上, 衔接向下, 顺, 逆, 同

        :ivar 是否向上: 判断是否为向上方向
        :ivar 是否向下: 判断是否为向下方向
        :ivar 是否包含: 判断是否为包含关系
        :ivar 是否缺口: 判断是否有缺口
        :ivar 是否衔接: 判断是否为首尾衔接
    """
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def 翻转(self) -> "相对方向":
        """返回方向的对立面：向上↔向下, 向上缺口↔向下缺口, 顺↔逆, 衔接向上↔衔接向下, 同不变"""
        ...
    def 是否向上(self) -> bool:
        """判断是否为向上方向（向上/向上缺口/衔接向上）"""
        ...
    def 是否向下(self) -> bool:
        """判断是否为向下方向（向下/向下缺口/衔接向下）"""
        ...
    def 是否包含(self) -> bool:
        """判断是否为包含关系（顺/逆/同）"""
        ...
    def 是否缺口(self) -> bool:
        """判断是否有缺口（向下缺口/向上缺口）"""
        ...
    def 是否衔接(self) -> bool:
        """判断是否为首尾衔接（衔接向下/衔接向上）"""
        ...
    @classmethod
    def 分析(cls, 前高: float, 前低: float, 后高: float, 后低: float) -> "相对方向":
        """
        分析两个价格区间（前、后）的相对位置关系。

                :param 前高: 前方价格区间的最高价
                :param 前低: 前方价格区间的最低价
                :param 后高: 后方价格区间的最高价
                :param 后低: 后方价格区间的最低价
                :return: 相对方向枚举值（向上/向下/向上缺口/向下缺口/衔接向上/衔接向下/顺/逆/同）
                :raises RuntimeError: 无法识别的方向
        """
        ...

class 分型结构(Enum):
    """
    分型结构 — 描述三根K线构成的顶底分型形态。

        类属性: 上, 下, 顶, 底, 散
    """
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    @classmethod
    def 分析(cls, 左: Any, 中: Any, 右: Any, 可以逆序包含: bool = False, 忽视顺序包含: bool = False) -> Optional["分型结构"]:
        """
        分析左中右三个元素构成的分型形态。

                :param 左: 左侧元素（必须有 高/低 属性）
                :param 中: 中间元素
                :param 右: 右侧元素
                :param 可以逆序包含: True 时允许逆序包含关系（如 右>左>中）
                :param 忽视顺序包含: True 时跳过顺序包含检查（如 左>中>右）
                :return: 分型结构（顶/底/上/下/散），无法判定返回 None
        """
        ...

class 缺口:
    """
    缺口 — 描述价格区间之间的缺口（未重叠部分）。

        :ivar 高: 缺口上沿
        :ivar 低: 缺口下沿
    """
    @property
    def 高(self) -> float: ...
    @高.setter
    def 高(self, value: float) -> None: ...
    @property
    def 低(self) -> float: ...
    @低.setter
    def 低(self, value: float) -> None: ...
    def __init__(self, 高: float, 低: float) -> None:
        """
        :param 高: 缺口上沿
                :param 低: 缺口下沿
        """
        ...
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    @classmethod
    def 居中截取区间(cls, 起点: float, 终点: float, 比例: float = 0.15) -> Optional["缺口"]:
        """
        以原区间中心为基准，向两侧各取总长度的 `比例` 作为新区间。
                如果新区间超出原边界则裁剪到边界；若完全越界则返回 None。

                :param 起点: 区间起始值
                :param 终点: 区间结束值
                :param 比例: 向两侧扩展的长度占总长度的比例（0~1 之间）
                :return: Optional[缺口]
        """
        ...

class 指标:
    """指标 — 静态工具类，提供K线取值的辅助方法。"""
    @classmethod
    def K线取值(cls, k线: "K线", 指标计算方式: Any) -> Any:
        """
        根据计算方式从K线中取值

                :param k线: K线对象
                :param 指标计算方式: "开"/"高"/"低"/"收"/"高低均值"/"高低收均值"/"开高低收均值"
                :return: 对应价格
        """
        ...

class 平滑异同移动平均线:
    """
    平滑异同移动平均线（MACD）— 基于EMA快慢线差值的趋势指标。

        计算字段（输入）:
            时间戳: datetime / 收盘价: float
            快线周期: int (默认12) / 慢线周期: int (默认26) / 信号周期: int (默认9)

        输出字段:
            EMA快线: float — 短期EMA值
            EMA慢线: float — 长期EMA值
            DIF: float — 快线减慢线
            DEA: float — DIF的信号线EMA
            MACD柱子: float — DIF减DEA
    """
    @classmethod
    def 首次计算(cls, 初始收盘价: float, 初始时间: datetime, 快线周期: int = 12, 慢线周期: int = 26, 信号周期: int = 9) -> "平滑异同移动平均线":
        """
        首次计算MACD指标（没有历史数据时使用）

                :param 初始收盘价: 第一个数据点的收盘价
                :param 初始时间: 第一个数据点的时间戳
                :param 快线周期: 快线EMA周期
                :param 慢线周期: 慢线EMA周期
                :param 信号周期: 信号线EMA周期
                :return: MACD指标对象
        """
        ...
    @classmethod
    def 首次计算_K线(cls, k线: "K线", 计算方式: str, 快线周期: int = 12, 慢线周期: int = 26, 信号周期: int = 9) -> "平滑异同移动平均线":
        """
        :param k线: 原始K线
                :param 计算方式: 指标计算方式（开/高/低/收/均值等）
                :param 快线周期: 快线EMA周期
                :param 慢线周期: 慢线EMA周期
                :param 信号周期: 信号线EMA周期
                :return: MACD指标对象
        """
        ...
    @classmethod
    def 增量计算(cls, 前一个MACD: "平滑异同移动平均线", 当前收盘价: float, 当前时间: datetime) -> "平滑异同移动平均线":
        """
        基于前一个MACD指标增量计算当前MACD指标
                适用于实时交易系统或流式数据处理

                :param 前一个MACD: 前一个周期的MACD指标对象
                :param 当前收盘价: 当前K线的收盘价
                :param 当前时间: 当前K线的时间戳
                :return: 当前MACD指标对象
                :raises RuntimeError: 前一个MACD中快线EMA或慢线EMA为None时抛出
        """
        ...
    @classmethod
    def 增量计算_K线(cls, 前一个MACD: "平滑异同移动平均线", 当前K线: "K线", 计算方式: "str") -> "平滑异同移动平均线":
        """
        :param 前一个MACD: 前一个MACD指标对象
                :param 当前K线: 当前K线
                :param 计算方式: 指标计算方式
                :return: 当前MACD指标对象
        """
        ...

class 相对强弱指数:
    """
    相对强弱指数 (RSI) 指标
        使用 Wilder 平滑（RMA）进行增量计算，提供完整的中间平滑值，
        并支持对RSI值计算SMA（简单移动平均）
    """
    @classmethod
    def 首次计算(cls, 初始收盘价: float, 初始时间: datetime, 周期: int = 14, 超买阈值: float = 70.0, 超卖阈值: float = 30.0, RSI_SMA周期: Optional[int] = None) -> "相对强弱指数":
        """
        首次计算RSI（没有足够历史数据时使用）
                此时无法计算真实RSI，设为 None，但记录初始收盘价作为起点

                :param 初始收盘价: 第一个数据点的收盘价
                :param 初始时间: 第一个数据点的时间戳
                :param 周期: RSI周期
                :param 超买阈值: 超买阈值
                :param 超卖阈值: 超卖阈值
                :param RSI_SMA周期: RSI的SMA周期（可选）
                :return: RSI指标对象
        """
        ...
    @classmethod
    def 首次计算_K线(cls, k线: "K线", 计算方式: str, 周期: int = 14, 超买阈值: float = 70.0, 超卖阈值: float = 30.0, RSI_SMA周期: Optional[int] = None) -> "相对强弱指数":
        """
        :param k线: 原始K线
                :param 计算方式: 指标计算方式
                :param 周期: RSI周期
                :param 超买阈值: 超买阈值
                :param 超卖阈值: 超卖阈值
                :param RSI_SMA周期: RSI的SMA周期
                :return: RSI指标对象
        """
        ...
    @classmethod
    def 增量计算(cls, 前一个RSI: "相对强弱指数", 当前收盘价: float, 当前时间: datetime) -> "相对强弱指数":
        """
        基于前一个RSI指标增量计算当前RSI
                支持可选的RSI_SMA（简单移动平均）

                :param 前一个RSI: 前一个RSI指标对象
                :param 当前收盘价: 当前收盘价
                :param 当前时间: 当前时间戳
                :return: 当前RSI指标对象
        """
        ...
    @classmethod
    def 增量计算_K线(cls, 前一个RSI: "相对强弱指数", 当前K线: "K线", 计算方式: "str") -> "相对强弱指数":
        """
        :param 前一个RSI: 前一个RSI指标对象
                :param 当前K线: 当前K线
                :param 计算方式: 指标计算方式
                :return: 当前RSI指标对象
        """
        ...

class 随机指标:
    """
    KDJ 随机指标 (Stochastic Oscillator)
        使用标准参数：N=9, M1=3, M2=3
        支持增量计算，需提供当前K线的最高价、最低价、收盘价
    """
    @classmethod
    def 首次计算(cls, 初始最高价: float, 初始最低价: float, 初始收盘价: float, 初始时间: datetime, N: int = 9, M1: int = 3, M2: int = 3, 超买阈值: float = 80.0, 超卖阈值: float = 20.0) -> "随机指标":
        """
        首次计算KDJ（无历史数据时）
                此时无法计算RSV和K/D/J，仅记录初始三价，初始化队列

                :param 初始最高价: 第一个数据点的最高价
                :param 初始最低价: 第一个数据点的最低价
                :param 初始收盘价: 第一个数据点的收盘价
                :param 初始时间: 第一个数据点的时间戳
                :param N: RSV周期
                :param M1: K值平滑周期
                :param M2: D值平滑周期
                :param 超买阈值: 超买阈值
                :param 超卖阈值: 超卖阈值
                :return: KDJ指标对象
        """
        ...
    @classmethod
    def 首次计算_K线(cls, k线: "K线", 计算方式: str, RSV周期: int = 9, K值平滑周期: int = 3, D值平滑周期: int = 3, 超买阈值: float = 80.0, 超卖阈值: float = 20.0) -> "随机指标":
        """
        :param k线: 原始K线
                :param 计算方式: 指标计算方式（未使用，仅为接口一致）
                :param RSV周期: RSV周期
                :param K值平滑周期: K值平滑周期
                :param D值平滑周期: D值平滑周期
                :param 超买阈值: 超买阈值
                :param 超卖阈值: 超卖阈值
                :return: KDJ指标对象
        """
        ...
    @classmethod
    def 增量计算(cls, 前一个KDJ: "随机指标", 当前最高价: float, 当前最低价: float, 当前收盘价: float, 当前时间: datetime) -> "随机指标":
        """
        基于前一个KDJ对象和当前三价，增量计算当前KDJ值

                :param 前一个KDJ: 前一个KDJ指标对象
                :param 当前最高价: 当前最高价
                :param 当前最低价: 当前最低价
                :param 当前收盘价: 当前收盘价
                :param 当前时间: 当前时间戳
                :return: 当前KDJ指标对象
        """
        ...
    @classmethod
    def 增量计算_K线(cls, 前一个KDJ: "随机指标", 当前K线: "K线", 计算方式: "str") -> "随机指标":
        """
        :param 前一个KDJ: 前一个KDJ指标对象
                :param 当前K线: 当前K线
                :param 计算方式: 指标计算方式（未使用，仅为接口一致）
                :return: 当前KDJ指标对象
        """
        ...

class 背驰分析:
    """
    背驰分析 — 静态方法容器，提供背驰/背离检测算法。

        方法:
          MACD背驰(进入段, 离开段, K线序列, 方式?) — MACD柱状线面积背驰
          斜率背驰(进入段, 离开段, 序列, 配置?) — 线段斜率背驰
          测度背驰(进入段, 离开段, 序列, 配置?) — 线段测度背驰
          全量背驰(进入段, 离开段, ...) — 综合所有背驰检测方式
          配置背驰(进入段, 离开段, ...) — 根据配置选择检测方式
    """
    @staticmethod
    def MACD背驰(进入段: "虚线", 离开段: "虚线", K线序列: List["K线"], 方式: str = "总") -> bool:
        """
        MACD柱状线面积背驰

                :param 进入段: 进入中枢的虚线
                :param 离开段: 离开中枢的虚线
                :param K线序列: 完整K线序列
                :param 方式: "总" 或 "阳"/"阴"
                :return: 背驰为True
        """
        ...
    @staticmethod
    def 斜率背驰(进入段: "虚线", 离开段: "虚线") -> bool:
        """
        价格斜率背驰

                :param 进入段: 进入中枢的虚线
                :param 离开段: 离开中枢的虚线
                :return: 背驰为True
        """
        ...
    @staticmethod
    def 测度背驰(进入段: "虚线", 离开段: "虚线") -> bool:
        """
        价格测度背驰（欧氏距离）

                :param 进入段: 进入中枢的虚线
                :param 离开段: 离开中枢的虚线
                :return: 背驰为True
        """
        ...
    @staticmethod
    def 全量背驰(进入段: "虚线", 离开段: "虚线", 普K序列: List["K线"]) -> bool:
        """
        判断是否满足全部三种背驰条件（MACD + 测度 + 斜率）

                :param 进入段: 进入中枢的线段
                :param 离开段: 离开中枢的线段
                :param 普K序列: 完整K线序列
                :return: 三者全满足返回True
        """
        ...
    @staticmethod
    def 任意背驰(进入段: "虚线", 离开段: "虚线", 普K序列: List["K线"]) -> bool:
        """
        判断是否满足任一背驰条件

                :param 进入段: 进入中枢的线段
                :param 离开段: 离开中枢的线段
                :param 普K序列: 完整K线序列
                :return: 任一满足返回True
        """
        ...
    @staticmethod
    def 配置背驰(进入段: "虚线", 离开段: "虚线", 普K序列: List["K线"], 配置: 缠论配置) -> bool:
        """
        根据配置选择对应的背驰检测组合

                :param 进入段: 进入中枢的线段
                :param 离开段: 离开中枢的线段
                :param 普K序列: 完整K线序列
                :param 配置: 缠论配置（控制MACD/测度/斜率开关）
                :return: 背驰结果
        """
        ...
    @staticmethod
    def 任选背驰(进入段: "虚线", 离开段: "虚线", 普K序列: List["K线"]) -> bool:
        """
        三个背驰条件中至少两个满足即视为背驰

                :param 进入段: 进入中枢的线段
                :param 离开段: 离开中枢的线段
                :param 普K序列: 完整K线序列
                :return: 至少两个满足返回True
        """
        ...
    @staticmethod
    def 背驰模式(进入段: "虚线", 离开段: "虚线", 普K序列: List["K线"], 配置: 缠论配置, 模式: str) -> bool:
        """
        根据模式字符串选择背驰检测策略

                :param 进入段: 进入中枢的线段
                :param 离开段: 离开中枢的线段
                :param 普K序列: 完整K线序列
                :param 配置: 缠论配置
                :param 模式: "全量"/"任意"/"配置"/"相对"
                :return: 背驰结果
        """
        ...

class K线(object):
    """
    原始 K 线 — OHLCV 数据，可内嵌 MACD/RSI/KDJ 指标。

        :ivar 标识: 标识符
        :ivar 序号: 序号
        :ivar 周期: K线周期（秒）
        :ivar 时间戳: 时间戳
        :ivar 高: 最高价
        :ivar 低: 最低价
        :ivar 开盘价: 开盘价
        :ivar 收盘价: 收盘价
        :ivar 成交量: 成交量
        :ivar macd: MACD指标对象
        :ivar rsi: RSI指标对象
        :ivar kdj: KDJ指标对象
        :ivar 方向: 涨跌方向（只读）
    """
    @property
    def 标识(self) -> str: ...
    @标识.setter
    def 标识(self, value: str) -> None: ...
    @property
    def 序号(self) -> int: ...
    @序号.setter
    def 序号(self, value: int) -> None: ...
    @property
    def 周期(self) -> int: ...
    @周期.setter
    def 周期(self, value: int) -> None: ...
    @property
    def 时间戳(self) -> datetime: ...
    @时间戳.setter
    def 时间戳(self, value: datetime) -> None: ...
    @property
    def 开盘价(self) -> float: ...
    @开盘价.setter
    def 开盘价(self, value: float) -> None: ...
    @property
    def 最高价(self) -> float: ...
    @最高价.setter
    def 最高价(self, value: float) -> None: ...
    @property
    def 最低价(self) -> float: ...
    @最低价.setter
    def 最低价(self, value: float) -> None: ...
    @property
    def 收盘价(self) -> float: ...
    @收盘价.setter
    def 收盘价(self, value: float) -> None: ...
    @property
    def 成交量(self) -> float: ...
    @成交量.setter
    def 成交量(self, value: float) -> None: ...
    @property
    def macd(self) -> 平滑异同移动平均线: ...
    @macd.setter
    def macd(self, value: 平滑异同移动平均线) -> None: ...
    @property
    def rsi(self) -> 相对强弱指数: ...
    @rsi.setter
    def rsi(self, value: 相对强弱指数) -> None: ...
    @property
    def kdj(self) -> 随机指标: ...
    @kdj.setter
    def kdj(self, value: 随机指标) -> None: ...
    def __init__(self, 标识: str, 序号: int, 周期: int, 时间戳: datetime, 开盘价: float, 最高价: float, 最低价: float, 收盘价: float, 成交量: float, macd: 平滑异同移动平均线 = None, rsi: 相对强弱指数 = None, kdj: 随机指标 = None) -> Any:
        """
        :param 标识: K线标识符
                :param 序号: K线序号
                :param 周期: K线周期（秒）
                :param 时间戳: K线时间
                :param 开盘价: 开盘价
                :param 最高价: 最高价
                :param 最低价: 最低价
                :param 收盘价: 收盘价
                :param 成交量: 成交量
                :param macd: MACD指标对象（可选）
                :param rsi: RSI指标对象（可选）
                :param kdj: KDJ指标对象（可选）
        """
        ...
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def 方向(self) -> 相对方向:
        """:return: 相对方向.向上（开盘<收盘）或 相对方向.向下（开盘>收盘）"""
        ...
    def __bytes__(self) -> bytes: ...
    @classmethod
    def 创建普K(cls, 标识: str, 时间戳: datetime, 开盘价: float, 最高价: float, 最低价: float, 收盘价: float, 成交量: float, 序号: int, 周期: int) -> "K线":
        """
        快捷构造普通K线

                :param 标识: K线标识符
                :param 时间戳: K线时间
                :param 开盘价: 开盘价
                :param 最高价: 最高价
                :param 最低价: 最低价
                :param 收盘价: 收盘价
                :param 成交量: 成交量
                :param 序号: K线序号
                :param 周期: K线周期（秒）
                :return: K线实例
        """
        ...
    @classmethod
    def 保存到DAT文件(cls, 路径: str, K线序列: List["K线"]) -> Any:
        """
        将K线序列保存为二进制DAT文件

                :param 路径: 保存路径
                :param K线序列: K线列表
        """
        ...
    @classmethod
    def 读取大端字节数组(cls, 字节组: bytes, 周期: int, 标识: str) -> "K线":
        """
        从大端字节序二进制数据反序列化K线（兼容.dat/.nb文件格式）

                :param 字节组: 二进制数据（48字节）
                :param 周期: 周期（秒）
                :param 标识: K线标识
                :return: K线实例
        """
        ...
    @classmethod
    def 获取MACD(cls, K线序列: List["K线"], 始: "K线", 终: "K线") -> Dict[str, float]:
        """
        计算指定K线区间的MACD柱面积

                :param K线序列: 完整K线序列
                :param 始: 起始K线
                :param 终: 终点K线
                :return: {"阳": 正值面积和, "阴": 负值面积和, "合": 净面积, "总": 绝对面积和}
        """
        ...
    @staticmethod
    def 截取(序列: List["K线"], 始: "K线", 终: "K线") -> List["K线"]:
        """
        按起止K线截取K线子序列

                :param 序列: 完整K线序列
                :param 始: 起始K线
                :param 终: 终点K线
                :return: K线子列表
        """
        ...

class 缠论K线(object):
    """
    缠论K线 — 经包含处理后的标准化K线，有方向和分型结构标记。

        :ivar 序号: 序号
        :ivar 时间戳: 时间戳
        :ivar 高: 最高价
        :ivar 低: 最低价
        :ivar 方向: 运行方向
        :ivar 分型: 分型结构（顶/底/上/下等）
        :ivar 周期: 周期（秒）
        :ivar 标识: 标识符
        :ivar 分型特征值: 分型特征值（顶取高，底取低）
        :ivar 原始起始序号: 合并前的原始K线起始序号
        :ivar 原始结束序号: 合并前的原始K线结束序号
        :ivar 标的K线: 对应的原始K线
        :ivar 买卖点信息: 买卖点信息（预留）
    """
    @property
    def 序号(self) -> int: ...
    @序号.setter
    def 序号(self, value: int) -> None: ...
    @property
    def 时间戳(self) -> datetime: ...
    @时间戳.setter
    def 时间戳(self, value: datetime) -> None: ...
    @property
    def 最高价(self) -> float: ...
    @最高价.setter
    def 最高价(self, value: float) -> None: ...
    @property
    def 最低价(self) -> float: ...
    @最低价.setter
    def 最低价(self, value: float) -> None: ...
    @property
    def 最终方向(self) -> 相对方向: ...
    @最终方向.setter
    def 最终方向(self, value: 相对方向) -> None: ...
    @property
    def 普K(self) -> "K线": ...
    @普K.setter
    def 普K(self, value: "K线") -> None: ...
    @property
    def 原始起始序号(self) -> int: ...
    @原始起始序号.setter
    def 原始起始序号(self, value: int) -> None: ...
    @property
    def 原始结束序号(self) -> int: ...
    @原始结束序号.setter
    def 原始结束序号(self, value: int) -> None: ...
    @property
    def 分型(self) -> Optional[分型结构]: ...
    @分型.setter
    def 分型(self, value: Optional[分型结构]) -> None: ...
    def __init__(self, 序号: int, 时间戳: datetime, 最高价: float, 最低价: float, 最终方向: 相对方向, 普K: "K线", 原始起始序号: int, 原始结束序号: int, 分型: Optional[分型结构] = None) -> Any:
        """
        :param 序号: 缠论K线序号
                :param 时间戳: 时间戳
                :param 最高价: 最高价
                :param 最低价: 最低价
                :param 最终方向: 方向
                :param 普K: 对应的原始K线
                :param 原始起始序号: 合并起始序号
                :param 原始结束序号: 合并结束序号
                :param 分型: 分型结构（可选）
        """
        ...
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def 镜像(self) -> Any:
        """
        创建当前缠K的浅拷贝副本

                :return: 新的缠论K线实例
        """
        ...
    def 与MACD柱子匹配(self) -> bool:
        """:return: 底分型时MACD柱<0，顶分型时MACD柱>0"""
        ...
    def 与RSI匹配(self) -> bool:
        """:return: 底分型时RSI < RSI_SMA，顶分型时RSI > RSI_SMA"""
        ...
    def 与KDJ匹配(self) -> bool:
        """:return: 底分型时K<D，顶分型时K>D"""
        ...
    @classmethod
    def 时间戳对齐(cls, 基线: List["缠论K线"], k线: "缠论K线") -> Any:
        """
        在基线序列中找到与k线时间戳对齐的时间戳

                :param 基线: 参照缠K序列
                :param k线: 需要对齐的缠K
                :return: 对齐后的时间戳
        """
        ...
    @classmethod
    def 创建缠K(cls, 时间戳: datetime, 高: float, 低: float, 方向: 相对方向, 结构: 分型结构, 原始序号: int, 普k: "K线", 之前: Optional["缠论K线"] = None) -> "缠论K线":
        """
        创建新的缠论K线

                :param 时间戳: K线时间
                :param 高: 最高价
                :param 低: 最低价
                :param 方向: K线运行方向
                :param 结构: 分型结构
                :param 原始序号: 原始K线序号
                :param 普k: 标的原始K线
                :param 之前: 前一缠K（可选，自动校验包含关系）
                :return: 缠论K线实例
                :raises ValueError: 包含关系错误
        """
        ...
    @classmethod
    def 兼并(cls, 之前缠K: Optional["缠论K线"], 当前缠K: "缠论K线", 当前普K: "K线", 配置: 缠论配置) -> Tuple[Optional["缠论K线"], Optional[str]]:
        """
        K线包含处理（合并）

                :param 之前缠K: 前一缠K
                :param 当前缠K: 当前待处理的缠K
                :param 当前普K: 当前原始K线
                :param 配置: 缠论配置
                :return: (新缠K或None, 操作模式:"添加"/"替换"/None)
        """
        ...
    @classmethod
    def 分析(cls, 当前K线: "K线", 缠K序列: List["缠论K线"], 普K序列: List["K线"], 配置: 缠论配置) -> tuple[str, Optional["分型"]]:
        """
        分析K线，执行指标计算+包含处理+分型判定

                :param 当前K线: 新到的原始K线
                :param 缠K序列: 现有缠K序列（会被原地修改）
                :param 普K序列: 现有普K序列（会被原地修改）
                :param 配置: 缠论配置
                :return: (操作状态, 新形成的分型或None)
        """
        ...
    @staticmethod
    def 截取(序列: List["缠论K线"], 始: "缠论K线", 终: "缠论K线") -> List["缠论K线"]:
        """
        :param 序列: 缠K序列
                :param 始: 起始缠K
                :param 终: 终点缠K
                :return: 缠K子列表
        """
        ...

class 分型(object):
    """
    分型 — 由左中右三根缠论K线构成的顶/底分型结构。

        :ivar 左: 左侧缠K（可能为None）
        :ivar 中: 中间缠K（分型顶点/底点）
        :ivar 右: 右侧缠K（可能为None）
        :ivar 结构: 分型结构（顶/底/上/下/包含）
        :ivar 时间戳: 分型时间戳
        :ivar 分型特征值: 顶分型取最高价，底分型取最低价
        :ivar 关系组: 左中、中右、左右三对相对方向
        :ivar 强度: 分型强度（强/中/弱/未知）
        :ivar 与MACD柱子分型匹配: 是否与MACD柱子分型匹配
    """
    @property
    def 左(self) -> Optional[缠论K线]: ...
    @左.setter
    def 左(self, value: Optional[缠论K线]) -> None: ...
    @property
    def 中(self) -> 缠论K线: ...
    @中.setter
    def 中(self, value: 缠论K线) -> None: ...
    @property
    def 右(self) -> Optional[缠论K线]: ...
    @右.setter
    def 右(self, value: Optional[缠论K线]) -> None: ...
    def __init__(self, 左: Optional[缠论K线], 中: 缠论K线, 右: Optional[缠论K线]) -> Any:
        """
        :param 左: 左侧缠K
                :param 中: 中间缠K
                :param 右: 右侧缠K
        """
        ...
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def 关系组(self) -> Optional[Tuple[相对方向, 相对方向, 相对方向]]:
        """
        左、中、右三对相对方向关系

                :return: (左中关系, 中右关系, 左右关系) 或 None
        """
        ...
    def 强度(self) -> Any:
        """
        分型强度（强/中/弱/未知）

                :return: 强度字符串
        """
        ...
    def 与MACD柱子分型匹配(self) -> bool:
        """:return: 底分型时左右MACD柱 > 中MACD柱，顶分型时左右MACD柱 < 中MACD柱"""
        ...
    @classmethod
    def 判断分型(cls, 左: "分型", 右: "分型", 模式: str = "中") -> bool:
        """
        判断两个分型是否相同（identity比较）

                :param 左: 左分型
                :param 右: 右分型
                :param 模式: 比较模式（默认"中"）
                :return: 是否为同一对象
        """
        ...
    @staticmethod
    def 从缠K序列中获取分型(K线序列: List[缠论K线], 中: 缠论K线) -> "分型":
        """
        从缠K序列中提取以指定缠K为中元素的分型

                :param K线序列: 缠K列表
                :param 中: 中间K线
                :return: 分型实例（右可能为None）
        """
        ...
    @staticmethod
    def 向序列中添加(分型序列: List["分型"], 当前分型: "分型") -> Any:
        """
        向分型序列尾部添加，自动校验顶底交替

                :param 分型序列: 现有分型列表
                :param 当前分型: 待添加分型
                :raises ValueError: 首元素非顶底、连续同向分型时抛出
        """
        ...

class 虚线(object):
    """
    虚线 — 笔/线段的通用数据结构，持有一组分型端点（文=起点分型, 武=终点分型）。

        :ivar 标识: 类型标识（"笔"/"线段"/"扩展线段"等）
        :ivar 序号: 序号
        :ivar 级别: 级别
        :ivar 文: 起点分型
        :ivar 武: 终点分型
        :ivar 有效性: 是否有效
        :ivar 基础序列: 内部虚线序列（笔序列/线段序列）
        :ivar 特征序列: 特征序列列表（线段专用）
        :ivar 实_中枢序列: 实中枢列表
        :ivar 虚_中枢序列: 虚中枢列表
        :ivar 合_中枢序列: 合中枢列表
        :ivar 确认K线: 确认K线
        :ivar 模式: 买卖点模式
        :ivar 前一缺口: 前一缺口
        :ivar 前一结束位置: 前一笔/线段的终点
        :ivar 短路修正: 是否短路修正
        :ivar 方向: 运行方向（只读）
        :ivar 高: 最高价（只读）
        :ivar 低: 最低价（只读）
        :ivar 笔序列: 所有笔（只读）
        :ivar 图表标题: 图表显示标题（只读）
    """
    @property
    def 序号(self) -> int: ...
    @序号.setter
    def 序号(self, value: int) -> None: ...
    @property
    def 标识(self) -> str: ...
    @标识.setter
    def 标识(self, value: str) -> None: ...
    @property
    def 文(self) -> 分型: ...
    @文.setter
    def 文(self, value: 分型) -> None: ...
    @property
    def 武(self) -> 分型: ...
    @武.setter
    def 武(self, value: 分型) -> None: ...
    @property
    def 级别(self) -> int: ...
    @级别.setter
    def 级别(self, value: int) -> None: ...
    @property
    def 有效性(self) -> bool: ...
    @有效性.setter
    def 有效性(self, value: bool) -> None: ...
    def __init__(self, 序号: int, 标识: str, 文: 分型, 武: 分型, 级别: int, 有效性: bool = True) -> Any:
        """
        :param 序号: 序号
                :param 标识: 类型标识
                :param 文: 起点分型
                :param 武: 终点分型
                :param 级别: 级别
                :param 有效性: 是否有效
        """
        ...
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def 笔序列(self) -> Any:
        """笔序列"""
        ...
    def 图表标题(self) -> str:
        """:return: 图表显示标题"""
        ...
    def 方向(self) -> "相对方向":
        """
        :return: 运行方向
                :raises RuntimeError: 无法识别的方向
        """
        ...
    def 高(self) -> float:
        """虚线区间的最高价（向上线段为终点分型最高价，向下线段为起点分型最高价）"""
        ...
    def 低(self) -> float:
        """虚线区间的最低价（向下线段为终点分型最低价，向上线段为起点分型最低价）"""
        ...
    def 之前是(self, 之前: "虚线") -> bool:
        """
        :param 之前: 前一条虚线
                :return: 当前虚线的起点是否为前一条虚线的终点
        """
        ...
    def 之后是(self, 之后: "虚线") -> bool:
        """
        :param 之后: 后一条虚线
                :return: 当前虚线的终点是否为后一条虚线的起点
        """
        ...
    def 获取普K序列(self, 观察员: "观察者") -> List[K线]:
        """
        :param 观察员: 观察者实例
                :return: 区间内的原始K线列表
        """
        ...
    def 获取缠K序列(self, 观察员: "观察者") -> List[缠论K线]:
        """
        :param 观察员: 观察者实例
                :return: 区间内的缠K列表
        """
        ...
    def 获取数据文本(self) -> str:
        """获取用于保存的数据文本"""
        ...
    @classmethod
    def 创建笔(cls, 文: 分型, 武: 分型, 有效性: bool = True) -> "虚线":
        """
        :param 文: 起点分型
                :param 武: 终点分型
                :param 有效性: 是否有效
                :return: 虚线实例（标识="笔"）
        """
        ...
    @classmethod
    def 创建线段(cls, 虚线序列: List["虚线"]) -> "虚线":
        """
        :param 虚线序列: 构成线段的虚线列表（笔）
                :return: 虚线实例（标识="线段"或"线段<...>"）
        """
        ...
    @classmethod
    def 缠K买卖点模式(cls, 模式: str, 缠K: "缠论K线", 配置: 缠论配置) -> Any:
        """
        :param 模式: "全量"/"任意"/"配置"/"相对"
                :param 缠K: 待检测的缠论K线
                :param 配置: 缠论配置
                :return: 是否满足买卖点条件
        """
        ...
    @classmethod
    def 买卖点配置匹配(cls, 缠K: "缠论K线", 配置: 缠论配置) -> bool:
        """根据配置中的指标开关检测缠K匹配情况（MACD/KDJ/RSI组合）"""
        ...
    @classmethod
    def 买卖点任意匹配(cls, 缠K: "缠论K线") -> bool:
        """
        :param 缠K: 缠论K线
                :return: MACD/KDJ/RSI 任一匹配
        """
        ...
    @classmethod
    def 买卖点全量匹配(cls, csl: Any, 缠K: "缠论K线") -> bool:
        """
        :param 缠K: 缠论K线
                :return: MACD/KDJ/RSI 全部匹配
        """
        ...
    @classmethod
    def 买卖点相对匹配(cls, 缠K: "缠论K线") -> bool:
        """
        :param 缠K: 缠论K线
                :return: 三个指标中至少两个匹配
        """
        ...
    @classmethod
    def 计算MACD柱子均值(cls, 普K序列: List[K线], 实线: "虚线") -> float:
        """
        :param 普K序列: 完整K线序列
                :param 实线: 虚线（笔/线段）
                :return: 区间内MACD柱绝对值的平均值
        """
        ...
    @classmethod
    def 计算MACD柱子均值_阴(cls, 普K序列: List[K线], 实线: "虚线") -> float:
        """
        :param 普K序列: 完整K线序列
                :param 实线: 虚线
                :return: 区间内负数MACD柱绝对值的平均值，无负数时返回False
        """
        ...
    @classmethod
    def 计算MACD柱子均值_阳(cls, 普K序列: List[K线], 实线: "虚线") -> float:
        """
        :param 普K序列: 完整K线序列
                :param 实线: 虚线
                :return: 区间内正数MACD柱绝对值的平均值，无正数时返回False
        """
        ...
    @classmethod
    def 武之全量MACD均值(cls, 普K序列: List[K线], 实线: "虚线") -> bool:
        """
        :param 普K序列: 完整K线序列
                :param 实线: 虚线
                :return: 终点K线的MACD柱是否小于区间均值（全量背驰信号）
        """
        ...
    @classmethod
    def 武之MACD均值(cls, 普K序列: List[K线], 实线: "虚线") -> bool:
        """
        :param 普K序列: 完整K线序列
                :param 实线: 虚线
                :return: 根据方向选择阳/阴均值判断
        """
        ...
    @classmethod
    def 武之MACD均值_阴(cls, 普K序列: List[K线], 实线: "虚线") -> bool:
        """
        :param 普K序列: 完整K线序列
                :param 实线: 虚线
                :return: 终点MACD柱绝对值小于阴均值时True（背驰）
        """
        ...
    @classmethod
    def 武之MACD均值_阳(cls, 普K序列: List[K线], 实线: "虚线") -> bool:
        """
        :param 普K序列: 完整K线序列
                :param 实线: 虚线
                :return: 终点MACD柱绝对值小于阳均值时True（背驰）
        """
        ...
    @classmethod
    def 武之MACD极值(cls, 普K序列: List[K线], 实线: "虚线") -> bool:
        """
        :param 普K序列: 完整K线序列
                :param 实线: 虚线
                :return: 终点MACD柱是否为区间内极值
        """
        ...
    @classmethod
    def 计算K线序列MACD趋向背驰(cls, 普K序列: Sequence["K线"], 方向: 相对方向) -> Any:
        """
        计算K线序列的MACD柱/DIF/DEA趋向背驰（三元素判断）

                :param 普K序列: K线序列
                :param 方向: 运行方向
                :return: [柱子背驰, DIF背驰, DEA背驰]
        """
        ...
    @classmethod
    @lru_cache(maxsize=128)
    def 买卖意义(cls, 实线: "虚线", 观察员: "观察者") -> Tuple[bool, str]:
        """
        静止是相对的，而运动是绝对的

                :param 实线: 虚线（笔/线段）
                :param 观察员: 观察者实例
                :return: (是否具有买卖意义, 原因字符串)
        """
        ...
    @classmethod
    def 计算MACD柱子分段(cls, k线序列: List["K线"]) -> Tuple[List[List["K线"]], ...]:
        """
        :param k线序列: K线序列
                :return: 按正负分段的MACD柱列表
        """
        ...
    @classmethod
    def 密集区域按间隔(cls, 交叉标记: List[int], 最大间隔: int = 5, 最少交叉数: int = 3) -> List[Tuple[int, int, int]]:
        """
        交叉标记: 长度为len(macd_list)的列表，0=无交叉, 1=金叉, -1=死叉
                最大间隔: 相邻交叉索引差 ≤ 此值则归入同一密集区
                最少交叉数: 一个密集区内至少包含的交叉次数

                :return: [(起始交叉索引, 结束交叉索引, 区内交叉次数), ...]
        """
        ...
    @classmethod
    def 统计MACD行为(cls, 普K序列: List[K线], 最大间隔: int = 8, 最少交叉数: int = 3) -> dict:
        """
        :param 普K序列: K线序列
                :param 最大间隔: 最大间隔 8
                :param 最少交叉数: 最少交叉数 3
                :return: 统计字典
        """
        ...
    @classmethod
    def 获取_武(cls, 实线: 虚线) -> 分型:
        """
        递归获取虚线的终点分型（笔直接返回武，线段递归到底层笔的武）
                :param 实线: 虚线
                :return: 分型
        """
        ...

class 笔(object):
    """
    笔 — 纯静态方法容器，提供笔划分算法的所有函数。

        主要方法:
            获取缠K数量 — 获取笔内缠K数量
            停顿 — 检测笔的停顿点
            自检 — 校验笔的有效性
            获取所有停顿位置 — 获取笔内所有停顿位置
            获取起始分型 — 获取笔的起始分型
            生成笔序列 — 从分型序列生成笔序列
            是否背驰过 — 判断笔是否经过背驰
    """
    @staticmethod
    def 获取缠K数量(缠K序列: List[缠论K线], 笔序列: List[虚线], 配置: 缠论配置) -> int:
        """
        获取笔内有效缠K数量（考虑笔弱化等配置）

                :param 缠K序列: 候选笔的基础缠K序列
                :param 笔序列: 已有笔序列
                :param 配置: 缠论配置
                :return: 有效缠K数量
        """
        ...
    @staticmethod
    def 次高(缠K序列: List[缠论K线], 笔内相同终点取舍: bool) -> 缠论K线:
        """
        次高

                :param 缠K序列: 缠K序列
                :param 笔内相同终点取舍: 终点取舍方式
                :return: 次高缠K
        """
        ...
    @staticmethod
    def 次低(缠K序列: List[缠论K线], 笔内相同终点取舍: bool) -> 缠论K线:
        """
        次低

                :param 缠K序列: 缠K序列
                :param 笔内相同终点取舍: 终点取舍方式
                :return: 次低缠K
        """
        ...
    @staticmethod
    def 实际高点(缠K序列: List[缠论K线], 笔内相同终点取舍: bool) -> 缠论K线:
        """
        实际高点

                :param 缠K序列: 缠K序列
                :param 笔内相同终点取舍: 终点取舍方式
                :return: 实际高点缠K
        """
        ...
    @staticmethod
    def 实际低点(缠K序列: List[缠论K线], 笔内相同终点取舍: bool) -> 缠论K线:
        """
        实际低点

                :param 缠K序列: 缠K序列
                :param 笔内相同终点取舍: 终点取舍方式
                :return: 实际低点缠K
        """
        ...
    @staticmethod
    def 相对关系(筆: 虚线, 配置: 缠论配置) -> bool:
        """
        相对关系

                :param 筆: 笔虚线
                :param 配置: 缠论配置
                :return: 方向是否匹配
        """
        ...
    @classmethod
    def 分析(cls, 当前分型: Optional[分型], 分型序列: List[分型], 笔序列: List[虚线], 缠K序列: List[缠论K线], 普K序列: List[K线], 递归层次: int, 配置: 缠论配置) -> Any:
        """
        笔划分核心递归算法

                :param 当前分型: 新形成的分型（可能为None）
                :param 分型序列: 现有分型列表（原地修改）
                :param 笔序列: 现有笔列表（原地修改）
                :param 缠K序列: 缠K序列
                :param 普K序列: 普K序列
                :param 递归层次: 递归深度计数
                :param 配置: 缠论配置
                :return: 递归层次（int）
        """
        ...
    @staticmethod
    def 以文会友(笔序列: List[虚线], 文: 分型) -> Optional[虚线]:
        """
        以文会友

                :param 笔序列: 笔列表
                :param 文: 起点分型
                :return: 匹配的笔或None
        """
        ...
    @staticmethod
    def 以武会友(笔序列: List[虚线], 武: 分型) -> Optional[虚线]:
        """
        以武会友

                :param 笔序列: 笔列表
                :param 武: 终点分型
                :return: 匹配的笔或None
        """
        ...
    @staticmethod
    def 根据缠K找笔(笔序列: List[虚线], 缠K: "缠论K线", 偏移: int = 1) -> Any:
        """
        根据缠K找笔

                :param 笔序列: 笔列表
                :param 缠K: 缠论K线
                :param 偏移: 序号偏移量
                :return: 包含该缠K的笔或None
        """
        ...
    @classmethod
    def 自检(cls, 筆: 虚线, 观察员: "观察者") -> bool:
        """
        校验笔的有效性：高/低点是否为实际极值

                :param 筆: 待校验的笔
                :param 观察员: 观察者
                :return: 有效为True
        """
        ...
    @classmethod
    def 获取所有停顿位置(cls, 筆: 虚线, 观察员: "观察者") -> List[虚线]:
        """
        获取笔内所有可能的停顿位置（用于背驰检测）

                :param 筆: 笔
                :param 观察员: 观察者
                :return: 停顿位置的笔列表
        """
        ...
    @classmethod
    def 是否背驰过(cls, 当前筆: 虚线, 观察员: "观察者") -> List[缠论K线]:
        """
        判断笔内是否发生过MACD趋向背驰

                :param 当前筆: 笔
                :param 观察员: 观察者
                :return: 发生背驰的停顿位置列表
        """
        ...

class 线段特征(list):
    """
    线段特征 — list 子类，持有一组同向虚线（笔），是线段划分的中间结构。

        继承 list，元素为虚线。特征序列用于线段划分算法中的包含处理。

        :ivar 序号: 序号
        :ivar 标识: 标识
        :ivar 线段方向: 线段运行方向（特征序列方向为其翻转）
        :ivar 文: 第一个元素的起点分型（根据方向选取极值）
        :ivar 武: 最后一个元素的终点分型（根据方向选取极值）
        :ivar 高: 最高价
        :ivar 低: 最低价
        :ivar 方向: 特征序列方向（线段方向的翻转）
        :ivar 图表标题: 图表标题
    """
    @property
    def 标识(self) -> str: ...
    @标识.setter
    def 标识(self, value: str) -> None: ...
    @property
    def 基础序列(self) -> List[虚线]: ...
    @基础序列.setter
    def 基础序列(self, value: List[虚线]) -> None: ...
    @property
    def 线段方向(self) -> 相对方向: ...
    @线段方向.setter
    def 线段方向(self, value: 相对方向) -> None: ...
    def __init__(self, 标识: str, 基础序列: List[虚线], 线段方向: 相对方向) -> Any:
        """
        :param 标识: 标识符
                :param 基础序列: 基础虚线列表
                :param 线段方向: 线段运行方向
        """
        ...
    def 图表标题(self) -> str:
        """:return: 图表标题"""
        ...
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def 文(self) -> 分型:
        """起点分型（向上线段取高高中的最大者，向下线段取低低中的最小者）"""
        ...
    def 武(self) -> 分型:
        """终点分型（向上线段取高高中的最大者，向下线段取低低中的最小者）"""
        ...
    def 高(self) -> float:
        """:return: 文和武中分型特征值的较大者"""
        ...
    def 低(self) -> float:
        """:return: 文和武中分型特征值的较小者"""
        ...
    def 方向(self) -> 相对方向:
        """:return: 特征序列方向（线段方向的翻转）"""
        ...
    def 添加(self, 待添加虚线: Union[虚线]) -> Any:
        """
        :param 待添加虚线: 待添加的虚线
                :raises ValueError: 方向不匹配时抛出
        """
        ...
    def 删除(self, 待删除虚线: Union[虚线]) -> Any:
        """
        :param 待删除虚线: 待删除的虚线
                :raises ValueError: 方向不匹配时抛出
        """
        ...
    @classmethod
    def 新建(cls, 虚线序列: List[虚线], 线段方向: 相对方向) -> "线段特征":
        """
        :param 虚线序列: 基础虚线列表
                :param 线段方向: 线段方向
                :return: 线段特征实例
        """
        ...
    @classmethod
    def 静态分析(cls, 虚线序列: List[虚线], 线段方向: 相对方向, 四象: str, 是否忽视: bool = False) -> List["线段特征"]:
        """
        静态分析虚线序列，生成特征序列

                :param 虚线序列: 笔/虚线列表
                :param 线段方向: 线段运行方向
                :param 四象: "老阳"/"老阴"/"小阳"/"少阴"
                :param 是否忽视: True时不严格处理缺口包含
                :return: 特征序列列表
        """
        ...
    @classmethod
    def 获取分型序列(cls, 特征序列: List) -> List[Any]:
        """
        从特征序列提取特征分型序列

                :param 特征序列: 线段特征列表
                :return: 特征分型列表
        """
        ...

class 特征分型:
    """
    特征分型 — 由左右中三个线段特征构成的顶/底分型，用于线段划分算法。

        :ivar 左: 左侧特征序列
        :ivar 中: 中间特征序列
        :ivar 右: 右侧特征序列
        :ivar 结构: 分型结构（顶/底）
    """
    @property
    def 左(self) -> 线段特征: ...
    @左.setter
    def 左(self, value: 线段特征) -> None: ...
    @property
    def 中(self) -> 线段特征: ...
    @中.setter
    def 中(self, value: 线段特征) -> None: ...
    @property
    def 右(self) -> 线段特征: ...
    @右.setter
    def 右(self, value: 线段特征) -> None: ...
    @property
    def 结构(self) -> 分型结构: ...
    @结构.setter
    def 结构(self, value: 分型结构) -> None: ...
    def __init__(self, 左: 线段特征, 中: 线段特征, 右: 线段特征, 结构: 分型结构) -> Any:
        """
        :param 左: 左侧特征序列
                :param 中: 中间特征序列
                :param 右: 右侧特征序列
                :param 结构: 分型结构
        """
        ...
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...

class 线段(object):
    """
    线段 — 纯静态方法容器，提供线段划分算法的所有函数。

        主要方法:
            检查线段破坏(前一线段, 当前线段) — 检查线段是否被破坏
            获取线段特征序列(笔序列, 线段方向, 需要被合并方向序列) — 提取特征序列
            线段有缺口(特征序列, 线段方向, 当前分型序列, 特征分型序列) — 判断线段是否有缺口
            判断新线段(笔序列) — 判断是否形成新线段
            静态分析(观察员) — 静态线段分析
            生成线段序列(笔序列, 观察员) — 从笔序列生成线段序列
    """
    @classmethod
    def 添加虚线(cls, 段: 虚线, 筆: 虚线) -> Any:
        """
        向线段中添加一笔

                :param 段: 线段
                :param 筆: 待添加的笔
                :raises ValueError: 不连续或标识不符时抛出
        """
        ...
    @classmethod
    def 武斗(cls, 段: 虚线, 武: 分型, 行号: int) -> Any:
        """
        更新线段的终点分型（武）

                :param 段: 线段
                :param 武: 新的终点分型
                :param 行号: 调用行号（用于调试）
        """
        ...
    @classmethod
    def 特征分型终结(cls, 段: 虚线) -> bool:
        """
        是否符合特征序列正常分型终结

                :param 段: 线段
                :return: 是否终结
        """
        ...
    @classmethod
    def 特征序列状态(cls, 段: 虚线) -> Tuple[bool, bool, bool]:
        """
        :param 段: 线段
                :return: (左是否存在, 中是否存在, 右是否存在)
        """
        ...
    @classmethod
    def 获取缺口(cls, 段: 虚线) -> Optional[缺口]:
        """
        获取线段特征序列第一二元素间的缺口

                :param 段: 线段
                :return: 缺口或None
        """
        ...
    @classmethod
    def 四象(cls, 段: 虚线) -> str:
        """
        判断线段的四象属性

                :param 段: 线段
                :return: "老阳"(向下线段后继缺口向上线段) / "老阴"(向上线段后继缺口向下线段) / "小阳"(普通向上) / "少阴"(普通向下)
        """
        ...
    @classmethod
    def 设置特征序列(cls, 段: 虚线, 序列: Any, 行号: Any) -> Any:
        """
        设置特征序列

                :param 段: 线段
                :param 序列: 特征序列三元组 (左,中,右)
                :param 行号: 调用行号
        """
        ...
    @classmethod
    def 刷新特征序列(cls, 段: 虚线, 配置: 缠论配置) -> Any:
        """
        刷新特征序列

                :param 段: 线段
                :param 配置: 缠论配置
        """
        ...
    @classmethod
    def 分割序列(cls, 段: 虚线, 所属中枢: Optional["中枢"] = None) -> Tuple[List[虚线], List[虚线], List[虚线], Optional[虚线]]:
        """
        将线段基础序列分割为前/后/第三买卖/贯穿伤四部分

                :param 段: 线段
                :param 所属中枢: 所属中枢（用于第三买卖点检测）
                :return: (前序列, 后序列, 第三买卖线, 贯穿伤)
        """
        ...
    @classmethod
    def 刷新(cls, 段: 虚线, 配置: 缠论配置) -> Any:
        """
        刷新线段的特征序列和内部中枢序列

                :param 段: 线段
                :param 配置: 缠论配置
        """
        ...
    @classmethod
    def 序列重置(cls, 段: 虚线, 序列: Sequence) -> Any:
        """
        序列重置

                :param 段: 线段
                :param 序列: 参考序列
        """
        ...
    @classmethod
    def 查找贯穿伤(cls, 段: 虚线) -> Optional[虚线]:
        """
        查找贯穿伤

                :param 段: 线段
                :return: 贯穿伤虚线或None
        """
        ...
    @classmethod
    def 获取内部中枢序列(cls, 段: 虚线, 配置: 缠论配置) -> Tuple[List["中枢"], List["中枢"], List["中枢"]]:
        """
        获取内部中枢序列

                :param 段: 线段
                :param 配置: 缠论配置
                :return: (虚中枢列表, 实中枢列表, 合中枢列表)
        """
        ...
    @classmethod
    def 基础判断(cls, 左: 虚线, 中: 虚线, 右: 虚线, 关系序列: List[相对方向]) -> bool:
        """
        连续三笔且重叠

                :param 左: 左侧虚线
                :param 中: 中间虚线
                :param 右: 右侧虚线
                :param 关系序列: 允许的方向关系
                :return: 是否满足基础条件
        """
        ...
    @classmethod
    def 分析(cls, 笔序列: List[虚线], 线段序列: List[虚线], 配置: 缠论配置, 层级: int = 0, 关系序列: Any = [相对方向.向上, 相对方向.向下]) -> None:
        """
        线段划分核心递归算法

                :param 笔序列: 笔列表
                :param 线段序列: 线段列表（原地修改）
                :param 配置: 缠论配置
                :param 层级: 递归深度
                :param 关系序列: 允许的方向关系
        """
        ...
    @classmethod
    def 武终(cls, 段: 虚线, 行号: int) -> Any:
        """
        武终

                :param 段: 线段
                :param 行号: 调用行号
        """
        ...
    @classmethod
    def 验证序列(cls, 段: 虚线, 序列: Sequence) -> Any:
        """
        验证序列

                :param 段: 线段
                :param 序列: 参考序列
        """
        ...
    @classmethod
    def 扩展分析(cls, 虚线序列: List[虚线], 线段序列: List[虚线], 配置: 缠论配置) -> None:
        """
        即同级别分析
                将笔看成线段

                :param 虚线序列: 基础虚线列表
                :param 线段序列: 扩展线段列表（原地修改）
                :param 配置: 缠论配置
        """
        ...
    @classmethod
    @lru_cache(maxsize=128)
    def 判断线段内部是否背驰(cls, 当前段: 虚线, 观察员: 观察者) -> bool:
        """
        判断线段内部是否发生背驰（基于内部中枢和MACD）

                :param 当前段: 线段
                :param 观察员: 观察者
                :return: bool
        """
        ...
    @classmethod
    def 获取所有停顿位置(cls, 段: 虚线, 观察员: "观察者") -> List[虚线]:
        """
        获取所有停顿位置

                :param 段: 线段
                :param 观察员: 观察者
                :return: 停顿位置的线段列表
        """
        ...
    @classmethod
    def 是否背驰过(cls, 当前段: 虚线, 观察员: "观察者") -> List[缠论K线]:
        """
        判断线段内是否发生过背驰（遍历所有停顿位置）

                :param 当前段: 线段
                :param 观察员: 观察者
                :return: 背驰点列表
        """
        ...

class 中枢(object):
    """
    中枢 — 三段虚线重叠区间构成的价格中枢，支持延伸和扩展。

        :ivar 序号: 序号
        :ivar 标识: 标识
        :ivar 级别: 级别
        :ivar 基础序列: 构成中枢的基础虚线（至少3条）
        :ivar 第三买卖线: 第三类买卖点关联虚线
        :ivar 本级_第三买卖线: 本级第三类买卖点虚线
        :ivar 高: 中枢上沿（虚线低点的最大值）
        :ivar 低: 中枢下沿（虚线高点的最小值）
        :ivar 高高: 全区间最高价
        :ivar 低低: 全区间最低价
        :ivar 文: 起点分型
        :ivar 武: 终点分型
        :ivar 方向: 中枢方向（首条虚线的方向翻转）
        :ivar 离开段: 最后一条虚线
    """
    @property
    def 序号(self) -> int: ...
    @序号.setter
    def 序号(self, value: int) -> None: ...
    @property
    def 标识(self) -> str: ...
    @标识.setter
    def 标识(self, value: str) -> None: ...
    @property
    def 级别(self) -> int: ...
    @级别.setter
    def 级别(self, value: int) -> None: ...
    @property
    def 基础序列(self) -> List[虚线]: ...
    @基础序列.setter
    def 基础序列(self, value: List[虚线]) -> None: ...
    def __init__(self, 序号: int, 标识: str, 级别: int, 基础序列: List[虚线]) -> Any:
        """
        :param 序号: 序号
                :param 标识: 标识
                :param 级别: 级别
                :param 基础序列: 基础虚线列表
        """
        ...
    def 添加虚线(self, 实线: 虚线) -> Any:
        """
        向中枢添加新虚线（延伸），重置第三买卖线

                :param 实线: 新虚线
        """
        ...
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def 图表标题(self) -> str:
        """:return: 图表标题"""
        ...
    def 离开段(self) -> 虚线:
        """:return: 最后一条虚线"""
        ...
    def 方向(self) -> 相对方向:
        """:return: 中枢方向（首条虚线的方向翻转）"""
        ...
    def 高(self) -> float:
        """:return: 中枢上沿（前三段中虚线高点的最小值）"""
        ...
    def 低(self) -> float:
        """:return: 中枢下沿（前三段中虚线低点的最大值）"""
        ...
    def 高高(self) -> float:
        """:return: 全区间最高价"""
        ...
    def 低低(self) -> float:
        """:return: 全区间最低价"""
        ...
    def 文(self) -> 分型:
        """:return: 起点分型"""
        ...
    def 武(self) -> 分型:
        """:return: 终点分型"""
        ...
    def 获取数据文本(self) -> str:
        """获取用于保存的数据文本"""
        ...
    def 完整性(self, 虚实: str) -> Any:
        """
        判断中枢是否完整（是否有第三买卖点或内部中枢离开）

                详情见 教你炒股票 43：有关背驰的补习课(2007-04-06 15:31:28)
                不完整时 下一个中枢大概率会与当前中枢发生扩展！

                :param 虚实: "实"/"虚"/"合"
                :return: 完整为True
        """
        ...
    def 获取序列(self) -> List[虚线]:
        """
        获取中枢的完整虚线序列（基础序列+第三买卖线）

                :return: 虚线列表
        """
        ...
    def 获取扩展中枢(self, 扩展中枢: List, 配置: 缠论配置) -> Any:
        """
        当基础序列>=9时，从中枢中提取扩展线段中枢

                :param 扩展中枢: 存放扩展中枢的列表
                :param 配置: 缠论配置
        """
        ...
    def 校验合法性(self, 序列: Sequence[虚线], 中枢序列: List["中枢"]) -> bool:
        """
        校验当前中枢在给定序列中是否仍然合法

                :param 序列: 基础虚线序列
                :param 中枢序列: 中枢列表
                :return: 合法为True，不合法会原地裁剪基础序列
        """
        ...
    def 设置第三买卖线(self, 线: Union[虚线, None]) -> Any:
        """
        设置第三类买卖点关联虚线

                :param 线: 第三买卖虚线或None
        """
        ...
    def 当前状态(self) -> Any:
        """
        获取中枢当前状态：中枢之中/中枢之上/中枢之下

                详情见 教你炒股票 49：利润率最大的操作模式(2007-04-26 08:16:56)
                当前中枢最后一段所处的位置关系

                一、当下在该中枢之中。
                    因为在中枢里，由于这时候怎么演化都是对的，不操作是最好的操作，等待其演化第二、三类，
                    当然，如果你技术好点，可以判断出次级别的第二类买点，这些买点很多情况下都是在中枢中出现的，那当然也是可以参与的。
                    但如果没有这种技术，那就有了再说了。只把握你自己当下技术水平能把握的机会，这才是最重要的。
                二、当下在该中枢之下。
                    1.当下之前未出现该中枢第三类卖点。
                    2.当下之前已出现该中枢第三类卖点（正出现也包括在这种情况下，按最严格的定义，这最精确的卖点，是瞬间完成的，而具有操作意义的第三类卖点，其实是一个包含该最精确卖点的足够小区间）
                三、当下在该中枢之上。
                    1.当下之前未出现该中枢第三类买点。
                    2.当下之前已出现该中枢第三类买点。

                :return: "中枢之中" / "中枢之上" / "中枢之下"
        """
        ...
    @classmethod
    def 基础检查(cls, 左: 虚线, 中: 虚线, 右: 虚线) -> bool:
        """
        检查三条虚线是否构成中枢（连续且重叠）

                :param 左: 左侧虚线
                :param 中: 中间虚线
                :param 右: 右侧虚线
                :return: bool
        """
        ...
    @classmethod
    def 创建(cls, 左: 虚线, 中: 虚线, 右: 虚线, 级别: int, 标识: str = "") -> "中枢":
        """
        从三条连续且重叠的虚线创建中枢

                :param 左: 左侧虚线
                :param 中: 中间虚线
                :param 右: 右侧虚线
                :param 级别: 中枢级别
                :param 标识: 中枢标识前缀
                :return: 中枢实例
        """
        ...
    @classmethod
    def 从序列中获取中枢(cls, 虚线序列: Sequence[虚线], 起始方向: 相对方向, 标识: str) -> Optional["中枢"]:
        """
        从虚线序列中按起始方向查找第一个中枢

                :param 虚线序列: 虚线列表
                :param 起始方向: 第一条虚线的方向
                :param 标识: 中枢标识前缀
                :return: 中枢或None
        """
        ...
    @classmethod
    def 向中枢序列尾部添加(cls, 中枢序列: List["中枢"], 待添加中枢: "中枢") -> Any:
        """
        :param 中枢序列: 中枢列表
                :param 待添加中枢: 待添加的中枢
        """
        ...
    @classmethod
    def 从中枢序列尾部弹出(cls, 中枢序列: List["中枢"], 待弹出中枢: "中枢") -> Optional["中枢"]:
        """
        :param 中枢序列: 中枢列表
                :param 待弹出中枢: 待弹出的中枢
                :return: 弹出的中枢或None
        """
        ...
    @classmethod
    def 分析(cls, 虚线序列: Sequence[虚线], 中枢序列: List["中枢"], 跳过首部: bool = True, 标识: str = "", 层级: int = 0) -> None:
        """
        中枢识别核心递归算法

                :param 虚线序列: 基础虚线列表
                :param 中枢序列: 中枢列表（原地修改）
                :param 跳过首部: True: 跳过首元素中枢
                :param 标识: 中枢标识前缀
                :param 层级: 递归深度
        """
        ...

class 观察者:
    """
    观察者 — 单周期缠论分析器，接收K线流式输入并逐层计算所有层级序列。

        核心入口为 增加原始K线(普K)，每收到一根新K线就增量更新：
        原始K线 → 缠论K线（包含处理+合并）→ 分型 → 笔 → 线段 → 中枢 → 买卖点

        :ivar 标识: "{符号}:{周期}"
        :ivar 当前K线: 最后一根原始K线
        :ivar 当前缠K: 最后一根缠论K线
        :ivar 普通K线序列 / 缠论K线序列 / 分型序列 / 笔序列 / 线段序列 / 中枢序列 等
    """
    @property
    def 符号(self) -> str: ...
    @符号.setter
    def 符号(self, value: str) -> None: ...
    @property
    def 周期(self) -> int: ...
    @周期.setter
    def 周期(self, value: int) -> None: ...
    @property
    def 配置(self) -> 缠论配置: ...
    @配置.setter
    def 配置(self, value: 缠论配置) -> None: ...
    def __init__(self, 符号: str, 周期: int, 配置: 缠论配置) -> Any:
        """
        初始化观察者

                :param 符号: 交易对符号
                :param 周期: K线周期（秒）
                :param 配置: 缠论配置
        """
        ...
    def 观察员(self) -> Any:
        """观察员（自引用）"""
        ...
    def 标识(self) -> str:
        ''':return: "{符号}:{周期}"'''
        ...
    def 当前K线(self) -> Optional["K线"]:
        """:return: 最后一根原始K线"""
        ...
    def 当前缠K(self) -> Optional["缠论K线"]:
        """:return: 最后一根缠论K线"""
        ...
    def 重置基础序列(self) -> Any:
        """清空所有分析序列，重置为初始状态"""
        ...
    @final
    def 增加原始K线(self, 普K: K线) -> Any:
        """
        核心入口 — 投喂一根原始K线，增量更新所有层级

                :param 普K: 新到的原始K线

                处理流程: 原始K线→缠论K线（包含处理+合并）→分型→笔→线段→中枢→买卖点
        """
        ...
    def 测试_保存数据(self, root: str = None) -> Any:
        """
        拆分各序列数据，单独存文件，文件名为对应变量名

                :param root: 保存根目录（可选）
        """
        ...
    def 识别买卖点(self) -> Any:
        """识别买卖点（占位方法，具体逻辑在子类或Rust核心中实现）"""
        ...
    def 静态重新分析(self) -> Any:
        """静态重新分析（占位方法）"""
        ...
    def 加载本地数据(self, 文件路径: str) -> Any: ...
    @classmethod
    def 读取数据文件(cls, 文件路径: str, 配置: Any = 缠论配置()) -> Self:
        """
        :param 文件路径: 数据文件路径 格式如: btcusd-300-1631772074-1632222374.nb
                :param 配置: 缠论配置
                :return: 观察者实例
        """
        ...

class K线合成器:
    """
    K线合成器 — 将小周期K线合成为大周期K线，支持多周期级联合成。

        :ivar 标识: 合成器标识
        :ivar 周期组: 从小到大排列的周期组
        :ivar 当前K线: 各周期当前K线
        :ivar 合成K线列表: 各周期已合成的K线列表
    """
    @property
    def 标识(self) -> str: ...
    @标识.setter
    def 标识(self, value: str) -> None: ...
    @property
    def 周期组(self) -> List[int]: ...
    @周期组.setter
    def 周期组(self, value: List[int]) -> None: ...
    @property
    def 事件回调(self) -> Optional[Callable]: ...
    @事件回调.setter
    def 事件回调(self, value: Optional[Callable]) -> None: ...
    def __init__(self, 标识: str, 周期组: List[int], 事件回调: Optional[Callable] = None) -> Any:
        """
        初始化K线合成器

                :param 标识: 合成器标识
                :param 周期组: 从小到大排列的周期列表
                :param 事件回调: 可选，K线合成完成时的回调函数(信号类型, 标识, 周期, 完成K线)
        """
        ...
    def 设置事件回调(self, 回调函数: Callable) -> Any:
        """
        设置事件回调函数

                :param 回调函数: 回调函数
        """
        ...
    def 投喂(self, 时间戳: datetime, 开: float, 高: float, 低: float, 收: float, 量: float) -> Any:
        """
        投喂原始tick数据

                :param 时间戳: 时间戳
                :param 开: 开盘价
                :param 高: 最高价
                :param 低: 最低价
                :param 收: 收盘价
                :param 量: 成交量
        """
        ...
    def 投喂K线(self, 普K: K线) -> Any:
        """
        统一入口 — 投喂最小周期K线，自动合成大周期并分发给各周期观察者

                :param 普K: 最小周期原始K线
        """
        ...
    def 获取当前K线(self, 周期: int) -> Optional[K线]:
        """
        获取指定周期当前正在合成的K线

                :param 周期: 目标周期
                :return: 当前K线或None
        """
        ...

class 立体分析器:
    """
    立体分析器 — 多周期缠论分析器，内部包含K线合成器 + 每周期一个观察者。

        通过最小周期合成大周期K线，各周期观察者独立分析，实现多周期联立。

        :ivar 周期组: 所有分析周期
        :ivar 观察员: 各周期对应的观察者
        :ivar K线合成器: 内部K线合成器
    """
    @property
    def 符号(self) -> str: ...
    @符号.setter
    def 符号(self, value: str) -> None: ...
    @property
    def 周期组(self) -> List[int]: ...
    @周期组.setter
    def 周期组(self, value: List[int]) -> None: ...
    @property
    def 配置(self) -> 缠论配置: ...
    @配置.setter
    def 配置(self, value: 缠论配置) -> None: ...
    @property
    def 配置组(self) -> Dict[int, 缠论配置]: ...
    @配置组.setter
    def 配置组(self, value: Dict[int, 缠论配置]) -> None: ...
    def __init__(self, 符号: str, 周期组: List[int], 配置: 缠论配置 = 缠论配置(), 配置组: Dict[int, 缠论配置] = dict()) -> Any:
        """
        初始化多周期立体分析器

                :param 符号: 交易对符号
                :param 周期组: 所有分析周期列表（第一个为最小输入周期）
                :param 配置: 默认配置
                :param 配置组: 各周期独立配置 {周期: 缠论配置}
        """
        ...
    def 投喂K线(self, 普K: K线) -> Any:
        """
        统一入口 — 投喂最小周期K线，自动合成大周期并分发给各周期观察者

                :param 普K: 最小周期原始K线
                :raises RuntimeError: 当投喂的K线周期与输入周期不匹配时
        """
        ...
    def 测试_保存数据(self) -> Any:
        """拆分各序列数据，单独存文件，文件名为对应变量名"""
        ...

def 转化为时间戳(ts: Any) -> int: ...
def 转化为时间戳_数字(ts: Any) -> int: ...
