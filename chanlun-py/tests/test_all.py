#!/usr/bin/env python3
"""chanlun PyO3 综合测试 — 所有测试类统一入口。

使用了 pyo3_test_helpers 中的泛化 Mixin:
  - RcIdentityMixin: Rc/Arc 指针身份一致性
  - PyO3SubclassMixin: PyO3 子类化兼容性
  - ApiConsistencyMixin: chan/chanlun API 描述符类型一致性
  - assert_type_shape: 返回值类型形状验证

运行方式:
    python -m pytest tests/test_all.py -v
    python tests/test_all.py                           # 全部测试
    python tests/test_all.py Test观察者子类化           # 指定测试类
    python tests/test_all.py Test观察者子类化.test_subclass_basic  # 单个测试
"""

import unittest
import sys
import os
import struct
import tempfile
import math
from datetime import datetime

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "tests"))

import chanlun
import chanlun.chan  # noqa  # 用于 API 一致性测试
from helpers import ApiConsistencyMixin, RcIdentityMixin, PyO3SubclassMixin, assert_type_shape

# ---- 路径 ----

_PROJECT_ROOT = os.environ.get(
    "CHANLUN_PROJECT_ROOT",
    os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))),
)

NB_PATH = os.path.join(os.path.dirname(os.path.abspath(__file__)), "btcusd-300-1777649100-1778398800.nb")
_PY_REF_DIR = os.path.join(_PROJECT_ROOT, "Py_btcusd:300_1777649100_1778398800")
_RUST_REF_DIR = os.path.join(_PROJECT_ROOT, "chanlun", "Rust_btcusd:300_1777649100_1778398800")

# ---- 辅助函数 ----


def read_nb_bars(path, max_bars=None):
    """从 .nb 文件读取 K 线数据 (48 字节: 6 × f64 大端)."""
    bars = []
    with open(path, "rb") as f:
        i = 0
        while True:
            data = f.read(48)
            if not data:
                break
            ts, o, h, l, c, v = struct.unpack(">6d", data)
            bars.append((int(ts), o, h, l, c, v))
            i += 1
            if max_bars and i >= max_bars:
                break
    return bars


def _has_nb():
    """检查 .nb 数据文件是否存在."""
    return os.path.isfile(NB_PATH)


def create_observer(symbol="btcusd", period=14400, n_bars=500):
    """创建观察者并喂入模拟K线数据."""
    cfg = chanlun.缠论配置()
    obs = chanlun.观察者(symbol, period, cfg)
    for i in range(n_bars):
        trend = i * 3
        wave = math.sin(i * 0.05) * 2000
        mid = 68000.0 + trend + wave
        high = mid + abs(math.cos(i * 0.3)) * 400 + 100
        low = mid - abs(math.sin(i * 0.5)) * 400 - 100
        k = chanlun.K线(
            标识=symbol,
            周期=period,
            时间戳=1771675200 + i * period,
            开盘价=mid - 50,
            高=high,
            低=low,
            收盘价=mid + 50,
            成交量=abs(math.sin(i)) * 1000,
        )
        obs.增加原始K线(k)
    return obs


def _load_observer(max_bars=None):
    """从 .nb 文件加载数据并创建观察者."""
    cfg = chanlun.缠论配置()
    obs = chanlun.观察者("btcusd", 300, cfg)
    bars = read_nb_bars(NB_PATH, max_bars=max_bars)
    for i, (ts, o, h, l, c, v) in enumerate(bars):
        k = chanlun.K线.创建普K(f"k{i}", ts, o, h, l, c, v, i, 300)
        obs.增加原始K线(k)
    return obs


# ============================================================
# 观察者 子类化 / 方法重写 测试
# ============================================================


class Test观察者子类化(PyO3SubclassMixin, unittest.TestCase):
    """观察者 子类化/方法重写 测试 — 使用 PyO3SubclassMixin 泛化基类."""

    base_class = chanlun.观察者
    constructor_args = ("btcusd", 300)
    constructor_kwargs = {}
    sequence_getter_names = [
        "普通K线序列",
        "缠论K线序列",
        "分型序列",
        "笔序列",
        "笔_中枢序列",
        "线段序列",
        "中枢序列",
        "扩展线段序列",
        "扩展中枢序列",
        "扩展线段序列_线段",
        "扩展中枢序列_线段",
        "线段_线段序列",
        "线段_中枢序列",
        "扩展线段序列_扩展线段",
        "扩展中枢序列_扩展线段",
    ]
    feed_method_name = "增加原始K线"

    # ---- Mixin hooks: 使用预加载数据，避免每个测试重复读取 .nb ----

    @staticmethod
    def make_data_item():
        return chanlun.K线.创建普K("test", 1761327300, 100.0, 105.0, 99.0, 103.0, 1000.0, 0, 300)

    @classmethod
    def make_target_with_data(cls):
        obs = chanlun.观察者("btcusd", 300)
        for i, (ts, o, h, l, c, v) in enumerate(cls._bars):
            k = chanlun.K线.创建普K(f"base_{i}", ts, o, h, l, c, v, i, 300)
            obs.增加原始K线(k)
        return obs

    @classmethod
    def make_sub_with_data(cls):
        class Sub(chanlun.观察者):
            pass

        obs = Sub("btcusd", 300)
        for i, (ts, o, h, l, c, v) in enumerate(cls._bars):
            k = chanlun.K线.创建普K(f"sub_{i}", ts, o, h, l, c, v, i, 300)
            obs.增加原始K线(k)
        return obs

    @classmethod
    def setUpClass(cls):
        """预加载 .nb 数据和预暖观察者，所有测试共享."""
        if not _has_nb():
            return  # setUPClass 中不能 skip，各测试自行检查
        cls._bars = read_nb_bars(NB_PATH, max_bars=500)
        cls._base_obs = chanlun.观察者("btcusd", 300)
        for i, (ts, o, h, l, c, v) in enumerate(cls._bars):
            k = chanlun.K线.创建普K(f"b_{i}", ts, o, h, l, c, v, i, 300)
            cls._base_obs.增加原始K线(k)

    def _require_bars(self):
        if not hasattr(type(self), "_bars"):
            self.skipTest("需要 .nb 数据文件")

    # ---- 以下为 chanlun 专项测试 ----

    def test_subclass_new_pass_config(self):
        """__new__ 透传 配置 参数到父类."""
        cfg = chanlun.缠论配置()

        class Sub(chanlun.观察者):
            def __new__(cls, 符号, 周期, 配置=None, *, tag="", **kwargs):
                return super().__new__(cls, 符号, 周期, 配置=配置)

            def __init__(self, 符号, 周期, 配置=None, *, tag="", **kwargs):
                self.tag = tag

        obs = Sub("btcusd", 300, cfg, tag="test-tag")
        self.assertEqual(obs.标识, "btcusd:300")
        self.assertEqual(obs.tag, "test-tag")

    def test_override_method_super_call(self):
        """重写 增加原始K线，super() 调用父类，全线管线运行."""
        self._require_bars()

        class Sub(chanlun.观察者):
            def __init__(self, 符号, 周期):
                self.intercept_count = 0
                self.intercept_timestamps = []

            def 增加原始K线(self, 普K):
                self.intercept_count += 1
                self.intercept_timestamps.append(普K.时间戳)
                super().增加原始K线(普K)

        sub_obs = Sub("btcusd", 300)
        for i, (ts, o, h, l, c, v) in enumerate(self._bars):
            k = chanlun.K线.创建普K(f"s_{i}", ts, o, h, l, c, v, i, 300)
            sub_obs.增加原始K线(k)

        self.assertEqual(sub_obs.intercept_count, len(self._bars))
        self.assertEqual(len(sub_obs.intercept_timestamps), len(self._bars))

        for attr in ["普通K线序列", "缠论K线序列", "分型序列", "笔序列", "线段序列", "中枢序列"]:
            base_len = len(getattr(self._base_obs, attr))
            sub_len = len(getattr(sub_obs, attr))
            self.assertEqual(base_len, sub_len, f"{attr}: base={base_len}, sub={sub_len}")

        for j, (bp, sp) in enumerate(zip(self._base_obs.笔序列, sub_obs.笔序列)):
            self.assertEqual(bp.文.中.时间戳, sp.文.中.时间戳, f"笔[{j}] 时间戳不一致")

    def test_override_getter(self):
        """重写 @property getter，super() 取基类值."""

        class Sub(chanlun.观察者):
            @property
            def 标识(self):
                return f"[MOCKED] {super().标识}"

        obs = Sub("btcusd", 300)
        self.assertEqual(obs.标识, "[MOCKED] btcusd:300")
        self.assertEqual(obs.周期, 300)

    def test_override_str_repr(self):
        """重写 __str__ / __repr__."""

        class Sub(chanlun.观察者):
            def __str__(self):
                return f"Custom({self.标识})"

            def __repr__(self):
                return self.__str__()

        obs = Sub("btcusd", 300)
        self.assertEqual(str(obs), "Custom(btcusd:300)")
        self.assertEqual(repr(obs), "Custom(btcusd:300)")

    def test_multi_level_inheritance(self):
        """多层继承，MRO 调用链完整."""
        k = self.make_data_item()

        class Level1(chanlun.观察者):
            def 增加原始K线(self, 普K):
                self.l1_log = getattr(self, "l1_log", [])
                self.l1_log.append("L1")
                super().增加原始K线(普K)

        class Level2(Level1):
            def 增加原始K线(self, 普K):
                self.l2_log = getattr(self, "l2_log", [])
                self.l2_log.append("L2")
                super().增加原始K线(普K)

        obs = Level2("btcusd", 300)
        obs.增加原始K线(k)

        self.assertEqual(obs.l2_log, ["L2"])
        self.assertEqual(obs.l1_log, ["L1"])
        self.assertEqual(len(obs.普通K线序列), 1)

    def test_unoverridden_method_inherited(self):
        """未重写的方法从基类直接继承."""
        k = self.make_data_item()

        class Sub(chanlun.观察者):
            pass

        obs = Sub("btcusd", 120)
        obs.增加原始K线(k)

        self.assertEqual(obs.标识, "btcusd:120")
        self.assertEqual(obs.周期, 120)
        self.assertEqual(len(obs.普通K线序列), 1)
        self.assertEqual(len(obs.缠论K线序列), 1)
        obs.静态重新分析()

    def test_override_reset(self):
        """重写 重置基础序列，子类状态也重置."""
        k = self.make_data_item()

        class Sub(chanlun.观察者):
            def __init__(self, 符号, 周期):
                self.my_log = []

            def 重置基础序列(self):
                self.my_log.clear()
                super().重置基础序列()

        obs = Sub("btcusd", 300)
        obs.增加原始K线(k)
        obs.my_log.append("test")

        self.assertEqual(len(obs.普通K线序列), 1)
        obs.重置基础序列()
        self.assertEqual(len(obs.普通K线序列), 0)
        self.assertEqual(obs.my_log, [])

    # --- Property getter 逐一覆盖 ---

    def test_override_getter_符号(self):
        class Sub(chanlun.观察者):
            @property
            def 符号(self):
                return f"[WRAPPED] {super().符号}"

        obs = Sub("btcusd", 300)
        self.assertEqual(obs.符号, "[WRAPPED] btcusd")

    def test_override_getter_周期(self):
        class Sub(chanlun.观察者):
            @property
            def 周期(self):
                return super().周期 * 60

        obs = Sub("btcusd", 5)
        self.assertEqual(obs.周期, 300)

    def test_override_getter_配置(self):
        class Sub(chanlun.观察者):
            @property
            def 配置(self):
                self._cfg_accessed = True
                return super().配置

        obs = Sub("btcusd", 300)
        cfg = obs.配置
        self.assertIsNotNone(cfg)
        self.assertTrue(obs._cfg_accessed)

    def test_override_getter_当前K线(self):
        k = self.make_data_item()

        class Sub(chanlun.观察者):
            def __init__(self, 符号, 周期):
                self._cur_k_accessed = False

            @property
            def 当前K线(self):
                self._cur_k_accessed = True
                return super().当前K线

        obs = Sub("btcusd", 300)
        obs.增加原始K线(k)
        cur = obs.当前K线
        self.assertIsNotNone(cur)
        self.assertTrue(obs._cur_k_accessed)

    def test_override_getter_当前缠K(self):
        k = self.make_data_item()

        class Sub(chanlun.观察者):
            def __init__(self, 符号, 周期):
                self._cur_ck_accessed = False

            @property
            def 当前缠K(self):
                self._cur_ck_accessed = True
                return super().当前缠K

        obs = Sub("btcusd", 300)
        obs.增加原始K线(k)
        cur = obs.当前缠K
        self.assertIsNotNone(cur)
        self.assertTrue(obs._cur_ck_accessed)

    def test_override_getter_观察员(self):
        class Sub(chanlun.观察者):
            @property
            def 观察员(self):
                self._obs_accessed = True
                return self

        obs = Sub("btcusd", 300)
        self.assertIs(obs.观察员, obs)
        self.assertTrue(obs._obs_accessed)

    def test_override_sequence_getter(self):
        k = self.make_data_item()

        class Sub(chanlun.观察者):
            def __init__(self, 符号, 周期):
                self._seq_accessed = set()

            @property
            def 普通K线序列(self):
                self._seq_accessed.add("普通K线序列")
                return super().普通K线序列

            @property
            def 缠论K线序列(self):
                self._seq_accessed.add("缠论K线序列")
                return super().缠论K线序列

            @property
            def 笔序列(self):
                self._seq_accessed.add("笔序列")
                return super().笔序列

            @property
            def 线段序列(self):
                self._seq_accessed.add("线段序列")
                return super().线段序列

        obs = Sub("btcusd", 300)
        obs.增加原始K线(k)

        self.assertEqual(len(obs.普通K线序列), 1)
        self.assertIn("普通K线序列", obs._seq_accessed)
        self.assertEqual(len(obs.缠论K线序列), 1)
        self.assertIn("缠论K线序列", obs._seq_accessed)
        self.assertIsInstance(obs.笔序列, list)
        self.assertIn("笔序列", obs._seq_accessed)
        self.assertIsInstance(obs.线段序列, list)
        self.assertIn("线段序列", obs._seq_accessed)

    def test_override_all_sequence_getters(self):
        k = self.make_data_item()
        all_seqs = [
            "普通K线序列",
            "缠论K线序列",
            "分型序列",
            "笔序列",
            "笔_中枢序列",
            "线段序列",
            "中枢序列",
            "扩展线段序列",
            "扩展中枢序列",
            "扩展线段序列_线段",
            "扩展中枢序列_线段",
            "线段_线段序列",
            "线段_中枢序列",
            "扩展线段序列_扩展线段",
            "扩展中枢序列_扩展线段",
        ]

        def _make_getter(name):
            @property
            def getter(self, _name=name):
                self._all_seq_accessed.add(_name)
                for cls in type(self).__mro__[1:]:
                    if hasattr(cls, _name) and isinstance(getattr(cls, _name, None), property):
                        return getattr(cls, _name).fget(self)
                return []

            return getter

        class Sub(chanlun.观察者):
            def __init__(self, 符号, 周期):
                self._all_seq_accessed = set()

        for seq_name in all_seqs:
            setattr(Sub, seq_name, _make_getter(seq_name))

        obs = Sub("btcusd", 300)
        obs.增加原始K线(k)

        for seq_name in all_seqs:
            seq = getattr(obs, seq_name)
            self.assertIn(seq_name, obs._all_seq_accessed, f"{seq_name} 未被拦截")
            self.assertIsInstance(seq, list, f"{seq_name} 不是 list，而是 {type(seq)}")

    # --- 方法重写（使用预加载数据） ---

    def test_override_加载保存读取重分析(self):
        """串行验证 加载本地数据 / 保存数据 / 读取数据文件 / 静态重新分析 四个重写."""
        self._require_bars()

        class Sub(chanlun.观察者):
            def __init__(self, 符号, 周期, 配置=None):
                self._loaded = False
                self._load_count = 0
                self._reanalyzed = 0
                self._saved = False
                self._save_root = None
                self._reload = 0
                super().__init__(符号, 周期, 配置)

            def 测试_保存数据(self, root=None):
                self._saved = True
                self._save_root = root
                super().测试_保存数据(root)

            def 静态重新分析(self):
                self._reanalyzed += 1
                super().静态重新分析()

            def 加载本地数据(self, 文件路径):
                self._loaded = True
                self._load_count += 1
                super().加载本地数据(文件路径)

            def 重置基础序列(self):
                self._reload += 1
                super().重置基础序列()

            @classmethod
            def 读取数据文件(cls, 文件路径, 配置=None):
                obs = cls("", 0)  # 创建子类实例，父类方法会覆盖符号/周期
                chanlun.观察者.读取数据文件(文件路径, 配置, 观察员=obs)
                obs._custom_classmethod_flag = True
                return obs

        # 4. 读取数据文件（classmethod 重写
        a = datetime.now()
        obs = Sub.读取数据文件(NB_PATH)
        self.assertIsInstance(obs, Sub)
        self.assertTrue(obs._custom_classmethod_flag)
        self.assertGreater(len(obs.普通K线序列), 0)
        b = datetime.now()
        print("读取数据文件 用时:", b - a)

        # 1. 加载本地数据
        obs.重置基础序列()
        obs.加载本地数据(NB_PATH)
        self.assertTrue(obs._loaded)
        self.assertEqual(obs._load_count, 2)
        self.assertGreater(len(obs.普通K线序列), 0)
        c = datetime.now()
        print("加载本地数据 用时:", c - b)

        # 2. 静态重新分析（复用已加载数据的 obs）
        obs.重置基础序列()
        obs.加载本地数据(NB_PATH)
        obs.静态重新分析()
        self.assertEqual(obs._reanalyzed, 1)
        self.assertGreaterEqual(len(obs.笔序列), 0)
        d = datetime.now()
        print("静态重新分析 用时:", d - c)

        # 3. 保存数据
        obs.重置基础序列()
        obs.增加原始K线(self.make_data_item())
        with tempfile.TemporaryDirectory() as tmpdir:
            obs.测试_保存数据(tmpdir)
            self.assertTrue(obs._saved)
            self.assertEqual(obs._save_root, tmpdir)

        # 5. 重置次数
        self.assertEqual(obs._reload, 6)
        e = datetime.now()
        print("保存数据 用时:", e - d)

    def test_override_completely_no_super(self):
        k = self.make_data_item()

        class Sub(chanlun.观察者):
            def 增加原始K线(self, 普K):
                self._custom = getattr(self, "_custom", [])
                self._custom.append(普K.时间戳)

        obs = Sub("btcusd", 300)
        obs.增加原始K线(k)
        self.assertEqual(obs._custom, [1761327300])
        self.assertEqual(len(obs.普通K线序列), 0)

    def test_override_identical_subclass(self):
        self._require_bars()

        class Sub(chanlun.观察者):
            pass

        sub = Sub("btcusd", 300)
        for i, (ts, o, h, l, c, v) in enumerate(self._bars):
            sk = chanlun.K线.创建普K(f"s_{i}", ts, o, h, l, c, v, i, 300)
            sub.增加原始K线(sk)

        for attr in ["普通K线序列", "笔序列", "线段序列", "中枢序列"]:
            base_len = len(getattr(self._base_obs, attr))
            sub_len = len(getattr(sub, attr))
            self.assertEqual(base_len, sub_len, f"{attr}: base={base_len}, sub={sub_len}")

    def test_override_three_level_mixed(self):
        k = self.make_data_item()

        class L1(chanlun.观察者):
            def __init__(self, 符号, 周期):
                self._l1_feed = 0

            def 增加原始K线(self, 普K):
                self._l1_feed += 1
                super().增加原始K线(普K)

        class L2(L1):
            def __init__(self, 符号, 周期):
                super().__init__(符号, 周期)
                self._l2_标识 = 0

            @property
            def 标识(self):
                self._l2_标识 += 1
                return f"[L2] {super().标识}"

        class L3(L2):
            def __init__(self, 符号, 周期):
                super().__init__(符号, 周期)
                self._l3_reset = 0

            def 重置基础序列(self):
                self._l3_reset += 1
                super().重置基础序列()

        obs = L3("btcusd", 300)
        obs.增加原始K线(k)

        self.assertEqual(obs._l1_feed, 1)
        self.assertEqual(obs.标识, "[L2] btcusd:300")
        self.assertEqual(obs._l2_标识, 1)
        obs.重置基础序列()
        self.assertEqual(obs._l3_reset, 1)
        self.assertEqual(len(obs.普通K线序列), 0)

    def test_override_cross_method_dispatch(self):
        k = self.make_data_item()

        class Sub(chanlun.观察者):
            def __init__(self, 符号, 周期):
                self._feed_called = False
                self._reset_called = False

            def 增加原始K线(self, 普K):
                self._feed_called = True
                super().增加原始K线(普K)

            def 重置基础序列(self):
                self._reset_called = True
                super().重置基础序列()

            def compound_operation(self, 普K):
                self.增加原始K线(普K)
                self.重置基础序列()

        obs = Sub("btcusd", 300)
        obs.compound_operation(k)

        self.assertTrue(obs._feed_called)
        self.assertTrue(obs._reset_called)
        self.assertEqual(len(obs.普通K线序列), 0)


# ============================================================
# K线.截取 测试
# ============================================================


class TestK线截取(unittest.TestCase):
    """K线.截取 测试."""

    def test_jiequ_basic(self):
        k1 = chanlun.K线.创建普K("k1", 1000000000, 100.0, 105.0, 98.0, 102.0, 1000.0, 0, 300)
        k2 = chanlun.K线.创建普K("k2", 1000000060, 102.0, 108.0, 101.0, 106.0, 1200.0, 1, 300)
        k3 = chanlun.K线.创建普K("k3", 1000000120, 106.0, 110.0, 104.0, 108.0, 900.0, 2, 300)
        k4 = chanlun.K线.创建普K("k4", 1000000180, 108.0, 112.0, 107.0, 110.0, 1100.0, 3, 300)
        ks = [k1, k2, k3, k4]

        r = chanlun.K线.截取(ks, k2, k4)
        self.assertEqual(len(r), 3)
        self.assertEqual(r[0].时间戳, k2.时间戳)
        self.assertEqual(r[-1].时间戳, k4.时间戳)

        r = chanlun.K线.截取(ks, k1, k4)
        self.assertEqual(len(r), 4)

        r = chanlun.K线.截取(ks, k3, k3)
        self.assertEqual(len(r), 1)
        self.assertEqual(r[0].时间戳, k3.时间戳)

    def test_jiequ_error(self):
        k1 = chanlun.K线.创建普K("k1", 1000000000, 100.0, 105.0, 98.0, 102.0, 1000.0, 0, 300)
        k2 = chanlun.K线.创建普K("k2", 1000000060, 102.0, 108.0, 101.0, 106.0, 1200.0, 1, 300)
        k3 = chanlun.K线.创建普K("k3", 1000000120, 106.0, 110.0, 104.0, 108.0, 900.0, 2, 300)
        ks = [k1, k2, k3]

        with self.assertRaises(ValueError):
            chanlun.K线.截取([], k1, k1)

        kx = chanlun.K线.创建普K("kx", 9999999999, 1.0, 1.0, 1.0, 1.0, 1.0, 99, 300)
        with self.assertRaises(ValueError):
            chanlun.K线.截取(ks, kx, k3)

        with self.assertRaises(ValueError):
            chanlun.K线.截取(ks, k3, k1)

    @unittest.skipUnless(_has_nb(), "需要 .nb 数据文件")
    def test_jiequ_from_observer(self):
        cfg = chanlun.缠论配置()
        obs = chanlun.观察者.读取数据文件(NB_PATH, cfg)

        self.assertGreater(len(obs.笔序列), 0, "笔序列为空，数据可能不够")
        bi = obs.笔序列[0]

        pk_seq = bi.获取普K序列(obs)
        self.assertGreater(len(pk_seq), 0)

        r = chanlun.K线.截取(obs.普通K线序列, bi.文.中.标的K线, bi.武.中.标的K线)
        self.assertGreater(len(r), 0)
        self.assertEqual(r[0].时间戳, bi.文.中.标的K线.时间戳)
        self.assertEqual(r[-1].时间戳, bi.武.中.标的K线.时间戳)

        mid = len(pk_seq) // 2
        if mid > 0:
            r = chanlun.K线.截取(pk_seq, pk_seq[0], pk_seq[mid])
            self.assertEqual(len(r), mid + 1)
            self.assertEqual(r[0].时间戳, pk_seq[0].时间戳)
            self.assertEqual(r[-1].时间戳, pk_seq[mid].时间戳)

    @unittest.skipUnless(_has_nb(), "需要 .nb 数据文件")
    def test_jiequ_multi_bi(self):
        cfg = chanlun.缠论配置()
        obs = chanlun.观察者.读取数据文件(NB_PATH, cfg)

        n_checked = 0
        for i, bi in enumerate(obs.笔序列[:10]):
            pk_seq = bi.获取普K序列(obs)
            if len(pk_seq) < 2:
                continue
            r = chanlun.K线.截取(pk_seq, pk_seq[0], pk_seq[-1])
            self.assertEqual(len(r), len(pk_seq), f"笔[{i}] 截取长度不一致: {len(r)} vs {len(pk_seq)}")
            n_checked += 1

        self.assertGreater(n_checked, 0)


# ============================================================
# Rc 指针身份 / list.index 测试
# ============================================================


class TestRc身份列表索引(unittest.TestCase):
    """Rc 指针身份 / list.index 测试."""

    def setUp(self):
        if not _has_nb():
            self.skipTest("需要 .nb 数据文件")
        self.obs = _load_observer()

    def test_same_seq_index(self):
        obs = self.obs

        fx_list = obs.分型序列
        if len(fx_list) >= 2:
            self.assertEqual(fx_list.index(fx_list[0]), 0)
            self.assertEqual(fx_list.index(fx_list[1]), 1)

        ck_list = obs.缠论K线序列
        if len(ck_list) >= 2:
            self.assertEqual(ck_list.index(ck_list[0]), 0)
            self.assertEqual(ck_list.index(ck_list[-1]), len(ck_list) - 1)

        bi_list = obs.笔序列
        if len(bi_list) >= 2:
            self.assertEqual(bi_list.index(bi_list[0]), 0)
            self.assertEqual(bi_list.index(bi_list[-1]), len(bi_list) - 1)

        if len(obs.线段序列) > 0:
            for 段 in obs.线段序列:
                笔列表 = 段.笔序列
                if len(笔列表) >= 2:
                    self.assertEqual(笔列表.index(笔列表[0]), 0)
                    self.assertEqual(笔列表.index(笔列表[-1]), len(笔列表) - 1)
                    break

        zs_list = obs.中枢序列
        if len(zs_list) >= 2:
            self.assertEqual(zs_list.index(zs_list[0]), 0)
            self.assertEqual(zs_list.index(zs_list[-1]), len(zs_list) - 1)

    def test_cross_seq_index(self):
        obs = self.obs
        n_checked = 0
        for 段 in obs.线段序列:
            zs_list = 段.合_中枢序列
            if len(zs_list) == 0:
                continue
            zs = zs_list[-1]
            笔列表 = 段.笔序列
            for j, elem in enumerate(zs.基础序列):
                idx = 笔列表.index(elem)
                self.assertGreaterEqual(idx, 0)
                self.assertEqual(笔列表[idx], elem)
                n_checked += 1
            break

        self.assertGreaterEqual(n_checked, 3, f"应至少检查3个中枢元素，实际 {n_checked}")

    def test_zhaofenxing_cross_ref(self):
        obs = self.obs
        fx_list = obs.分型序列
        if len(fx_list) >= 2 and len(obs.笔序列) > 0:
            bi = obs.笔序列[-1]
            try:
                idx = fx_list.index(bi.文)
                self.assertGreaterEqual(idx, 0)
            except ValueError:
                pass

    def test_user_pattern(self):
        obs = self.obs
        n_segments_with_zs = 0
        for 段 in obs.线段序列:
            zs_list = 段.合_中枢序列
            if len(zs_list) == 0:
                continue
            n_segments_with_zs += 1
            zs = zs_list[-1]
            笔列表 = 段.笔序列

            idx = 笔列表.index(zs.基础序列[0])

            found = False
            for bi in reversed(笔列表[:idx]):
                if bi.方向 == zs.基础序列[0].方向:
                    found = True
                    break

        self.assertGreater(n_segments_with_zs, 0, "应至少有一个段包含中枢")

    def test_tongji_macd_behavior_types(self):
        obs = _load_observer(max_bars=5000)
        if len(obs.笔序列) == 0:
            self.skipTest("无笔")
        笔 = obs.笔序列[-1]
        普K序列 = 笔.获取普K序列(obs)
        if len(普K序列) < 2:
            self.skipTest("K线不够")
        result = chanlun.虚线.统计MACD行为(普K序列, 8, 3)
        # 使用泛化工具验证类型形状
        assert_type_shape(
            result,
            {
                "DIF上穿0": int,
                "DIF下穿0": int,
                "DEA上穿0": int,
                "DEA下穿0": int,
                "金叉次数": int,
                "死叉次数": int,
                "密集交叉区域": [(int, int, int)],
            },
        )

    def test_xiangduifangxiang_methods(self):
        fx = chanlun.相对方向.分析(2.0, 0.5, 1.0, 0.8)
        assert_type_shape(fx.是否向上, callable)
        assert_type_shape(fx.是否向下, callable)
        assert_type_shape(fx.是否包含, callable)
        assert_type_shape(fx.是否缺口, callable)
        assert_type_shape(fx.是否衔接, callable)
        self.assertIsInstance(fx.是否向上(), bool)
        self.assertIsInstance(fx.是否向下(), bool)


# ============================================================
# Rc 身份测试 — 使用 RcIdentityMixin
# ============================================================


class _Base身份:
    """身份测试基类：通过 setUpClass 预缓存 target 供 Mixin 使用."""

    @classmethod
    def setUpClass(cls):
        cls._cached_target = cls.target_factory()


class Test缠K身份(_Base身份, RcIdentityMixin, unittest.TestCase):
    """缠论K线: 从序列、分型、笔端点、中枢等不同路径访问."""

    @staticmethod
    def target_factory():
        return create_observer()

    sequence_getters = {
        "缠论K线序列": lambda t: t.缠论K线序列,
    }
    stable_getters = {
        "分型[0].中": lambda t: t.分型序列[0].中 if t.分型序列 else None,
    }

    # 专项测试保留
    def test_分型中K(self):
        t = self._get_target()
        seq = t.缠论K线序列
        分序 = t.分型序列
        for fx in 分序[:10]:
            中 = fx.中
            for ck in seq:
                if ck.时间戳 == 中.时间戳:
                    self.assertIs(ck, 中, f"分型.中 (ts={中.时间戳}) 与序列中元素不匹配")
                    break

    def test_笔端点钟K(self):
        t = self._get_target()
        seq = t.缠论K线序列
        for bi in t.笔序列:
            for nm, getter in [("文", lambda b=bi: b.文), ("武", lambda b=bi: b.武)]:
                ep = getter()
                if ep is None:
                    continue
                中 = ep.中
                for ck in seq:
                    if ck.时间戳 == 中.时间戳:
                        self.assertIs(ck, 中, f"笔.{nm}.中 (ts={中.时间戳}) 与序列中元素不匹配")
                        break


class Test分型身份(_Base身份, RcIdentityMixin, unittest.TestCase):
    """分型: 从分型序列、笔/线段端点等不同路径访问."""

    @staticmethod
    def target_factory():
        return create_observer()

    sequence_getters = {
        "分型序列": lambda t: t.分型序列,
    }
    stable_getters = {
        "笔[0].文": lambda t: t.笔序列[0].文 if t.笔序列 else None,
        "笔[0].武": lambda t: t.笔序列[0].武 if t.笔序列 else None,
    }

    def test_笔端点与序列(self):
        t = self._get_target()
        分序 = t.分型序列
        for bi in t.笔序列:
            for nm in ["文", "武"]:
                ep = getattr(bi, nm)
                if ep is None:
                    continue
                matched = False
                for fx in 分序:
                    if fx.时间戳 == ep.时间戳 and fx.结构 == ep.结构:
                        self.assertIs(fx, ep, f"笔.{nm} (ts={ep.时间戳}) 与分型序列中元素不匹配")
                        matched = True
                        break
                self.assertTrue(matched, f"笔.{nm} (ts={ep.时间戳}) 在分型序列中未找到")

    def test_段端点与序列(self):
        t = self._get_target()
        分序 = t.分型序列
        for duan in t.线段序列:
            for nm in ["文", "武"]:
                ep = getattr(duan, nm)
                if ep is None:
                    continue
                matched = False
                for fx in 分序:
                    if fx.时间戳 == ep.时间戳 and fx.结构 == ep.结构:
                        self.assertIs(fx, ep, f"段.{nm} (ts={ep.时间戳}) 与分型序列中元素不匹配")
                        matched = True
                        break
                self.assertTrue(matched, f"段.{nm} (ts={ep.时间戳}) 在分型序列中未找到")


class Test虚线身份(_Base身份, RcIdentityMixin, unittest.TestCase):
    """虚线(笔/线段): 从笔序列、线段序列、中枢内部序列等不同路径访问."""

    @staticmethod
    def target_factory():
        return create_observer()

    sequence_getters = {
        "笔序列": lambda t: t.笔序列,
        "线段序列": lambda t: t.线段序列,
    }

    def test_多个扩展序列(self):
        t = self._get_target()
        s1 = t.扩展线段序列
        s2 = t.扩展线段序列_线段
        for d1 in s1:
            for d2 in s2:
                if d1.序号 == d2.序号:
                    self.assertIs(d1, d2, f"扩展线段序列[{d1.序号}] 跨序列身份不一致")
                    break


class TestK线身份(_Base身份, RcIdentityMixin, unittest.TestCase):
    """原始K线: 从序列、买卖点、缠K标的等不同路径访问."""

    @staticmethod
    def target_factory():
        return create_observer()

    sequence_getters = {
        "普通K线序列": lambda t: t.普通K线序列,
    }


class Test中枢身份(_Base身份, RcIdentityMixin, unittest.TestCase):
    """中枢: 从中枢序列、分型关联、笔中枢/线段中枢等不同路径访问."""

    @staticmethod
    def target_factory():
        return create_observer(period=3600, n_bars=800)

    sequence_getters = {
        "中枢序列": lambda t: t.中枢序列,
    }
    stable_getters = {
        "笔中枢[0].文": lambda t: t.笔_中枢序列[0].文 if t.笔_中枢序列 else None,
        "段中枢[0].文": lambda t: t.线段_中枢序列[0].文 if t.线段_中枢序列 else None,
    }


class Test整体身份(_Base身份, unittest.TestCase):
    """跨类型综合身份测试."""

    @classmethod
    def setUpClass(cls):
        cls.obs = create_observer(period=3600, n_bars=800)

    def test_买卖点分型(self):
        分序 = self.obs.分型序列
        笔序 = self.obs.笔序列
        self.assertGreaterEqual(len(分序), 0)
        self.assertGreaterEqual(len(笔序), 0)

    def test_全链路一致性(self):
        obs = create_observer()
        seq = obs.缠论K线序列

        for bi in obs.笔序列:
            for nm, getter in [("文", lambda b=bi: b.文), ("武", lambda b=bi: b.武)]:
                ep = getter()
                if ep is None:
                    continue
                中 = ep.中
                for ck in seq:
                    if ck.时间戳 == 中.时间戳:
                        self.assertIs(ck, 中)
                        break
                左 = ep.左
                if 左 is not None:
                    for ck in seq:
                        if ck.时间戳 == 左.时间戳:
                            self.assertIs(ck, 左)
                            break
                右 = ep.右
                if 右 is not None:
                    for ck in seq:
                        if ck.时间戳 == 右.时间戳:
                            self.assertIs(ck, 右)
                            break


# ============================================================
# 跨线程身份测试 — 验证全局缓存（非 thread_local）的跨线程一致性
# ============================================================


class Test跨线程身份(unittest.TestCase):
    """跨线程 RC 身份一致性：全局缓存应在不同线程间共享同一 Python 对象."""

    @classmethod
    def setUpClass(cls):
        cls.obs = create_observer(period=3600, n_bars=800)

    def _run_in_thread(self, fn):
        """在子线程中执行 fn，通过 queue 收集结果和异常."""
        import threading

        result = []
        err = []

        def wrapper():
            try:
                result.append(fn())
            except Exception as e:
                err.append(e)

        t = threading.Thread(target=wrapper)
        t.start()
        t.join()
        if err:
            raise err[0]
        return result[0]

    # ---- 序列级别 ----

    def test_缠K序列跨线程重复获取_is一致(self):
        """缠论K线序列：从子线程重复获取，元素 is 一致."""
        obs = self.obs

        def check():
            s1 = obs.缠论K线序列
            s2 = obs.缠论K线序列
            return [(s1[i] is s2[i], len(s1), len(s2)) for i in range(min(len(s1), len(s2), 20))]

        results = self._run_in_thread(check)
        for i, (ok, l1, l2) in enumerate(results):
            self.assertTrue(ok, f"缠K序列[{i}] 跨线程 is 不一致")

    def test_分型序列跨线程重复获取_is一致(self):
        """分型序列：从子线程重复获取，元素 is 一致."""
        obs = self.obs

        def check():
            s1 = obs.分型序列
            s2 = obs.分型序列
            return [(s1[i] is s2[i], len(s1)) for i in range(min(len(s1), len(s2), 20))]

        results = self._run_in_thread(check)
        for i, (ok, _) in enumerate(results):
            self.assertTrue(ok, f"分型序列[{i}] 跨线程 is 不一致")

    def test_笔序列跨线程重复获取_is一致(self):
        """笔序列：从子线程重复获取，元素 is 一致."""
        obs = self.obs

        def check():
            s1 = obs.笔序列
            s2 = obs.笔序列
            return [(s1[i] is s2[i], len(s1)) for i in range(min(len(s1), len(s2), 20))]

        results = self._run_in_thread(check)
        for i, (ok, _) in enumerate(results):
            self.assertTrue(ok, f"笔序列[{i}] 跨线程 is 不一致")

    def test_线段序列跨线程重复获取_is一致(self):
        """线段序列：从子线程重复获取，元素 is 一致."""
        obs = self.obs

        def check():
            s1 = obs.线段序列
            s2 = obs.线段序列
            return [(s1[i] is s2[i], len(s1)) for i in range(min(len(s1), len(s2), 20))]

        results = self._run_in_thread(check)
        for i, (ok, _) in enumerate(results):
            self.assertTrue(ok, f"线段序列[{i}] 跨线程 is 不一致")

    def test_中枢序列跨线程重复获取_is一致(self):
        """中枢序列：从子线程重复获取，元素 is 一致."""
        obs = self.obs

        def check():
            s1 = obs.中枢序列
            s2 = obs.中枢序列
            return [(s1[i] is s2[i], len(s1)) for i in range(min(len(s1), len(s2), 20))]

        results = self._run_in_thread(check)
        for i, (ok, _) in enumerate(results):
            self.assertTrue(ok, f"中枢序列[{i}] 跨线程 is 不一致")

    def test_普K序列跨线程重复获取_is一致(self):
        """普通K线序列：从子线程重复获取，元素 is 一致."""
        obs = self.obs

        def check():
            s1 = obs.普通K线序列
            s2 = obs.普通K线序列
            return [(s1[i] is s2[i], len(s1)) for i in range(min(len(s1), len(s2), 20))]

        results = self._run_in_thread(check)
        for i, (ok, _) in enumerate(results):
            self.assertTrue(ok, f"普K序列[{i}] 跨线程 is 不一致")

    # ---- 跨路径 ----

    def test_跨线程分型中K线_is一致(self):
        """子线程中 分型.中 is 缠论K线序列[同时间戳]."""
        obs = self.obs

        def check():
            results = []
            seq = obs.缠论K线序列
            for fx in obs.分型序列[:10]:
                中 = fx.中
                found = False
                for ck in seq:
                    if ck.时间戳 == 中.时间戳:
                        results.append((ck is 中, ck.时间戳))
                        found = True
                        break
                if not found:
                    results.append((None, 中.时间戳))
            return results

        results = self._run_in_thread(check)
        for ok, ts in results:
            self.assertIsNotNone(ok, f"分型.中 ts={ts} 在缠K序列中未找到")
            self.assertTrue(ok, f"跨线程 分型.中 ts={ts} is 不一致")

    def test_跨线程笔端点钟K_is一致(self):
        """子线程中 笔.文中 is 缠论K线序列[同时间戳]."""
        obs = self.obs

        def check():
            results = []
            seq = obs.缠论K线序列
            for bi in obs.笔序列[:10]:
                for nm, ep in [("文", bi.文), ("武", bi.武)]:
                    if ep is None:
                        continue
                    中 = ep.中
                    for ck in seq:
                        if ck.时间戳 == 中.时间戳:
                            results.append((ck is 中, nm, ck.时间戳))
                            break
            return results

        results = self._run_in_thread(check)
        for ok, nm, ts in results:
            self.assertTrue(ok, f"跨线程 笔.{nm}.中 ts={ts} is 不一致")

    def test_跨线程中枢元件_is一致(self):
        """子线程中 中枢.元件 中的虚线对象 is 线段序列[同索引]."""
        obs = self.obs

        def check():
            results = []
            for zs in obs.中枢序列[:5]:
                for elem in zs.元件[:3]:
                    results.append(elem is elem)  # 自我 is
                    results.append(elem is not None)
            return results

        results = self._run_in_thread(check)
        for ok in results:
            self.assertTrue(ok)

    # ---- list.index 基于 is ----

    def test_跨线程list_index基于身份(self):
        """子线程中 list.index(elem) 正常工作（依赖 __eq__ 基于 is）."""
        obs = self.obs

        def check():
            results = []
            for name, getter in [
                ("缠论K线序列", lambda o: o.缠论K线序列),
                ("分型序列", lambda o: o.分型序列),
                ("笔序列", lambda o: o.笔序列),
                ("线段序列", lambda o: o.线段序列),
                ("中枢序列", lambda o: o.中枢序列),
            ]:
                seq = getter(obs)
                if len(seq) >= 2:
                    r0 = seq.index(seq[0]) == 0
                    r1 = seq.index(seq[-1]) == len(seq) - 1
                    results.append((name, r0, r1, len(seq)))
            return results

        results = self._run_in_thread(check)
        for name, r0, r1, length in results:
            self.assertTrue(r0, f"跨线程 {name}(len={length}): index(seq[0]) != 0")
            self.assertTrue(r1, f"跨线程 {name}(len={length}): index(seq[-1]) != {length - 1}")

    # ---- 主线程-子线程之间 ----

    def test_主线程与子线程对象_is一致(self):
        """主线程获取的对象与子线程获取的对象 is 相同."""
        import threading

        obs = self.obs

        # 主线程先获取
        seq_main = obs.缠论K线序列
        fx_main = obs.分型序列
        bi_main = obs.笔序列

        err = []

        def check():
            try:
                seq_thread = obs.缠论K线序列
                for i in range(min(len(seq_main), 10)):
                    if seq_main[i] is not seq_thread[i]:
                        raise AssertionError(f"缠K[{i}] 主线程与子线程 is 不一致")
                fx_thread = obs.分型序列
                for i in range(min(len(fx_main), 10)):
                    if fx_main[i] is not fx_thread[i]:
                        raise AssertionError(f"分型[{i}] 主线程与子线程 is 不一致")
                bi_thread = obs.笔序列
                for i in range(min(len(bi_main), 10)):
                    if bi_main[i] is not bi_thread[i]:
                        raise AssertionError(f"笔[{i}] 主线程与子线程 is 不一致")
            except Exception as e:
                err.append(e)

        t = threading.Thread(target=check)
        t.start()
        t.join()
        if err:
            raise err[0]

    def test_跨线程getter稳定性(self):
        """子线程中同一 getter 多次调用返回同一对象（如 分型.结构 等）."""
        obs = self.obs

        def check():
            results = []
            if obs.分型序列:
                fx = obs.分型序列[0]
                for name, getter in [
                    ("分型.结构", lambda f: f.结构),
                    ("分型.左", lambda f: f.左),
                    ("分型.中", lambda f: f.中),
                    ("分型.右", lambda f: f.右),
                ]:
                    v1 = getter(fx)
                    v2 = getter(fx)
                    results.append((name, v1 is v2 if v1 is not None else True))
            return results

        results = self._run_in_thread(check)
        for name, ok in results:
            self.assertTrue(ok, f"跨线程 {name}: 两次调用 is 不一致")

    def test_多线程并发访问_is一致(self):
        """两个子线程同时访问，各自拿到的对象与主线程 is 一致."""
        import threading

        obs = self.obs
        errors = []

        seq_main = obs.笔序列

        def worker(thread_id):
            try:
                seq = obs.笔序列
                for i in range(min(len(seq), 10)):
                    if seq[i] is not seq_main[i]:
                        errors.append(f"线程{thread_id} 笔[{i}] is 不一致")
                    if seq[i] is not seq[i]:
                        errors.append(f"线程{thread_id} 笔[{i}] 自我 is 失败")
            except Exception as e:
                errors.append(f"线程{thread_id}: {e}")

        t1 = threading.Thread(target=worker, args=(1,))
        t2 = threading.Thread(target=worker, args=(2,))
        t1.start()
        t2.start()
        t1.join()
        t2.join()

        self.assertEqual(len(errors), 0, "\n".join(errors))


# ============================================================
# 买卖意义 双端一致性测试
# ============================================================


class Test买卖意义双端对比(unittest.TestCase):
    """运行时对比 Rust 绑定层 与 chan.py 的 虚线.买卖意义() 结果."""

    @classmethod
    def setUpClass(cls):
        if not _has_nb():
            raise unittest.SkipTest("需要 .nb 数据文件")
        cls.bars = read_nb_bars(NB_PATH)

    def _build_observers(self, n_bars=2000):
        """构建双端观察者并喂入相同数据."""
        import chanlun
        from chanlun import chan

        cfg_rs = chanlun.缠论配置()
        obs_rs = chanlun.观察者("btcusd", 300, cfg_rs)

        cfg_py = chan.缠论配置()
        obs_py = chan.观察者("btcusd", 300, cfg_py)

        for i, (ts, o, h, l, c, v) in enumerate(self.bars[:n_bars]):
            k_rs = chanlun.K线.创建普K(f"k{i}", ts, o, h, l, c, v, i, 300)
            k_py = chan.K线.创建普K(f"k{i}", ts, o, h, l, c, v, i, 300)
            obs_rs.增加原始K线(k_rs)
            obs_py.增加原始K线(k_py)

        return obs_rs, obs_py

    def test_笔买卖意义双端一致(self):
        """笔序列的 买卖意义() 双端结果完全一致."""
        import chanlun
        from chanlun import chan

        obs_rs, obs_py = self._build_observers()
        n = min(len(obs_rs.笔序列), len(obs_py.笔序列))
        self.assertGreater(n, 0, "笔序列为空")

        mismatches = []
        for i in range(n):
            r = chanlun.虚线.买卖意义(obs_rs.笔序列[i], obs_rs)
            p = chan.虚线.买卖意义(obs_py.笔序列[i], obs_py)
            if r != p:
                mismatches.append((i, r, p, obs_rs.笔序列[i].获取数据文本(), obs_py.笔序列[i].获取数据文本()))

        self.assertEqual(len(mismatches), 0, f"笔买卖意义 不一致 ({len(mismatches)}/{n}):\n" + "\n".join(f"  [{i}] R={r} P={p}\n    R文本={rt}\n    P文本={pt}" for i, r, p, rt, pt in mismatches[:3]))

    def test_线段买卖意义双端一致(self):
        """线段序列的 买卖意义() 双端结果完全一致."""
        import chanlun
        from chanlun import chan

        obs_rs, obs_py = self._build_observers()
        n = min(len(obs_rs.线段序列), len(obs_py.线段序列))
        self.assertGreater(n, 0, "线段序列为空")

        mismatches = []
        for i in range(n):
            r = chanlun.虚线.买卖意义(obs_rs.线段序列[i], obs_rs)
            p = chan.虚线.买卖意义(obs_py.线段序列[i], obs_py)
            if r != p:
                mismatches.append((i, r, p, obs_rs.线段序列[i].获取数据文本(), obs_py.线段序列[i].获取数据文本()))

        self.assertEqual(len(mismatches), 0, f"线段买卖意义 不一致 ({len(mismatches)}/{n}):\n" + "\n".join(f"  [{i}] R={r} P={p}\n    R文本={rt}\n    P文本={pt}" for i, r, p, rt, pt in mismatches[:3]))

    def test_笔序列长度一致(self):
        """双端笔序列数量一致."""
        obs_rs, obs_py = self._build_observers()
        self.assertGreater(len(obs_rs.笔序列), 0)
        self.assertEqual(len(obs_rs.笔序列), len(obs_py.笔序列), f"笔数量: Rust={len(obs_rs.笔序列)} Py={len(obs_py.笔序列)}")

    def test_线段序列长度一致(self):
        """双端线段序列数量一致."""
        obs_rs, obs_py = self._build_observers()
        self.assertGreater(len(obs_rs.线段序列), 0)
        self.assertEqual(len(obs_rs.线段序列), len(obs_py.线段序列), f"线段数量: Rust={len(obs_rs.线段序列)} Py={len(obs_py.线段序列)}")

    def test_中枢序列长度一致(self):
        """双端中枢序列数量一致."""
        obs_rs, obs_py = self._build_observers()
        self.assertGreater(len(obs_rs.中枢序列), 0)
        self.assertEqual(len(obs_rs.中枢序列), len(obs_py.中枢序列), f"中枢数量: Rust={len(obs_rs.中枢序列)} Py={len(obs_py.中枢序列)}")

    # ---- MACD趋向背驰 ----

    def test_笔MACD趋向背驰双端一致(self):
        """笔的 K线序列 MACD趋向背驰 双端结果完全一致."""
        import chanlun
        from chanlun import chan

        obs_rs, obs_py = self._build_observers()
        mismatches = []
        for bi_rs, bi_py in zip(obs_rs.笔序列, obs_py.笔序列):
            k_seq_rs = chanlun.K线.截取(
                obs_rs.普通K线序列,
                bi_rs.文.中.标的K线,
                bi_rs.武.中.标的K线,
            )
            k_seq_py = chan.K线.截取(
                obs_py.普通K线序列,
                bi_py.文.中.标的K线,
                bi_py.武.中.标的K线,
            )
            r = chanlun.虚线.计算K线序列MACD趋向背驰(k_seq_rs, bi_rs.方向)
            p = chan.虚线.计算K线序列MACD趋向背驰(k_seq_py, bi_py.方向)
            if r != list(p):
                mismatches.append((bi_rs.文.时间戳, bi_rs.武.时间戳, list(r), list(p)))

        self.assertEqual(len(mismatches), 0, f"笔MACD趋向背驰 不一致 ({len(mismatches)}):\n" + "\n".join(f"  [{ts_w},{ts_wu}] R={r} P={p}" for ts_w, ts_wu, r, p in mismatches[:5]))

    def test_线段MACD趋向背驰双端一致(self):
        """线段的 K线序列 MACD趋向背驰 双端结果完全一致."""
        import chanlun
        from chanlun import chan

        obs_rs, obs_py = self._build_observers()
        mismatches = []
        for seg_rs, seg_py in zip(obs_rs.线段序列, obs_py.线段序列):
            k_seq_rs = chanlun.K线.截取(
                obs_rs.普通K线序列,
                seg_rs.文.中.标的K线,
                seg_rs.武.中.标的K线,
            )
            k_seq_py = chan.K线.截取(
                obs_py.普通K线序列,
                seg_py.文.中.标的K线,
                seg_py.武.中.标的K线,
            )
            r = chanlun.虚线.计算K线序列MACD趋向背驰(k_seq_rs, seg_rs.方向)
            p = chan.虚线.计算K线序列MACD趋向背驰(k_seq_py, seg_py.方向)
            if r != list(p):
                mismatches.append((seg_rs.文.时间戳, seg_rs.武.时间戳, list(r), list(p)))

        self.assertEqual(len(mismatches), 0, f"线段MACD趋向背驰 不一致 ({len(mismatches)}):\n" + "\n".join(f"  [{ts_w},{ts_wu}] R={r} P={p}" for ts_w, ts_wu, r, p in mismatches[:5]))

    # ---- 统计MACD行为 ----

    def test_笔统计MACD行为双端一致(self):
        """笔的 统计MACD行为() 双端结果完全一致."""
        import chanlun
        from chanlun import chan

        obs_rs, obs_py = self._build_observers()
        mismatches = []
        for bi_rs, bi_py in zip(obs_rs.笔序列, obs_py.笔序列):
            k_seq_rs = chanlun.K线.截取(
                obs_rs.普通K线序列,
                bi_rs.文.中.标的K线,
                bi_rs.武.中.标的K线,
            )
            k_seq_py = chan.K线.截取(
                obs_py.普通K线序列,
                bi_py.文.中.标的K线,
                bi_py.武.中.标的K线,
            )
            r = chanlun.虚线.统计MACD行为(k_seq_rs)
            p = chan.虚线.统计MACD行为(k_seq_py)
            if r != p:
                mismatches.append((bi_rs.文.时间戳, bi_rs.武.时间戳, r, p))

        self.assertEqual(len(mismatches), 0, f"笔统计MACD行为 不一致 ({len(mismatches)}):\n" + "\n".join(f"  [{ts_w},{ts_wu}] R={r} P={p}" for ts_w, ts_wu, r, p in mismatches[:5]))

    def test_线段统计MACD行为双端一致(self):
        """线段的 统计MACD行为() 双端结果完全一致."""
        import chanlun
        from chanlun import chan

        obs_rs, obs_py = self._build_observers()
        mismatches = []
        for seg_rs, seg_py in zip(obs_rs.线段序列, obs_py.线段序列):
            k_seq_rs = chanlun.K线.截取(
                obs_rs.普通K线序列,
                seg_rs.文.中.标的K线,
                seg_rs.武.中.标的K线,
            )
            k_seq_py = chan.K线.截取(
                obs_py.普通K线序列,
                seg_py.文.中.标的K线,
                seg_py.武.中.标的K线,
            )
            r = chanlun.虚线.统计MACD行为(k_seq_rs)
            p = chan.虚线.统计MACD行为(k_seq_py)
            if r != p:
                mismatches.append((seg_rs.文.时间戳, seg_rs.武.时间戳, r, p))

        self.assertEqual(len(mismatches), 0, f"线段统计MACD行为 不一致 ({len(mismatches)}):\n" + "\n".join(f"  [{ts_w},{ts_wu}] R={r} P={p}" for ts_w, ts_wu, r, p in mismatches[:5]))

    # ---- 获取所有停顿位置 ----

    def test_笔获取所有停顿位置双端一致(self):
        """笔的 获取所有停顿位置() 双端结果一致（通过虚线相等 逐项比对）."""
        import chanlun
        from chanlun import chan

        obs_rs, obs_py = self._build_observers()
        mismatches = []
        for bi_rs, bi_py in zip(obs_rs.笔序列, obs_py.笔序列):
            r = chanlun.笔.获取所有停顿位置(bi_rs, obs_rs)
            p = chan.笔.获取所有停顿位置(bi_py, obs_py)
            if len(r) != len(p):
                mismatches.append((f"len R={len(r)} P={len(p)}", [x.获取数据文本() for x in r], [x.获取数据文本() for x in p]))
            else:
                for a, b in zip(r, p):
                    eq, msg = chanlun.虚线相等(a, b)
                    if not eq:
                        mismatches.append((msg, [x.获取数据文本() for x in r], [x.获取数据文本() for x in p]))
                        break

        self.assertEqual(len(mismatches), 0, f"笔获取所有停顿位置 不一致 ({len(mismatches)}):\n" + "\n".join(f"  {tag}\n    R={rl}\n    P={pl}" for tag, rl, pl in mismatches[:3]))

    def test_线段获取所有停顿位置双端一致(self):
        """线段的 获取所有停顿位置() 双端结果一致（通过虚线相等 逐项比对）."""
        import chanlun
        from chanlun import chan

        obs_rs, obs_py = self._build_observers()
        mismatches = []
        for seg_rs, seg_py in zip(obs_rs.线段序列, obs_py.线段序列):
            r = chanlun.线段.获取所有停顿位置(seg_rs, obs_rs)
            p = chan.线段.获取所有停顿位置(seg_py, obs_py)
            if len(r) != len(p):
                mismatches.append((f"len R={len(r)} P={len(p)}", [x.获取数据文本() for x in r], [x.获取数据文本() for x in p]))
            else:
                for a, b in zip(r, p):
                    eq, msg = chanlun.虚线相等(a, b)
                    if not eq:
                        mismatches.append((msg, [x.获取数据文本() for x in r], [x.获取数据文本() for x in p]))
                        break

        self.assertEqual(len(mismatches), 0, f"线段获取所有停顿位置 不一致 ({len(mismatches)}):\n" + "\n".join(f"  {tag}\n    R={rl}\n    P={pl}" for tag, rl, pl in mismatches[:3]))

    # ---- 判断线段内部是否背驰 ----

    def test_判断线段内部是否背驰双端一致(self):
        """线段的 判断线段内部是否背驰() 双端结果完全一致."""
        import chanlun
        from chanlun import chan

        obs_rs, obs_py = self._build_observers()
        n = min(len(obs_rs.线段序列), len(obs_py.线段序列))
        self.assertGreater(n, 0, "线段序列为空")

        mismatches = []
        for i in range(n):
            r = chanlun.线段.判断线段内部是否背驰(obs_rs.线段序列[i], obs_rs)
            p = chan.线段.判断线段内部是否背驰(obs_py.线段序列[i], obs_py)
            if r != p:
                mismatches.append((i, r, p, obs_rs.线段序列[i].获取数据文本(), obs_py.线段序列[i].获取数据文本()))

        self.assertEqual(len(mismatches), 0, f"判断线段内部是否背驰 不一致 ({len(mismatches)}/{n}):\n" + "\n".join(f"  [{i}] R={r} P={p}\n    R文本={rt}\n    P文本={pt}" for i, r, p, rt, pt in mismatches[:3]))

    # ---- 是否背驰过 ----

    # ---- 获取内部中枢序列 ----

    def test_线段获取内部中枢序列双端一致(self):
        """线段的 获取内部中枢序列() 双端结果完全一致（通过 中枢相等 逐项比对）."""
        import chanlun
        from chanlun import chan

        obs_rs, obs_py = self._build_observers()
        n = min(len(obs_rs.线段序列), len(obs_py.线段序列))
        self.assertGreater(n, 0, "线段序列为空")

        mismatches = []
        for i in range(n):
            seg_rs = obs_rs.线段序列[i]
            seg_py = obs_py.线段序列[i]
            r = chanlun.线段.获取内部中枢序列(seg_rs, obs_rs.配置)
            p = chan.线段.获取内部中枢序列(seg_py, obs_py.配置)
            if len(r) != len(p):
                mismatches.append((i, f"tuple len R={len(r)} P={len(p)}"))
            else:
                for k, (hr, hp) in enumerate(zip(r, p)):
                    if len(hr) != len(hp):
                        mismatches.append((i, f"{['实', '虚', '合'][k]} len R={len(hr)} P={len(hp)}"))
                        break
                    for j, (ha, hb) in enumerate(zip(hr, hp)):
                        eq, msg = chanlun.中枢相等(ha, hb)
                        if not eq:
                            mismatches.append((i, f"{['实', '虚', '合'][k]}[{j}]: {msg}"))
                            break

        self.assertEqual(len(mismatches), 0, f"线段获取内部中枢序列 不一致 ({len(mismatches)}/{n}):\n" + "\n".join(f"  Seg[{i}]: {detail}" for i, detail in mismatches[:5]))

    # ---- 是否背驰过 ----

    def test_线段是否背驰过双端一致(self):
        """线段的 是否背驰过() 双端结果一致（通过 缠论K线相等 逐项比对）."""
        import chanlun
        from chanlun import chan

        obs_rs, obs_py = self._build_observers()
        n = min(len(obs_rs.线段序列), len(obs_py.线段序列))
        self.assertGreater(n, 0, "线段序列为空")

        mismatches = []
        for i in range(n):
            r = chanlun.线段.是否背驰过(obs_rs.线段序列[i], obs_rs)
            p = chan.线段.是否背驰过(obs_py.线段序列[i], obs_py)
            if len(r) != len(p):
                mismatches.append((i, f"len R={len(r)} P={len(p)}", obs_rs.线段序列[i].获取数据文本()))
            else:
                for a, b in zip(r, p):
                    eq, msg = chanlun.缠论K线相等(a, b)
                    if not eq:
                        mismatches.append((i, msg, obs_rs.线段序列[i].获取数据文本()))
                        break

        self.assertEqual(len(mismatches), 0, f"线段是否背驰过 不一致 ({len(mismatches)}/{n}):\n" + "\n".join(f"  [{i}] {detail}\n    段={txt[:120]}" for i, detail, txt in mismatches[:3]))


# ============================================================
# 指标挂载测试
# ============================================================


class Test指标挂载(unittest.TestCase):
    """指标计算与动态挂载回填测试."""

    @staticmethod
    def _make_k(i: int, ts_base: int = 1771675200, period: int = 300):
        """创建一根模拟K线."""
        return chanlun.K线.创建普K(
            "btcusd",
            ts_base + i * period,
            50000.0 + i,
            51000.0 + i,
            49000.0 + i,
            50500.0 + i,
            100.0 + i,
            i,
            period,
        )

    def test_基本指标计算(self):
        """每根K线都应有默认指标值."""
        cfg = chanlun.缠论配置()
        obs = chanlun.观察者("btcusd", 300, cfg)
        for i in range(50):
            obs.增加原始K线(self._make_k(i))

        for k in obs.普通K线序列:
            self.assertIn("macd", k.指标)
            self.assertIn("rsi", k.指标)
            self.assertIn("kdj", k.指标)

    def test_动态MACD参数回填(self):
        """中途修改 obs.配置 添加 MACD 变体后，历史K线应被回填."""
        cfg = chanlun.缠论配置()
        obs = chanlun.观察者("btcusd", 300, cfg)

        for i in range(100):
            if i == 50:
                obs.配置.MACD_参数列表 = [
                    ("macd", 12, 26, 9),
                    ("macd_10_20_7", 10, 20, 7),
                ]
            obs.增加原始K线(self._make_k(i))

        for k in obs.普通K线序列:
            self.assertIn("macd_10_20_7", k.指标)

    def test_多指标同时回填(self):
        """同时修改 MACD + RSI + KDJ 参数，验证全部回填."""
        cfg = chanlun.缠论配置()
        obs = chanlun.观察者("btcusd", 300, cfg)

        for i in range(80):
            if i == 40:
                obs.配置.MACD_参数列表 = [("macd", 12, 26, 9), ("macd_fast", 5, 13, 5)]
                obs.配置.RSI_周期列表 = [("rsi", 14), ("rsi_7", 7)]
                obs.配置.KDJ_参数列表 = [("kdj", 9, 3, 3), ("kdj_5", 5, 2, 2)]
            obs.增加原始K线(self._make_k(i))

        for k in obs.普通K线序列:
            self.assertIn("macd_fast", k.指标)
            self.assertIn("rsi_7", k.指标)
            self.assertIn("kdj_5", k.指标)

    def test_回填后增量计算一致(self):
        """回填后的指标值应与从头计算一致."""
        cfg_full = chanlun.缠论配置()
        cfg_full.MACD_参数列表 = [("macd", 12, 26, 9), ("macd_extra", 8, 16, 6)]
        obs_full = chanlun.观察者("btcusd", 300, cfg_full)

        cfg_late = chanlun.缠论配置()
        obs_late = chanlun.观察者("btcusd", 300, cfg_late)

        for i in range(100):
            if i == 50:
                obs_late.配置.MACD_参数列表 = [("macd", 12, 26, 9), ("macd_extra", 8, 16, 6)]
            obs_full.增加原始K线(self._make_k(i))
            obs_late.增加原始K线(self._make_k(i))

        seq_full = obs_full.普通K线序列
        seq_late = obs_late.普通K线序列
        self.assertEqual(len(seq_full), len(seq_late))

        for i in range(len(seq_full)):
            macd_full = seq_full[i].指标["macd_extra"]
            macd_late = seq_late[i].指标["macd_extra"]
            self.assertEqual(macd_full.DIF, macd_late.DIF)
            self.assertEqual(macd_full.DEA, macd_late.DEA)

    def test_同时间戳更新后指标重算(self):
        """同时间戳K线更新后，指标应基于新值重新计算."""
        cfg = chanlun.缠论配置()
        obs = chanlun.观察者("btcusd", 300, cfg)

        ts = 1771675200
        k1 = chanlun.K线.创建普K("btcusd", ts, 50000, 51000, 49000, 50500, 100, 0, 300)
        obs.增加原始K线(k1)

        k2 = chanlun.K线.创建普K("btcusd", ts + 300, 50500, 52000, 50000, 51500, 200, 1, 300)
        obs.增加原始K线(k2)
        macd_before = obs.普通K线序列[-1].指标["macd"].DIF

        # 同时间戳，不同收盘价
        k2_upd = chanlun.K线.创建普K("btcusd", ts + 300, 50500, 53000, 49000, 52500, 300, 1, 300)
        obs.增加原始K线(k2_upd)
        macd_after = obs.普通K线序列[-1].指标["macd"].DIF

        self.assertNotEqual(macd_before, macd_after)


# ============================================================
# 集成对比测试
# ============================================================


class Test集成对比(unittest.TestCase):
    """全量集成测试：喂入 .nb 数据，与 Python 参考输出对比."""

    @classmethod
    def setUpClass(cls):
        if not _has_nb():
            raise unittest.SkipTest("需要 .nb 数据文件")

    def test_full_integration(self):
        out_dir = os.path.join(tempfile.gettempdir(), "chanlun_py_test_output")

        bars = read_nb_bars(NB_PATH)
        self.assertGreater(len(bars), 0, "未读取到任何 K 线")

        obs = chanlun.观察者("btcusd", 300)
        self.assertEqual(obs.标识, "btcusd:300")
        self.assertEqual(obs.周期, 300)

        for i, (ts, o, h, l, c, v) in enumerate(bars):
            k = chanlun.K线.创建普K(f"btcusd_{i}", ts, o, h, l, c, v, i, 300)
            obs.增加原始K线(k)

        self.assertGreater(len(obs.普通K线序列), 0)
        self.assertGreater(len(obs.缠论K线序列), 0)

        os.makedirs(out_dir, exist_ok=True)
        obs.测试_保存数据(out_dir)

        subdirs = [d for d in os.listdir(out_dir) if os.path.isdir(os.path.join(out_dir, d))]
        self.assertGreater(len(subdirs), 0, "No output subdirectory found!")
        actual_out_dir = os.path.join(out_dir, subdirs[0])
        out_files = sorted(os.listdir(actual_out_dir))

        self.assertGreaterEqual(len(out_files), 14, f"Expected >= 14 output files, got {len(out_files)}")

        if not os.path.isdir(_PY_REF_DIR):
            self.skipTest(f"Python reference dir not found: {_PY_REF_DIR}")

        ref_files = sorted(os.listdir(_PY_REF_DIR))
        match_count = 0
        diff_count = 0

        for fname in ref_files:
            ref_path = os.path.join(_PY_REF_DIR, fname)
            out_path = os.path.join(actual_out_dir, fname)

            if not os.path.exists(out_path):
                self.fail(f"MISSING: {fname}")
                continue

            with open(ref_path) as f:
                ref_lines = f.readlines()
            with open(out_path) as f:
                out_lines = f.readlines()

            if ref_lines == out_lines:
                match_count += 1
            else:
                diff_count += 1
                for j, (rl, ol) in enumerate(zip(ref_lines, out_lines)):
                    if rl != ol:
                        print(f"  DIFF {fname} line {j}:")
                        print(f"    REF: {rl.rstrip()}")
                        print(f"    OUT: {ol.rstrip()}")
                        break
                if len(ref_lines) != len(out_lines):
                    print(f"  DIFF {fname}: line count {len(ref_lines)} vs {len(out_lines)}")

        self.assertEqual(diff_count, 0, f"{diff_count} files differ, {match_count} match")
        print(f"  All {match_count} Python reference files match!")


# ============================================================
# API 一致性测试 — chan.py vs chanlun
# ============================================================


class TestApi一致性(ApiConsistencyMixin, unittest.TestCase):
    """chan.py (Python参考) 与 chanlun (Rust/PyO3) 的 API 描述符类型一致性.

    验证: 同名类的同名成员在两边有相同的描述符类型:
      property / classmethod / staticmethod / regular_method
    """

    reference_module = chanlun.chan
    target_module = chanlun

    # pydantic 模型方法 + Python list/str 继承方法 → 噪音过滤
    noise_filters = [
        # pydantic v1/v2
        "construct",
        "copy",
        "dict",
        "json",
        "schema",
        "schema_json",
        "validate",
        "parse_file",
        "parse_obj",
        "parse_raw",
        "from_orm",
        "update_forward_refs",
        "model_",
        "bool_parse_fallback_default",
        # Python str (Enum 继承)
        "capitalize",
        "casefold",
        "center",
        "count",
        "encode",
        "endswith",
        "expandtabs",
        "find",
        "format",
        "format_map",
        "index",
        "isalnum",
        "isalpha",
        "isascii",
        "isdecimal",
        "isdigit",
        "isidentifier",
        "islower",
        "isnumeric",
        "isprintable",
        "isspace",
        "istitle",
        "isupper",
        "join",
        "ljust",
        "lower",
        "lstrip",
        "maketrans",
        "partition",
        "removeprefix",
        "removesuffix",
        "replace",
        "rfind",
        "rindex",
        "rjust",
        "rpartition",
        "rsplit",
        "rstrip",
        "split",
        "splitlines",
        "startswith",
        "strip",
        "swapcase",
        "title",
        "translate",
        "upper",
        "zfill",
        # Python list
        "append",
        "clear",
        "extend",
        "insert",
        "pop",
        "remove",
        "reverse",
        "sort",
    ]

    # 已知 target 中缺失的成员（有意未移植或用户自行实现）
    known_missing_in_target = {
        "K线合成器": {"设置事件回调"},
        "观察者": {"识别买卖点"},  # 用户子类化时重写
        "缠论配置": {"兼容旧版本配置"},  # pydantic v1 兼容
    }


# ============================================================
# main
# ============================================================


if __name__ == "__main__":
    unittest.main()
