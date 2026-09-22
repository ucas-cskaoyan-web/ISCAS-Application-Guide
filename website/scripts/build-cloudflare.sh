#!/usr/bin/env bash
# Build the ISCAS guide for Cloudflare Pages.
#
# Required in the build image: node/npm, rustup/cargo, nu and zstd.
# The custom Trunk revision is pinned so a future upstream change cannot
# silently alter the generated site.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
SITE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TOOLS_DIR="$SITE_ROOT/.ci-tools"
TRUNK_REV="4758424b9c026b79cfe94fde1ef318df4f9c9216"
PUBLIC_URL="${PUBLIC_URL:-/}"

# Keep downloaded helper binaries local to the site checkout. This also makes
# the same script usable from Git Bash on Windows.
mkdir -p "$TOOLS_DIR/bin"
export PATH="$TOOLS_DIR/bin:$PATH"

cd "$SITE_ROOT"

require_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "缺少构建工具：$1" >&2
    exit 1
  fi
}

require_command node
require_command npm
require_command rustup
require_command cargo
require_command nu
require_command zstd

echo "安装前端依赖..."
npm ci

echo "准备 Rust WASM 目标..."
rustup target add wasm32-unknown-unknown

TRUNK_BIN="$TOOLS_DIR/bin/trunk"
if [[ "$(uname -s)" == MINGW* || "$(uname -s)" == MSYS* || "$(uname -s)" == CYGWIN* ]]; then
  TRUNK_BIN="$TOOLS_DIR/bin/trunk.exe"
fi
if [ ! -x "$TRUNK_BIN" ]; then
  echo "编译固定版本的 Trunk ($TRUNK_REV)..."
  cargo install \
    --git https://github.com/bigsaltyfishes/trunk.git \
    --rev "$TRUNK_REV" \
    --locked \
    --root "$TOOLS_DIR" \
    trunk
fi

echo "构建网站 (PUBLIC_URL=$PUBLIC_URL)..."
"$TRUNK_BIN" build --release --public-url "$PUBLIC_URL"

# Cloudflare Pages uses this rule for history-based client-side routes.
cp "$SITE_ROOT/_redirects" "$SITE_ROOT/dist/_redirects"

test -f "$SITE_ROOT/dist/index.html"
echo "构建完成：$SITE_ROOT/dist"
