#!/usr/bin/env bash
# 中科院软件所（ISCAS）考研报考指南 - Linux 构建脚本
# 供 Cloudflare Pages / CI 等 Linux 环境使用
#
# 用法：
#   bash build-site.sh          # 构建静态网站到 site/（首次自动安装锁定依赖）

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
cd "$SCRIPT_DIR"

# ---- 智能选择 Python 解释器（本地 venv 优先，CI 用系统 python） ----
if [ -x "$REPO_ROOT/.venv/Scripts/python.exe" ]; then
  PY="$REPO_ROOT/.venv/Scripts/python.exe"          # Windows repository venv
elif [ -x "$SCRIPT_DIR/.venv/Scripts/python.exe" ]; then
  PY="$SCRIPT_DIR/.venv/Scripts/python.exe"          # Windows
elif [ -x "$SCRIPT_DIR/.venv/bin/python" ]; then
  PY="$SCRIPT_DIR/.venv/bin/python"                   # Linux venv
else
  PY="python"                             # Cloudflare Pages / CI
fi

# ---- 检查构建依赖（缺失时自动安装 requirements.txt 锁定依赖） ----
# Cloudflare Pages 每次构建都是全新环境，必须自动安装，否则构建会失败。
if ! "$PY" -c "import material" 2>/dev/null; then
  echo "首次运行：安装构建依赖（requirements.txt）..."
  "$PY" -m pip install -q -r requirements.txt
  if ! "$PY" -c "import material" 2>/dev/null; then
    echo "依赖安装失败，请检查网络后重试" >&2
    exit 1
  fi
fi

# ---- 同步内容到构建源目录（仓库 markdown 仍是唯一内容源） ----
echo "同步内容到 docs_src/ ..."
rm -rf "$SCRIPT_DIR/docs_src"
mkdir -p docs_src
cp -r "$REPO_ROOT/初试准备" "$REPO_ROOT/复试准备" "$REPO_ROOT/上岸经验分享" "$REPO_ROOT/毕业去向" docs_src/
cp "$REPO_ROOT/经验分享投稿模板.md" "$REPO_ROOT/CONTRIBUTORS.md" "$REPO_ROOT/免责声明.md" docs_src/

# ---- 网站层匿名化（学校名 → 层次，仅改 docs_src 副本，源文件不动） ----
"$PY" anonymize.py docs_src

# 首页已经使用公开层次描述；匿名化后再复制，避免替换链接中的原始文件路径。
cp "$SCRIPT_DIR/homepage.md" docs_src/index.md

# ---- 构建 ----
"$PY" -m mkdocs build
echo "构建完成：$(pwd)/site/index.html"
