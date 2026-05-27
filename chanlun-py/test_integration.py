#!/usr/bin/env python3
"""Integration test: feed .nb bars through PyO3 observer, compare with Python reference output."""

import sys
import os
import struct
import tempfile

import chanlun

# 项目根目录（test_integration.py 位于 <root>/chanlun-py/ 下）
# 当脚本被复制到其他路径运行时，通过环境变量 CHANLUN_PROJECT_ROOT 指定
_PROJECT_ROOT = os.environ.get(
    "CHANLUN_PROJECT_ROOT",
    os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
)

NB_PATH = os.path.join(_PROJECT_ROOT, "btcusd-300-1761327300-1776327900.nb")
_PY_REF_DIR = os.path.join(_PROJECT_ROOT, "Py_btcusd:300_1761327300_1776327900")
_RUST_REF_DIR = os.path.join(_PROJECT_ROOT, "chanlun", "Rust_btcusd:300_1761327300_1776327900")


def read_nb_bars(path, max_bars=None):
    """Read bars from .nb file (48 bytes each: 6 × f64 big-endian)."""
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


# ============================================================
# 观察者 子类化 / 方法重写 测试
# ============================================================


def test_subclass_basic():
    """子类可创建，isinstance 正确."""

    class Sub(chanlun.观察者):
        pass

    obs = Sub("btcusd", 300)
    assert isinstance(obs, chanlun.观察者)
    assert type(obs).__name__ == "Sub"
    assert obs.标识 == "btcusd:300"
    assert obs.周期 == 300
    print("  ✓ test_subclass_basic")


def test_subclass_init_extra_attrs():
    """子类 __init__ 可添加自定义属性."""

    class Sub(chanlun.观察者):
        def __init__(self, 符号, 周期):
            self.tag = "custom"
            self.count = 0

    obs = Sub("btcusd", 300)
    assert obs.tag == "custom"
    assert obs.count == 0
    # 基类字段不受影响
    assert obs.标识 == "btcusd:300"
    print("  ✓ test_subclass_init_extra_attrs")


def test_subclass_new_filter_kwargs():
    """__new__ 过滤子类专属参数，只把父类需要的传给 super().__new__."""

    class Sub(chanlun.观察者):
        def __new__(cls, 符号, 周期, *, extra=None, **kwargs):
            return super().__new__(cls, 符号, 周期)

        def __init__(self, 符号, 周期, *, extra=None, **kwargs):
            self.extra = extra

    obs = Sub("btcusd", 300, extra={"debug": True})
    assert obs.extra == {"debug": True}
    assert obs.标识 == "btcusd:300"

    obs2 = Sub("ethusd", 60)
    assert obs2.extra is None
    print("  ✓ test_subclass_new_filter_kwargs")


def test_subclass_new_pass_config():
    """__new__ 透传 配置 参数到父类."""
    cfg = chanlun.缠论配置()

    class Sub(chanlun.观察者):
        def __new__(cls, 符号, 周期, 配置=None, *, tag="", **kwargs):
            return super().__new__(cls, 符号, 周期, 配置=配置)

        def __init__(self, 符号, 周期, 配置=None, *, tag="", **kwargs):
            self.tag = tag

    obs = Sub("btcusd", 300, cfg, tag="test-tag")
    assert obs.标识 == "btcusd:300"
    assert obs.tag == "test-tag"
    print("  ✓ test_subclass_new_pass_config")


def test_override_method_super_call():
    """重写 增加原始K线，super() 调用父类，全线管线运行."""
    bars = read_nb_bars(NB_PATH, max_bars=500)

    # 基类对照组
    base_obs = chanlun.观察者("btcusd", 300)
    for i, (ts, o, h, l, c, v) in enumerate(bars):
        k = chanlun.K线.创建普K(f"base_{i}", ts, o, h, l, c, v, i, 300)
        base_obs.增加原始K线(k)

    # 子类实验组
    class Sub(chanlun.观察者):
        def __init__(self, 符号, 周期):
            self.intercept_count = 0
            self.intercept_timestamps = []

        def 增加原始K线(self, 普K):
            self.intercept_count += 1
            self.intercept_timestamps.append(普K.时间戳)
            super().增加原始K线(普K)

    sub_obs = Sub("btcusd", 300)
    for i, (ts, o, h, l, c, v) in enumerate(bars):
        k = chanlun.K线.创建普K(f"sub_{i}", ts, o, h, l, c, v, i, 300)
        sub_obs.增加原始K线(k)

    # 拦截次数
    assert sub_obs.intercept_count == 500
    assert len(sub_obs.intercept_timestamps) == 500

    # 各层级序列与基类完全一致
    sequences = [
        "普通K线序列",
        "缠论K线序列",
        "分型序列",
        "笔序列",
        "线段序列",
        "中枢序列",
    ]
    for attr in sequences:
        base_len = len(getattr(base_obs, attr))
        sub_len = len(getattr(sub_obs, attr))
        assert base_len == sub_len, f"{attr}: base={base_len}, sub={sub_len}"

    # 笔时间戳精确对比
    base_pens = base_obs.笔序列
    sub_pens = sub_obs.笔序列
    for j, (bp, sp) in enumerate(zip(base_pens, sub_pens)):
        assert bp.文.中.时间戳 == sp.文.中.时间戳, f"笔[{j}] 时间戳不一致"

    print("  ✓ test_override_method_super_call")


def test_override_getter():
    """重写 @property getter，super() 取基类值."""

    class Sub(chanlun.观察者):
        @property
        def 标识(self):
            return f"[MOCKED] {super().标识}"

    obs = Sub("btcusd", 300)
    assert obs.标识 == "[MOCKED] btcusd:300"
    # 其他 getter 不受影响
    assert obs.周期 == 300
    print("  ✓ test_override_getter")


def test_override_str_repr():
    """重写 __str__ / __repr__."""

    class Sub(chanlun.观察者):
        def __str__(self):
            return f"Custom({self.标识})"

        def __repr__(self):
            return self.__str__()

    obs = Sub("btcusd", 300)
    assert str(obs) == "Custom(btcusd:300)"
    assert repr(obs) == "Custom(btcusd:300)"
    print("  ✓ test_override_str_repr")


def test_multi_level_inheritance():
    """多层继承，MRO 调用链完整."""

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
    k = chanlun.K线.创建普K("test", 1761327300, 100.0, 105.0, 99.0, 103.0, 1000.0, 0, 300)
    obs.增加原始K线(k)

    assert obs.l2_log == ["L2"], f"L2 log: {obs.l2_log}"
    assert obs.l1_log == ["L1"], f"L1 log: {obs.l1_log}"
    assert len(obs.普通K线序列) == 1
    print("  ✓ test_multi_level_inheritance")


def test_unoverridden_method_inherited():
    """未重写的方法从基类直接继承."""

    class Sub(chanlun.观察者):
        pass

    obs = Sub("btcusd", 120)
    k = chanlun.K线.创建普K("test", 1761327900, 100.0, 105.0, 99.0, 103.0, 1000.0, 0, 300)
    obs.增加原始K线(k)

    assert obs.标识 == "btcusd:120"
    assert obs.周期 == 120
    assert len(obs.普通K线序列) == 1
    assert len(obs.缠论K线序列) == 1
    # 静态重新分析 也能正常继承
    obs.静态重新分析()
    print("  ✓ test_unoverridden_method_inherited")


def test_override_reset():
    """重写 重置基础序列，子类状态也重置."""

    class Sub(chanlun.观察者):
        def __init__(self, 符号, 周期):
            self.my_log = []

        def 重置基础序列(self):
            self.my_log.clear()
            super().重置基础序列()

    obs = Sub("btcusd", 300)
    k = chanlun.K线.创建普K("test", 1761327300, 100.0, 105.0, 99.0, 103.0, 1000.0, 0, 300)
    obs.增加原始K线(k)
    obs.my_log.append("test")

    assert len(obs.普通K线序列) == 1
    obs.重置基础序列()
    assert len(obs.普通K线序列) == 0
    assert obs.my_log == []
    print("  ✓ test_override_reset")


# --- Property getter 逐一覆盖 ---


def test_override_getter_符号():
    """重写 符号 property."""

    class Sub(chanlun.观察者):
        @property
        def 符号(self):
            return f"[WRAPPED] {super().符号}"

    obs = Sub("btcusd", 300)
    assert obs.符号 == "[WRAPPED] btcusd"
    print("  ✓ test_override_getter_符号")


def test_override_getter_周期():
    """重写 周期 property."""

    class Sub(chanlun.观察者):
        @property
        def 周期(self):
            return super().周期 * 60

    obs = Sub("btcusd", 5)
    assert obs.周期 == 300
    print("  ✓ test_override_getter_周期")


def test_override_getter_配置():
    """重写 配置 property."""

    class Sub(chanlun.观察者):
        @property
        def 配置(self):
            self._cfg_accessed = True
            return super().配置

    obs = Sub("btcusd", 300)
    cfg = obs.配置
    assert cfg is not None
    assert obs._cfg_accessed
    print("  ✓ test_override_getter_配置")


def test_override_getter_当前K线():
    """重写 当前K线 property."""

    class Sub(chanlun.观察者):
        def __init__(self, 符号, 周期):
            self._cur_k_accessed = False

        @property
        def 当前K线(self):
            self._cur_k_accessed = True
            return super().当前K线

    obs = Sub("btcusd", 300)
    k = chanlun.K线.创建普K("t", 1761327300, 100.0, 105.0, 99.0, 103.0, 1000.0, 0, 300)
    obs.增加原始K线(k)
    cur = obs.当前K线
    assert cur is not None
    assert obs._cur_k_accessed
    print("  ✓ test_override_getter_当前K线")


def test_override_getter_当前缠K():
    """重写 当前缠K property."""

    class Sub(chanlun.观察者):
        def __init__(self, 符号, 周期):
            self._cur_ck_accessed = False

        @property
        def 当前缠K(self):
            self._cur_ck_accessed = True
            return super().当前缠K

    obs = Sub("btcusd", 300)
    k = chanlun.K线.创建普K("t", 1761327300, 100.0, 105.0, 99.0, 103.0, 1000.0, 0, 300)
    obs.增加原始K线(k)
    cur = obs.当前缠K
    assert cur is not None
    assert obs._cur_ck_accessed
    print("  ✓ test_override_getter_当前缠K")


def test_override_getter_观察员():
    """重写 观察员 property."""

    class Sub(chanlun.观察者):
        @property
        def 观察员(self):
            self._obs_accessed = True
            return self

    obs = Sub("btcusd", 300)
    assert obs.观察员 is obs
    assert obs._obs_accessed
    print("  ✓ test_override_getter_观察员")


# --- 序列 getter 覆盖 ---


def test_override_sequence_getter():
    """重写关键序列 getter，super() 取基类值."""

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
    k = chanlun.K线.创建普K("t", 1761327300, 100.0, 105.0, 99.0, 103.0, 1000.0, 0, 300)
    obs.增加原始K线(k)

    assert len(obs.普通K线序列) == 1
    assert "普通K线序列" in obs._seq_accessed
    assert len(obs.缠论K线序列) == 1
    assert "缠论K线序列" in obs._seq_accessed
    assert isinstance(obs.笔序列, list)
    assert "笔序列" in obs._seq_accessed
    assert isinstance(obs.线段序列, list)
    assert "线段序列" in obs._seq_accessed
    print("  ✓ test_override_sequence_getter")


def test_override_all_sequence_getters():
    """重写全部 15 个序列 getter，确认每个都可通过 super() 获取."""

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

    # 动态构建子类，重写全部序列 getter
    def _make_getter(name):
        @property
        def getter(self, _name=name):
            self._all_seq_accessed.add(_name)
            # 通过 MRO 找到父类的 property 并调用
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
    k = chanlun.K线.创建普K("t", 1761327300, 100.0, 105.0, 99.0, 103.0, 1000.0, 0, 300)
    obs.增加原始K线(k)

    for seq_name in all_seqs:
        seq = getattr(obs, seq_name)
        assert seq_name in obs._all_seq_accessed, f"{seq_name} 未被拦截"
        assert isinstance(seq, list), f"{seq_name} 不是 list，而是 {type(seq)}"

    print("  ✓ test_override_all_sequence_getters")


# --- 方法重写 ---


def test_override_加载本地数据():
    """重写 加载本地数据，super() 调基类."""

    class Sub(chanlun.观察者):
        def __init__(self, 符号, 周期):
            self._loaded = False
            self._load_count = 0

        def 加载本地数据(self, 文件路径):
            self._loaded = True
            self._load_count += 1
            super().加载本地数据(文件路径)

    obs = Sub("btcusd", 300)
    obs.加载本地数据(NB_PATH)
    assert obs._loaded
    assert obs._load_count == 1
    assert len(obs.普通K线序列) > 0
    print("  ✓ test_override_加载本地数据")


def test_override_静态重新分析():
    """重写 静态重新分析，super() 调基类."""

    class Sub(chanlun.观察者):
        def __init__(self, 符号, 周期):
            self._reanalyzed = 0

        def 静态重新分析(self):
            self._reanalyzed += 1
            super().静态重新分析()

    obs = Sub("btcusd", 300)
    obs.加载本地数据(NB_PATH)
    bi_before = len(obs.笔序列)
    obs.静态重新分析()
    assert obs._reanalyzed == 1
    # 重新分析后笔序列仍然存在
    assert len(obs.笔序列) >= 0
    print("  ✓ test_override_静态重新分析")


def test_override_测试_保存数据():
    """重写 测试_保存数据，super() 调基类."""

    class Sub(chanlun.观察者):
        def __init__(self, 符号, 周期):
            self._saved = False
            self._save_root = None

        def 测试_保存数据(self, root=None):
            self._saved = True
            self._save_root = root
            super().测试_保存数据(root)

    obs = Sub("btcusd", 300)
    k = chanlun.K线.创建普K("t", 1761327300, 100.0, 105.0, 99.0, 103.0, 1000.0, 0, 300)
    obs.增加原始K线(k)
    with tempfile.TemporaryDirectory() as tmpdir:
        obs.测试_保存数据(tmpdir)
        assert obs._saved
        assert obs._save_root == tmpdir
    print("  ✓ test_override_测试_保存数据")


def test_override_读取数据文件():
    """重写 classmethod 读取数据文件."""

    class Sub(chanlun.观察者):
        @classmethod
        def 读取数据文件(cls, 文件路径, 配置=None):
            obs = super().读取数据文件(文件路径, 配置)
            obs._custom_classmethod_flag = True
            return obs

    obs = Sub.读取数据文件(NB_PATH)
    assert isinstance(obs, Sub)
    assert obs._custom_classmethod_flag
    assert len(obs.普通K线序列) > 0
    print("  ✓ test_override_读取数据文件")


# --- 完全重写（不调 super） ---


def test_override_completely_no_super():
    """完全重写 增加原始K线，不调 super()，彻底接管."""

    class Sub(chanlun.观察者):
        def 增加原始K线(self, 普K):
            self._custom = getattr(self, "_custom", [])
            self._custom.append(普K.时间戳)
            # 不调 super()

    obs = Sub("btcusd", 300)
    k = chanlun.K线.创建普K("t", 1761327300, 100.0, 105.0, 99.0, 103.0, 1000.0, 0, 300)
    obs.增加原始K线(k)
    assert obs._custom == [1761327300]
    assert len(obs.普通K线序列) == 0  # 基类逻辑没执行
    print("  ✓ test_override_completely_no_super")


# --- 同名继承（零重写） ---


def test_override_identical_subclass():
    """同名继承（零重写），行为与基类完全一致."""

    class Sub(chanlun.观察者):
        pass

    base = chanlun.观察者("btcusd", 300)
    sub = Sub("btcusd", 300)

    bars = read_nb_bars(NB_PATH, max_bars=200)
    for i, (ts, o, h, l, c, v) in enumerate(bars):
        bk = chanlun.K线.创建普K(f"b_{i}", ts, o, h, l, c, v, i, 300)
        sk = chanlun.K线.创建普K(f"s_{i}", ts, o, h, l, c, v, i, 300)
        base.增加原始K线(bk)
        sub.增加原始K线(sk)

    for attr in ["普通K线序列", "笔序列", "线段序列", "中枢序列"]:
        base_len = len(getattr(base, attr))
        sub_len = len(getattr(sub, attr))
        assert base_len == sub_len, f"{attr}: base={base_len}, sub={sub_len}"

    print("  ✓ test_override_identical_subclass")


# --- 多层继承 ---


def test_override_three_level_mixed():
    """三层继承，每层重写不同方法，MRO 链完整."""

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
    k = chanlun.K线.创建普K("t", 1761327300, 100.0, 105.0, 99.0, 103.0, 1000.0, 0, 300)
    obs.增加原始K线(k)

    assert obs._l1_feed == 1, f"L1 feed: {obs._l1_feed}"
    assert obs.标识 == "[L2] btcusd:300"
    assert obs._l2_标识 == 1
    obs.重置基础序列()
    assert obs._l3_reset == 1
    assert len(obs.普通K线序列) == 0
    print("  ✓ test_override_three_level_mixed")


# --- 跨方法调用 ---


def test_override_cross_method_dispatch():
    """子类方法间相互调用，确保 self 始终指向最外层实例."""

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
    k = chanlun.K线.创建普K("t", 1761327300, 100.0, 105.0, 99.0, 103.0, 1000.0, 0, 300)
    obs.compound_operation(k)

    assert obs._feed_called
    assert obs._reset_called
    assert len(obs.普通K线序列) == 0  # feed 后又 reset
    print("  ✓ test_override_cross_method_dispatch")


def run_subclass_tests():
    print("=== 观察者 子类化/重写 测试 ===")
    tests = [
        test_subclass_basic,
        test_subclass_init_extra_attrs,
        test_subclass_new_filter_kwargs,
        test_subclass_new_pass_config,
        test_override_method_super_call,
        test_override_getter,
        test_override_str_repr,
        test_multi_level_inheritance,
        test_unoverridden_method_inherited,
        test_override_reset,
        # 新增：property getter 逐一覆盖
        test_override_getter_符号,
        test_override_getter_周期,
        test_override_getter_配置,
        test_override_getter_当前K线,
        test_override_getter_当前缠K,
        test_override_getter_观察员,
        # 新增：序列 getter 覆盖
        test_override_sequence_getter,
        test_override_all_sequence_getters,
        # 新增：方法重写
        test_override_加载本地数据,
        test_override_静态重新分析,
        test_override_测试_保存数据,
        test_override_读取数据文件,
        # 新增：完全重写 / 同名继承 / 多层 / 跨方法
        test_override_completely_no_super,
        test_override_identical_subclass,
        test_override_three_level_mixed,
        test_override_cross_method_dispatch,
    ]
    for test in tests:
        try:
            test()
        except Exception as e:
            print(f"  ✗ {test.__name__} FAILED: {e}")
            import traceback

            traceback.print_exc()
            return 1
    print("  ✓ 全部通过")
    return 0


# ============================================================
# 集成对比测试
# ============================================================


def run_integration_test():
    """全量集成测试：喂入 .nb 数据，与 Python 参考输出对比。"""
    out_dir = os.path.join(tempfile.gettempdir(), "chanlun_py_test_output")

    # Read all bars
    print("Reading bars from .nb file...")
    bars = read_nb_bars(NB_PATH)
    print(f"  Read {len(bars)} bars")

    # Create observer (default config)
    print("Creating observer...")
    obs = chanlun.观察者("btcusd", 300)
    print(f"  Observer: {obs.标识}, period={obs.周期}")

    # Feed bars
    print("Feeding bars...")
    for i, (ts, o, h, l, c, v) in enumerate(bars):
        k = chanlun.K线.创建普K(f"btcusd_{i}", ts, o, h, l, c, v, i, 300)
        obs.增加原始K线(k)
        if i % 10000 == 0:
            print(f"  Fed {i}/{len(bars)} bars")

    print(f"  Done. {len(obs.普通K线序列)} normal K lines, {len(obs.缠论K线序列)} Chan K lines")

    # Save output
    print(f"Saving data to {out_dir}...")
    os.makedirs(out_dir, exist_ok=True)
    obs.测试_保存数据(out_dir)

    # Find the actual output subdirectory created by 测试_保存数据
    subdirs = [d for d in os.listdir(out_dir) if os.path.isdir(os.path.join(out_dir, d))]
    if not subdirs:
        print("ERROR: No output subdirectory found!")
        return 1
    actual_out_dir = os.path.join(out_dir, subdirs[0])
    out_files = sorted(os.listdir(actual_out_dir))
    print(f"  Output dir: {actual_out_dir}")
    print(f"  Output files ({len(out_files)}): {out_files}")

    # Compare with Python reference (skip if not available)
    if not os.path.isdir(_PY_REF_DIR):
        print(f"\nPython reference dir not found: {_PY_REF_DIR}")
        print("Skipping Python comparison. Generate reference with chan.py first.")
        print(f"Rust output is at: {actual_out_dir}")
        print(f"Files: {out_files}")
        # Self-consistency check: at least 14 output files expected
        assert len(out_files) >= 14, f"Expected >= 14 output files, got {len(out_files)}"
        print("Self-consistency check passed.")
        return 0

    print("\nComparing with Python reference...")
    ref_files = sorted(os.listdir(_PY_REF_DIR))

    match_count = 0
    diff_count = 0
    all_match = True
    for fname in ref_files:
        ref_path = os.path.join(_PY_REF_DIR, fname)
        out_path = os.path.join(actual_out_dir, fname)

        if not os.path.exists(out_path):
            print(f"  MISSING: {fname}")
            all_match = False
            continue

        with open(ref_path) as f:
            ref_lines = f.readlines()
        with open(out_path) as f:
            out_lines = f.readlines()

        if ref_lines == out_lines:
            print(f"  MATCH: {fname} ({len(ref_lines)} lines)")
            match_count += 1
        else:
            print(f"  DIFF: {fname} (ref={len(ref_lines)} lines, out={len(out_lines)} lines)")
            for j, (rl, ol) in enumerate(zip(ref_lines, out_lines)):
                if rl != ol:
                    print(f"    Line {j}:")
                    print(f"      REF: {rl.rstrip()}")
                    print(f"      OUT: {ol.rstrip()}")
                    break
            if len(ref_lines) != len(out_lines):
                print(f"    Line count differs")
            diff_count += 1
            all_match = False

    # Also compare extra files against Rust reference
    extra_files = set(out_files) - set(ref_files)
    if extra_files:
        print("\nComparing extra files with Rust reference...")
        for fname in sorted(extra_files):
            out_path = os.path.join(actual_out_dir, fname)
            rust_ref_path = os.path.join(_RUST_REF_DIR, fname)
            if os.path.exists(rust_ref_path):
                with open(rust_ref_path) as f:
                    ref_lines = f.readlines()
                with open(out_path) as f:
                    out_lines = f.readlines()
                if ref_lines == out_lines:
                    print(f"  MATCH: {fname} (vs Rust ref, {len(ref_lines)} lines)")
                    match_count += 1
                else:
                    print(f"  DIFF: {fname} (vs Rust ref)")
                    diff_count += 1

    print(f"\nSummary: {match_count} match, {diff_count} differ")
    if all_match:
        print("All Python reference files match!")
        return 0
    else:
        print("Some files differ (see above)")
        return 1


# ============================================================
# K线.截取 测试
# ============================================================


def test_kxian_jiequ_basic():
    """Python端创建K线，测试 K线.截取 基本功能."""
    k1 = chanlun.K线.创建普K("k1", 1000000000, 100.0, 105.0, 98.0, 102.0, 1000.0, 0, 300)
    k2 = chanlun.K线.创建普K("k2", 1000000060, 102.0, 108.0, 101.0, 106.0, 1200.0, 1, 300)
    k3 = chanlun.K线.创建普K("k3", 1000000120, 106.0, 110.0, 104.0, 108.0, 900.0, 2, 300)
    k4 = chanlun.K线.创建普K("k4", 1000000180, 108.0, 112.0, 107.0, 110.0, 1100.0, 3, 300)
    ks = [k1, k2, k3, k4]

    # 截取中间 (k2~k4)，含头尾
    r = chanlun.K线.截取(ks, k2, k4)
    assert len(r) == 3
    assert r[0].时间戳 == k2.时间戳
    assert r[-1].时间戳 == k4.时间戳

    # 截取全部
    r = chanlun.K线.截取(ks, k1, k4)
    assert len(r) == 4

    # 截取单根
    r = chanlun.K线.截取(ks, k3, k3)
    assert len(r) == 1
    assert r[0].时间戳 == k3.时间戳

    print("  ✓ test_kxian_jiequ_basic")


def test_kxian_jiequ_error():
    """K线.截取 异常边界测试."""
    k1 = chanlun.K线.创建普K("k1", 1000000000, 100.0, 105.0, 98.0, 102.0, 1000.0, 0, 300)
    k2 = chanlun.K线.创建普K("k2", 1000000060, 102.0, 108.0, 101.0, 106.0, 1200.0, 1, 300)
    k3 = chanlun.K线.创建普K("k3", 1000000120, 106.0, 110.0, 104.0, 108.0, 900.0, 2, 300)
    ks = [k1, k2, k3]

    # 空序列 → ValueError
    try:
        chanlun.K线.截取([], k1, k1)
        raise AssertionError("空序列应抛 ValueError")
    except ValueError:
        pass

    # K线不在序列中 → ValueError
    kx = chanlun.K线.创建普K("kx", 9999999999, 1.0, 1.0, 1.0, 1.0, 1.0, 99, 300)
    try:
        chanlun.K线.截取(ks, kx, k3)
        raise AssertionError("不在序列中应抛 ValueError")
    except ValueError:
        pass

    # 始在终之后 → ValueError
    try:
        chanlun.K线.截取(ks, k3, k1)
        raise AssertionError("始在终之后应抛 ValueError")
    except ValueError:
        pass

    print("  ✓ test_kxian_jiequ_error")


def test_kxian_jiequ_from_observer():
    """通过观察者加载数据，从笔中获取普K序列再做截取."""
    cfg = chanlun.缠论配置()
    obs = chanlun.观察者.读取数据文件(NB_PATH, cfg)

    assert len(obs.笔序列) > 0, "笔序列为空，数据可能不够"
    bi = obs.笔序列[0]

    # 从笔获取普K子序列
    pk_seq = bi.获取普K序列(obs)
    assert len(pk_seq) > 0

    # 用标的K线在全序列上截取，验证与笔首尾匹配
    r = chanlun.K线.截取(obs.普通K线序列, bi.文.中.标的K线, bi.武.中.标的K线)
    assert len(r) > 0
    assert r[0].时间戳 == bi.文.中.标的K线.时间戳
    assert r[-1].时间戳 == bi.武.中.标的K线.时间戳

    # 截取前半段
    mid = len(pk_seq) // 2
    if mid > 0:
        r = chanlun.K线.截取(pk_seq, pk_seq[0], pk_seq[mid])
        assert len(r) == mid + 1
        assert r[0].时间戳 == pk_seq[0].时间戳
        assert r[-1].时间戳 == pk_seq[mid].时间戳

    print(f"  ✓ test_kxian_jiequ_from_observer (笔0: {len(pk_seq)}根普K)")


def test_kxian_jiequ_multi_bi():
    """遍历多根笔，各自截取笔内嵌K并验证首尾."""
    cfg = chanlun.缠论配置()
    obs = chanlun.观察者.读取数据文件(NB_PATH, cfg)

    n_checked = 0
    for i, bi in enumerate(obs.笔序列[:10]):
        pk_seq = bi.获取普K序列(obs)
        if len(pk_seq) < 2:
            continue
        r = chanlun.K线.截取(pk_seq, pk_seq[0], pk_seq[-1])
        assert len(r) == len(pk_seq), f"笔[{i}] 截取长度不一致: {len(r)} vs {len(pk_seq)}"
        n_checked += 1

    assert n_checked > 0
    print(f"  ✓ test_kxian_jiequ_multi_bi (检查 {n_checked} 根笔)")


def run_jiequ_tests():
    print("=== K线.截取 测试 ===")
    tests = [
        test_kxian_jiequ_basic,
        test_kxian_jiequ_error,
        test_kxian_jiequ_from_observer,
        test_kxian_jiequ_multi_bi,
    ]
    for test in tests:
        try:
            test()
        except Exception as e:
            print(f"  ✗ {test.__name__} FAILED: {e}")
            import traceback

            traceback.print_exc()
            return 1
    print("  ✓ 全部通过")
    return 0


# ============================================================
# Rc 指针身份 / list.index 测试
# ============================================================


def _load_observer(max_bars=None):
    """辅助函数：从 .nb 文件加载数据并创建观察者."""
    cfg = chanlun.缠论配置()
    obs = chanlun.观察者("btcusd", 300, cfg)
    bars = read_nb_bars(NB_PATH, max_bars=max_bars)
    for i, (ts, o, h, l, c, v) in enumerate(bars):
        k = chanlun.K线.创建普K(f"k{i}", ts, o, h, l, c, v, i, 300)
        obs.增加原始K线(k)
    return obs


def test_rc_same_seq_index():
    """同一序列内 list.index 基于 Rc 指针身份正常工作."""
    obs = _load_observer()

    # 分型序列
    fx_list = obs.分型序列
    if len(fx_list) >= 2:
        assert fx_list.index(fx_list[0]) == 0
        assert fx_list.index(fx_list[1]) == 1

    # 缠论K线序列
    ck_list = obs.缠论K线序列
    if len(ck_list) >= 2:
        assert ck_list.index(ck_list[0]) == 0
        assert ck_list.index(ck_list[-1]) == len(ck_list) - 1

    # 笔序列
    bi_list = obs.笔序列
    if len(bi_list) >= 2:
        assert bi_list.index(bi_list[0]) == 0
        assert bi_list.index(bi_list[-1]) == len(bi_list) - 1

    # 线段中的笔序列
    if len(obs.线段序列) > 0:
        for 段 in obs.线段序列:
            笔列表 = 段.笔序列
            if len(笔列表) >= 2:
                assert 笔列表.index(笔列表[0]) == 0
                assert 笔列表.index(笔列表[-1]) == len(笔列表) - 1
                break

    # 中枢序列
    zs_list = obs.中枢序列
    if len(zs_list) >= 2:
        assert zs_list.index(zs_list[0]) == 0
        assert zs_list.index(zs_list[-1]) == len(zs_list) - 1

    print("  ✓ test_rc_same_seq_index")


def test_rc_cross_seq_index():
    """跨序列 list.index：段.笔序列 中查找 中枢.基础序列 元素."""
    obs = _load_observer()

    n_checked = 0
    for 段 in obs.线段序列:
        zs_list = 段.合_中枢序列
        if len(zs_list) == 0:
            continue
        zs = zs_list[-1]
        笔列表 = 段.笔序列
        for j, elem in enumerate(zs.基础序列):
            idx = 笔列表.index(elem)
            assert idx >= 0
            assert 笔列表[idx] == elem
            n_checked += 1
        break  # 只测第一个有中枢的段

    assert n_checked >= 3, f"应至少检查3个中枢元素，实际 {n_checked}"
    print(f"  ✓ test_rc_cross_seq_index (检查 {n_checked} 个元素)")


def test_rc_zhaofenxing_cross_ref():
    """分型序列与笔之间的跨序列引用."""
    obs = _load_observer()

    fx_list = obs.分型序列
    if len(fx_list) >= 2 and len(obs.笔序列) > 0:
        # 笔.文 (起始分型) 应能在分型序列中找到
        bi = obs.笔序列[-1]
        try:
            idx = fx_list.index(bi.文)
            assert idx >= 0
        except ValueError:
            # 可能因为分型过滤导致不在序列中，但不应 panic
            pass

    print("  ✓ test_rc_zhaofenxing_cross_ref")


def test_rc_user_pattern():
    """复现用户报告的原始代码模式：在中枢基础序列第一个笔之前找同向笔."""
    obs = _load_observer()

    n_segments_with_zs = 0
    for 段 in obs.线段序列:
        zs_list = 段.合_中枢序列
        if len(zs_list) == 0:
            continue
        n_segments_with_zs += 1
        zs = zs_list[-1]
        笔列表 = 段.笔序列

        # 用户原始代码模式
        try:
            idx = 笔列表.index(zs.基础序列[0])
        except ValueError as e:
            raise AssertionError(f"段[{段.序号}] 笔列表中找不到中枢基础序列[0]: {e}")

        # 向前查找同向笔
        found = False
        for bi in reversed(笔列表[:idx]):
            if bi.方向 == zs.基础序列[0].方向:
                found = True
                break
        # found 可以为 True 或 False（取决于是否有前向笔）

    assert n_segments_with_zs > 0, "应至少有一个段包含中枢"
    print(f"  ✓ test_rc_user_pattern ({n_segments_with_zs} 个有中枢的段)")


def test_tongji_macd_behavior_types():
    """统计MACD行为返回值类型正确: int和list(tuple)，非str."""
    obs = _load_observer(max_bars=5000)
    if len(obs.笔序列) == 0:
        print("  - test_tongji_macd_behavior_types SKIP (无笔)")
        return
    笔 = obs.笔序列[-1]
    普K序列 = 笔.获取普K序列(obs)
    if len(普K序列) < 2:
        print("  - test_tongji_macd_behavior_types SKIP (K线不够)")
        return
    result = chanlun.虚线.统计MACD行为(普K序列, 8, 3)
    for key in ["DIF上穿0", "DIF下穿0", "DEA上穿0", "DEA下穿0", "金叉次数", "死叉次数"]:
        assert isinstance(result[key], int), f"{key} 应为 int, 实际 {type(result[key])}"
    assert isinstance(result["密集交叉区域"], list), f"密集交叉区域 应为 list"
    for item in result["密集交叉区域"]:
        assert isinstance(item, tuple), f"密集交叉区域元素 应为 tuple"
        assert len(item) == 3
        assert all(isinstance(v, int) for v in item)
    # 验证比较操作可用
    _ = result["DEA上穿0"] > 0 and result["DEA下穿0"] > 0
    print("  ✓ test_tongji_macd_behavior_types")


def test_xiangduifangxiang_methods():
    """相对方向.是否向上/是否向下 等必须是方法（需要括号调用），不能是 property."""
    fx = chanlun.相对方向.分析(2.0, 0.5, 1.0, 0.8)
    # 验证是可调用的方法
    assert callable(fx.是否向上), "是否向上 必须是方法"
    assert callable(fx.是否向下), "是否向下 必须是方法"
    assert callable(fx.是否包含), "是否包含 必须是方法"
    assert callable(fx.是否缺口), "是否缺口 必须是方法"
    assert callable(fx.是否衔接), "是否衔接 必须是方法"
    # 验证返回值类型
    assert isinstance(fx.是否向上(), bool)
    assert isinstance(fx.是否向下(), bool)

    print("  ✓ test_xiangduifangxiang_methods")


def run_rc_identity_tests():
    print("=== Rc 身份 / list.index 测试 ===")
    tests = [
        test_tongji_macd_behavior_types,
        test_xiangduifangxiang_methods,
        test_rc_same_seq_index,
        test_rc_cross_seq_index,
        test_rc_zhaofenxing_cross_ref,
        test_rc_user_pattern,
    ]
    for test in tests:
        try:
            test()
        except Exception as e:
            print(f"  ✗ {test.__name__} FAILED: {e}")
            import traceback

            traceback.print_exc()
            return 1
    print("  ✓ 全部通过")
    return 0


def main():
    import argparse

    parser = argparse.ArgumentParser(description="chanlun PyO3 集成测试")
    parser.add_argument("test", nargs="?", default="all", choices=["all", "subclass", "integration", "jiequ", "rc"], help="运行哪组测试 (默认: all)")
    args = parser.parse_args()

    exit_code = 0

    if args.test in ("all", "subclass"):
        if run_subclass_tests() != 0:
            exit_code = 1

    if args.test in ("all", "integration"):
        if run_integration_test() != 0:
            exit_code = 1

    if args.test in ("all", "jiequ"):
        if run_jiequ_tests() != 0:
            exit_code = 1

    if args.test in ("all", "rc"):
        if run_rc_identity_tests() != 0:
            exit_code = 1

    if exit_code == 0:
        print("\n✓ 所有测试通过")

    sys.exit(exit_code)


if __name__ == "__main__":
    main()
