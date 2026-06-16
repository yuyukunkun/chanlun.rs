#!/usr/bin/env bash
set -e
cd "$(dirname "$0")"

echo "=== 1/4 清除 Python 缓存 ==="
find . -type d -name __pycache__ -exec rm -rf {} + 2>/dev/null
find . -type f -name "*.pyc" -delete 2>/dev/null
echo "  Python 缓存已清除"

echo "=== 2/4 清除 Cargo 编译缓存 ==="
rm -rf chanlun/target chanlun-py/target
echo "  target/ 已清除"

echo "=== 3/4 构建 Release ==="
cd chanlun-py
maturin build --release
echo "  构建完成"

echo "=== 4/4 安装 ==="
pip install --break-system-packages --force-reinstall --no-deps \
    target/wheels/chanlun-*.whl
echo "  安装完成"

echo
echo "✓ 清理 + 构建 + 安装完毕"
echo "  pip show chanlun | grep Version"
pip show chanlun 2>/dev/null | grep Version
