#!/usr/bin/env python3
"""检测 Rust 源码文件头部是否有 MIT 协议，若无则自动注入。

用法:
    python3 check_license.py                # 检测 chanlun/ 和 chanlun-py/ 下所有 .rs
    python3 check_license.py --check-only   # 仅检测，不修改
    python3 check_license.py --fix          # 检测并修复
    python3 check_license.py path/to/dir    # 指定目录
"""

import argparse
import os
import sys
from pathlib import Path


def find_repo_root() -> Path:
    """从脚本位置向上查找仓库根目录（含 LICENSE 文件的目录）。"""
    current = Path(__file__).resolve().parent
    while current != current.parent:
        if (current / "LICENSE").exists():
            return current
        current = current.parent
    # Fallback: 脚本所在目录的父目录
    return Path(__file__).resolve().parent.parent


def build_license_header(license_path: Path) -> str:
    """读取 LICENSE 文件并格式化为 Rust 块注释头。"""
    lines = license_path.read_text(encoding="utf-8").rstrip("\n").split("\n")
    header_lines = ["/*"]
    for line in lines:
        if line.strip():
            header_lines.append(f" * {line}")
        else:
            header_lines.append(" *")
    header_lines.append(" */")
    header_lines.append("")  # 末尾空行分隔
    return "\n".join(header_lines) + "\n"


def has_license_header(file_path: Path) -> bool:
    """检测文件头部是否已包含块注释风格的 MIT License。"""
    try:
        with open(file_path, "r", encoding="utf-8") as f:
            head = f.read(512)
    except (OSError, UnicodeDecodeError):
        return True
    return head.lstrip().startswith("/*") and "MIT License" in head


def strip_old_license_header(text: str) -> str:
    """去除文件中已有的 // 风格 license header（重新注入前调用）。"""
    stripped = text.lstrip("\n")
    if stripped.startswith("// MIT License"):
        # 找到 // 注释块结束位置（第一个非 // 非空行）
        lines = stripped.split("\n")
        end_idx = 0
        for i, line in enumerate(lines):
            if line.startswith("//") or line.strip() == "":
                end_idx = i + 1
            else:
                break
        return "\n".join(lines[end_idx:])
    return text


def inject_license(file_path: Path, header: str) -> bool:
    """将 license header 注入文件头部。返回 True 表示已修改。"""
    original = file_path.read_text(encoding="utf-8")
    # 已有块注释风格则跳过
    if original.lstrip().startswith("/*") and "MIT License" in original[:512]:
        return False
    # 去除旧的 // 风格 header（如果存在）
    cleaned = strip_old_license_header(original)
    file_path.write_text(header + cleaned, encoding="utf-8")
    return True


def collect_rs_files(roots: list[Path]) -> list[Path]:
    """递归收集所有 .rs 文件，排除 target/ 等构建产物目录。"""
    exclude_dirs = {"target", ".git", "__pycache__", "dist", "build", ".venv", "venv"}
    files = []
    for root in roots:
        if not root.is_dir():
            continue
        for path in root.rglob("*.rs"):
            if any(excl in path.parts for excl in exclude_dirs):
                continue
            files.append(path)
    return sorted(files)


def main() -> int:
    parser = argparse.ArgumentParser(description="检测 Rust 源码 MIT 协议头")
    parser.add_argument(
        "paths",
        nargs="*",
        help="要检测的目录（默认: chanlun 和 chanlun-py 源码目录）",
    )
    parser.add_argument(
        "--check-only",
        action="store_true",
        help="仅检测，不修改文件",
    )
    parser.add_argument(
        "--fix",
        action="store_true",
        help="检测并自动注入缺失的协议头（默认行为）",
    )
    args = parser.parse_args()

    repo_root = find_repo_root()
    license_path = repo_root / "LICENSE"

    if not license_path.exists():
        print(f"错误: 未找到 LICENSE 文件 ({license_path})", file=sys.stderr)
        return 1

    header = build_license_header(license_path)

    # 确定扫描目录
    if args.paths:
        roots = [Path(p).resolve() for p in args.paths]
    else:
        roots = [
            repo_root / "chanlun" / "src",
            repo_root / "chanlun-py" / "src",
            repo_root / "chanlun" / "tests",
            repo_root / "chanlun-py" / "tests",
        ]
        roots = [r for r in roots if r.is_dir()]

    if not roots:
        print("错误: 未找到任何源码目录", file=sys.stderr)
        return 1

    rs_files = collect_rs_files(roots)

    if not rs_files:
        print("未找到 .rs 文件")
        return 0

    missing = []
    injected = []

    for f in rs_files:
        if has_license_header(f):
            continue
        missing.append(f)
        if not args.check_only:
            if inject_license(f, header):
                injected.append(f)

    if args.check_only:
        if missing:
            print(f"缺失 MIT 协议头: {len(missing)} 个文件")
            for f in missing:
                print(f"  {f}")
            return 1
        else:
            print(f"全部 {len(rs_files)} 个 .rs 文件均已包含 MIT 协议头")
            return 0
    else:
        if injected:
            print(f"已注入 MIT 协议头: {len(injected)} 个文件")
            for f in injected:
                print(f"  {f}")
        if missing:
            already = len(missing) - len(injected)
            if already > 0:
                print(f"已有协议头: {already} 个文件（无需修改）")
        total = len(rs_files) - len(missing)
        print(f"总计: {len(rs_files)} 个 .rs 文件, {total} 个已含协议头")
        return 0


if __name__ == "__main__":
    sys.exit(main())
