#!/usr/bin/env python3
"""版本号自增 — pyproject.toml(2605.X) + Cargo.toml(26.5.X)

规则: YYMM 匹配当前年月 → patch 自增; 不匹配 → 重置为 NEW_YYMM.1
仅当版本号需要变化时才修改文件。
"""

import os
import re
import sys
from datetime import date

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
PYPROJECT = os.path.join(SCRIPT_DIR, "pyproject.toml")
CARGO_TOML = os.path.join(SCRIPT_DIR, "Cargo.toml")


def read_file(path: str) -> str:
    with open(path, "r") as f:
        return f.read()


def write_file(path: str, content: str) -> None:
    with open(path, "w") as f:
        f.write(content)


def main() -> None:
    today = date.today()
    new_yymm = today.strftime("%y%m")  # 2605

    # --- pyproject.toml ---
    pyproject = read_file(PYPROJECT)
    m = re.search(r'^version\s*=\s*"(\d{4})\.(\d+)"', pyproject, re.MULTILINE)
    if not m:
        print("ERROR: 无法从 pyproject.toml 解析版本号", file=sys.stderr)
        sys.exit(1)

    old_yymm, old_patch = m.group(1), int(m.group(2))

    if old_yymm == new_yymm:
        new_patch = old_patch + 1
    else:
        new_patch = 1

    new_py_ver = f"{new_yymm}.{new_patch}"
    if old_yymm == new_yymm and old_patch == new_patch:
        print(f"版本号未变: {new_py_ver}")
        return

    new_pyproject = re.sub(
        r'^(version\s*=\s*)"\d{4}\.\d+"',
        rf'\1"{new_py_ver}"',
        pyproject,
        flags=re.MULTILINE,
    )
    write_file(PYPROJECT, new_pyproject)
    print(f'pyproject.toml:  {m.group(0).split("=")[1].strip()} → "{new_py_ver}"')

    # --- Cargo.toml ---
    new_cargo_ver = f"{new_yymm[:2]}.{int(new_yymm[2:])}.{new_patch}"  # 26.5.2
    cargo = read_file(CARGO_TOML)
    new_cargo = re.sub(
        r'^(version\s*=\s*)"\d+\.\d+\.\d+"',
        rf'\1"{new_cargo_ver}"',
        cargo,
        flags=re.MULTILINE,
    )
    write_file(CARGO_TOML, new_cargo)
    print(f'Cargo.toml:     → "{new_cargo_ver}"')


if __name__ == "__main__":
    main()
