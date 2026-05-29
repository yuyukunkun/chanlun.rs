"""pyo3_test_helpers — 可复用的 PyO3 测试工具包。

提供四个核心模块:

    rc_identity      — Rc/Arc 指针身份一致性测试 Mixin
    subclass         — PyO3 #[pyclass(subclass)] 子类化兼容性测试 Mixin
    type_shape       — 返回值类型形状验证工具
    api_consistency  — 两个模块间 API 描述符类型一致性测试 Mixin

所有 Mixin 都是纯 Python，不依赖 pytest，与 unittest.TestCase 配合使用。
下游项目复制此目录即可复用。
"""

from .api_consistency import ApiConsistencyMixin
from .rc_identity import RcIdentityMixin
from .subclass import PyO3SubclassMixin
from .type_shape import assert_type_shape, TypeShapeAssertions

__all__ = [
    "ApiConsistencyMixin",
    "RcIdentityMixin",
    "PyO3SubclassMixin",
    "assert_type_shape",
    "TypeShapeAssertions",
]
