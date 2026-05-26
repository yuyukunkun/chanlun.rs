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

    # Compare with Python reference
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


def main():
    import argparse

    parser = argparse.ArgumentParser(description="chanlun PyO3 集成测试")
    parser.add_argument("test", nargs="?", default="all", choices=["all", "subclass", "integration"], help="运行哪组测试 (默认: all)")
    args = parser.parse_args()

    exit_code = 0

    if args.test in ("all", "subclass"):
        if run_subclass_tests() != 0:
            exit_code = 1

    if args.test in ("all", "integration"):
        if run_integration_test() != 0:
            exit_code = 1

    if exit_code == 0:
        print("\n✓ 所有测试通过")

    sys.exit(exit_code)


if __name__ == "__main__":
    main()
