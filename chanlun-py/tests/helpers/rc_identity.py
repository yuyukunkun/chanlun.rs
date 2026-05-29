"""Rc/Arc 指针身份一致性测试 Mixin。

验证：同一个 Rust Rc<T>/Arc<T> 无论通过哪条路径到达 Python，
始终返回相同的 PyObject（`a is b` 为 True）。

用法::

    class TestMyLib(RcIdentityMixin, unittest.TestCase):
        # 必须: 创建被测对象实例（每个 test_ 调用一次）
        @staticmethod
        def target_factory():
            return make_fresh_instance()

        # 必须: 序列 getter —— (名称, target → list)
        # Mixin 会验证: 同一 getter 调用两次，list[i] is list[j]
        sequence_getters = {
            "主序列": lambda t: t.items,
            "子序列": lambda t: t.children,
        }

        # 可选: 跨路径身份断言 —— (名称, (target → obj_a, target → obj_b))
        # Mixin 会验证: obj_a is obj_b
        cross_path_assertions = [
            ("序列[0] 与 首元素.父", lambda t: t.items[0], lambda t: t.items[0].parent),
        ]

        # 可选: getter 稳定性 —— (名称, target → obj)
        # Mixin 会验证: obj is obj (两次调用返回同一对象)
        stable_getters = {
            "首元素.属性": lambda t: t.items[0].attr,
        }

        # 可选: 序列长度检查的最小值（默认不检查，设为 >0 开启）
        min_sequence_lengths = {
            "主序列": 3,
            "子序列": 2,
        }
"""

import unittest


class RcIdentityMixin:
    """Rc/Arc 指针身份一致性测试 Mixin。

    子类必须定义:
        target_factory: Callable[[], Any]
        sequence_getters: dict[str, Callable[[Any], list]]

    子类可选定义:
        cross_path_assertions: list[tuple[str, Callable, Callable]]
        stable_getters: dict[str, Callable]
        min_sequence_lengths: dict[str, int]
    """

    target_factory = None
    sequence_getters: dict = {}
    cross_path_assertions: list = []
    stable_getters: dict = {}
    min_sequence_lengths: dict = {}

    def _get_target(self):
        """惰性获取 target，首次调用后缓存在类上。避免 setUpClass MRO 冲突."""
        cls = type(self)
        # 每次测试重新创建——但这会太慢。用类级别缓存。
        # 子类应在 setUpClass 中调用 self._get_target() 或自己设置 cls._cached_target。
        if not hasattr(cls, "_cached_target"):
            if cls.target_factory is None:
                raise unittest.SkipTest(f"{cls.__name__} 未定义 target_factory")
            cls._cached_target = cls.target_factory()
        return cls._cached_target

    # ---- 序列 getter 稳定性 ----

    def test_序列重复获取身份一致(self):
        """同一序列 getter 调用两次，对应位置元素 is 相同."""
        t = self._get_target()
        for name, getter in self.sequence_getters.items():
            seq1 = getter(t)
            seq2 = getter(t)
            self.assertEqual(len(seq1), len(seq2), f"{name}: 两次获取长度不同")
            check_n = min(len(seq1), 10)
            for i in range(check_n):
                self.assertIs(seq1[i], seq2[i], f"{name}[{i}] 身份不一致")

    def test_序列最小长度(self):
        """序列长度至少达到配置的最小值."""
        t = self._get_target()
        for name, getter in self.sequence_getters.items():
            if name in self.min_sequence_lengths:
                min_len = self.min_sequence_lengths[name]
                actual = len(getter(t))
                self.assertGreaterEqual(actual, min_len, f"{name} 长度 {actual} < {min_len}")

    # ---- 跨路径身份 ----

    def test_跨路径身份一致(self):
        """不同访问路径到达的同一 Rust 对象在 Python 侧 is 相同."""
        t = self._get_target()
        for i, (label, path_a, path_b) in enumerate(self.cross_path_assertions):
            obj_a = path_a(t)
            obj_b = path_b(t)
            self.assertIsNotNone(obj_a, f"[{i}] {label}: path_a 返回 None")
            self.assertIsNotNone(obj_b, f"[{i}] {label}: path_b 返回 None")
            self.assertIs(obj_a, obj_b, f"[{i}] {label}: 身份不一致")

    # ---- getter 稳定性 ----

    def test_getter重复调用身份一致(self):
        """同一 getter 调用两次返回同一 PyObject."""
        t = self._get_target()
        for name, getter in self.stable_getters.items():
            obj1 = getter(t)
            obj2 = getter(t)
            self.assertIs(obj1, obj2, f"{name}: 两次调用返回不同对象")

    # ---- list.index 基于 is ----

    def test_list_index_基于身份(self):
        """list.index(elem) 正常工作（依赖 __eq__ 基于 is 比较）."""
        t = self._get_target()
        for name, getter in self.sequence_getters.items():
            seq = getter(t)
            if len(seq) >= 2:
                self.assertEqual(seq.index(seq[0]), 0, f"{name}: index(seq[0]) != 0")
                self.assertEqual(seq.index(seq[-1]), len(seq) - 1, f"{name}: index(seq[-1]) != {len(seq) - 1}")
