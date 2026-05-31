#!/usr/bin/env bash
set -euo pipefail

CARGO_APK_BIN="${CARGO_APK:-cargo-apk}"
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="$ROOT_DIR/dist/android"
TARGET="${TARGET:-aarch64-linux-android}"

if ! command -v "$CARGO_APK_BIN" >/dev/null 2>&1; then
  echo "cargo-apk is required. Install it with: cargo install cargo-apk" >&2
  exit 1
fi

cd "$ROOT_DIR"
"$CARGO_APK_BIN" apk build --release --target "$TARGET"

rm -rf "$DIST_DIR"
mkdir -p "$DIST_DIR"

find "$ROOT_DIR/target/$TARGET/release/apk" -maxdepth 2 -type f -name '*.apk' -exec cp {} "$DIST_DIR/" \;

echo "$DIST_DIR"
