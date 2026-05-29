"""PyO3 返回值的 Python 类型形状验证工具。

验证 PyO3 导出的函数/方法返回值类型正确：
- int 不是 str/float
- list 元素是 tuple 不是 list
- 方法是 callable 不是 property
- 返回值结构（嵌套类型）符合预期

用法::

    from helpers.type_shape import assert_type_shape

    result = mylib.compute(some_input)
    assert_type_shape(result, {
        "count": int,
        "ratio": float,
        "label": str,
        "items": [(int, str, bool)],    # list of 3-tuples
        "nested": {"key": int},
    })
"""

import unittest


def assert_type_shape(obj, schema, path=""):
    """验证 obj 的类型形状与 schema 一致。

    schema 支持:
        - type: obj 必须是该类型实例
        - [inner]: obj 必须是 list，每个元素验证 inner
        - (t1, t2, ...): obj 必须是 tuple，每字段验证对应类型
        - {key: inner}: obj 必须是 dict，递归验证
        - callable: obj 必须是 callable（函数/方法）
    """
    if isinstance(schema, type):
        _check_type(obj, schema, path)
    elif isinstance(schema, list):
        _check_list(obj, schema, path)
    elif isinstance(schema, tuple):
        _check_tuple(obj, schema, path)
    elif isinstance(schema, dict):
        _check_dict(obj, schema, path)
    elif schema is callable:
        _check_callable(obj, path)
    else:
        raise ValueError(f"{path}: 不支持的 schema 类型 {type(schema)}")


def _check_type(obj, expected, path):
    assert isinstance(obj, expected), f"{path}: 期望 {expected.__name__}, 实际 {type(obj).__name__}"


def _check_list(obj, schema, path):
    assert isinstance(obj, list), f"{path}: 期望 list, 实际 {type(obj).__name__}"
    if len(schema) == 1:
        inner = schema[0]
        for i, item in enumerate(obj):
            assert_type_shape(item, inner, f"{path}[{i}]")


def _check_tuple(obj, schema, path):
    assert isinstance(obj, tuple), f"{path}: 期望 tuple, 实际 {type(obj).__name__}"
    assert len(obj) == len(schema), f"{path}: 期望 tuple 长度 {len(schema)}, 实际 {len(obj)}"
    for i, (item, inner) in enumerate(zip(obj, schema)):
        assert_type_shape(item, inner, f"{path}[{i}]")


def _check_dict(obj, schema, path):
    assert isinstance(obj, dict), f"{path}: 期望 dict, 实际 {type(obj).__name__}"
    for key, inner in schema.items():
        assert key in obj, f"{path}: 缺少键 '{key}'"
        assert_type_shape(obj[key], inner, f"{path}['{key}']")


def _check_callable(obj, path):
    assert callable(obj), f"{path}: 期望 callable, 实际 {type(obj).__name__}"


# ---- TestCase mixin ----


class TypeShapeAssertions:
    """提供 assert_type_shape 便捷方法的 mixin."""

    def assertTypeShape(self, obj, schema, path=""):
        """断言 obj 的类型形状与 schema 一致."""
        assert_type_shape(obj, schema, path)
