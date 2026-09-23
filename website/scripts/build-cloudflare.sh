#!/usr/bin/env bash
# Build the ISCAS guide for Cloudflare Pages.
#
# Required in the build image: node/npm, rustup/cargo and nu.
# The custom Trunk revision is pinned so a future upstream change cannot
# silently alter the generated site.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
SITE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TOOLS_DIR="$SITE_ROOT/.ci-tools"
TRUNK_REV="4758424b9c026b79cfe94fde1ef318df4f9c9216"
NU_VERSION="0.115.1"
NU_SHA256="d11d825241f6504a3617c535fa725a9dd6d009c86d7b19fb3168b47635b9d8b0"
PUBLIC_URL="${PUBLIC_URL:-/}"

# Keep downloaded helper binaries local to the site checkout. This also makes
# the same script usable from Git Bash on Windows.
mkdir -p "$TOOLS_DIR/bin"
export PATH="$TOOLS_DIR/bin:$PATH"

# Trunk's CLI parser accepts boolean values here, while some CI environments
# expose the conventional numeric form.
if [ "${NO_COLOR:-}" = "1" ]; then
  export NO_COLOR="true"
fi

cd "$SITE_ROOT"

require_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "缺少构建工具：$1" >&2
    exit 1
  fi
}

require_command node
require_command npm

if ! command -v rustup >/dev/null 2>&1; then
  require_command curl
  echo "安装 Rust 工具链..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --profile minimal --default-toolchain nightly
  export PATH="$HOME/.cargo/bin:$PATH"
fi

if ! command -v nu >/dev/null 2>&1; then
  require_command curl
  require_command tar
  require_command sha256sum
  echo "安装 NuShell $NU_VERSION..."
  NU_ARCHIVE="$TOOLS_DIR/nu-$NU_VERSION-x86_64-unknown-linux-gnu.tar.gz"
  curl -fL --retry 3 \
    "https://github.com/nushell/nushell/releases/download/$NU_VERSION/nu-$NU_VERSION-x86_64-unknown-linux-gnu.tar.gz" \
    -o "$NU_ARCHIVE"
  echo "$NU_SHA256  $NU_ARCHIVE" | sha256sum -c -
  tar -xzf "$NU_ARCHIVE" -C "$TOOLS_DIR"
  cp "$TOOLS_DIR/nu-$NU_VERSION-x86_64-unknown-linux-gnu/nu" "$TOOLS_DIR/bin/nu"
  chmod +x "$TOOLS_DIR/bin/nu"
fi

require_command rustup
require_command cargo
require_command nu

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
