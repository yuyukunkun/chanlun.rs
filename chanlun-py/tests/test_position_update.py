"""Position.update() 集成测试 — 验证 Rust 核心状态机与 Python 行为一致。

测试覆盖：
- 基础开多/开空/平多/平空
- 间隔限制
- 止损/超时
- pairs 盈亏计算
- 时间倒退容错
- 空信号字典容错
- dump/load 含状态
"""

import pytest
from datetime import datetime, timezone
from chanlun._chanlun import Position, Event, Factor, Signal, Operate


# ---- 辅助函数 ----


def 开多事件(k3="中枢", v2="三买"):
    s = Signal(k1="14400", k2="D1MO3", k3=k3, v2=v2)
    return Event(Operate.LO, [Factor(signals_all=[s])])


def 平多事件(k3="中枢", v2="三卖"):
    s = Signal(k1="14400", k2="D1MO3", k3=k3, v2=v2)
    return Event(Operate.LE, [Factor(signals_all=[s])])


def 开空事件(k3="中枢", v2="三卖"):
    s = Signal(k1="14400", k2="D1MO3", k3=k3, v2=v2)
    return Event(Operate.SO, [Factor(signals_all=[s])])


def 平空事件(k3="中枢", v2="三买"):
    s = Signal(k1="14400", k2="D1MO3", k3=k3, v2=v2)
    return Event(Operate.SE, [Factor(signals_all=[s])])


def 信号字典(symbol="btc", dt=None, close=50000.0, bid=1, **kwargs):
    """构造信号字典（含 OHLCV + 信号键）。"""
    if dt is None:
        dt = datetime.now(timezone.utc)
    d = {"symbol": symbol, "dt": dt, "close": close, "id": bid}
    d.update(kwargs)
    return d


# ---- 构造 ----


def test_构造状态初始化为默认值():
    p = Position(symbol="btc", opens=[开多事件()], name="测试")
    assert p.pos == 0
    assert p.pos_changed is False
    assert p.operates == []
    assert p.holds == []


# ---- update: 开仓 ----


def test_update_开多():
    p = Position(symbol="btc", opens=[开多事件()], name="测试")
    p.update(信号字典(**{"14400_D1MO3_中枢": "任意_三买_任意_0"}))
    assert p.pos == 1
    assert p.pos_changed is True
    assert len(p.operates) == 1
    assert p.operates[0]["op"] == Operate.LO
    assert len(p.holds) == 1
    assert p.holds[0]["pos"] == 1


def test_update_开空():
    p = Position(symbol="btc", opens=[开空事件()], name="测试")
    p.update(信号字典(**{"14400_D1MO3_中枢": "任意_三卖_任意_0"}))
    assert p.pos == -1
    assert p.operates[0]["op"] == Operate.SO


def test_update_开多_已持仓_不重复开仓():
    p = Position(symbol="btc", opens=[开多事件()], name="测试")
    p.update(信号字典(dt=datetime(2020, 1, 1, tzinfo=timezone.utc), bid=1, **{"14400_D1MO3_中枢": "任意_三买_任意_0"}))
    assert p.pos == 1
    assert len(p.operates) == 1
    # 第二次相同信号，已多头，不再开仓
    p.update(信号字典(dt=datetime(2020, 1, 1, 1, tzinfo=timezone.utc), bid=2, **{"14400_D1MO3_中枢": "任意_三买_任意_0"}))
    assert p.pos == 1
    assert len(p.operates) == 1  # 无新操作


# ---- update: 平仓 ----


def test_update_开多后平多():
    p = Position(symbol="btc", opens=[开多事件()], exits=[平多事件()], name="测试")
    # Step 1: LO
    p.update(信号字典(dt=datetime(2020, 1, 1, tzinfo=timezone.utc), bid=1, **{"14400_D1MO3_中枢": "任意_三买_任意_0"}))
    assert p.pos == 1
    # Step 2: LE (next day to allow exit when T0=False)
    p.update(信号字典(dt=datetime(2020, 1, 2, tzinfo=timezone.utc), bid=2, **{"14400_D1MO3_中枢": "任意_三卖_任意_0"}))
    assert p.pos == 0, f"Expected pos=0, got {p.pos}"
    assert p.operates[-1]["op"] == Operate.LE


def test_update_开空后平空():
    p = Position(symbol="btc", opens=[开空事件()], exits=[平空事件()], name="测试")
    p.update(信号字典(dt=datetime(2020, 1, 1, tzinfo=timezone.utc), bid=1, **{"14400_D1MO3_中枢": "任意_三卖_任意_0"}))
    assert p.pos == -1
    p.update(信号字典(dt=datetime(2020, 1, 2, tzinfo=timezone.utc), bid=2, **{"14400_D1MO3_中枢": "任意_三买_任意_0"}))
    assert p.pos == 0
    assert p.operates[-1]["op"] == Operate.SE


# ---- update: 止损 ----


def test_update_多头止损():
    p = Position(symbol="btc", opens=[开多事件()], name="测试", stop_loss=500)
    p.update(信号字典(dt=datetime(2020, 1, 1, tzinfo=timezone.utc), bid=1, close=50000.0, **{"14400_D1MO3_中枢": "任意_三买_任意_0"}))
    assert p.pos == 1
    # Price drops to 47000: (47000/50000 - 1) = -0.06 = -600 BP < -500 BP stop_loss
    p.update(信号字典(dt=datetime(2020, 1, 2, tzinfo=timezone.utc), bid=2, close=47000.0, **{"14400_D1MO3_中枢": "任意_无_任意_0"}))
    assert p.pos == 0, "Should be stopped out"
    assert "止损" in p.operates[-1]["op_desc"]


def test_update_空头止损():
    p = Position(symbol="btc", opens=[开空事件()], name="测试", stop_loss=500)
    p.update(信号字典(dt=datetime(2020, 1, 1, tzinfo=timezone.utc), bid=1, close=50000.0, **{"14400_D1MO3_中枢": "任意_三卖_任意_0"}))
    assert p.pos == -1
    # Price rises to 53000: (1 - 53000/50000) = -0.06 = -600 BP < -500 BP stop_loss
    p.update(信号字典(dt=datetime(2020, 1, 2, tzinfo=timezone.utc), bid=2, close=53000.0, **{"14400_D1MO3_中枢": "任意_无_任意_0"}))
    assert p.pos == 0, "Should be stopped out"
    assert "止损" in p.operates[-1]["op_desc"]


# ---- update: 超时 ----


def test_update_多头超时():
    p = Position(symbol="btc", opens=[开多事件()], name="测试", timeout=5)
    p.update(信号字典(dt=datetime(2020, 1, 1, tzinfo=timezone.utc), bid=1, close=50000.0, **{"14400_D1MO3_中枢": "任意_三买_任意_0"}))
    assert p.pos == 1
    # bid diff=9 > timeout=5
    p.update(信号字典(dt=datetime(2020, 1, 2, tzinfo=timezone.utc), bid=10, close=50000.0, **{"14400_D1MO3_中枢": "任意_无_任意_0"}))
    assert p.pos == 0, "Should be timed out"
    assert "超时" in p.operates[-1]["op_desc"]


# ---- update: 间隔限制 ----


def test_update_间隔限制():
    p = Position(symbol="btc", opens=[开多事件()], name="测试", interval=3600)
    # Create fresh position, open, test interval protection
    p2 = Position(symbol="btc", opens=[开多事件()], name="测试", interval=3600)
    p2.update(信号字典(dt=datetime(2020, 1, 1, tzinfo=timezone.utc), bid=1, close=50000, **{"14400_D1MO3_中枢": "任意_三买_任意_0"}))
    assert len(p2.operates) == 1
    # Within interval, same day (T0=False) — no new open
    p2.update(信号字典(dt=datetime(2020, 1, 1, 1, tzinfo=timezone.utc), bid=2, close=50000, **{"14400_D1MO3_中枢": "任意_三买_任意_0"}))
    assert len(p2.operates) == 1  # No new operate (already long, interval not elapsed)


# ---- update: 边界条件 ----


def test_update_时间倒退_跳过():
    p = Position(symbol="btc", opens=[开多事件()], name="测试")
    dt1 = datetime(2020, 1, 2, tzinfo=timezone.utc)
    dt2 = datetime(2020, 1, 1, tzinfo=timezone.utc)  # earlier
    p.update(信号字典(dt=dt1, **{"14400_D1MO3_中枢": "任意_三买_任意_0"}))
    operates_before = len(p.operates)
    p.update(信号字典(dt=dt2, **{"14400_D1MO3_中枢": "任意_三买_任意_0"}))
    assert len(p.operates) == operates_before  # skipped


def test_update_空事件列表():
    p = Position(symbol="btc", opens=[], name="空")
    p.update(信号字典(dt=datetime(2020, 1, 1, tzinfo=timezone.utc), **{"14400_D1MO3_中枢": "任意_三买_任意_0"}))
    assert p.pos == 0
    assert len(p.holds) == 1


def test_update_无匹配事件_仅追加holds():
    p = Position(symbol="btc", opens=[开多事件()], name="测试")
    p.update(信号字典(dt=datetime(2020, 1, 1, tzinfo=timezone.utc), **{"14400_D1MO3_中枢": "任意_无_任意_0"}))
    assert p.pos == 0
    assert p.operates == []
    assert len(p.holds) == 1


def test_update_缺键错误():
    """信号字典缺少事件所需 key 时抛 ValueError。"""
    p = Position(symbol="btc", opens=[开多事件()], name="测试")
    with pytest.raises(ValueError, match="不在信号列表中"):
        # 空信号字典缺少 "14400_D1MO3_中枢" 键
        p.update(信号字典(dt=datetime(2020, 1, 1, tzinfo=timezone.utc)))


def test_update_T0模式_同一天可操作():
    p = Position(symbol="btc", opens=[开多事件()], exits=[平多事件()], name="测试", T0=True)
    dt = datetime(2020, 1, 1, 0, 0, tzinfo=timezone.utc)
    p.update(信号字典(dt=dt, bid=1, **{"14400_D1MO3_中枢": "任意_三买_任意_0"}))
    assert p.pos == 1
    # Same day, T0=True → 允许平仓
    p.update(信号字典(dt=dt.replace(hour=1), bid=2, **{"14400_D1MO3_中枢": "任意_三卖_任意_0"}))
    assert p.pos == 0


# ---- pairs ----


def test_pairs_空():
    p = Position(symbol="btc", opens=[开多事件()], name="测试")
    assert p.pairs == []


def test_pairs_单笔开平_多头盈利():
    p = Position(symbol="btc", opens=[开多事件()], exits=[平多事件()], name="测试")
    p.update(信号字典(dt=datetime(2020, 1, 1, tzinfo=timezone.utc), bid=1, close=50000.0, **{"14400_D1MO3_中枢": "任意_三买_任意_0"}))
    p.update(信号字典(dt=datetime(2020, 1, 2, tzinfo=timezone.utc), bid=2, close=51000.0, **{"14400_D1MO3_中枢": "任意_三卖_任意_0"}))
    pairs = p.pairs
    assert len(pairs) == 1
    assert pairs[0]["交易方向"] == "多头"
    assert pairs[0]["开仓价格"] == 50000.0
    assert pairs[0]["平仓价格"] == 51000.0
    # (51000/50000 - 1) * 10000 = 200 BP
    assert pairs[0]["盈亏比例"] == pytest.approx(200.0, abs=0.1)


def test_pairs_单笔开平_空头盈利():
    p = Position(symbol="btc", opens=[开空事件()], exits=[平空事件()], name="测试")
    p.update(信号字典(dt=datetime(2020, 1, 1, tzinfo=timezone.utc), bid=1, close=50000.0, **{"14400_D1MO3_中枢": "任意_三卖_任意_0"}))
    p.update(信号字典(dt=datetime(2020, 1, 2, tzinfo=timezone.utc), bid=2, close=48000.0, **{"14400_D1MO3_中枢": "任意_三买_任意_0"}))
    pairs = p.pairs
    assert len(pairs) == 1
    assert pairs[0]["交易方向"] == "空头"
    assert pairs[0]["开仓价格"] == 50000.0
    assert pairs[0]["平仓价格"] == 48000.0
    # (1 - 48000/50000) * 10000 = 400 BP
    assert pairs[0]["盈亏比例"] == pytest.approx(400.0, abs=0.1)


def test_pairs_持仓天数():
    p = Position(symbol="btc", opens=[开多事件()], exits=[平多事件()], name="测试")
    p.update(信号字典(dt=datetime(2020, 1, 1, tzinfo=timezone.utc), bid=1, close=50000.0, **{"14400_D1MO3_中枢": "任意_三买_任意_0"}))
    # 3 days later
    p.update(信号字典(dt=datetime(2020, 1, 4, tzinfo=timezone.utc), bid=2, close=51000.0, **{"14400_D1MO3_中枢": "任意_三卖_任意_0"}))
    assert p.pairs[0]["持仓天数"] == pytest.approx(3.0, abs=0.1)


# ---- dump/load ----


def test_dump_with_data():
    p = Position(symbol="btc", opens=[开多事件()], name="测试")
    p.update(信号字典(dt=datetime(2020, 1, 1, tzinfo=timezone.utc), bid=1, close=50000.0, **{"14400_D1MO3_中枢": "任意_三买_任意_0"}))
    raw = p.dump(with_data=True)
    assert "pairs" in raw
    assert "holds" in raw
    assert raw["symbol"] == "btc"
    assert len(raw["holds"]) == 1


def test_dump_without_data():
    p = Position(symbol="btc", opens=[开多事件()], name="测试")
    raw = p.dump(with_data=False)
    assert "symbol" in raw
    assert "pairs" not in raw


def test_load_roundtrip():
    from chanlun.chan_external import Position as PyPos

    p = PyPos(symbol="btc", opens=[开多事件()], name="测试", timeout=500)
    p2 = PyPos.load(p.dump())
    assert p2.symbol == p.symbol
    assert p2.name == p.name
    assert p2.timeout == 500
    assert p2.pos == 0  # 新构造，状态初始


# ---- 信号字典 dt 类型兼容 ----


def test_update_dt_支持int时间戳():
    """验证 update() 支持 int Unix 时间戳（除 datetime 外）。"""
    p = Position(symbol="btc", opens=[开多事件()], name="测试")
    ts = int(datetime(2020, 1, 1, tzinfo=timezone.utc).timestamp())
    p.update(信号字典(dt=ts, **{"14400_D1MO3_中枢": "任意_三买_任意_0"}))
    assert p.pos == 1


def test_update_dt_支持float时间戳():
    """验证 update() 支持 float Unix 时间戳。"""
    p = Position(symbol="btc", opens=[开多事件()], name="测试")
    ts = datetime(2020, 1, 1, tzinfo=timezone.utc).timestamp()
    p.update(信号字典(dt=ts, **{"14400_D1MO3_中枢": "任意_三买_任意_0"}))
    assert p.pos == 1
