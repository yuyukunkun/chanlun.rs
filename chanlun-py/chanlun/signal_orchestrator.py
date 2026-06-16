"""信号编排器 — Rust 优先 + Python 回退的混合信号计算。

Rust 注册表优先（编译时 #[signal] 宏注册），Python import_by_name 回退。

使用方式::

    分析器 = 立体分析器("btcusd", [300, 900, 3600], 配置)
    编排器 = SignalOrchestrator(分析器, 信号配置=[...])

    for k in k线列表:
        分析器.投喂K线(k)
        编排器.更新()
        print(编排器.信号字典)
"""

import sys
from collections import OrderedDict
from typing import Any, Callable, Dict, List, Optional

from loguru import logger

from chanlun import 观察者
from chanlun._chanlun import (
    信号引擎 as _RustSignalEngine,
    list_signals as _rust_list_signals,
)


class SignalOrchestrator:
    """混合信号编排器：Rust 注册表优先，Python import_by_name 回退。"""

    def __init__(
        self,
        分析器,
        信号配置: Optional[List[Dict]] = None,
        信号模块: str = "chanlun.signals",
    ):
        self._分析器 = 分析器
        self._观察者字典 = {p: 分析器._单体分析器[p] for p in 分析器.周期组}
        self._基础周期 = 分析器.周期组[0]
        self._信号模块 = 信号模块

        # 分类存储
        self._rust_configs: List[Dict] = []
        self._python_configs: List[Dict] = []
        self._python_func_cache: Dict[str, Callable] = {}

        # 结果容器
        self.信号: Dict[str, str] = {}
        self.行情: Dict[str, Any] = {}

        # 初始设置
        self.信号配置 = 信号配置 or []

    # ── 信号配置 property ──

    @property
    def 信号配置(self) -> List[Dict]:
        return self._信号配置

    @信号配置.setter
    def 信号配置(self, value: List[Dict]):
        可用周期 = set(self._分析器.周期组)
        rust_names = set(_rust_list_signals())

        self._rust_configs = []
        self._python_configs = []

        for c in self._去重配置(value):
            freq = c.get("freq")
            if freq is not None:
                周期秒 = int(freq)
                if 周期秒 not in 可用周期:
                    raise ValueError(f"信号配置 freq={freq}({周期秒}s) 不在分析器周期组 {sorted(可用周期)} 中\n  信号: {c.get('name', '?')}")

            name = c.get("name", "")
            # 分类：含 '.' 的显式 Python 路径 → Python；短名查 Rust 注册表
            if "." in name:
                self._python_configs.append(c)
            elif name in rust_names:
                self._rust_configs.append(c)
            else:
                self._python_configs.append(c)

        self._信号配置 = value
        self._预加载Python信号函数()

    # ── 更新 ──

    def 更新(self):
        """执行所有信号：Rust 批量优先，Python 逐个回退。"""
        self.信号.clear()
        self.行情.clear()

        # 0. 始终确保指标已计算（幂等），Rust/Python 信号都需要
        _RustSignalEngine(信号配置=[]).自动挂载指标(self._分析器)

        # 1. Rust 批量执行
        if self._rust_configs:
            rust_cfgs = []
            for c in self._rust_configs:
                freq = int(c.get("freq", 0))
                rust_cfgs.append({"name": c["name"], "freq": str(freq)})

            engine = _RustSignalEngine(信号配置=rust_cfgs)
            engine.自动挂载指标(self._分析器)

            result = engine.更新_完整(self._分析器)
            if result.get("signals"):
                for k, v in result["signals"].items():
                    if v != "任意_任意_任意_0":
                        self.信号[k] = v
            if result.get("market"):
                self.行情 = dict(result["market"])

        # 2. Python 回退（逐个 import_by_name 调用）
        for config in self._python_configs:
            try:
                result = self._执行Python信号函数(config)
                if result:
                    for k, v in result.items():
                        if v != "任意_任意_任意_0":
                            self.信号[k] = v
            except Exception:
                logger.exception(f"Python 信号函数执行失败: {config.get('name')}")

        # 3. 补充行情（若 Rust 引擎未提供）
        if not self.行情:
            self._提取行情()

    # ── Python 信号函数执行 ──

    def _执行Python信号函数(self, config: Dict) -> Optional[OrderedDict]:
        """执行单个 Python 信号函数（import_by_name 动态导入）。"""
        param = dict(config)
        sig_name = param.pop("name")
        sig_func = self._python_func_cache.get(sig_name) or self._解析信号函数(sig_name)
        if sig_func is None:
            logger.warning(f"信号函数未找到: {sig_name}")
            return None

        freq = param.get("freq", None)
        if freq is not None:
            周期秒 = int(freq)
            obs = self._观察者字典.get(周期秒)
            if obs is None:
                logger.warning(f"未找到周期 {freq} 的观察者")
                return None
        else:
            obs = self
        return sig_func(obs, **param)

    # ── 辅助方法 ──

    def _去重配置(self, configs: List[Dict]) -> List[Dict]:
        seen = set()
        unique = []
        for c in configs:
            key = (
                c.get("name"),
                frozenset((k, str(v)) for k, v in c.items() if k != "name"),
            )
            if key not in seen:
                seen.add(key)
                unique.append(c)
        return unique

    def _预加载Python信号函数(self):
        for config in self._python_configs:
            name = config.get("name", "")
            if name and name not in self._python_func_cache:
                self._python_func_cache[name] = None
        for name in list(self._python_func_cache.keys()):
            try:
                self._python_func_cache[name] = self._解析信号函数(name)
            except Exception:
                logger.warning(f"预加载信号函数失败: {name}")

    @staticmethod
    def _解析信号函数(name: str) -> Optional[Callable]:
        """动态导入信号函数（与旧 信号计算器 逻辑一致）。"""
        if "." not in name:
            return __import__(name)

        module_name, func_name = name.rsplit(".", 1)
        main_mod = sys.modules.get("__main__")
        if main_mod is not None and hasattr(main_mod, func_name):
            return getattr(main_mod, func_name)

        module = __import__(module_name, fromlist=[func_name])
        return getattr(module, func_name, None)

    def _提取行情(self):
        """从基础周期观察者提取 OHLCV 行情。"""
        obs = self._观察者字典.get(self._基础周期)
        if obs is None:
            return
        klines = obs.普通K线序列
        if not klines:
            return
        k = klines[-1]
        self.行情 = {
            "symbol": obs.符号,
            "dt": k.时间戳,
            "id": k.序号,
            "open": k.开盘价,
            "high": k.高,
            "low": k.低,
            "close": k.收盘价,
            "vol": k.成交量,
        }

    # ── 公共属性 ──

    @property
    def 信号字典(self) -> dict:
        """合并信号 + 行情（与 Position.update() 兼容）。"""
        return {**self.信号, **self.行情}

    def 获取周期观察者(self, freq: str) -> Optional[观察者]:
        """按频率字符串获取观察者。"""
        return self._观察者字典.get(int(freq))

    def 从信号列表提取配置(self, 信号序列: List[str]):
        """从信号字符串列表解析配置（Rust 模板 + Python SignalsParser 双路径）。"""
        from chanlun._chanlun import get_signal_template

        if not 信号序列:
            return

        rust_names = set(_rust_list_signals())
        configs = []
        seen = set()

        for sig_key in 信号序列:
            matched = False
            # 1) 尝试 Rust 模板匹配
            for name in rust_names:
                template = get_signal_template(name)
                if template is None:
                    continue
                from chanlun.parse import parse as _parse

                parsed = _parse(template, sig_key)
                if parsed is not None:
                    entry = {"name": name}
                    entry.update(parsed.named)
                    key = (name, frozenset((k, str(v)) for k, v in entry.items() if k != "name"))
                    if key not in seen:
                        seen.add(key)
                        configs.append(entry)
                    matched = True
                    break
            # 2) Python SignalsParser 回退
            if not matched:
                try:
                    from chanlun.chan_external import SignalsParser

                    sp = SignalsParser(signals_module=self._信号模块)
                    py_configs = sp.parse([sig_key])
                    for c in py_configs:
                        key = (c.get("name"), frozenset((k, str(v)) for k, v in c.items() if k != "name"))
                        if key not in seen:
                            seen.add(key)
                            configs.append(c)
                except Exception:
                    logger.warning(f"无法解析信号 key: {sig_key}")

        self.信号配置 = configs


def get_signals_config(signal_keys: list, signals_module: str = "chanlun.signals") -> List[Dict]:
    """从 Rust 注册表 + Python SignalsParser 生成信号配置（双路径）。

    根据信号 key 字符串，优先用 Rust 注册表模板匹配，失败则回退到 Python SignalsParser。
    """
    from chanlun._chanlun import list_signals, get_signal_template

    rust_names = set(list_signals())
    configs = []
    seen = set()
    unmatched = []

    for sig_key in signal_keys:
        matched = False
        # 1) Rust 模板
        for name in rust_names:
            template = get_signal_template(name)
            if template is None:
                continue
            from chanlun.parse import parse as _parse

            parsed = _parse(template, sig_key)
            if parsed is not None:
                entry = {"name": name}
                entry.update(parsed.named)
                key = (name, frozenset((k, str(v)) for k, v in entry.items() if k != "name"))
                if key not in seen:
                    seen.add(key)
                    configs.append(entry)
                matched = True
                break
        # 2) 回退到 Python
        if not matched:
            unmatched.append(sig_key)

    if unmatched:
        try:
            from chanlun.chan_external import SignalsParser

            sp = SignalsParser(signals_module=signals_module)
            py_configs = sp.parse(unmatched)
            for c in py_configs:
                key = (c.get("name"), frozenset((k, str(v)) for k, v in c.items() if k != "name"))
                if key not in seen:
                    seen.add(key)
                    configs.append(c)
        except Exception:
            logger.warning(f"SignalsParser 无法解析: {unmatched}")

    return configs
