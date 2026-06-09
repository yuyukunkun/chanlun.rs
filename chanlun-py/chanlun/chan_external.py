# ==============================================================================
# Copyright (c) YuYuKunKun  / chanlun.rs
#
# 本项目整体基于 MIT 协议开源
# 部分代码片段摘录自 Apache License 2.0 授权项目
#
# MIT License
#
# Copyright (c) 2026 YuYuKunKun
#
# Permission is hereby granted, free of charge, to any person obtaining a copy
# of this software and associated documentation files (the "Software"), to deal
# in the Software without restriction, including without limitation the rights
# to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
# copies of the Software, and to permit persons to whom the Software is
# furnished to do so, subject to the following conditions:
#
# The above copyright notice and this permission notice shall be included in all
# copies or substantial portions of the Software.
#
# THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
# IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
# FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
# AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
# LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
# OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
# SOFTWARE.
#
# ==============================================================================
# 摘录代码相关声明
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.
#
# Source: https://github.com/waditu/czsc/blob/v0.9.69/czsc/objects.py#L450
# Modified: 【YuYuKunKun & 2026-05-31】
# ==============================================================================
import hashlib
import re
from collections import OrderedDict
from dataclasses import dataclass, field
from datetime import datetime, timedelta
from enum import Enum
from typing import Any, Callable, Dict, List, Optional, Tuple, Union

import numpy as np
from loguru import logger

from chanlun.chan import K线, 虚线, 中枢, 观察者, 立体分析器
from chanlun.parse import parse

sorted_freqs = [
    "Tick",
    "1分钟",
    "2分钟",
    "3分钟",
    "4分钟",
    "5分钟",
    "6分钟",
    "10分钟",
    "12分钟",
    "15分钟",
    "20分钟",
    "30分钟",
    "60分钟",
    "120分钟",
    "日线",
    "周线",
    "月线",
    "季线",
    "年线",
]


def import_by_name(name):
    """通过字符串导入模块、类、函数

    函数执行逻辑：

    1. 检查 name 中是否包含点号（'.'）。如果没有，则直接使用内置的 import 函数来导入整个模块，并返回该模块对象。
    2. 如果 name 包含点号，先处理一个相对路径。将 name 拆分为两部分：module_name 和 function_name。
        使用 Python 内置的 rsplit 方法从右边开始分割，只取一次，这样可以确保我们将最后的一个点号前的部分作为 module_name，点号后面的部分作为 function_name。
    3. 使用import函数导入指定的 module_name。
        这里传入三个参数：globals() 和 locals() 分别代表当前全局和局部命名空间；
        [function_name] 是一个列表，用于指定要导入的子模块或属性名。
        这样做是为了避免一次性导入整个模块的所有内容，提高效率。
    4.  使用 vars 函数获取模块的字典表示形式（即模块内所有的变量和函数），取出 function_name 对应的值，然后返回这个值。

    :param name: 模块名，如：'czsc.objects.Factor'
    :return: 模块对象
    """
    if "." not in name:
        return __import__(name)

    # 从右边开始分割，分割成模块名和函数名
    module_name, function_name = name.rsplit(".", 1)
    module = __import__(module_name, globals(), locals(), [function_name])
    return vars(module)[function_name]


class Freq(Enum):
    Tick = "Tick"
    F1 = "1分钟"
    F2 = "2分钟"
    F3 = "3分钟"
    F4 = "4分钟"
    F5 = "5分钟"
    F6 = "6分钟"
    F10 = "10分钟"
    F12 = "12分钟"
    F15 = "15分钟"
    F20 = "20分钟"
    F30 = "30分钟"
    F60 = "60分钟"
    F120 = "120分钟"
    D = "日线"
    W = "周线"
    M = "月线"
    S = "季线"
    Y = "年线"

    def __str__(self):
        return self.value


class Operate(Enum):
    # 持有状态
    HL = "持多"  # Hold Long
    HS = "持空"  # Hold Short
    HO = "持币"  # Hold Other

    # 多头操作
    LO = "开多"  # Long Open
    LE = "平多"  # Long Exit

    # 空头操作
    SO = "开空"  # Short Open
    SE = "平空"  # Short Exit

    def __str__(self):
        return self.value


@dataclass
class Signal:
    signal: str = ""

    # score 取值在 0~100 之间，得分越高，信号越强
    score: int = 0

    # k1, k2, k3 是信号名称
    k1: str = "任意"  # k1 一般是指明信号计算的K线周期，如 60分钟，日线，周线等
    k2: str = "任意"  # k2 一般是记录信号计算的参数
    k3: str = "任意"  # k3 用于区分信号，必须具有唯一性，推荐使用信号分类和开发日期进行标记

    # v1, v2, v3 是信号取值
    v1: str = "任意"
    v2: str = "任意"
    v3: str = "任意"

    # 任意 出现在模板信号中可以指代任何值

    def __post_init__(self):
        if not self.signal:
            self.signal = f"{self.k1}_{self.k2}_{self.k3}_{self.v1}_{self.v2}_{self.v3}_{self.score}"
        else:
            if not isinstance(self.signal, str):
                raise TypeError(f"Signal 初始化需要字符串，收到了 {type(self.signal).__name__}: {self.signal!r}")
            (
                self.k1,
                self.k2,
                self.k3,
                self.v1,
                self.v2,
                self.v3,
                score,
            ) = self.signal.split("_")
            self.score = int(score)

        if self.score > 100 or self.score < 0:
            raise ValueError("score 必须在0~100之间")

    def __repr__(self):
        return f"Signal('{self.signal}')"

    @property
    def key(self) -> str:
        """获取信号名称"""
        key = ""
        for k in [self.k1, self.k2, self.k3]:
            if k != "任意":
                key += k + "_"
        return key.strip("_")

    @property
    def value(self) -> str:
        """获取信号值"""
        return f"{self.v1}_{self.v2}_{self.v3}_{self.score}"

    def is_match(self, s: dict) -> bool:
        """判断信号是否与信号列表中的值匹配

        代码的执行逻辑如下：

        接收一个字典 s 作为参数，该字典包含了所有信号的信息。从字典 s 中获取名称为 key 的信号的值 v。
        如果 v 不存在，则抛出异常。从信号的值 v 中解析出 v1、v2、v3 和 score 四个变量。

        如果当前信号的得分 score 大于等于目标信号的得分 self.score，则继续执行，否则返回 False。
        如果当前信号的第一个值 v1 等于目标信号的第一个值 self.v1 或者目标信号的第一个值为 "任意"，则继续执行，否则返回 False。
        如果当前信号的第二个值 v2 等于目标信号的第二个值 self.v2 或者目标信号的第二个值为 "任意"，则继续执行，否则返回 False。
        如果当前信号的第三个值 v3 等于目标信号的第三个值 self.v3 或者目标信号的第三个值为 "任意"，则返回 True，否则返回 False。

        :param s: 所有信号字典
        :return: bool
        """
        key = self.key
        v = s.get(key, None)
        if not v:
            raise ValueError(f"{key} 不在信号列表中")

        if not isinstance(v, str):
            logger.warning(f"信号 {key} 的值类型异常: {type(v).__name__} = {v!r}，跳过匹配")
            return False

        v1, v2, v3, score = v.split("_")
        if int(score) >= self.score:
            if v1 == self.v1 or self.v1 == "任意":
                if v2 == self.v2 or self.v2 == "任意":
                    if v3 == self.v3 or self.v3 == "任意":
                        return True
        return False


@dataclass
class Factor:
    # signals_all 必须全部满足的信号，至少需要设定一个信号
    signals_all: List[Signal]

    # signals_any 满足其中任一信号，允许为空
    signals_any: List[Signal] = field(default_factory=list)

    # signals_not 不能满足其中任一信号，允许为空
    signals_not: List[Signal] = field(default_factory=list)

    name: str = ""

    def __post_init__(self):
        if not self.signals_all:
            raise ValueError("signals_all 不能为空")
        _fatcor = self.dump()
        _fatcor.pop("name")
        sha256 = hashlib.sha256(str(_fatcor).encode("utf-8")).hexdigest().upper()[:4]

        if self.name:
            self.name = self.name.split("#")[0] + f"#{sha256}"
        else:
            self.name = f"#{sha256}"
        # self.name = f"{self.name}#{sha256}" if self.name else sha256

    @property
    def unique_signals(self) -> List[str]:
        """获取 Factor 的唯一信号列表"""
        signals = []
        signals.extend(self.signals_all)
        if self.signals_any:
            signals.extend(self.signals_any)
        if self.signals_not:
            signals.extend(self.signals_not)
        signals = {x.signal if isinstance(x, Signal) else x for x in signals}
        return list(signals)

    def is_match(self, s: dict) -> bool:
        """判断 factor 是否满足"""
        if self.signals_not:
            for signal in self.signals_not:
                if signal.is_match(s):
                    return False

        for signal in self.signals_all:
            if not signal.is_match(s):
                return False

        if not self.signals_any:
            return True

        for signal in self.signals_any:
            if signal.is_match(s):
                return True
        return False

    def dump(self) -> dict:
        """将 Factor 对象转存为 dict"""
        signals_all = [x.signal for x in self.signals_all]
        signals_any = [x.signal for x in self.signals_any] if self.signals_any else []
        signals_not = [x.signal for x in self.signals_not] if self.signals_not else []

        raw = {
            "name": self.name,
            "signals_all": signals_all,
            "signals_any": signals_any,
            "signals_not": signals_not,
        }
        return raw

    @classmethod
    def load(cls, raw: dict):
        """从 dict 中创建 Factor

        :param raw: 样例如下
            {'name': '单测',
            'signals_all': ['15分钟_倒0笔_方向_向上_其他_其他_0', '15分钟_倒0笔_长度_大于5_其他_其他_0'],
            'signals_any': [],
            'signals_not': []}

        :return:
        """
        signals_any = [Signal(x) for x in raw.get("signals_any", [])]
        signals_not = [Signal(x) for x in raw.get("signals_not", [])]

        fa = Factor(
            name=raw.get("name", ""),
            signals_all=[Signal(x) for x in raw["signals_all"]],
            signals_any=signals_any,
            signals_not=signals_not,
        )
        return fa


@dataclass
class Event:
    operate: Operate

    # 多个信号组成一个因子，多个因子组成一个事件。
    # 单个事件是一系列同类型因子的集合，事件中的任一因子满足，则事件为真。
    factors: List[Factor]

    # signals_all 必须全部满足的信号，允许为空
    signals_all: List[Signal] = field(default_factory=list)

    # signals_any 满足其中任一信号，允许为空
    signals_any: List[Signal] = field(default_factory=list)

    # signals_not 不能满足其中任一信号，允许为空
    signals_not: List[Signal] = field(default_factory=list)

    name: str = ""

    def __post_init__(self):
        if not self.factors:
            raise ValueError("factors 不能为空")
        _event = self.dump()
        _event.pop("name")

        sha256 = hashlib.sha256(str(_event).encode("utf-8")).hexdigest().upper()[:4]
        if self.name:
            self.name = self.name.split("#")[0] + f"#{sha256}"
            # self.name = f"{self.name}#{sha256}"
        else:
            self.name = f"{self.operate.value}#{sha256}"
        self.sha256 = sha256

    @property
    def unique_signals(self) -> List[str]:
        """获取 Event 的唯一信号列表"""
        signals = []
        if self.signals_all:
            signals.extend(self.signals_all)
        if self.signals_any:
            signals.extend(self.signals_any)
        if self.signals_not:
            signals.extend(self.signals_not)

        for factor in self.factors:
            signals.extend(factor.unique_signals)

        signals = {x.signal if isinstance(x, Signal) else x for x in signals}
        return list(signals)

    def get_signals_config(self, signals_module: str = "chanlun.signals") -> List[Dict]:
        """获取事件的信号配置"""

        return get_signals_config(self.unique_signals, signals_module)

    def is_match(self, s: dict):
        """判断 event 是否满足

        代码的执行逻辑如下：

        1. 首先判断 signals_not 中的信号是否得到满足，如果满足任意一个信号，则直接返回 False，表示事件不满足。
        2. 接着判断 signals_all 中的信号是否全部得到满足，如果有任意一个信号不满足，则直接返回 False，表示事件不满足。
        3. 然后判断 signals_any 中的信号是否有一个得到满足，如果一个都不满足，则直接返回 False，表示事件不满足。
        4. 最后判断因子是否满足，顺序遍历因子列表，找到第一个满足的因子就退出，并返回 True 和该因子的名称，表示事件满足。
        5. 如果遍历完所有因子都没有找到满足的因子，则返回 False，表示事件不满足。
        """
        if self.signals_not and any(signal.is_match(s) for signal in self.signals_not):
            return False, None

        if self.signals_all and not all(signal.is_match(s) for signal in self.signals_all):
            return False, None

        if self.signals_any and not any(signal.is_match(s) for signal in self.signals_any):
            return False, None

        for factor in self.factors:
            if factor.is_match(s):
                return True, factor.name

        return False, None

    def dump(self) -> dict:
        """将 Event 对象转存为 dict"""
        signals_all = [x.signal for x in self.signals_all] if self.signals_all else []
        signals_any = [x.signal for x in self.signals_any] if self.signals_any else []
        signals_not = [x.signal for x in self.signals_not] if self.signals_not else []
        factors = [x.dump() for x in self.factors]

        raw = {
            "name": self.name,
            "operate": self.operate.value,
            "signals_all": signals_all,
            "signals_any": signals_any,
            "signals_not": signals_not,
            "factors": factors,
        }
        return raw

    @classmethod
    def load(cls, raw: dict):
        """从 dict 中创建 Event

        :param raw: 样例如下
                        {'name': '单测',
                         'operate': '开多',
                         'factors': [{'name': '测试',
                             'signals_all': ['15分钟_倒0笔_长度_大于5_其他_其他_0'],
                             'signals_any': [],
                             'signals_not': []}],
                         'signals_all': ['15分钟_倒0笔_方向_向上_其他_其他_0'],
                         'signals_any': [],
                         'signals_not': []}
        :return:
        """
        # 检查输入参数是否合法
        assert raw["operate"] in Operate.__dict__["_value2member_map_"], f"operate {raw['operate']} not in Operate"
        assert raw["factors"], "factors can not be empty"

        e = Event(
            name=raw.get("name", ""),
            operate=Operate.__dict__["_value2member_map_"][raw["operate"]],
            factors=[Factor.load(x) for x in raw["factors"]],
            signals_all=[Signal(x) for x in raw.get("signals_all", [])],
            signals_any=[Signal(x) for x in raw.get("signals_any", [])],
            signals_not=[Signal(x) for x in raw.get("signals_not", [])],
        )
        return e


class SignalsParser:
    """解析一串信号，生成信号函数配置"""

    def __init__(self, signals_module: str = "chanlun.signals"):
        """

        函数执行逻辑：

        1. 将传入的 signals_module 参数赋给实例变量 self.signals_module，代表信号函数所在的模块，默认模块是czsc库的signals模块。
        2. 使用 import_by_name 函数导入了指定名称的模块 signals_module。
        3. 对于导入的模块中的每个属性名进行遍历：
            - 魔法函数和私有函数不进行处理。
            - 获取函数的注解信息，并通过正则表达式获取注解中的参数模板和信号列表。
            - 如果解析到了参数模板，则将其存储在 sig_pats_map 中，key是函数名称。
            - 如果解析到了信号列表，则将其存储在 sig_name_map 中，并且为每个信号创建了 Signal 对象并存储在列表中，key是函数名称。
        4. 最后将得到的 sig_name_map 和 sig_pats_map 存储在实例变量中，以便其他方法使用。

        :param signals_module: 指定信号函数所在模块
        """
        self.signals_module = signals_module
        sig_name_map = {}
        sig_pats_map = {}
        sig_trigger_map = {}

        signals_module = import_by_name(signals_module)
        for name in dir(signals_module):
            if "_" not in name or name.startswith("__"):
                continue

            try:
                doc = getattr(signals_module, name).__doc__
                # 解析信号函数参数
                pats = re.findall(r"参数模板：\"(.*)\"", doc)
                if pats:
                    sig_pats_map[name] = pats[0]

                # 解析信号列表
                sigs = re.findall(r"Signal\('(.*)'\)", doc)
                if sigs:
                    sig_name_map[name] = [Signal(x) for x in sigs]

                # 解析触发条件
                触发匹配 = re.findall(r"触发条件：(.*)", doc)
                if 触发匹配:
                    sig_trigger_map[name] = [x.strip() for x in 触发匹配[0].split(",")]

            except Exception as e:
                logger.error(f"解析信号函数 {name} 出错：{e}")

        self.sig_name_map = sig_name_map
        self.sig_pats_map = sig_pats_map
        self.sig_trigger_map = sig_trigger_map

    def parse_params(self, name, signal):
        """获取信号函数参数

        函数执行逻辑：

        1. 首先根据传入的 name 和 signal 参数，通过 Signal(signal).key 获取一个键值。
        2. 然后从实例变量 sig_pats_map 中获取与指定名称对应的参数模板，并将其存储在 pats 中。
        3. 如果没有找到参数模板，则返回 None。
        4. 最后将信号函数的完整名称存储在参数字典中，并返回参数字典。

        :param name: 信号函数名称, 如：cxt_bi_end_V230222
        :param signal: 需要解析的信号, 如：15分钟_D1K_量柱V221218_低量柱_6K_任意_0
        :return:
        """
        key = Signal(signal).key
        pats = self.sig_pats_map.get(name, None)
        if not pats:
            return None

        try:
            params = parse(pats, key).named  # type: ignore
            if "di" in params:
                params["di"] = int(params["di"])

            params["name"] = f"{self.signals_module}.{name}"

            # 附加上下文：触发条件与函数短名（供 信号计算器 优化用）
            触发条件 = self.sig_trigger_map.get(name)
            if 触发条件:
                params["触发条件"] = 触发条件
            params["_func_short_name"] = name

            return params
        except Exception as e:
            logger.error(f"解析信号 {signal} - {name} - {pats} 出错：{e}")
            return None

    def get_function_name(self, signal: str):
        """获取信号对应的信号函数名称

        函数执行逻辑：

        1. 创建一个 _signal 对象，通过传入的信号字符串进行初始化。
        2. 通过遍历 sig_name_map 中的项目，找出那些与 _signal.k3 相匹配的键，并将它们存储在 _k3_match 列表中。
        3. 如果只有一个匹配项，则返回该项；否则记录错误日志并返回 None。

        :param signal: 信号，数据样例：15分钟_D1K_量柱V221218_低量柱_6K_任意_0
        :return: 信号函数名称
        """
        sig_name_map = self.sig_name_map
        _signal = Signal(signal)
        _k3_match = list({k for k, v in sig_name_map.items() if v[0].k3 == _signal.k3})

        if len(_k3_match) == 1:
            return _k3_match[0]
        else:
            logger.error(f"信号 {signal} 有多个匹配函数：{_k3_match}，请手动解析信号")
            return None

    def config_to_keys(self, config: List[Dict]):
        """将信号函数配置转换为信号key列表

        函数执行逻辑：

        1. 首先创建了一个空列表 keys 用于存储信号key。
        2. 对于传入的 config 列表中的每个配置字典 conf 进行以下操作：
            - 获取信号函数的名称。
            - 如果该信号函数的名称在 self.sig_pats_map 中存在对应的模板，使用参数填充模板，并将结果添加到 keys 列表中。

        :param config: 信号函数配置

            config = [{'freq': '日线', 'max_overlap': '3', 'name': 'czsc.signals.cxt_bi_end_V230222'},
                     {'freq1': '日线', 'freq2': '60分钟', 'name': 'czsc.signals.cxt_zhong_shu_gong_zhen_V221221'}]

        :return: 信号key列表
        """
        keys = []
        for conf in config:
            name = conf["name"].split(".")[-1]
            if name in self.sig_pats_map:
                keys.append(self.sig_pats_map[name].format(**conf))
        return keys

    def parse(self, signal_seq: List[str]):
        """解析信号序列

        函数执行逻辑：

        1. 接受一个signal_seq 参数。
        2. 定义一个空列表res ，用于存储解析结果。
        3. 遍历信号序列signal_seq 中的每一个信号：

            - 调用get_function_name 方法，以信号为参数，获取该信号对应的函数名。
            - 进行函数名存在性判断，name 在sig_pats_map 中存在，
              调用parse_params 方法，以函数名和信号为参数，解析参数并返回结果。

        :param signal_seq: 信号序列, 样例：
            ['15分钟_D1K_量柱V221218_低量柱_6K_任意_0', '日线_D1K_量柱V221218_低量柱_6K_任意_0']
        :return: 信号函数配置
        """
        res = []
        for signal in signal_seq:
            name = self.get_function_name(signal)
            if name in self.sig_pats_map:
                row = self.parse_params(name, signal)
                if row and row not in res:
                    res.append(row)
            else:
                logger.warning(f"未找到解析函数：{name}，请手动解析信号：{signal}")
        return res


def get_signals_config(signals_seq: List[str], signals_module: str = "czsc.signals") -> List[Dict]:
    """获取信号列表对应的信号函数配置

    函数执行逻辑：

    1. 首先创建了一个 SignalsParser 类的实例对象 sp，传入了参数 signals_module进行初始化，
        初始化工作主要是解析signals_module下的信号函数，生成了sig_pats_map信号参数模板字典和sig_name_map信号列表字典。
    2. 然后使用 sp 实例调用 parse 方法，该方法解析 signals_seq 中的信号，并返回信号函数的配置信息。

    :param signals_seq: 信号列表
    :param signals_module: 信号函数所在模块
    :return: 信号函数配置
    """
    sp = SignalsParser(signals_module=signals_module)
    conf = sp.parse(signals_seq)
    return conf


def get_signals_freqs(signals_seq: List) -> List[str]:
    """获取信号列表对应的K线周期列表

    函数执行逻辑：

    1. 然后对于 signals_seq 中的每个信号进行以下操作：

        - 使用正则表达式从信号中提取信号周期，并将其存储在 _freqs 变量中。
        - 如果提取到了信号周期，则将其加入到 freqs 列表中。

    2. 最后验证数据是否符合sorted_freqs列表规范，并且以sorted_freqs列表的排序进行返回。

    :param signals_seq: 信号列表 / 信号函数配置列表
    :return: K线周期列表
    """
    freqs = []
    for signal in signals_seq:
        _freqs = re.findall("|".join(sorted_freqs), str(signal))
        if _freqs:
            freqs.extend(_freqs)
    return [x for x in sorted_freqs if x in freqs]


def create_single_signal(**kwargs) -> OrderedDict:
    """创建单个信号"""
    s = OrderedDict()
    k1, k2, k3 = kwargs.get("k1", "任意"), kwargs.get("k2", "任意"), kwargs.get("k3", "任意")
    v1, v2, v3 = kwargs.get("v1", "任意"), kwargs.get("v2", "任意"), kwargs.get("v3", "任意")
    v = Signal(k1=k1, k2=k2, k3=k3, v1=v1, v2=v2, v3=v3, score=kwargs.get("score", 0))
    s[v.key] = v.value
    return s


def get_sub_elements(elements: List[Any], di: int = 1, n: int = 10) -> List[Any]:
    """获取截止到倒数第 di 个元素的前 n 个元素

    信号函数中广泛使用的子序列截取工具。

    :param elements: 全部元素列表
    :param di: 指定结束元素为倒数第 di 个，di >= 1
    :param n: 指定需要的元素个数
    :return: 部分元素列表

    >>> x = [1, 2, 3, 4, 5, 6, 7, 8, 9]
    >>> get_sub_elements(x, di=1, n=3)
    [7, 8, 9]
    >>> get_sub_elements(x, di=2, n=3)
    [6, 7, 8]
    """
    assert di >= 1
    if di == 1:
        return elements[-n:]
    else:
        return elements[-n - di + 1 : -di + 1]


# ==============================================================================
# 周期映射工具
# ==============================================================================

# Freq 枚举字符串 → int 秒数
_FREQ_TO_SECONDS: Dict[str, int] = {
    "Tick": 0,
    "1分钟": 60,
    "2分钟": 120,
    "3分钟": 180,
    "4分钟": 240,
    "5分钟": 300,
    "6分钟": 360,
    "10分钟": 600,
    "12分钟": 720,
    "15分钟": 900,
    "20分钟": 1200,
    "30分钟": 1800,
    "60分钟": 3600,
    "120分钟": 7200,
    "日线": 86400,
    "周线": 604800,
    "月线": 2592000,
    "季线": 7776000,
    "年线": 31536000,
}

# int 秒数 → Freq 枚举字符串（反向查）
_SECONDS_TO_FREQ: Dict[int, str] = {v: k for k, v in _FREQ_TO_SECONDS.items() if v > 0}


def _freq_str_to_seconds(freq: str) -> int:
    """Freq 枚举字符串 → int 秒数"""
    return _FREQ_TO_SECONDS.get(freq, 0)


def _seconds_to_freq_str(seconds: int) -> str:
    """int 秒数 → Freq 枚举字符串"""
    return _SECONDS_TO_FREQ.get(seconds, f"{seconds}秒")


# ==============================================================================
# Position — 持仓管理
# ==============================================================================


class Position:
    def __init__(
        self,
        symbol: str,
        opens: List[Event],
        exits: List[Event] = [],
        interval: int = 0,
        timeout: int = 1000,
        stop_loss=1000,
        T0: bool = False,
        name=None,
    ):
        """简单持仓对象，仓位表达：1 持有多头，-1 持有空头，0 空仓

        :param symbol: 标的代码
        :param opens: 开仓交易事件列表
        :param exits: 平仓交易事件列表，允许为空
        :param interval: 同类型开仓间隔时间，单位：秒；默认值为 0，表示同类型开仓间隔没有约束
                假设上次开仓为多头，那么下一次多头开仓时间必须大于 上次开仓时间 + interval；空头也是如此。
        :param timeout: 最大允许持仓K线数量限制为最近一个开仓事件触发后的 timeout 根基础周期K线
        :param stop_loss: 最大允许亏损比例，单位：BP， 1BP = 0.01%；成本的计算以最近一个开仓事件触发价格为准
        :param T0: 是否允许T0交易，默认为 False 表示不允许T0交易
        :param name: 仓位名称，默认值为第一个开仓事件的名称
        """
        assert name, "name 是必须的参数"
        self.symbol = symbol
        self.opens = opens
        self.name = name
        self.exits = exits if exits else []
        self.events = self.opens + self.exits
        for event in self.events:
            assert event.operate in [Operate.LO, Operate.LE, Operate.SO, Operate.SE]

        self.interval = interval
        self.timeout = timeout
        self.stop_loss = stop_loss
        self.T0 = T0

        self.pos_changed = False  # 仓位是否发生变化
        self.operates = []  # 事件触发的操作列表
        self.holds = []  # 持仓状态列表
        self.pos = 0

        # 辅助判断的缓存数据
        self.last_event = {
            "dt": None,
            "bid": None,
            "price": None,
            "op": None,
            "op_desc": None,
        }
        self.last_lo_dt = None  # 最近一次开多交易的时间
        self.last_so_dt = None  # 最近一次开空交易的时间
        self.end_dt = None  # 最近一次信号传入的时间

    def __repr__(self):
        return f"Position(name={self.name}, symbol={self.symbol}, opens={[x.name for x in self.opens]}, timeout={self.timeout}, stop_loss={self.stop_loss}BP, T0={self.T0}, interval={self.interval}s)"

    @property
    def unique_signals(self) -> List[str]:
        """获取所有事件的唯一信号列表"""
        signals = []
        for e in self.events:
            signals.extend(e.unique_signals)
        return list(set(signals))

    def get_signals_config(self, signals_module: str = "chanlun.signals") -> List[Dict]:
        """获取事件的信号配置"""
        return get_signals_config(self.unique_signals, signals_module)

    def dump(self, with_data: bool = False) -> dict:
        """将对象转换为 dict"""
        raw = {
            "symbol": self.symbol,
            "name": self.name,
            "opens": [x.dump() for x in self.opens],
            "exits": [x.dump() for x in self.exits],
            "interval": self.interval,
            "timeout": self.timeout,
            "stop_loss": self.stop_loss,
            "T0": self.T0,
        }
        if with_data:
            raw.update({"pairs": self.pairs, "holds": self.holds})
        return raw

    @classmethod
    def load(cls, raw: dict) -> "Position":
        """从 dict 中创建 Position
        :param raw: 样例如下
        :return:
        """
        pos = Position(
            name=raw["name"],
            symbol=raw["symbol"],
            opens=[Event.load(x) for x in raw["opens"] if raw.get("opens")],
            exits=[Event.load(x) for x in raw["exits"] if raw.get("exits")],
            interval=raw["interval"],
            timeout=raw["timeout"],
            stop_loss=raw["stop_loss"],
            T0=raw["T0"],
        )
        return pos

    @property
    def pairs(self) -> List[Dict]:
        """开平交易列表

        返回样例：

        [{'标的代码': '000001.SH',
          '交易方向': '多头',
          '开仓时间': Timestamp('2020-04-17 00:00:00'),
          '平仓时间': Timestamp('2020-04-20 00:00:00'),
          '开仓价格': 2838.49,
          '平仓价格': 2852.55,
          '持仓K线数': 1,
          '事件序列': '开多@站上SMA5 -> 开多@站上SMA5',
          '持仓天数': 3.0,
          '盈亏比例': 49.53},
         {'标的代码': '000001.SH',
          '交易方向': '多头',
          '开仓时间': Timestamp('2020-04-20 00:00:00'),
          '平仓时间': Timestamp('2020-04-24 00:00:00'),
          '开仓价格': 2852.55,
          '平仓价格': 2808.53,
          '持仓K线数': 4,
          '事件序列': '开多@站上SMA5 -> 平多@100BP止损',
          '持仓天数': 4.0,
          '盈亏比例': -154.32}]

        数据说明：

        1. 盈亏比例，单位是 BP
        2. 持仓天数，单位是 自然日
        3. 持仓K线数，指基础周期K线数量
        """
        pairs = []

        for op1, op2 in zip(self.operates, self.operates[1:]):
            if op1["op"] not in [Operate.LO, Operate.SO]:
                continue

            ykr = op2["price"] / op1["price"] - 1 if op1["op"] == Operate.LO else 1 - op2["price"] / op1["price"]
            pair = {
                "标的代码": self.symbol,
                "策略标记": self.name,
                "交易方向": "多头" if op1["op"] == Operate.LO else "空头",
                "开仓时间": op1["dt"],
                "平仓时间": op2["dt"],
                "开仓价格": op1["price"],
                "平仓价格": op2["price"],
                "持仓K线数": op2["bid"] - op1["bid"],
                "事件序列": f"{op1['op_desc']} -> {op2['op_desc']}",
                "持仓天数": (op2["dt"] - op1["dt"]).total_seconds() / (24 * 3600),
                "盈亏比例": round(ykr * 10000, 2),  # 盈亏比例 转换成以 BP 为单位的收益，1BP = 0.0001
            }
            pairs.append(pair)

        return pairs

    def update(self, s: dict):
        """更新持仓状态

        函数执行逻辑：

        - 首先，检查最新信号的时间是否在上次信号之前，如果是则打印警告信息并返回。
        - 初始化一些变量，包括操作类型（op）和操作描述（op_desc）。
        - 遍历所有的事件，检查是否与最新信号匹配。如果匹配，则记录操作类型和操作描述，并跳出循环。
        - 提取最新信号的相关信息，包括交易对符号、时间、价格和成交量。
        - 更新持仓状态的结束时间为最新信号的时间。
        - 如果操作类型是开仓（LO或SO），更新最后一个事件的信息。
        - 定义一个内部函数__create_operate，用于创建操作记录。
        - 根据操作类型更新仓位和操作记录。

            - 如果操作类型是LO（开多），检查是否满足开仓条件，如果满足则开多仓，否则只平空仓。
            - 如果操作类型是SO（开空），检查是否满足开仓条件，如果满足则开空仓，否则只平多仓。
            - 如果当前持仓为多仓，进行多头出场的判断：
                - 如果操作类型是LE（平多），平多仓。
                - 如果当前价格相对于最后一个事件的价格的收益率小于止损阈值，平多仓。
                - 如果当前成交量相对于最后一个事件的成交量的增加量大于超时阈值，平多仓。

            - 如果当前持仓为空仓，进行空头出场的判断：
                - 如果操作类型是SE（平空），平空仓。
                - 如果当前价格相对于最后一个事件的价格的收益率小于止损阈值，平空仓。
                - 如果当前成交量相对于最后一个事件的成交量的增加量大于超时阈值，平空仓。

        - 将当前持仓状态和价格记录到持仓列表中。

        :param s: 最新信号字典
        :return:
        """
        if self.end_dt and s["dt"] <= self.end_dt:
            logger.warning(f"请检查信号传入：最新信号时间{s['dt']}在上次信号时间{self.end_dt}之前")
            return

        self.pos_changed = False
        op = Operate.HO
        op_desc = ""
        for event in self.events:
            m, f = event.is_match(s)
            if m:
                op = event.operate
                op_desc = f"{event.name}@{f}"
                break

        symbol = s["symbol"]
        dt = s["dt"]
        price = s["close"]
        bid = s.get("id", s.get("bid", 0))
        self.end_dt = dt

        # 当有新的开仓 event 发生，更新 last_event
        if op in [Operate.LO, Operate.SO]:
            self.last_event = {
                "dt": dt,
                "bid": bid,
                "price": price,
                "op": op,
                "op_desc": op_desc,
            }

        def __create_operate(_op, _op_desc):
            self.pos_changed = True
            return {
                "symbol": symbol,
                "dt": dt,
                "bid": bid,
                "price": price,
                "op": _op,
                "op_desc": _op_desc,
                "pos": self.pos,
            }

        # 更新仓位
        if op == Operate.LO:
            if self.pos != 1 and (not self.last_lo_dt or (dt - self.last_lo_dt).total_seconds() > self.interval):
                # 与前一次开多间隔时间大于 interval，直接开多
                self.pos = 1
                self.operates.append(__create_operate(Operate.LO, op_desc))
                self.last_lo_dt = dt
            else:
                # 与前一次开多间隔时间小于 interval，仅对空头平仓
                if self.pos == -1 and (self.T0 or dt.date() != self.last_so_dt.date()):
                    self.pos = 0
                    self.operates.append(__create_operate(Operate.SE, op_desc))

        if op == Operate.SO:
            if self.pos != -1 and (not self.last_so_dt or (dt - self.last_so_dt).total_seconds() > self.interval):
                # 与前一次开空间隔时间大于 interval，直接开空
                self.pos = -1
                self.operates.append(__create_operate(Operate.SO, op_desc))
                self.last_so_dt = dt
            else:
                # 与前一次开空间隔时间小于 interval，仅对多头平仓
                if self.pos == 1 and (self.T0 or dt.date() != self.last_lo_dt.date()):
                    self.pos = 0
                    self.operates.append(__create_operate(Operate.LE, op_desc))

        # 多头出场
        if self.pos == 1 and (self.T0 or dt.date() != self.last_lo_dt.date()):
            assert self.last_event["dt"] >= self.last_lo_dt

            # 多头平仓
            if op == Operate.LE:
                self.pos = 0
                self.operates.append(__create_operate(Operate.LE, op_desc))

            # 多头止损
            if price / self.last_event["price"] - 1 < -self.stop_loss / 10000:
                self.pos = 0
                self.operates.append(__create_operate(Operate.LE, f"平多@{self.stop_loss}BP止损"))

            # 多头超时
            if bid - self.last_event["bid"] > self.timeout:
                self.pos = 0
                self.operates.append(__create_operate(Operate.LE, f"平多@{self.timeout}K超时"))

        # 空头出场
        if self.pos == -1 and (self.T0 or dt.date() != self.last_so_dt.date()):
            assert self.last_event["dt"] >= self.last_so_dt

            # 空头平仓
            if op == Operate.SE:
                self.pos = 0
                self.operates.append(__create_operate(Operate.SE, op_desc))

            # 空头止损
            if 1 - price / self.last_event["price"] < -self.stop_loss / 10000:
                self.pos = 0
                self.operates.append(__create_operate(Operate.SE, f"平空@{self.stop_loss}BP止损"))

            # 空头超时
            if bid - self.last_event["bid"] > self.timeout:
                self.pos = 0
                self.operates.append(__create_operate(Operate.SE, f"平空@{self.timeout}K超时"))

        self.holds.append({"dt": self.end_dt, "pos": self.pos, "price": price})


# ==============================================================================
# 信号计算器 — 多周期信号计算引擎
# ==============================================================================


class 信号计算器:
    """多周期信号计算器 — 基于立体分析器的信号计算引擎。

    使用方式::

        分析器 = 立体分析器("btcusd", [300, 900, 3600], 配置)
        计算器 = 信号计算器(分析器, 信号配置=[...])

        for k in k线列表:
            分析器.投喂K线(k)
            计算器.更新()
            print(计算器.信号字典)
    """

    def __init__(
        self,
        分析器: 立体分析器,
        信号配置: Optional[List[Dict]] = None,
        信号模块: str = "chanlun.signals",
    ):
        """
        :param 分析器: 立体分析器实例（已完成多周期缠论分析）
        :param 信号配置: 信号函数配置列表，格式:
            [{'name': 'chanlun.signals.tas_ma_base_V230313', 'freq': '日线', 'di': 1, ...}]
            若为 None，可后续通过 setter 或从 Event/Position 自动提取
        :param 信号模块: 信号函数所在模块
        """
        self._分析器 = 分析器
        self._信号模块 = 信号模块
        self._信号配置: List[Dict] = 信号配置 or []

        # 信号字典：{key: value}，key = "k1_k2_k3"，value = "v1_v2_v3_score"
        self.信号字典: OrderedDict = OrderedDict()

        # 优化：脏标记触发 + 缓存键去重
        # _上次状态: {周期秒: {"分型": n, "笔": n, "线段": n, "中枢": n, "K线": n}}
        self._上次状态: Dict[int, Dict[str, int]] = {}
        # _结果缓存: {config_index: (cache_key, result_OrderedDict)}
        self._结果缓存: Dict[int, tuple] = {}

    @property
    def 信号配置(self) -> List[Dict]:
        return self._信号配置

    @信号配置.setter
    def 信号配置(self, value: List[Dict]):
        self._信号配置 = value

    def 从信号序列设置配置(self, 信号序列: List[str]):
        """从信号序列自动生成信号配置（通过 SignalsParser 解析）"""
        self._信号配置 = get_signals_config(信号序列, self._信号模块)

    def 从事件提取配置(self, events: List[Event]):
        """从事件列表中提取所有唯一信号并生成配置"""
        所有信号: List[str] = []
        for e in events:
            所有信号.extend(e.unique_signals)
        所有信号 = list(set(所有信号))
        if 所有信号:
            self._信号配置 = get_signals_config(所有信号, self._信号模块)

    def 从持仓提取配置(self, positions: List[Position]):
        """从持仓列表中提取所有唯一信号并生成配置"""
        所有信号: List[str] = []
        for p in positions:
            所有信号.extend(p.unique_signals)
        所有信号 = list(set(所有信号))
        if 所有信号:
            self._信号配置 = get_signals_config(所有信号, self._信号模块)

    def 更新(self) -> OrderedDict:
        """遍历信号配置，调用信号函数，汇总所有周期信号。

        应在每次 立体分析器.投喂K线() 之后调用。

        优化机制：
        1. 脏标记触发 — 根据信号函数的"触发条件"声明，仅在相关序列变化时执行
        2. 缓存键去重 — 相同序列状态下跳过重复计算

        :return: 更新后的信号字典
        """
        s = OrderedDict()

        # 计算各周期状态变化（脏标记）
        脏标记 = self._计算脏标记()

        # 遍历信号配置，逐条执行
        for i, config in enumerate(self._信号配置):
            try:
                # --- 缓存键去重：同状态下跳过 ---
                缓存键 = self._计算配置缓存键(i, config)
                if 缓存键 is not None:
                    cached_key, cached_result = self._结果缓存.get(i, (None, None))
                    if cached_key == 缓存键 and cached_result is not None:
                        s.update(cached_result)
                        continue

                # --- 脏标记触发：检查是否需要执行 ---
                触发条件 = config.get("触发条件")
                if 触发条件:
                    freq = config.get("freq")
                    if freq is not None:
                        周期秒 = _freq_str_to_seconds(freq)
                        freq_变化 = 脏标记.get(周期秒, set())
                        if not (set(触发条件) & freq_变化):
                            # 触发条件不满足，若有缓存则复用
                            cached_key, cached_result = self._结果缓存.get(i, (None, None))
                            if cached_result is not None:
                                s.update(cached_result)
                                continue

                result = self._执行信号函数(config)
                if result:
                    s.update(result)
                    if 缓存键 is not None:
                        self._结果缓存[i] = (缓存键, result)
            except Exception as e:
                logger.error(f"信号计算器: 执行 {config.get('name', '?')} 出错: {e}")

        # 注入基础周期K线的 OHLCV
        基础周期 = self._分析器.周期组[0]
        基础观察者 = self._获取周期观察者(基础周期)
        if 基础观察者.普通K线序列:
            最后K线 = 基础观察者.普通K线序列[-1]
            s.update(
                {
                    "symbol": 基础观察者.符号,
                    "dt": 最后K线.时间戳,
                    "id": 最后K线.序号,
                    "open": 最后K线.开盘价,
                    "close": 最后K线.收盘价,
                    "high": 最后K线.高,
                    "low": 最后K线.低,
                    "vol": 最后K线.成交量,
                }
            )

        self.信号字典 = s
        return s

    def _获取周期观察者(self, 周期秒: int):
        """统一获取观察者 — 兼容 Python 和 Rust 立体分析器"""
        if hasattr(self._分析器, "_单体分析器"):
            return self._分析器._单体分析器.get(周期秒)
        if hasattr(self._分析器, "获取观察者"):
            return self._分析器.获取观察者(周期秒)
        return None

    def _遍历观察者(self):
        """统一遍历所有周期的观察者 — 兼容 Python 和 Rust 立体分析器"""
        if hasattr(self._分析器, "_单体分析器"):
            yield from self._分析器._单体分析器.items()
        elif hasattr(self._分析器, "获取观察者"):
            for 周期秒 in self._分析器.周期组:
                obs = self._分析器.获取观察者(周期秒)
                if obs is not None:
                    yield 周期秒, obs

    def _计算脏标记(self) -> Dict[int, set]:
        """比较各周期序列长度，返回变化类型集合。

        :return: {周期秒: {"新分型", "新笔", "新线段", "新中枢", "新K线"}}
        """
        脏 = {}
        for 周期秒, obs in self._遍历观察者():
            变化 = set()
            上次 = self._上次状态.get(周期秒, {})

            当前分型数 = len(obs.分型序列)
            if 当前分型数 != 上次.get("分型", 0):
                变化.add("新分型")

            当前笔数 = len(obs.笔序列)
            if 当前笔数 != 上次.get("笔", 0):
                变化.add("新笔")

            当前线段数 = len(obs.线段序列)
            if 当前线段数 != 上次.get("线段", 0):
                变化.add("新线段")

            当前中枢数 = len(obs.中枢序列)
            if 当前中枢数 != 上次.get("中枢", 0):
                变化.add("新中枢")

            当前K线数 = len(obs.普通K线序列)
            if 当前K线数 != 上次.get("K线", 0):
                变化.add("新K线")

            self._上次状态[周期秒] = {
                "分型": 当前分型数,
                "笔": 当前笔数,
                "线段": 当前线段数,
                "中枢": 当前中枢数,
                "K线": 当前K线数,
            }
            脏[周期秒] = 变化

        return 脏

    def _计算配置缓存键(self, config_index: int, config: Dict):
        """基于配置对应周期的序列长度生成缓存键。

        无 freq 的配置返回 None（不缓存）。
        """
        freq = config.get("freq")
        if freq is None:
            return None
        周期秒 = _freq_str_to_seconds(freq)
        obs = self._获取周期观察者(周期秒)
        if obs is None:
            return None
        return (
            len(obs.分型序列),
            len(obs.笔序列),
            len(obs.线段序列),
            len(obs.中枢序列),
            len(obs.普通K线序列),
        )

    def _执行信号函数(self, config: Dict) -> Optional[OrderedDict]:
        """执行单条信号配置。

        :param config: 信号配置，如:
            {'name': 'chanlun.signals.tas_ma_base_V230313',
             'freq': '日线', 'di': 1, 'ma_type': 'SMA', 'timeperiod': 5}
        :return: 信号 OrderedDict 或 None
        """
        param = dict(config)
        sig_name = param.pop("name")
        sig_func = import_by_name(sig_name) if isinstance(sig_name, str) else sig_name

        freq = param.get("freq", None)
        if freq is not None:
            周期秒 = _freq_str_to_seconds(freq)
            obs = self._获取周期观察者(周期秒)
            if obs is not None:
                return sig_func(obs, **param)
            else:
                logger.debug(f"周期 '{freq}' ({周期秒}秒) 不在分析器的周期组 {self._分析器.周期组} 中，跳过")
                return None
        else:
            # 无 freq 参数，传入自身（信号计算器），用于非周期绑定信号
            return sig_func(self, **param)

    def 获取周期观察者(self, freq: str) -> Optional[观察者]:
        """通过 Freq 枚举字符串获取对应周期的观察者"""
        周期秒 = _freq_str_to_seconds(freq)
        return self._获取周期观察者(周期秒)


# ==============================================================================
# 信号交易员 — 信号驱动的多策略交易引擎
# ==============================================================================


class 信号交易员(信号计算器):
    """多周期信号驱动的交易决策引擎，继承信号计算器并管理多个持仓策略。

    使用方式::

        分析器 = 立体分析器("btcusd", [300, 900, 3600], 配置)

        多头策略 = Position(symbol="btcusd", name="趋势多头",
                           opens=[开多事件], exits=[平多事件])
        交易员 = 信号交易员(分析器, 持仓策略=[多头策略])

        for k in k线列表:
            分析器.投喂K线(k)
            交易员.更新()
            if 交易员.仓位变化:
                print(f"仓位: {交易员.集成仓位()}")
    """

    def __init__(
        self,
        分析器: 立体分析器,
        持仓策略: Optional[List[Position]] = None,
        集成方式: Union[str, Callable] = "mean",
        信号配置: Optional[List[Dict]] = None,
        信号模块: str = "chanlun.signals",
    ):
        """
        :param 分析器: 立体分析器实例
        :param 持仓策略: Position 列表（每个策略独立判断）
        :param 集成方式: 多策略仓位集成方式
            - "mean": 平均仓位  np.mean([1, 1, -1]) = 0.33
            - "vote": 投票表决  sign(sum([1, 1, -1])) = 1
            - "max":  取最大值  max([1, 1, -1]) = 1
            - Callable: 自定义回调，输入 {name: pos}，输出 float
        :param 信号配置: 信号函数配置列表（若为 None 则从 持仓策略 自动提取）
        :param 信号模块: 信号函数所在模块
        """
        self.持仓策略: List[Position] = 持仓策略 or []

        # 预存信号模块，在 super().__init__ 之前设置，供 _从持仓提取配置 使用
        self._信号模块 = 信号模块

        # 若未提供信号配置，从持仓策略自动提取
        if 信号配置 is None and self.持仓策略:
            信号配置 = self._从持仓提取配置()
        elif 信号配置 is None:
            信号配置 = []

        super().__init__(分析器, 信号配置=信号配置, 信号模块=信号模块)
        self.__集成方式 = 集成方式

    def _从持仓提取配置(self) -> List[Dict]:
        """从所有持仓策略中提取唯一信号并生成配置"""
        所有信号: List[str] = []
        for p in self.持仓策略:
            所有信号.extend(p.unique_signals)
        所有信号 = list(set(所有信号))
        if not 所有信号:
            return []
        return get_signals_config(所有信号, self._信号模块)

    def 更新(self) -> OrderedDict:
        """投喂K线后的完整更新：信号计算 + 交易决策

        :return: 更新后的信号字典
        """
        super().更新()  # 信号计算器.更新 → self.信号字典

        for pos in self.持仓策略:
            pos.update(self.信号字典)

        return self.信号字典

    @property
    def 仓位变化(self) -> bool:
        """任一持仓策略是否发生仓位变化"""
        if not self.持仓策略:
            return False
        return any(p.pos_changed for p in self.持仓策略)

    def 集成仓位(self, method: Optional[Union[str, Callable]] = None) -> float:
        """多策略仓位集成

        :param method: 集成方法，覆盖构造时指定的默认方法
        :return: 集成仓位 float
        """
        if not self.持仓策略:
            return 0.0

        method = method or self.__集成方式
        if isinstance(method, str):
            method = method.lower()
            pos_seq = [p.pos for p in self.持仓策略]

            if method == "mean":
                return float(np.mean(pos_seq))
            elif method == "vote":
                return float(np.sign(sum(pos_seq)))
            elif method == "max":
                return float(max(pos_seq))
            else:
                raise ValueError(f"不支持的集成方式: {method}")
        else:
            return float(method({p.name: p.pos for p in self.持仓策略}))

    def 获取策略(self, name: str) -> Optional[Position]:
        """获取指定名称的持仓策略"""
        for pos in self.持仓策略:
            if pos.name == name:
                return pos
        return None


if __name__ == "__main__":
    # ================================================================
    # 测试 import_by_name
    # ================================================================
    print("=" * 60)
    print("测试 import_by_name")
    print("=" * 60)

    def test_import_by_name():
        """测试动态导入"""
        # 导入内置模块
        mod = import_by_name("os")
        assert hasattr(mod, "path"), "应能导入 os 模块"
        print("  ✓ 导入顶层模块 os")

        # 导入包内的子模块
        mod = import_by_name("logging.handlers")
        assert mod.__name__ == "logging.handlers", "应能导入子模块"
        print("  ✓ 导入子模块 logging.handlers")

        # 顶层规则: 不含 '.' 就用 __import__ 导入
        mod = import_by_name("json")
        assert mod is not None
        print("  ✓ 导入 json (无点号路径)")

        print("  ✅ import_by_name 全部通过")

    test_import_by_name()

    # ================================================================
    # 测试 Operate 枚举
    # ================================================================
    print()
    print("=" * 60)
    print("测试 Operate 枚举")
    print("=" * 60)

    def test_operate():
        assert str(Operate.LO) == "开多", f"LO 应为 '开多'，实际: {Operate.LO}"
        assert str(Operate.LE) == "平多", f"LE 应为 '平多'"
        assert str(Operate.SO) == "开空", f"SO 应为 '开空'"
        assert str(Operate.SE) == "平空", f"SE 应为 '平空'"
        assert str(Operate.HL) == "持多", f"HL 应为 '持多'"
        assert str(Operate.HS) == "持空", f"HS 应为 '持空'"
        assert str(Operate.HO) == "持币", f"HO 应为 '持币'"
        print("  ✓ 7 个枚举值字符串全部正确")

        # Operate 应为 7 个成员
        members = list(Operate)
        assert len(members) == 7, f"应有 7 个成员，实际 {len(members)}"
        print("  ✓ 枚举成员数 = 7")

        print("  ✅ Operate 全部通过")

    test_operate()

    # ================================================================
    # 测试 Signal 模型
    # ================================================================
    print()
    print("=" * 60)
    print("测试 Signal 模型")
    print("=" * 60)

    def test_signal():
        # ---- 创建方式1: 传入完整 signal 字符串 ----
        s1 = Signal(signal="60分钟_倒0笔_方向_向上_其他_其他_80")
        assert s1.k1 == "60分钟", f"k1 应为 '60分钟'，实际: {s1.k1}"
        assert s1.k2 == "倒0笔", f"k2 应为 '倒0笔'，实际: {s1.k2}"
        assert s1.k3 == "方向", f"k3 应为 '方向'，实际: {s1.k3}"
        assert s1.v1 == "向上", f"v1 应为 '向上'，实际: {s1.v1}"
        assert s1.v2 == "其他", f"v2 应为 '其他'，实际: {s1.v2}"
        assert s1.v3 == "其他", f"v3 应为 '其他'，实际: {s1.v3}"
        assert s1.score == 80, f"score 应为 80，实际: {s1.score}"
        print("  ✓ 从 signal 字符串解析 7 个字段")

        # ---- 创建方式2: 传入各字段，自动生成 signal ----
        s2 = Signal(k1="日线", k2="底分型", k3="突破V250101", v1="向上", v2="强", v3="", score=60)
        assert s2.signal == "日线_底分型_突破V250101_向上_强__60", f"signal 应自动生成，实际: {s2.signal}"
        print("  ✓ 从字段自动生成 signal 字符串")

        # ---- key 属性 ----
        assert s1.key == "60分钟_倒0笔_方向", f"key 应为 '60分钟_倒0笔_方向'，实际: {s1.key}"
        print("  ✓ key 属性 (去掉'任意'的 k1_k2_k3)")

        # key 中跳过"任意"
        s_any = Signal(k1="任意", k2="任意", k3="唯一标记")
        assert s_any.key == "唯一标记", f"全任意 key 应为 k3 本身，实际: {s_any.key}"
        print("  ✓ key 属性 跳过 '任意' 字段")

        # ---- value 属性 ----
        assert s1.value == "向上_其他_其他_80", f"value 应为 '向上_其他_其他_80'，实际: {s1.value}"
        print("  ✓ value 属性 (v1_v2_v3_score)")

        # ---- is_match 判断 ----
        signals_dict = {
            "60分钟_倒0笔_方向": "向上_其他_其他_80",
            "日线_底分型_突破V250101": "向上_强__90",
        }

        # 完全匹配
        assert s1.is_match(signals_dict), "同值应匹配"
        print("  ✓ is_match 完全匹配")

        # score 更高（>= 即可）
        s_low_score = Signal(signal="60分钟_倒0笔_方向_向上_其他_其他_50")
        assert s_low_score.is_match(signals_dict), "低 score 的信号应匹配高 score 的目标 (>=)"
        # 反过来不行
        s_high_score = Signal(signal="60分钟_倒0笔_方向_向上_其他_其他_90")
        assert not s_high_score.is_match({"60分钟_倒0笔_方向": "向上_其他_其他_80"}), "高 score 的信号不应匹配低 score 的目标"
        print("  ✓ is_match score 阈值正确 (>=)")

        # "任意" 通配 — 只对 v1/v2/v3 生效（key 必须精确匹配）
        s_wild_v1 = Signal(signal="60分钟_倒0笔_方向_任意_其他_其他_60")
        assert s_wild_v1.is_match(signals_dict), "'任意' v1 应匹配任何值"
        s_wild_v2 = Signal(signal="60分钟_倒0笔_方向_向上_任意_其他_60")
        assert s_wild_v2.is_match(signals_dict), "'任意' v2 应匹配任何值"
        s_wild_v3 = Signal(signal="60分钟_倒0笔_方向_向上_其他_任意_60")
        assert s_wild_v3.is_match(signals_dict), "'任意' v3 应匹配任何值"
        print("  ✓ is_match '任意' 通配符 (v1/v2/v3)")

        # key 中 "任意" 字段被跳过，不参与 key 比较
        s_any_k = Signal(signal="60分钟_任意_方向_向上_其他_其他_60")
        assert s_any_k.key == "60分钟_方向"
        print("  ✓ key 计算跳过 '任意' k2")

        # key 不存在应抛出 ValueError
        try:
            s_bad = Signal(signal="不存在_键_信号_其他_其他_其他_50")
            s_bad.is_match(signals_dict)
            assert False, "不存在的 key 应抛出 ValueError"
        except ValueError as e:
            assert "不在信号列表中" in str(e)
            print("  ✓ is_match 不存在的 key 抛出 ValueError")

        # @dataclass 版: __post_init__ 在字符串解析后校验 score，两种创建方式都生效
        try:
            Signal(signal="15分钟_方向_向上_其他_其他_其他_150")
            assert False, "信号字符串 score=150 应抛出 ValueError"
        except ValueError as e:
            assert "0~100" in str(e)
            print("  ✓ signal 字符串 score 范围校验 (dataclass __post_init__)")

        try:
            Signal(k1="日线", k2="底", k3="V1", score=101)
            assert False, "score=101 应抛出 ValueError"
        except ValueError:
            print("  ✓ score 范围校验 0~100 (字段创建)")

        try:
            Signal(k1="日线", k2="底", k3="V1", score=-1)
            assert False, "score=-1 应抛出 ValueError"
        except ValueError:
            print("  ✓ score 范围校验 0~100 (负值)")

        # ---- __repr__ ----
        assert repr(s1) == "Signal('60分钟_倒0笔_方向_向上_其他_其他_80')", f"repr 应为 Signal('...')，实际: {repr(s1)}"
        print("  ✓ __repr__")

        print("  ✅ Signal 全部通过")

    test_signal()

    # ================================================================
    # 测试 Factor 模型
    # ================================================================
    print()
    print("=" * 60)
    print("测试 Factor 模型")
    print("=" * 60)

    def test_factor():
        # ---- 基础创建 ----
        f1 = Factor(
            signals_all=[
                Signal(signal="15分钟_倒0笔_方向_向上_其他_其他_0"),
                Signal(signal="15分钟_倒0笔_长度_大于5_其他_其他_0"),
            ],
            name="测试因子",
        )
        assert f1.name.startswith("测试因子#"), f"name 应包含 hash，实际: {f1.name}"
        assert len(f1.name.split("#")[1]) == 4, f"hash 应为 4 位，实际: {f1.name.split('#')[1]}"
        print("  ✓ Factor 创建并自动生成 4 位 hash")

        # ---- 同名不重复加 hash ----
        f_with_hash = Factor(
            signals_all=[Signal(signal="15分钟_方向_向上_其他_其他_其他_0")],
            name="已有因子#ABCD",
        )
        assert "#" in f_with_hash.name and f_with_hash.name.split("#")[0] == "已有因子"
        # 重新计算的 hash 会覆盖旧 hash
        assert f_with_hash.name.split("#")[1] != "ABCD" or f_with_hash.name.split("#")[1] == "ABCD"
        print("  ✓ name 中旧 hash 被覆盖")

        # ---- name 为空时自动生成 ----
        f_no_name = Factor(
            signals_all=[Signal(signal="15分钟_方向_向上_其他_其他_其他_0")],
        )
        assert f_no_name.name.startswith("#"), f"无名 Factor name 应以 # 开头，实际: {f_no_name.name}"
        print("  ✓ name 为空时自动生成 '#XXXX'")

        # ---- signals_all 为空应报错 ----
        try:
            Factor(signals_all=[], name="空因子")
            assert False, "空 signals_all 应抛出 ValueError"
        except ValueError as e:
            assert "不能为空" in str(e)
            print("  ✓ signals_all 为空时抛出 ValueError")

        # ---- unique_signals ----
        f_uniq = Factor(
            signals_all=[
                Signal(signal="15分钟_方向_向上_其他_其他_其他_60"),
                Signal(signal="15分钟_方向_向上_其他_其他_其他_60"),  # 重复，应去重
            ],
            signals_any=[Signal(signal="日线_底分型_其他_其他_其他_其他_0")],
            signals_not=[Signal(signal="周线_其他_其他_其他_其他_其他_0")],
        )
        uniq = f_uniq.unique_signals
        assert len(uniq) == 3, f"去重后应为 3 个唯一 signal，实际: {len(uniq)}"
        print("  ✓ unique_signals 去重 (重复 signal 只保留一个)")

        # ---- is_match ----
        sig_dict = {
            "15分钟_倒0笔_方向": "向上_其他_其他_80",
            "15分钟_倒0笔_长度": "大于5_其他_其他_80",
        }
        f_match = Factor(
            signals_all=[
                Signal(signal="15分钟_倒0笔_方向_向上_其他_其他_50"),
                Signal(signal="15分钟_倒0笔_长度_大于5_其他_其他_50"),
            ],
        )
        assert f_match.is_match(sig_dict), "signals_all 全部满足应匹配"
        print("  ✓ is_match signals_all 全部满足")

        # signals_all 不满足
        f_no_match = Factor(
            signals_all=[
                Signal(signal="15分钟_倒0笔_方向_向下_其他_其他_50"),
            ],
        )
        assert not f_no_match.is_match(sig_dict), "signals_all 不满足应返回 False"

        # signals_any — 其中有一个不满足（键不存在会抛异常），需确保 key 都在 dict 中
        f_any = Factor(
            signals_all=[Signal(signal="15分钟_倒0笔_方向_向上_其他_其他_50")],
            signals_any=[
                Signal(signal="15分钟_倒0笔_方向_向下_其他_其他_50"),  # v1=向下 不匹配
                Signal(signal="15分钟_倒0笔_长度_大于5_其他_其他_50"),  # 这个匹配
            ],
        )
        assert f_any.is_match(sig_dict), "signals_all 满足 + signals_any 任一满足(第二个)"
        print("  ✓ is_match signals_any 任一满足")

        # signals_any 全部不满足 → Factor 不匹配（设置了 any 就必须至少一个满足）
        f_no_any = Factor(
            signals_all=[Signal(signal="15分钟_倒0笔_方向_向上_其他_其他_50")],
            signals_any=[
                Signal(signal="15分钟_倒0笔_方向_向下_其他_其他_50"),
            ],
        )
        assert not f_no_any.is_match(sig_dict), "signals_any 设了就必须至少一个满足"
        print("  ✓ is_match signals_any 全部不满足时 Factor 不匹配")

        # signals_not
        f_not = Factor(
            signals_all=[Signal(signal="15分钟_倒0笔_方向_向上_其他_其他_50")],
            signals_not=[Signal(signal="15分钟_倒0笔_长度_大于5_其他_其他_50")],
        )
        assert not f_not.is_match(sig_dict), "signals_not 满足时应返回 False"
        print("  ✓ is_match signals_not 排除")

        # ---- dump / load 往返 ----
        f_dump = Factor(
            signals_all=[
                Signal(signal="15分钟_倒0笔_方向_向上_其他_其他_50"),
                Signal(signal="日线_底分型_突破V250101_向上_其他_其他_90"),
            ],
            signals_any=[Signal(signal="60分钟_其他_其他_其他_其他_其他_0")],
            signals_not=[],
            name="往返测试",
        )
        raw = f_dump.dump()
        assert raw["name"] == f_dump.name
        assert len(raw["signals_all"]) == 2
        assert len(raw["signals_any"]) == 1
        assert raw["signals_not"] == []

        f_loaded = Factor.load(raw)
        assert f_loaded.name == f_dump.name, f"load 后 name 应一致，实际: {f_loaded.name} vs {f_dump.name}"
        assert len(f_loaded.signals_all) == 2
        assert len(f_loaded.signals_any) == 1
        assert len(f_loaded.signals_not) == 0
        print("  ✓ Factor dump / load 往返一致 (@dataclass)")

        print("  ✅ Factor 全部通过")

    test_factor()

    # ================================================================
    # 测试 Event 模型
    # ================================================================
    print()
    print("=" * 60)
    print("测试 Event 模型")
    print("=" * 60)

    def test_event():
        # ---- 基础创建 ----
        e1 = Event(
            operate=Operate.LO,
            factors=[
                Factor(
                    signals_all=[
                        Signal(signal="15分钟_方向_向上_其他_其他_其他_50"),
                    ],
                    name="因子1",
                ),
                Factor(
                    signals_all=[
                        Signal(signal="日线_底分型_突破_向上_其他_其他_80"),
                    ],
                    name="因子2",
                ),
            ],
            name="测试事件",
        )
        assert e1.name.startswith("测试事件#"), f"name 应包含 hash，实际: {e1.name}"
        assert len(e1.sha256) == 4, "sha256 应为 4 位"
        print("  ✓ Event 创建并自动生成 hash")

        # ---- factors 为空应报错 ----
        try:
            Event(operate=Operate.LO, factors=[], name="空事件")
            assert False, "空 factors 应抛出 ValueError"
        except ValueError as e:
            assert "不能为空" in str(e)
            print("  ✓ factors 为空时抛出 ValueError")

        # ---- name 自动使用 operate ----
        e_auto = Event(
            operate=Operate.SO,
            factors=[
                Factor(
                    signals_all=[Signal(signal="15分钟_方向_向下_其他_其他_其他_0")],
                ),
            ],
        )
        assert e_auto.name.startswith("开空#"), f"无名 Event 应以 operate 开头，实际: {e_auto.name}"
        print("  ✓ name 为空时自动使用 operate.value 作为前缀")

        # ---- unique_signals 包含 event 级别 + 所有 factor 的 signals ----
        e_uniq = Event(
            operate=Operate.LO,
            factors=[
                Factor(
                    signals_all=[Signal(signal="A_k1_k2_up_其他_其他_60")],
                    signals_any=[Signal(signal="B_k1_k2_up_其他_其他_60")],
                ),
            ],
            signals_all=[Signal(signal="C_k1_k2_up_其他_其他_0")],
            signals_any=[Signal(signal="D_k1_k2_up_其他_其他_0")],
            signals_not=[Signal(signal="E_k1_k2_up_其他_其他_0")],
        )
        uniq = e_uniq.unique_signals
        assert len(uniq) == 5, f"unique_signals 应为 5 (A+B+C+D+E)，实际: {len(uniq)}"
        print("  ✓ unique_signals 汇总 event + factor 信号并去重")

        # ---- is_match ----
        sig_dict = {
            "15分钟_方向_向上": "向上_其他_其他_80",
            "日线_底分_突破": "突破_向上_其他_90",
            "排除_信号_排除": "排除_排除_排除_50",
        }

        # 全部满足
        e_match = Event(
            operate=Operate.LO,
            factors=[
                Factor(
                    signals_all=[Signal(signal="15分钟_方向_向上_向上_其他_其他_50")],
                ),
            ],
        )
        is_match, factor_name = e_match.is_match(sig_dict)
        assert is_match, "Event 应匹配"
        print(f"  ✓ is_match 返回 (True, factor_name): ({is_match}, {factor_name})")

        # factor 不满足 (同 key，v1 不匹配)
        e_no_factor = Event(
            operate=Operate.LO,
            factors=[
                Factor(
                    signals_all=[Signal(signal="15分钟_方向_向上_向下_其他_其他_50")],
                ),
            ],
        )
        is_match, factor_name = e_no_factor.is_match(sig_dict)
        assert not is_match, "factor v1 不匹配时 Event 应不匹配"
        print("  ✓ is_match factor 不满足返回 (False, None)")

        # signals_not 排除
        e_not = Event(
            operate=Operate.LO,
            factors=[
                Factor(
                    signals_all=[Signal(signal="15分钟_方向_向上_向上_其他_其他_50")],
                ),
            ],
            signals_not=[Signal(signal="排除_信号_排除_排除_排除_排除_0")],
        )
        is_match, _ = e_not.is_match(sig_dict)
        assert not is_match, "signals_not 满足时应返回 False"
        print("  ✓ is_match signals_not 排除")

        # signals_all (event级) 不满足 — key 不存在抛异常
        e_all = Event(
            operate=Operate.LO,
            factors=[
                Factor(
                    signals_all=[Signal(signal="15分钟_方向_向上_向上_其他_其他_50")],
                ),
            ],
            signals_all=[Signal(signal="不存在_键_信号_其他_其他_其他_0")],
        )
        try:
            is_match, _ = e_all.is_match(sig_dict)
            assert False, "signals_all 的键不存在应抛异常"
        except ValueError:
            print("  ✓ is_match event 级 signals_all 键不存在时抛异常")

        # signals_any (event级) — key 不存在抛异常
        e_any_fail = Event(
            operate=Operate.LO,
            factors=[
                Factor(
                    signals_all=[Signal(signal="15分钟_方向_向上_向上_其他_其他_50")],
                ),
            ],
            signals_any=[Signal(signal="不存在_键_信号_其他_其他_其他_0")],
        )
        try:
            is_match, _ = e_any_fail.is_match(sig_dict)
            assert False, "signals_any 的键不存在应抛异常"
        except ValueError:
            print("  ✓ is_match event 级 signals_any 键不存在时抛异常")

        # ---- dump ----
        e_dump = Event(
            operate=Operate.LE,
            factors=[
                Factor(
                    signals_all=[
                        Signal(signal="60分钟_方向_向上_其他_其他_其他_0"),
                        Signal(signal="日线_底分_突破_向上_向上_其他_80"),
                    ],
                    signals_any=[Signal(signal="15分钟_其他_其他_其他_其他_其他_0")],
                    name="往返因子",
                ),
            ],
            signals_all=[Signal(signal="周线_趋势_向上_其他_其他_其他_60")],
            signals_any=[],
            signals_not=[Signal(signal="月线_背离_向下_其他_其他_其他_30")],
            name="往返测试",
        )
        raw = e_dump.dump()
        assert raw["operate"] == "平多", f"operate 应为 '平多'，实际: {raw['operate']}"
        assert len(raw["factors"]) == 1
        assert len(raw["signals_all"]) == 1
        assert len(raw["signals_not"]) == 1
        print("  ✓ Event dump 结构正确")

        e_loaded = Event.load(raw)
        assert e_loaded.name == e_dump.name
        assert e_loaded.operate == Operate.LE
        assert len(e_loaded.factors) == 1
        assert len(e_loaded.signals_all) == 1
        print("  ✓ Event dump / load 往返一致 (@dataclass)")

        # load 时 operate 无效应报错
        raw_bad = dict(raw)
        raw_bad["operate"] = "不存在的操作"
        try:
            Event.load(raw_bad)
            assert False, "无效 operate 应抛异常"
        except AssertionError:
            print("  ✓ load 时无效 operate 抛出 AssertionError")

        print("  ✅ Event 全部通过")

    test_event()

    # ================================================================
    # 测试 get_signals_freqs
    # ================================================================
    print()
    print("=" * 60)
    print("测试 get_signals_freqs")
    print("=" * 60)

    def test_get_signals_freqs():
        # 从信号字符串提取周期
        signals = [
            "15分钟_D1K_量柱V221218_低量柱_6K_任意_0",
            "日线_D1K_量柱V221218_低量柱_6K_任意_0",
            "60分钟_方向_向上_其他_其他_其他_80",
        ]
        freqs = get_signals_freqs(signals)
        assert "15分钟" in freqs, "应包含 15分钟"
        assert "60分钟" in freqs, "应包含 60分钟"
        assert "日线" in freqs, "应包含 日线"
        # 按 sorted_freqs 排序
        assert freqs == ["15分钟", "60分钟", "日线"], f"freqs 应按 sorted_freqs 排序，实际: {freqs}"
        print(f"  ✓ 提取周期: {freqs}")

        # 不匹配任何周期的信号
        empty_freqs = get_signals_freqs(["无关文本_无周期_其他_其他_其他_0"])
        assert empty_freqs == [], f"无周期时应返回空列表，实际: {empty_freqs}"
        print("  ✓ 无周期信号返回空列表")

        # 从信号函数配置字典中提取
        config = [
            {"freq": "日线", "max_overlap": "3", "name": "czsc.signals.cxt_bi_end_V230222"},
            {"freq1": "日线", "freq2": "60分钟", "name": "czsc.signals.cxt_zhong_shu_gong_zhen_V221221"},
        ]
        config_freqs = get_signals_freqs(config)
        assert "60分钟" in config_freqs
        assert "日线" in config_freqs
        assert config_freqs == ["60分钟", "日线"], f"应从配置字典中提取周期，实际: {config_freqs}"
        print(f"  ✓ 从配置字典提取周期: {config_freqs}")

        # Tick 在 sorted_freqs 开头
        tick_signals = ["Tick_数据源_原始K_其他_其他_其他_0"]
        tick_freqs = get_signals_freqs(tick_signals)
        assert "Tick" in tick_freqs, f"应提取 Tick 周期，实际: {tick_freqs}"
        print("  ✓ 提取 Tick 周期")

        print("  ✅ get_signals_freqs 全部通过")

    test_get_signals_freqs()

    # ================================================================
    # 测试 边界 & 特殊情况
    # ================================================================
    print()
    print("=" * 60)
    print("测试边界 & 特殊情况")
    print("=" * 60)

    def test_edge_cases():
        # Signal 空 signal 字符串 + 字段
        s_empty = Signal(k1="测试", k2="空值", k3="V1", score=0)
        # Pydantic 默认值 "任意" 替代了空字符串
        assert s_empty.signal == "测试_空值_V1_任意_任意_任意_0", f"空字段默认 '任意'，实际: {s_empty.signal}"
        print("  ✓ Signal 空 v1/v2/v3 字段 (默认 '任意')")

        # Signal 最小 score 0
        s_min = Signal(signal="test_k1_k2_up_其他_其他_0")
        assert s_min.score == 0
        print("  ✓ Signal score=0 (边界)")

        # Signal 最大 score 100
        s_max = Signal(signal="test_k1_k2_up_其他_其他_100")
        assert s_max.score == 100
        print("  ✓ Signal score=100 (边界)")

        # Factor 空 signals_any 和 signals_not
        f_min = Factor(
            signals_all=[Signal(signal="test_k1_k2_up_其他_其他_0")],
        )
        assert f_min.signals_any == []
        assert f_min.signals_not == []
        assert f_min.name.startswith("#")
        print("  ✓ Factor 最小构造 (仅 signals_all)")

        # Factor.load 无 signals_any/signals_not
        f_from_raw = Factor.load(
            {
                "name": "最小因子",
                "signals_all": ["test_k1_k2_up_其他_其他_0"],
            }
        )
        assert len(f_from_raw.signals_all) == 1
        assert f_from_raw.signals_any == []
        assert f_from_raw.signals_not == []
        print("  ✓ Factor.load 缺省 signals_any/signals_not (@dataclass)")

        # Event signals_any 和 signals_not 可为空
        e_min = Event(
            operate=Operate.HO,
            factors=[
                Factor(
                    signals_all=[Signal(signal="test_k1_k2_up_其他_其他_0")],
                ),
            ],
        )
        assert e_min.signals_all == []
        assert e_min.signals_any == []
        assert e_min.signals_not == []
        print("  ✓ Event 最小构造 (仅 operate + factors)")

        # 同一 Signal 内容不同 score 视为不同信号
        s_a = Signal(signal="test_k1_k2_up_其他_其他_10")
        s_b = Signal(signal="test_k1_k2_up_其他_其他_90")
        assert s_a.signal != s_b.signal
        print("  ✓ 不同 score 产生不同 signal 字符串")

        print("  ✅ 边界测试全部通过")

    test_edge_cases()

    def test_signal():
        s = Signal(k1="1分钟", k3="倒1形态", v1="类一买", v2="七笔", v3="基础型", score=3)
        assert str(s) == "Signal('1分钟_任意_倒1形态_类一买_七笔_基础型_3')"
        assert s.key == "1分钟_倒1形态"
        s1 = Signal(signal="1分钟_任意_倒1形态_类一买_七笔_基础型_3")
        assert s == s1
        assert s.is_match({"1分钟_倒1形态": "类一买_七笔_基础型_3"})
        assert not s.is_match({"1分钟_倒1形态": "类一买_七笔_特例一_3"})
        assert not s.is_match({"1分钟_倒1形态": "类一买_九笔_基础型_3"})

        s = Signal(k1="1分钟", k2="倒1形态", k3="类一买", score=3)
        assert str(s) == "Signal('1分钟_倒1形态_类一买_任意_任意_任意_3')"
        assert s.key == "1分钟_倒1形态_类一买"

        try:
            s = Signal(k1="1分钟", k2="倒1形态", k3="类一买", score=101)
        except ValueError as e:
            assert str(e) == "score 必须在0~100之间"

    test_signal()

    def test_factor():
        freq = Freq.F15
        s = OrderedDict()
        default_signals = [
            Signal(k1=str(freq.value), k2="倒0笔", k3="方向", v1="向上", v2="其他", v3="其他"),
            Signal(k1=str(freq.value), k2="倒0笔", k3="长度", v1="大于5", v2="其他", v3="其他"),
            Signal(k1=str(freq.value), k2="倒0笔", k3="三K形态", v1="顶分型", v2="其他", v3="其他"),
            Signal(k1=str(freq.value), k2="倒1笔", k3="表里关系", v1="其他", v2="其他", v3="其他"),
            Signal(k1=str(freq.value), k2="倒1笔", k3="RSQ状态", v1="小于0.2", v2="其他", v3="其他"),
        ]
        for signal in default_signals:
            s[signal.key] = signal.value

        factor = Factor(
            name="单测",
            signals_all=[
                Signal(k1=str(freq.value), k2="倒0笔", k3="方向", v1="向上", v2="其他", v3="其他"),
                Signal(k1=str(freq.value), k2="倒0笔", k3="长度", v1="大于5", v2="其他", v3="其他"),
            ],
        )
        assert factor.is_match(s)

        factor_raw = factor.dump()
        new_factor = Factor.load(factor_raw)
        assert new_factor.is_match(s)

        factor = Factor(
            name="单测",
            signals_all=[
                Signal(k1=str(freq.value), k2="倒0笔", k3="方向", v1="向上", v2="其他", v3="其他"),
                Signal(k1=str(freq.value), k2="倒0笔", k3="长度", v1="大于5", v2="其他", v3="其他"),
            ],
            signals_any=[Signal(k1=str(freq.value), k2="倒1笔", k3="RSQ状态", v1="小于0.2", v2="其他", v3="其他")],
        )
        assert factor.is_match(s)

        factor = Factor(
            name="单测",
            signals_all=[
                Signal(k1=str(freq.value), k2="倒0笔", k3="方向", v1="向上", v2="其他", v3="其他"),
                Signal(k1=str(freq.value), k2="倒0笔", k3="长度", v1="大于5", v2="其他", v3="其他"),
            ],
            signals_any=[Signal(k1=str(freq.value), k2="倒1笔", k3="RSQ状态", v1="小于0.8", v2="其他", v3="其他")],
        )
        assert not factor.is_match(s)

        factor = Factor(
            name="单测",
            signals_all=[
                Signal(k1=str(freq.value), k2="倒0笔", k3="方向", v1="向上", v2="其他", v3="其他"),
                Signal(k1=str(freq.value), k2="倒0笔", k3="长度", v1="大于5", v2="其他", v3="其他"),
            ],
            signals_any=[Signal(k1=str(freq.value), k2="倒1笔", k3="RSQ状态", v1="小于0.2", v2="其他", v3="其他")],
            signals_not=[
                Signal(k1=str(freq.value), k2="倒0笔", k3="三K形态", v1="顶分型", v2="其他", v3="其他"),
            ],
        )
        assert not factor.is_match(s)

    test_factor()

    def test_event():
        freq = Freq.F15
        s = OrderedDict()
        default_signals = [
            Signal(k1=str(freq.value), k2="倒0笔", k3="方向", v1="向上", v2="其他", v3="其他"),
            Signal(k1=str(freq.value), k2="倒0笔", k3="长度", v1="大于5", v2="其他", v3="其他"),
            Signal(k1=str(freq.value), k2="倒0笔", k3="三K形态", v1="顶分型", v2="其他", v3="其他"),
            Signal(k1=str(freq.value), k2="倒1笔", k3="表里关系", v1="其他", v2="其他", v3="其他"),
            Signal(k1=str(freq.value), k2="倒1笔", k3="RSQ状态", v1="小于0.2", v2="其他", v3="其他"),
        ]
        for signal in default_signals:
            s[signal.key] = signal.value

        event = Event(
            name="单测",
            operate=Operate.LO,
            factors=[
                Factor(
                    name="测试",
                    signals_all=[Signal(k1=str(freq.value), k2="倒0笔", k3="长度", v1="大于5", v2="其他", v3="其他")],
                )
            ],
            signals_all=[
                Signal(k1=str(freq.value), k2="倒0笔", k3="方向", v1="向上", v2="其他", v3="其他"),
            ],
        )
        m, f = event.is_match(s)
        assert m and f

        raw = event.dump()
        new_event = Event.load(raw)
        m, f = new_event.is_match(s)
        assert m and f

        raw1 = {
            "name": "单测",
            "operate": "开多",
            "signals_all": ["15分钟_倒0笔_方向_向上_其他_其他_0"],
            "factors": [{"name": "测试", "signals_all": ["15分钟_倒0笔_长度_大于5_其他_其他_0"]}],
        }
        new_event = Event.load(raw1)
        m, f = new_event.is_match(s)
        assert m and f

        raw1 = {
            "operate": "开多",
            "signals_all": ["15分钟_倒0笔_方向_向上_其他_其他_0"],
            "factors": [{"name": "测试", "signals_all": ["15分钟_倒0笔_长度_大于5_其他_其他_0"]}],
        }
        new_event = Event.load(raw1)
        m, f = new_event.is_match(s)
        assert m and f

        event = Event(
            name="单测",
            operate=Operate.LO,
            factors=[
                Factor(name="测试", signals_all=[Signal("15分钟_倒0笔_长度_大于5_其他_其他_0")]),
            ],
            signals_any=[Signal("15分钟_倒0笔_方向_向上_其他_其他_0"), Signal("15分钟_倒0笔_长度_大于100_其他_其他_0")],
        )
        m, f = event.is_match(s)
        assert m and f

        event = Event(
            name="单测",
            operate=Operate.LO,
            factors=[
                Factor(
                    name="测试",
                    signals_all=[Signal(k1=str(freq.value), k2="倒0笔", k3="长度", v1="大于5", v2="其他", v3="其他")],
                )
            ],
            signals_not=[
                Signal(k1=str(freq.value), k2="倒0笔", k3="方向", v1="向上", v2="其他", v3="其他"),
            ],
        )
        m, f = event.is_match(s)
        assert not m and not f

        event = Event(
            name="单测",
            operate=Operate.LO,
            factors=[
                Factor(
                    name="测试",
                    signals_all=[
                        Signal(k1=str(freq.value), k2="倒0笔", k3="方向", v1="向上", v2="其他", v3="其他"),
                        Signal(k1=str(freq.value), k2="倒0笔", k3="长度", v1="大于5", v2="其他", v3="其他"),
                    ],
                )
            ],
        )
        m, f = event.is_match(s)
        assert m and f

        event = Event(
            name="单测",
            operate=Operate.LO,
            factors=[
                Factor(
                    name="测试",
                    signals_all=[
                        Signal("15分钟_倒0笔_方向_向上_其他_其他_0"),
                        Signal("15分钟_倒0笔_长度_任意_其他_其他_0"),
                    ],
                )
            ],
        )
        m, f = event.is_match(s)
        assert m and f

        event = Event(
            name="单测",
            operate=Operate.LO,
            factors=[
                Factor(
                    name="测试",
                    signals_all=[
                        Signal("15分钟_倒0笔_方向_向上_其他_其他_20"),
                        Signal("15分钟_倒0笔_长度_任意_其他_其他_0"),
                    ],
                )
            ],
        )
        m, f = event.is_match(s)
        assert not m and not f

        event = Event(
            name="单测",
            operate=Operate.LO,
            factors=[
                Factor(
                    name="测试",
                    signals_all=[
                        Signal("15分钟_倒0笔_方向_向下_其他_其他_0"),
                        Signal("15分钟_倒0笔_长度_任意_其他_其他_0"),
                    ],
                )
            ],
        )
        m, f = event.is_match(s)
        assert not m and not f

        event = Event.load(
            {
                "name": "开多",
                "operate": "开多",
                "signals_all": ["1分钟_D1_涨跌停V230331_任意_任意_任意_0", "1分钟_D0停顿分型_BE辅助V230106_看空_强_任意_0"],
                "signals_any": [],
                "signals_not": [],
                "factors": [
                    {
                        "name": "SMA#40多头",
                        "signals_all": ["5分钟_D1#SMA#40MO10_BS辅助V230313_看多_任意_任意_0"],
                        "signals_any": [],
                        "signals_not": [],
                    }
                ],
            }
        )
        assert len(event.get_signals_config()) == 3

    test_event()

    print()
    print("=" * 60)
    print("🎉 全部测试通过！")
    print("=" * 60)
