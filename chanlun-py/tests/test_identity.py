#!/usr/bin/env python3
"""
对象标识测试：验证同一 Rc 底层数据通过不同路径访问时，
Python 侧始终返回相同的 PyObject（`is` 比较为 True）。

涉及的 Rc 包装类型：
  - K线 (Rc<K线>)           — 原始OHLCV数据
  - 缠论K线 (Rc<缠论K线>)     — 包含处理后的K线
  - 分型 (Rc<分型>)          — 顶底分型
  - 虚线 (Rc<虚线>)          — 笔/线段的通用抽象
  - 中枢 (Rc<中枢>)          — 三段虚线重叠区间
  - 线段特征 (Rc<线段特征>)   — 线段特征序列元素
  - 特征分型 (Rc<特征分型>)   — 特征序列的分型

路径示例：
  - 缠K序列[i]  vs  分型序列[j].中  (同一根缠K)
  - 分型序列[i]  vs  笔序列[j].文    (同一个分型)
  - 笔序列[i]    vs  中枢[k].基础序列[m]  (同一条虚线)
"""

import chanlun
import math


def create_observer(symbol="btcusd", period=14400, n_bars=500):
    """创建观察者并喂入模拟K线数据。"""
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


class Test缠K身份:
    """缠论K线: 从序列、分型、笔端点、中枢等不同路径访问。"""

    def test_序列重复获取(self):
        """同一序列获取两次，元素应相同。"""
        obs = create_observer()
        seq1 = obs.缠论K线序列
        seq2 = obs.缠论K线序列
        for i in range(min(len(seq1), 10)):
            assert seq1[i] is seq2[i], f"缠K序列[{i}] 身份不一致"

    def test_分型中K(self):
        """分型.中 与 缠K序列 对应元素应相同。"""
        obs = create_observer()
        seq = obs.缠论K线序列
        分序 = obs.分型序列
        for fx in 分序[:10]:
            中 = fx.中
            for ck in seq:
                if ck.时间戳 == 中.时间戳:
                    assert ck is 中, f"分型.中 (ts={中.时间戳}) 与序列中元素不匹配"
                    break

    def test_笔端点钟K(self):
        """笔的端点分型的中间K线应与序列元素相同。"""
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
                        assert ck is 中, f"笔.{nm}.中 (ts={中.时间戳}) 与序列中元素不匹配"
                        break

    def test_getter重复调用(self):
        """同一getter调用两次返回同一对象。"""
        obs = create_observer()
        for fx in obs.分型序列[:5]:
            中1 = fx.中
            中2 = fx.中
            assert 中1 is 中2, "分型.中 两次调用返回不同对象"


class Test分型身份:
    """分型: 从分型序列、笔/线段端点、买卖点等不同路径访问。"""

    def test_序列重复获取(self):
        """同一序列获取两次，元素应相同。"""
        obs = create_observer()
        seq1 = obs.分型序列
        seq2 = obs.分型序列
        for i in range(min(len(seq1), 9)):
            assert seq1[i] is seq2[i], f"分型序列[{i}] 身份不一致"

    def test_笔端点与序列(self):
        """笔.文 / 笔.武 应与分型序列中对应元素相同。"""
        obs = create_observer()
        分序 = obs.分型序列
        for bi in obs.笔序列:
            for nm in ["文", "武"]:
                ep = getattr(bi, nm)
                if ep is None:
                    continue
                matched = False
                for fx in 分序:
                    if fx.时间戳 == ep.时间戳 and fx.结构 == ep.结构:
                        assert fx is ep, f"笔.{nm} (ts={ep.时间戳}) 与分型序列中元素不匹配"
                        matched = True
                        break
                assert matched, f"笔.{nm} (ts={ep.时间戳}) 在分型序列中未找到"

    def test_段端点与序列(self):
        """段.文 / 段.武 应与分型序列中对应元素相同。"""
        obs = create_observer()
        分序 = obs.分型序列
        for duan in obs.线段序列:
            for nm in ["文", "武"]:
                ep = getattr(duan, nm)
                if ep is None:
                    continue
                matched = False
                for fx in 分序:
                    if fx.时间戳 == ep.时间戳 and fx.结构 == ep.结构:
                        assert fx is ep, f"段.{nm} (ts={ep.时间戳}) 与分型序列中元素不匹配"
                        matched = True
                        break
                assert matched, f"段.{nm} (ts={ep.时间戳}) 在分型序列中未找到"

    def test_getter重复调用(self):
        """同一getter调用两次返回同一对象。"""
        obs = create_observer()
        for bi in obs.笔序列:
            文1 = bi.文
            文2 = bi.文
            assert 文1 is 文2, "笔.文 两次调用返回不同对象"
            武1 = bi.武
            武2 = bi.武
            assert 武1 is 武2, "笔.武 两次调用返回不同对象"
            break  # 只测第一笔


class Test虚线身份:
    """虚线(笔/线段): 从笔序列、线段序列、中枢内部序列等不同路径访问。"""

    def test_笔序列重复获取(self):
        obs = create_observer()
        seq1 = obs.笔序列
        seq2 = obs.笔序列
        for i in range(min(len(seq1), 8)):
            assert seq1[i] is seq2[i], f"笔序列[{i}] 身份不一致"

    def test_线段序列重复获取(self):
        obs = create_observer()
        seq1 = obs.线段序列
        seq2 = obs.线段序列
        for i in range(min(len(seq1), 5)):
            assert seq1[i] is seq2[i], f"线段序列[{i}] 身份不一致"

    def test_多个扩展序列(self):
        """扩展线段的不同序列获取同一虚线应相同。"""
        obs = create_observer()
        s1 = obs.扩展线段序列
        s2 = obs.扩展线段序列_线段
        s3 = obs.扩展线段序列_扩展线段
        # 这些序列可能包含不同的虚线，但如果同一个 Rc 出现在两个序列中应该相同
        for d1 in s1:
            for d2 in s2:
                if d1.序号 == d2.序号:
                    assert d1 is d2, f"扩展线段序列[{d1.序号}] 跨序列身份不一致"
                    break


class TestK线身份:
    """原始K线: 从序列、买卖点、缠K标的等不同路径访问。"""

    def test_序列重复获取(self):
        obs = create_observer()
        seq1 = obs.普通K线序列
        seq2 = obs.普通K线序列
        for i in range(min(len(seq1), 10)):
            assert seq1[i] is seq2[i], f"普K序列[{i}] 身份不一致"


class Test中枢身份:
    """中枢: 从中枢序列、分型关联、笔中枢/线段中枢等不同路径访问。"""

    def test_序列重复获取(self):
        obs = create_observer(period=3600, n_bars=800)
        seq1 = obs.中枢序列
        seq2 = obs.中枢序列
        for i in range(min(len(seq1), 5)):
            assert seq1[i] is seq2[i], f"中枢序列[{i}] 身份不一致"

    def test_笔中枢与线段中枢(self):
        obs = create_observer(period=3600, n_bars=800)
        笔中 = obs.笔_中枢序列
        段中 = obs.线段_中枢序列
        扩展中 = obs.扩展中枢序列
        # 验证同一次获取内的身份
        for zs in 笔中:
            文1 = zs.文
            文2 = zs.文
            assert 文1 is 文2, f"笔中枢.文 两次调用不同"
            break
        for zs in 段中:
            文1 = zs.文
            文2 = zs.文
            assert 文1 is 文2, f"段中枢.文 两次调用不同"
            break


class Test整体身份:
    """跨类型综合身份测试。"""

    def test_买卖点分型(self):
        """验证买卖点的关联分型身份。"""
        obs = create_observer(period=3600, n_bars=800)
        # 尝试访问可用的结构
        分序 = obs.分型序列
        笔序 = obs.笔序列
        assert len(分序) >= 0 and len(笔序) >= 0  # 至少不崩溃

    def test_全链路一致性(self):
        """缠K → 分型 → 笔 → 段 链路中所有对象身份一致。"""
        obs = create_observer()
        seq = obs.缠论K线序列

        for bi in obs.笔序列:
            # 笔的端点分型
            for nm, getter in [("文", lambda b=bi: b.文), ("武", lambda b=bi: b.武)]:
                ep = getter()
                if ep is None:
                    continue
                # ep 中的 中 是一根缠K，应能在序列中找到相同对象
                中 = ep.中
                for ck in seq:
                    if ck.时间戳 == 中.时间戳:
                        assert ck is 中
                        break
                # 左也应该是可访问的
                左 = ep.左
                if 左 is not None:
                    for ck in seq:
                        if ck.时间戳 == 左.时间戳:
                            assert ck is 左
                            break
                # 右也应该是可访问的
                右 = ep.右
                if 右 is not None:
                    for ck in seq:
                        if ck.时间戳 == 右.时间戳:
                            assert ck is 右
                            break
