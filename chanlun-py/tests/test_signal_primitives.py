"""信号原语 Rust 移植后的跨语言一致性与边界行为测试。

验证 chanlun._chanlun 的 Signal/Factor/Event/Operate/Position 与移植前 Python 版本
行为一致（name hash 除外，已改为 Rust 确定性哈希）。
"""

import pytest
from chanlun._chanlun import Signal, Factor, Event, Operate, Position


# ---- Signal ----


def test_signal_parse_and_props():
    s = Signal("14400_D1MO3_中枢_中枢段DEA穿越2_三买_偏移0_100")
    assert s.k1 == "14400" and s.k3 == "中枢" and s.v2 == "三买" and s.score == 100
    assert s.key == "14400_D1MO3_中枢"
    assert s.value == "中枢段DEA穿越2_三买_偏移0_100"
    assert repr(s) == "Signal('14400_D1MO3_中枢_中枢段DEA穿越2_三买_偏移0_100')"


def test_signal_from_fields_default_任意():
    s = Signal(k1="14400", k2="D1MO3", k3="中枢", v2="三买")
    # v1/v3 缺省为 任意 → key 过滤后保留全部 k；value 含 任意
    assert s.key == "14400_D1MO3_中枢"
    assert s.value == "任意_三买_任意_0"


def test_signal_score_out_of_range():
    with pytest.raises(ValueError):
        Signal(k1="a", k2="b", k3="c", score=101)


def test_signal_is_match_missing_key_raises():
    s = Signal(k1="14400", k2="D1MO3", k3="中枢", v2="三买")
    with pytest.raises(ValueError):
        s.is_match({})


def test_signal_is_match_non_str_value_false():
    s = Signal(k1="14400", k2="D1MO3", k3="中枢", v2="三买")
    assert s.is_match({"14400_D1MO3_中枢": 123}) is False


def test_signal_is_match_hit():
    s = Signal(k1="14400", k2="D1MO3", k3="中枢", v2="三买")
    assert s.is_match({"14400_D1MO3_中枢": "x_三买_y_100"}) is True


def test_signal_is_match_v2_mismatch():
    s = Signal(k1="14400", k2="D1MO3", k3="中枢", v2="三买")
    assert s.is_match({"14400_D1MO3_中枢": "x_三卖_y_100"}) is False


# ---- Factor ----


def test_factor_empty_all_raises():
    with pytest.raises(ValueError):
        Factor(signals_all=[])


def test_factor_name_deterministic():
    s = Signal(k1="14400", k2="D1MO3", k3="中枢", v2="三买")
    f1 = Factor(signals_all=[s])
    f2 = Factor(signals_all=[Signal(k1="14400", k2="D1MO3", k3="中枢", v2="三买")])
    assert f1.name == f2.name
    assert f1.name.startswith("#") and len(f1.name) == 5  # #XXXX


def test_factor_not_short_circuit():
    s = Signal(k1="14400", k2="D1MO3", k3="中枢", v2="三买")
    f = Factor(signals_all=[s], signals_not=[s])
    assert f.is_match({"14400_D1MO3_中枢": "x_三买_y_100"}) is False


def test_factor_unique_signals_is_property():
    """unique_signals 必须是 property（匹配 Python @property），不带括号访问。"""
    s = Signal(k1="14400", k2="D1MO3", k3="中枢", v2="三买")
    f = Factor(signals_all=[s])
    assert f.unique_signals == [s.signal]  # 属性访问，非方法调用


# ---- Event ----


def test_event_empty_factors_raises():
    with pytest.raises(ValueError):
        Event(Operate.LO, [])


def test_event_name_uses_operate():
    s = Signal(k1="14400", k2="D1MO3", k3="中枢", v2="三买")
    e = Event(Operate.LO, [Factor(signals_all=[s])])
    assert e.name.startswith("开多#")


def test_event_match_returns_factor_name():
    s = Signal(k1="14400", k2="D1MO3", k3="中枢", v2="三买")
    e = Event(Operate.LO, [Factor(signals_all=[s])])
    ok, name = e.is_match({"14400_D1MO3_中枢": "x_三买_y_100"})
    assert ok and name


def test_event_multi_factor_or():
    """多 Factor OR：两 key 都在场，第一个不匹配、第二个匹配 → 返回第二个因子名。"""
    base = "14400"
    f1 = Factor(signals_all=[Signal(k1=base, k2="D1MO3", k3="中枢A", v2="三买")])
    f2 = Factor(signals_all=[Signal(k1=base, k2="D1MO3", k3="中枢B", v2="三买")])
    e = Event(Operate.LO, [f1, f2])
    d = {"14400_D1MO3_中枢A": "x_三卖_y_100", "14400_D1MO3_中枢B": "x_三买_y_100"}
    ok, name = e.is_match(d)
    assert ok and name == f2.name


# ---- Operate ----


def test_operate_value_and_eq():
    assert Operate.LO.value == "开多"
    assert Operate.LE.value == "平多"
    assert Operate.LO == Operate.LO
    assert Operate.LO in [Operate.LO, Operate.SO]  # update() 内部用法


# ---- Position（Rust 基类 + Python 子类）----


def test_position_requires_name():
    s = Signal(k1="14400", k2="D1MO3", k3="中枢", v2="三买")
    e = Event(Operate.LO, [Factor(signals_all=[s])])
    with pytest.raises((ValueError, TypeError)):
        Position(symbol="btc", opens=[e])


def test_position_config_getters():
    s = Signal(k1="14400", k2="D1MO3", k3="中枢", v2="三买")
    e = Event(Operate.LO, [Factor(signals_all=[s])])
    p = Position(symbol="btc", opens=[e], name="中枢", timeout=500, stop_loss=200, T0=True)
    assert p.symbol == "btc" and p.name == "中枢"
    assert p.timeout == 500 and p.stop_loss == 200 and p.T0 is True
    assert len(p.events) == 1
    assert p.unique_signals == [s.signal]


def test_position_subclassable_with_state():
    """验证 Rust 基类可被 Python 子类化，状态字段由 Rust 初始化。

    pos/pos_changed/operates/holds 等状态字段由 Rust 基类提供（只读 getter），
    初始值在构造时由 Rust #[new] 自动初始化。
    """
    s = Signal(k1="14400", k2="D1MO3", k3="中枢", v2="三买")
    e = Event(Operate.LO, [Factor(signals_all=[s])])
    p = Position(symbol="btc", opens=[e], name="中枢")
    assert p.name == "中枢"  # Rust 基类 getter
    assert p.pos == 0  # Rust 初始化为 0 (空仓)
    assert p.pos_changed is False
    assert p.operates == []
    assert p.holds == []


# ---- 序列化 dump/load ----


def test_operate_from_value():
    assert Operate.from_value("开多") == Operate.LO
    assert Operate.from_value("平空") == Operate.SE
    with pytest.raises(ValueError):
        Operate.from_value("不存在")


def test_factor_dump_load_roundtrip():
    s = Signal(k1="14400", k2="D1MO3", k3="中枢", v2="三买")
    f = Factor(signals_all=[s], name="测试")
    d = f.dump()
    assert d["name"] == f.name
    assert d["signals_all"] == [s.signal]
    assert d["signals_any"] == [] and d["signals_not"] == []
    f2 = Factor.load(d)
    assert f2.name == f.name  # 确定性哈希 → 同输入同名
    assert f2.unique_signals == f.unique_signals


def test_event_dump_load_roundtrip():
    s = Signal(k1="14400", k2="D1MO3", k3="中枢", v2="三买")
    e = Event(Operate.LO, [Factor(signals_all=[s])])
    d = e.dump()
    assert d["operate"] == "开多"
    assert len(d["factors"]) == 1
    e2 = Event.load(d)
    assert e2.name == e.name
    assert e2.operate == Operate.LO


def test_position_dump_load_roundtrip():
    """Position 序列化：Rust 基类 dump 配置 + Python 子类 with_data/load 返回子类实例。"""
    from chanlun.chan_external import Position as PositionExt, Signal as S, Factor as F, Event as E, Operate as O

    e = E(O.LO, [F(signals_all=[S(k1="14400", k2="D1MO3", k3="中枢", v2="三买")])])
    p = PositionExt(symbol="btc", opens=[e], name="中枢", timeout=500, T0=True)
    raw = p.dump()
    assert raw["symbol"] == "btc" and raw["T0"] is True and raw["timeout"] == 500
    assert len(raw["opens"]) == 1
    # with_data 附加 pairs/holds
    raw2 = p.dump(with_data=True)
    assert "pairs" in raw2 and "holds" in raw2
    # load 返回子类实例（含状态字段）
    p2 = PositionExt.load(raw)
    assert type(p2) is PositionExt
    assert p2.symbol == "btc" and p2.name == "中枢" and p2.timeout == 500
    assert p2.pos == 0  # 子类状态已初始化
