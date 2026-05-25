#!/usr/bin/env python3
"""Integration test: feed .nb bars through PyO3 observer, compare with Python reference output."""

import sys
import os
import struct

import chanlun


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


def main():
    nb_path = "/home/moscow/chanlun.rs/btcusd-300-1761327300-1776327900.nb"
    ref_dir = "/home/moscow/chanlun.rs/Py_btcusd:300_1761327300_1776327900"
    out_dir = "/tmp/chanlun_py_test_output"

    # Read all bars
    print("Reading bars from .nb file...")
    bars = read_nb_bars(nb_path)
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
    ref_files = sorted(os.listdir(ref_dir))

    match_count = 0
    diff_count = 0
    all_match = True
    for fname in ref_files:
        ref_path = os.path.join(ref_dir, fname)
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
    rust_ref_dir = "/home/moscow/chanlun.rs/chanlun/Rust_btcusd:300_1761327300_1776327900"
    extra_files = set(out_files) - set(ref_files)
    if extra_files:
        print("\nComparing extra files with Rust reference...")
        for fname in sorted(extra_files):
            out_path = os.path.join(actual_out_dir, fname)
            rust_ref_path = os.path.join(rust_ref_dir, fname)
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


if __name__ == "__main__":
    sys.exit(main())
