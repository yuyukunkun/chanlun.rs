# Sphinx 配置 — chanlun 缠论技术分析库
#
# 使用 MyST 解析器支持 Markdown，Furo 主题

import os
import sys

# 确保文档构建时能 import chanlun
sys.path.insert(0, os.path.abspath(".."))

# ---- 项目信息 ----
project = "chanlun"
copyright = "2026, YuWuKunCheng"
author = "YuWuKunCheng"
release = "26.5.103"
language = "zh_CN"

# ---- 扩展 ----
extensions = [
    "myst_parser",  # Markdown 支持
    "sphinx.ext.viewcode",  # 添加源码链接
    "sphinx.ext.intersphinx",  # 跨项目文档链接
]

# ---- MyST 配置 ----
myst_enable_extensions = [
    "colon_fence",  # ::: 围栏代码块
    "fieldlist",  # 字段列表
]
myst_heading_anchors = 3

# ---- 主题 ----
html_theme = "furo"
html_title = "chanlun — 缠论技术分析库"
html_theme_options = {
    "source_repository": "https://github.com/yuwukuncheng/chanlun.rs",
    "source_branch": "main",
    "source_directory": "docs/",
}

# ---- 跨项目链接 ----
intersphinx_mapping = {
    "python": ("https://docs.python.org/3", None),
}

add_module_names = False
