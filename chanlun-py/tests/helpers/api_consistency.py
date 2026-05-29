"""API 一致性测试 Mixin。

验证两个模块中同名类的公开成员描述符类型一致。
典型用途：对比 Python 参考实现 (chan.py) 与 Rust/PyO3 移植 (chanlun) 的 API 兼容性。

用法::

    class TestApi一致性(ApiConsistencyMixin, unittest.TestCase):
        reference_module = mylib.ref      # Python 参考实现
        target_module = mylib             # Rust/PyO3 移植

        # 可选: 已知差异（不会报错）
        known_missing_in_target = {
            "SomeClass": {"old_deprecated_method"},
        }
        known_descriptor_diffs = {
            # (class_name, member, ref_type, target_type)
        }

        # 可选: 成员名过滤（匹配则跳过，支持前缀用 "prefix_" 表示）
        noise_filters = ["model_", "parse_", "from_orm"]
"""

import unittest


def _classify_member(cls, attr_name):
    """返回描述符类型: property / classmethod / staticmethod / regular_method / None(data)."""
    # 优先检查元类字典中的描述符
    for klass in type(cls).__mro__:
        if attr_name in klass.__dict__:
            raw = klass.__dict__[attr_name]
            if isinstance(raw, property):
                return "property"
            elif isinstance(raw, classmethod):
                return "classmethod"
            elif isinstance(raw, staticmethod):
                return "staticmethod"
            break
    try:
        attr = getattr(cls, attr_name)
    except Exception:
        return None
    if callable(attr):
        return "regular_method"
    return None


def _is_noise(name, filters):
    for pat in filters:
        if pat == name:
            return True
        if pat.endswith("_") and name.startswith(pat):
            return True
    return False


def _get_classes(mod):
    """获取模块中所有公开的 type."""
    return {n: getattr(mod, n) for n in dir(mod) if not n.startswith("_") and isinstance(getattr(mod, n), type)}


class ApiConsistencyMixin:
    """API 一致性测试 Mixin。

    子类必须定义:
        reference_module: 参考模块 (Python 实现)
        target_module:   目标模块 (Rust/PyO3 移植)

    子类可选定义:
        known_missing_in_target: dict[str, set[str]]  — 已知 target 中缺失的成员
        known_descriptor_diffs: set[tuple]             — 已知描述符类型差异
        noise_filters: list[str]                       — 噪音成员名过滤
    """

    reference_module = None
    target_module = None
    known_missing_in_target: dict = {}
    known_descriptor_diffs: set = set()
    noise_filters: list = []

    @classmethod
    def setUpClass(cls):
        if cls.reference_module is None or cls.target_module is None:
            raise unittest.SkipTest(f"{cls.__name__} 未定义 reference_module / target_module")

    # ---- 描述符类型一致性 ----

    def test_共有成员描述符类型一致(self):
        """同名类的同名成员，描述符类型 (property/classmethod/staticmethod/regular) 一致."""
        ref_classes = _get_classes(self.reference_module)
        tgt_classes = _get_classes(self.target_module)
        shared = sorted(set(ref_classes) & set(tgt_classes))

        failures = []
        for cls_name in shared:
            ref_cls = ref_classes[cls_name]
            tgt_cls = tgt_classes[cls_name]

            ref_members = {}
            tgt_members = {}

            for attr_name in sorted(dir(ref_cls)):
                if attr_name.startswith("_") or _is_noise(attr_name, self.noise_filters):
                    continue
                cat = _classify_member(ref_cls, attr_name)
                if cat:
                    ref_members[attr_name] = cat

            for attr_name in sorted(dir(tgt_cls)):
                if attr_name.startswith("_") or _is_noise(attr_name, self.noise_filters):
                    continue
                cat = _classify_member(tgt_cls, attr_name)
                if cat:
                    tgt_members[attr_name] = cat

            shared_members = sorted(set(ref_members) & set(tgt_members))
            for member in shared_members:
                ref_cat = ref_members[member]
                tgt_cat = tgt_members[member]
                if ref_cat != tgt_cat:
                    diff_key = (cls_name, member, ref_cat, tgt_cat)
                    if diff_key not in self.known_descriptor_diffs:
                        failures.append(f"{cls_name}.{member}: ref={ref_cat}, tgt={tgt_cat}")

        if failures:
            self.fail("描述符类型不一致:\n  " + "\n  ".join(failures))

    # ---- 缺失成员检查 ----

    def test_参考模块成员在目标模块中存在(self):
        """chan 中的关键公开成员在 chanlun 中均有对应."""
        ref_classes = _get_classes(self.reference_module)
        tgt_classes = _get_classes(self.target_module)
        shared = sorted(set(ref_classes) & set(tgt_classes))

        failures = []
        for cls_name in shared:
            if cls_name not in self.known_missing_in_target:
                continue
            ref_cls = ref_classes[cls_name]
            tgt_cls = tgt_classes[cls_name]

            expected_missing = self.known_missing_in_target.get(cls_name, set())

            ref_members = set()
            for attr_name in sorted(dir(ref_cls)):
                if attr_name.startswith("_") or _is_noise(attr_name, self.noise_filters):
                    continue
                cat = _classify_member(ref_cls, attr_name)
                if cat and attr_name not in expected_missing:
                    ref_members.add(attr_name)

            tgt_members = set()
            for attr_name in sorted(dir(tgt_cls)):
                if attr_name.startswith("_") or _is_noise(attr_name, self.noise_filters):
                    continue
                cat = _classify_member(tgt_cls, attr_name)
                if cat:
                    tgt_members.add(attr_name)

            missing = ref_members - tgt_members - expected_missing
            for member in sorted(missing):
                failures.append(f"{cls_name}.{member}: ref={_classify_member(ref_cls, member)}, tgt=未导出")

        if failures:
            self.fail("参考模块中的成员在目标模块中缺失:\n  " + "\n  ".join(failures))

    # ---- 方法可调用性 ----

    def test_共有方法均可调用(self):
        """所有共有 regular_method 在两边都是 callable."""
        ref_classes = _get_classes(self.reference_module)
        tgt_classes = _get_classes(self.target_module)
        shared = sorted(set(ref_classes) & set(tgt_classes))

        failures = []
        for cls_name in shared:
            ref_cls = ref_classes[cls_name]
            tgt_cls = tgt_classes[cls_name]

            for attr_name in sorted(dir(ref_cls)):
                if attr_name.startswith("_") or _is_noise(attr_name, self.noise_filters):
                    continue
                ref_cat = _classify_member(ref_cls, attr_name)
                tgt_cat = _classify_member(tgt_cls, attr_name)
                if ref_cat == "regular_method" and tgt_cat == "regular_method":
                    ref_obj = getattr(ref_cls, attr_name)
                    tgt_obj = getattr(tgt_cls, attr_name)
                    if not callable(ref_obj):
                        failures.append(f"{cls_name}.{attr_name}: ref 不是 callable")
                    if not callable(tgt_obj):
                        failures.append(f"{cls_name}.{attr_name}: tgt 不是 callable")

        if failures:
            self.fail("方法不可调用:\n  " + "\n  ".join(failures))
