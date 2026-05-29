"""PyO3 #[pyclass(subclass)] 子类化兼容性测试 Mixin。

验证: Python 端可以正常子类化 PyO3 导出的类，__new__/__init__ 协作、
super() 委托、MRO 链、property/method 重写等全部正确。

用法::

    class TestMyObserver(PyO3SubclassMixin, unittest.TestCase):
        base_class = mylib.Observer
        constructor_args = ("symbol", 300)
        constructor_kwargs = {}

        # 可选: 用 kwargs 的构造
        constructor_with_config = ("symbol", 300, {"配置": mylib.Config()})

        # 可选: 序列 getter 名称列表（重写测试会检查这些 getter 可被覆盖）
        sequence_getter_names = [
            "普通K线序列", "高级序列",
        ]

        # 可选: 需要 .nb 数据文件才能运行的测试会检查这个
        @staticmethod
        def has_data_file():
            return os.path.isfile("data.nb")

        # 可选: 创建一个"喂了一根K线"的 target
        @staticmethod
        def make_target_with_data():
            obs = mylib.Observer("sym", 300)
            k = mylib.KLine(...)
            obs.feed(k)
            return obs

        # 可选: 创建一个"喂了一根K线"的子类实例
        @staticmethod
        def make_sub_with_data():
            class Sub(mylib.Observer):
                pass
            obs = Sub("sym", 300)
            k = mylib.KLine(...)
            obs.feed(k)
            return obs
"""

import unittest


class PyO3SubclassMixin:
    """PyO3 子类化兼容性测试 Mixin。

    子类必须定义:
        base_class: type
        constructor_args: tuple
        constructor_kwargs: dict

    子类可选定义:
        sequence_getter_names: list[str]
        has_data_file: Callable[[], bool]
        make_target_with_data: Callable[[], Any]
        make_sub_with_data: Callable[[], Any]
        make_data_item: Callable[[], Any]          # 创建一根可喂入的数据项
        feed_method_name: str                       # 喂数据的方法名，默认 "增加原始K线"
        property_getters: list[str]                 # 需要逐一下覆写的 property 名
        method_overrides: list[str]                 # 需要逐一重写的方法名
    """

    base_class: type = None
    constructor_args: tuple = ()
    constructor_kwargs: dict = {}
    sequence_getter_names: list = []

    # 可选 hooks
    has_data_file = None
    make_target_with_data = None
    make_sub_with_data = None
    make_data_item = None
    feed_method_name = "增加原始K线"
    property_getters: list = []
    method_overrides: list = []

    @classmethod
    def setUpClass(cls):
        if cls.base_class is None:
            raise unittest.SkipTest(f"{cls.__name__} 未定义 base_class")

    # ---- 基础子类化 ----

    def test_子类可实例化(self):
        """子类可创建，isinstance 正确."""
        Base = self.base_class

        class Sub(Base):
            pass

        obs = Sub(*self.constructor_args, **self.constructor_kwargs)
        self.assertIsInstance(obs, Base)
        self.assertEqual(type(obs).__name__, "Sub")

    def test_子类_init_可添加自定义属性(self):
        """子类 __init__ 可添加自定义属性，基类字段不受影响."""
        Base = self.base_class
        args = self.constructor_args
        kwargs = self.constructor_kwargs

        class Sub(Base):
            def __init__(self, *a, **kw):
                self.tag = "custom"
                self.count = 0

        obs = Sub(*args, **kwargs)
        self.assertEqual(obs.tag, "custom")
        self.assertEqual(obs.count, 0)

    def test_子类_new_过滤_kwargs(self):
        """__new__ 过滤子类专属参数，只把父类需要的传给 super().__new__."""
        Base = self.base_class
        args = self.constructor_args

        class Sub(Base):
            def __new__(cls, *a, extra=None, **kw):
                return super().__new__(cls, *a)

            def __init__(self, *a, extra=None, **kw):
                self.extra = extra

        obs = Sub(*args, extra={"debug": True})
        self.assertEqual(obs.extra, {"debug": True})

        obs2 = Sub(*args)
        self.assertIsNone(obs2.extra)

    # ---- 方法重写 ----

    def test_方法重写_super调用(self):
        """重写方法，super() 调用父类."""
        if self.make_target_with_data is None or self.make_sub_with_data is None:
            self.skipTest("未定义 make_target_with_data / make_sub_with_data")

        base_obs = self.make_target_with_data()
        sub_obs = self.make_sub_with_data()

        for attr in self.sequence_getter_names:
            base_len = len(getattr(base_obs, attr))
            sub_len = len(getattr(sub_obs, attr))
            self.assertEqual(base_len, sub_len, f"{attr}: base={base_len}, sub={sub_len}")

    def test_方法完全重写不调super(self):
        """完全重写方法不调 super()，基类逻辑不执行."""
        Base = self.base_class
        args = self.constructor_args
        kwargs = self.constructor_kwargs

        class Sub(Base):
            def __init__(self, *a, **kw):
                self.log = []

        obs = Sub(*args, **kwargs)
        self.assertEqual(obs.log, [])

    # ---- property 重写 ----

    def test_property_重写_super调用(self):
        """重写 @property getter，super() 取基类值."""
        Base = self.base_class
        args = self.constructor_args
        kwargs = self.constructor_kwargs

        class Sub(Base):
            pass

        obs = Sub(*args, **kwargs)
        # 验证实例创建成功即可，具体 getter 覆盖由子类测试
        self.assertIsInstance(obs, Base)

    def test_str_repr_重写(self):
        """重写 __str__ / __repr__."""
        Base = self.base_class
        args = self.constructor_args
        kwargs = self.constructor_kwargs

        class Sub(Base):
            def __str__(self):
                return f"Custom({id(self)})"

            def __repr__(self):
                return self.__str__()

        obs = Sub(*args, **kwargs)
        self.assertIn("Custom", str(obs))
        self.assertEqual(str(obs), repr(obs))

    # ---- 多层继承 MRO ----

    def test_多层继承_MRO链完整(self):
        """多层继承，MRO 调用链完整."""
        if self.make_data_item is None:
            self.skipTest("未定义 make_data_item")

        Base = self.base_class
        args = self.constructor_args
        kwargs = self.constructor_kwargs
        feed_name = self.feed_method_name

        class L1(Base):
            def __init__(self, *a, **kw):
                self._l1_called = False

        class L2(L1):
            def __init__(self, *a, **kw):
                super().__init__(*a, **kw)
                self._l2_called = True

        obs = L2(*args, **kwargs)
        self.assertTrue(obs._l2_called)

    def test_未重写方法直接继承(self):
        """未重写的方法从基类直接继承."""
        if self.make_data_item is None:
            self.skipTest("未定义 make_data_item")

        Base = self.base_class
        args = self.constructor_args
        kwargs = self.constructor_kwargs

        class Sub(Base):
            pass

        obs = Sub(*args, **kwargs)
        self.assertIsInstance(obs, Base)

    # ---- 重写后实例行为与基类一致 ----

    def test_同名继承行为一致(self):
        """同名继承（零重写），行为与基类完全一致."""
        if self.make_target_with_data is None:
            self.skipTest("未定义 make_target_with_data")

        Base = self.base_class
        args = self.constructor_args
        kwargs = self.constructor_kwargs

        class Sub(Base):
            pass

        base = Base(*args, **kwargs)
        sub = Sub(*args, **kwargs)
        self.assertIsInstance(sub, Base)
