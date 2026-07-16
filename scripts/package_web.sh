#!/usr/bin/env bash
# 打包 Web（wasm32 + WebGPU）到 dist/web；与桌面 Release 产物目录不冲突
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="$ROOT_DIR/dist/web"
CARGO_BIN="${CARGO:-cargo}"
FEATURES="${FEATURES:-webgpu}"

# trunk 常装在 ~/.cargo/bin，当前 shell PATH 可能没有
resolve_trunk() {
  if [[ -n "${TRUNK:-}" ]]; then
    echo "$TRUNK"
    return
  fi
  if command -v trunk >/dev/null 2>&1; then
    command -v trunk
    return
  fi
  if [[ -x "${HOME}/.cargo/bin/trunk" ]]; then
    echo "${HOME}/.cargo/bin/trunk"
    return
  fi
  echo "error: trunk not found (install: cargo install trunk; or set TRUNK=/path/to/trunk)" >&2
  exit 1
}

TRUNK_BIN="$(resolve_trunk)"

cd "$ROOT_DIR"

# trunk 0.21 把 NO_COLOR=1 解析成非法 --no-color 1；改成 true 或清掉
if [[ "${NO_COLOR:-}" == "1" ]]; then
  export NO_COLOR=true
fi

# 确保 wasm 目标已安装
if ! rustup target list --installed 2>/dev/null | grep -qx 'wasm32-unknown-unknown'; then
  rustup target add wasm32-unknown-unknown
fi

echo "trunk: $("$TRUNK_BIN" --version)"
echo "features: $FEATURES"
echo "dist: $DIST_DIR"

# Trunk.toml 已设 dist=dist/web；显式再传一次避免环境覆盖
# 与 macOS/Linux Release 可并行：产物目录不同（dist/web vs dist/*.app），
# 但若同时跑两个 cargo，会抢同一把 workspace 锁，排队即可，不会互相覆盖产物。
"$TRUNK_BIN" build --release --no-default-features --features "$FEATURES" --dist "$DIST_DIR"

echo "---"
du -sh "$DIST_DIR"/* 2>/dev/null | sort -hr | head -20
echo "web package: $DIST_DIR"
echo "serve: $TRUNK_BIN serve --release --no-default-features --features $FEATURES"
