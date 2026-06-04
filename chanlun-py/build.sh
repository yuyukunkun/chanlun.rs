#!/usr/bin/env bash
# ============================================================
# chanlun-py — 构建、安装、发布脚本
#
# 前置依赖:
#   pip install maturin
# ============================================================
set -euo pipefail

PROJECT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$PROJECT_DIR"

# 颜色输出
red()    { echo -e "\033[31m$*\033[0m"; }
green()  { echo -e "\033[32m$*\033[0m"; }
yellow() { echo -e "\033[33m$*\033[0m"; }

usage() {
    cat <<'EOF'
用法: ./build.sh <命令>

命令:
  build       开发模式编译（debug）
  release     发布模式编译（release，带优化）
  install     构建 wheel 并安装到当前 Python 环境
  install-rel 同 install
  develop     开发模式安装到 venv（需要激活虚拟环境）
  test        运行集成测试
  sdist       构建源码分发包
  wheel       构建 wheel 包（release）
  publish     发布至 PyPI（需先配置 TWINE_ 环境变量或 .pypirc）
  clean       清理构建产物
  bump        版本号自增（年月.次 → 2605.X）
  check       检查 pyproject.toml 配置是否合法

PyPI 发布流程:
  1. 确认 Cargo.toml 中 chanlun 依赖已切换为 crates.io 版本（非 path）
  2. 更新版本号: pyproject.toml + Cargo.toml
  3. ./build.sh check    — 验证配置
  4. ./build.sh publish  — 构建并推送至 PyPI

  环境变量:
    TWINE_USERNAME  — PyPI 用户名（或 __token__ 使用 API token）
    TWINE_PASSWORD  — PyPI 密码或 API token
    或通过 twine 的 ~/.pypirc 配置
EOF
}

cmd_build() {
    green "[build] 开发模式编译..."
    cargo build
    green "编译完成。使用 'maturin develop' 安装到 venv，或 './build.sh wheel' 构建 wheel"
}

cmd_release() {
    green "[release] 发布模式编译..."
    cargo build --release
}

cmd_bump() {
    green "[bump] 版本号自增..."
    python3 "$PROJECT_DIR/bump_version.py"
}

cmd_install() {
    cmd_bump
    green "[install] 构建 wheel 并安装..."
    maturin build --release 2>&1 | tail -3
    local wheel=$(ls -t target/wheels/*.whl 2>/dev/null | head -1)
    if [ -z "$wheel" ]; then
        red "未找到 wheel 文件"
        exit 1
    fi
    pip install --force-reinstall "$wheel" 2>&1 | tail -3
    green "安装完成!"
}

cmd_install_rel() {
    cmd_install
}

cmd_develop() {
    green "[develop] 开发模式安装（需要 venv）..."
    maturin develop 2>&1 | tail -3
}

cmd_test() {
    green "[test] 运行集成测试..."
    # 确保已安装
    python3 -c "import chanlun" 2>/dev/null || {
        yellow "chanlun 未安装，正在构建并安装..."
        maturin build --release 2>&1 | tail -3
        local wheel=$(ls -t target/wheels/*.whl 2>/dev/null | head -1)
        pip install --force-reinstall "$wheel" 2>&1 | tail -3
    }
    # 复制到临时目录运行，避免本地 chanlun/ 目录被优先导入
    local tmp_dir=$(mktemp -d 2>/dev/null || echo "${TMPDIR:-/tmp}/chanlun-test-$$")
    mkdir -p "$tmp_dir"
    cp test_integration.py "$tmp_dir/test_integration.py"
    CHANLUN_PROJECT_ROOT="$PROJECT_DIR/.." python3 "$tmp_dir/test_integration.py" "$@"
    rm -rf "$tmp_dir"
}

cmd_sdist() {
    green "[sdist] 构建源码分发包..."
    maturin sdist
    yellow "构建产物在: target/wheels/"
    ls -lh target/wheels/*.tar.gz 2>/dev/null || true
}

cmd_wheel() {
    cmd_bump
    green "[wheel] 构建 wheel 包..."
    maturin build --release
    yellow "构建产物在: target/wheels/"
    ls -lh target/wheels/*.whl 2>/dev/null || true
}

cmd_publish() {
    cmd_bump
    yellow "发布前请确认:"
    yellow "  1. Cargo.toml 中 chanlun 依赖已切换为 crates.io 版本"
    yellow "  2. 版本号已更新 (pyproject.toml + Cargo.toml)"
    yellow "  3. git 工作区干净且已打 tag"
    echo ""
    read -rp "确认发布至 PyPI? [y/N] " confirm
    if [[ "$confirm" != "y" && "$confirm" != "Y" ]]; then
        red "已取消"
        exit 0
    fi

    green "[publish] 构建并发布至 PyPI..."
    maturin publish

    green "发布完成!"
    yellow "安装: pip install chanlun"
}

cmd_clean() {
    green "[clean] 清理构建产物..."
    cargo clean
    rm -rf target/wheels/ dist/ build/ *.egg-info/
    find . -type d -name "__pycache__" -exec rm -rf {} + 2>/dev/null || true
    green "清理完成"
}

cmd_check() {
    green "[check] 验证 pyproject.toml..."
    python3 -c "
import tomllib
with open('pyproject.toml', 'rb') as f:
    data = tomllib.load(f)
print('  name:', data['project']['name'])
print('  version:', data['project']['version'])
print('  requires-python:', data['project']['requires-python'])
print('  build-backend:', data['build-system']['build-backend'])
print('  OK')
"
    green "配置验证通过"
}

# --- main ---
case "${1:-}" in
    build)       cmd_build ;;
    release)     cmd_release ;;
    install)     cmd_install ;;
    install-rel) cmd_install_rel ;;
    develop)     cmd_develop ;;
    test)        cmd_test ;;
    sdist)       cmd_sdist ;;
    wheel)       cmd_wheel ;;
    publish)     cmd_publish ;;
    clean)       cmd_clean ;;
    bump)        cmd_bump ;;
    check)       cmd_check ;;
    -h|--help|help) usage ;;
    *)           red "未知命令: ${1:-}"; usage; exit 1 ;;
esac
