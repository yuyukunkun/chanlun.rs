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
import sys
from collections import OrderedDict
from dataclasses import dataclass, field
from datetime import datetime, timedelta
from enum import Enum
from typing import Any, Callable, Dict, List, Optional, Tuple, Union

import numpy as np
from loguru import logger

from chanlun import K线, 虚线, 中枢, 观察者, 立体分析器
from chanlun.parse import parse

# 信号匹配原语已移植到 Rust 核心层（chanlun._chanlun）。
# Operate/Signal/Factor/Event/Position 改为从 Rust 导入；Position 在下方扩展为子类补 update 状态机。
from chanlun.signal_orchestrator import SignalOrchestrator

from chanlun._chanlun import (
    Signal,
    Factor,
    Event,
    Operate,
    Position as _PositionBase,
)


def import_by_name(name: str):
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

            except (OSError, ImportError, TypeError, ValueError, AttributeError) as e:
                logger.error(f"解析信号函数 {name} 出错：{e}")

        # 为每个 k3 生成独立 pattern（支持单函数多 k3 信号，如 youwukuncheng 的 3 个 k3）。
        # base pattern 末段是 k3，按 sig_name_map 里各 Signal 的 k3 逐一替换。
        _multi_pats: Dict[str, List[str]] = {}
        for _name, _base in sig_pats_map.items():
            _sigs = sig_name_map.get(_name, [])
            if _sigs:
                _prefix = _base.rsplit("_", 1)[0] if "_" in _base else _base
                _pats: List[str] = []
                for _s in _sigs:
                    _p = f"{_prefix}_{_s.k3}"
                    if _p not in _pats:
                        _pats.append(_p)
                _multi_pats[_name] = _pats
            else:
                _multi_pats[_name] = [_base]

        self.sig_name_map = sig_name_map
        self.sig_pats_map = _multi_pats  # name → List[pattern]（每个 k3 一个）
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
        pats_list = self.sig_pats_map.get(name, None)
        if not pats_list:
            return None

        for pats in pats_list:
            try:
                parsed = parse(pats, key)
            except (ValueError, KeyError, TypeError, AttributeError):
                continue
            if parsed is None:
                continue
            params = parsed.named
            if "di" in params:
                params["di"] = int(params["di"])

            params["name"] = f"{self.signals_module}.{name}"

            # 附加上下文：触发条件与函数短名（供 信号计算器 优化用）
            触发条件 = self.sig_trigger_map.get(name)
            if 触发条件:
                params["触发条件"] = 触发条件
            params["_func_short_name"] = name

            return params

        logger.error(f"解析信号 {signal} - {name} 出错：无匹配模式 {pats_list}")
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
        _k3_match = list({k for k, v in sig_name_map.items() for s in v if s.k3 == _signal.k3})

        # 多匹配时排除模板函数（以 "模板_" 开头）
        if len(_k3_match) > 1:
            non_template = [k for k in _k3_match if not k.startswith("模板_")]
            if len(non_template) == 1:
                return non_template[0]

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
                for pats in self.sig_pats_map[name]:
                    keys.append(pats.format(**conf))
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


def get_signals_config(signals_seq: List[str], signals_module: str = "") -> List[Dict]:
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


def create_single_signal(**kwargs) -> OrderedDict:
    """创建单个信号"""
    s = OrderedDict()
    k1, k2, k3 = kwargs.get("k1", "任意"), kwargs.get("k2", "任意"), kwargs.get("k3", "任意")
    v1, v2, v3 = kwargs.get("v1", "任意"), kwargs.get("v2", "任意"), kwargs.get("v3", "任意")
    v = Signal(k1=k1, k2=k2, k3=k3, v1=v1, v2=v2, v3=v3, score=kwargs.get("score", 0))
    s[v.key] = v.value
    return s


# ==============================================================================
# Position — 持仓管理
# ==============================================================================


class Position(_PositionBase):
    """持仓对象 — 配置 + 状态机均已迁移到 Rust 核心。

    仓位表达：1 持有多头，-1 持有空头，0 空仓。

    Rust 基类（chanlun._chanlun.Position）提供：
      - 配置字段 symbol/opens/exits/events/name/interval/timeout/stop_loss/T0（只读 getter）
      - 状态字段 pos/pos_changed/operates/holds/pairs（只读 getter）
      - update(信号字典) — 持仓状态机
      - dump(with_data) — 序列化（含可选状态）
      - load(raw) — 反序列化（静态方法）
      - unique_signals、__repr__
    本子类仅保留 get_signals_config（需 Python signals_module）。
    """

    def __init__(self, *args, **kwargs):
        # 状态字段已由 Rust #[new] 初始化；无需 Python 侧初始化。
        # 不调用 super().__init__()：PyO3 #[new] 已在 __new__ 阶段建好内部配置。
        pass

    def get_signals_config(self, signals_module: str = "") -> List[Dict]:
        """获取事件的信号配置"""
        return get_signals_config(self.unique_signals, signals_module)

    def dump(self, with_data: bool = False) -> dict:
        """序列化为 dict。Rust 基类 dump(with_data) 处理配置 + 可选状态。"""
        return super().dump(with_data=with_data)

    @classmethod
    def load(cls, raw: dict) -> "Position":
        """从 dict 反序列化为 Position（子类实例）；opens/exits 用 Rust Event.load 还原。"""
        return cls(
            symbol=raw["symbol"],
            name=raw["name"],
            opens=[Event.load(x) for x in raw.get("opens", [])],
            exits=[Event.load(x) for x in raw.get("exits", [])],
            interval=raw["interval"],
            timeout=raw["timeout"],
            stop_loss=raw["stop_loss"],
            T0=raw["T0"],
        )


class 信号计算器:
    """多周期信号计算引擎 — 基于观察者字典。

    不再依赖 立体分析器，直接接收 ``{周期秒: 观察者}`` 字典。

    使用方式::

        分析器 = 立体分析器("btcusd", [300, 900, 3600], 配置)
        观察者字典 = {p: 分析器._单体分析器[p] for p in 分析器.周期组}
        计算器 = 信号计算器(观察者字典, 基础周期=300, 信号配置=[...])

        for k in k线列表:
            分析器.投喂K线(k)
            计算器.更新()
            print(计算器.信号字典)
    """

    def __init__(
        self,
        分析器: 立体分析器,
        信号配置: Optional[List[Dict]] = None,
        信号模块: str = "",
    ):
        self._分析器 = 分析器
        self._观察者字典 = {p: 分析器._单体分析器[p] for p in 分析器.周期组}
        self._基础周期 = 分析器.周期组[0]
        self._信号模块 = 信号模块
        self._信号函数缓存: Dict[str, Callable] = {}
        self.信号: dict = {}
        self.行情: dict = {}
        self.信号配置 = 信号配置 or []
        self._自动挂载指标()

    @property
    def 信号字典(self) -> dict:  # 向后兼容：合并返回
        return {**self.信号, **self.行情}

    @property
    def 信号配置(self) -> List[Dict]:
        return self._信号配置

    @信号配置.setter
    def 信号配置(self, value: List[Dict]):
        可用周期 = set(self._分析器.周期组)
        for c in value:
            freq = c.get("freq")
            if freq is not None:
                周期秒 = int(freq)
                if 周期秒 not in 可用周期:
                    raise ValueError(f"信号配置 freq={freq}({周期秒}s) 不在分析器周期组 {sorted(可用周期)} 中\n  信号: {c.get('name', '?')}")
        self._信号配置 = self._去重配置(value)
        self._预加载信号函数()

    def _去重配置(self, configs: List[Dict]) -> List[Dict]:
        seen = set()
        unique = []
        for c in configs:
            key = (c.get("name"), frozenset((k, str(v)) for k, v in c.items() if k != "name"))
            if key not in seen:
                seen.add(key)
                unique.append(c)
            else:
                logger.warning(f"信号计算器: 重复信号配置已跳过 — {c.get('name', '?')} { {k: v for k, v in c.items() if k != 'name'} }")
        return unique

    def _预加载信号函数(self):
        for config in self._信号配置:
            name = config.get("name")
            if name and name not in self._信号函数缓存:
                try:
                    self._信号函数缓存[name] = self._解析信号函数(name)
                except (ImportError, ModuleNotFoundError, AttributeError, KeyError) as e:
                    logger.warning(f"信号计算器: 无法导入 {name} ({e})，跳过")

    @staticmethod
    def _解析信号函数(name: str):
        """解析信号函数名，返回可调用对象。

        当运行在 __main__ 上下文中且目标模块为 chan 时，优先使用 __main__
        命名空间中的函数，避免 import_by_name 触发 chan 模块的重复导入。
        """
        if "." in name:
            module_name, func_name = name.rsplit(".", 1)
            main_mod = sys.modules.get("__main__")
            if main_mod is not None and hasattr(main_mod, func_name):
                # 验证 __main__ 确实是目标模块（通过文件名判断）
                main_file = getattr(main_mod, "__file__", "")
                expected_path = module_name.replace(".", os.sep) + ".py"
                if main_file.endswith(expected_path):
                    return getattr(main_mod, func_name)

        return import_by_name(name)

    def _自动挂载指标(self):
        """根据信号配置参数，在对应周期的观察者上自动补全缺失的指标。"""
        from collections import defaultdict

        待补MACD: Dict[int, List[tuple]] = defaultdict(list)
        待补均线: Dict[int, List[tuple]] = defaultdict(list)

        for config in self._信号配置:
            name = config.get("name", "")
            freq = config.get("freq")
            if not freq:
                continue
            周期秒 = int(freq)
            if 周期秒 not in self._观察者字典:
                continue

            # MACD 类信号：从 config 解析 fast/slow/signal 参数
            if "macd" in name.lower() or "中枢" in name or "背驰" in name or "金叉" in name:
                fast = int(config.get("fast", config.get("快线周期", 13)))
                slow = int(config.get("slow", config.get("慢线周期", 31)))
                sig = int(config.get("signal", config.get("信号周期", 11)))
                key = f"macd_{fast}_{slow}_{sig}"
                if not any(t[0] == key for t in 待补MACD[周期秒]):
                    待补MACD[周期秒].append((key, "收", fast, slow, sig))

            # MA 类信号：从 config 解析 ma_type/timeperiod
            if "ma_" in name or "tas_ma" in name or "均线" in name:
                ma_type = config.get("ma_type", "SMA").upper()
                period = int(config.get("timeperiod", config.get("周期", 5)))
                key = f"{ma_type}_{period}"
                if not any(t[0] == key for t in 待补均线[周期秒]):
                    待补均线[周期秒].append((key, "收", ma_type, period))

        for 周期秒, macd_list in 待补MACD.items():
            cfg = self._观察者字典[周期秒].配置
            if not cfg.计算指标:
                cfg.计算指标 = True
            已有键 = {t[0] for t in cfg.MACD_参数列表}
            # 同时检查 (快线, 慢线, 信号) 参数避免只键名不同但参数相同的重复
            已有参数 = {(t[2], t[3], t[4]) for t in cfg.MACD_参数列表 if len(t) >= 5}
            新增 = [t for t in macd_list if t[0] not in 已有键 and (t[2], t[3], t[4]) not in 已有参数]
            if 新增:
                cfg.MACD_参数列表.extend(新增)
                if "macd" not in 已有键:
                    cfg.MACD_参数列表.insert(0, ("macd", "收", 新增[0][2], 新增[0][3], 新增[0][4]))
                logger.warning(f"信号计算器: 周期{周期秒}s 自动补全 MACD — {[t[0] for t in 新增]}")

        for 周期秒, ma_list in 待补均线.items():
            cfg = self._观察者字典[周期秒].配置
            if not cfg.计算指标:
                cfg.计算指标 = True
            已有 = {t[0] for t in cfg.均线参数列表}
            新增 = [t for t in ma_list if t[0] not in 已有]
            if 新增:
                cfg.均线参数列表.extend(新增)
                logger.warning(f"信号计算器: 周期{周期秒}s 自动补全 均线 — {[t[0] for t in 新增]}")

    def 从信号列表提取配置(self, 信号序列: List[str]):
        """从信号序列自动生成信号配置"""
        self.信号配置 = get_signals_config(list(set(信号序列)), self._信号模块)

    def 更新(self):
        """遍历信号配置，调用信号函数。结果写入 self.信号 和 self.行情。"""
        self.信号.clear()
        self.行情.clear()

        for config in self._信号配置:
            try:
                result = self._执行信号函数(config)
                if result:
                    for k, v in result.items():
                        if v != "任意_任意_任意_0":
                            self.信号[k] = v
            except (TypeError, ValueError, KeyError, AttributeError, IndexError) as e:
                logger.error(f"信号计算器: {config.get('name', '?')} 出错 — {e}")
                traceback.print_exc()

        # OHLCV 行情
        基础观察者 = self._观察者字典.get(self._基础周期)
        if 基础观察者 and 基础观察者.普通K线序列:
            最后K线 = 基础观察者.普通K线序列[-1]
            时间戳 = 最后K线.时间戳
            if isinstance(时间戳, (int, float)):
                时间戳 = datetime.fromtimestamp(int(时间戳))
            self.行情.update(
                symbol=基础观察者.符号,
                dt=时间戳,
                id=最后K线.序号,
                open=最后K线.开盘价,
                close=最后K线.收盘价,
                high=最后K线.高,
                low=最后K线.低,
                vol=最后K线.成交量,
            )

    def _执行信号函数(self, config: Dict) -> Optional[OrderedDict]:
        param = dict(config)
        sig_name = param.pop("name")
        sig_func = self._信号函数缓存.get(sig_name) or self._解析信号函数(sig_name)

        freq = param.get("freq")
        if freq is not None:
            周期秒 = int(freq)
            obs = self._观察者字典.get(周期秒)
            if obs is not None:
                return sig_func(obs, **param)
            else:
                raise KeyError(f"信号计算器: 未找到周期 {周期秒}s 的观察者，可用周期: {sorted(self._观察者字典.keys())}")
        else:
            return sig_func(self, **param)

    def 获取周期观察者(self, freq: str) -> Optional[观察者]:
        return self._观察者字典.get(int(freq))
