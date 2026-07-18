#!/usr/bin/env bash
set -euo pipefail

APP_NAME="OpenInfiniFactory"
BIN_NAME="oif"
CARGO_BIN="${CARGO:-cargo}"
TARGET_ARG=()
TARGET_DIR_SEGMENT="release"
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="$ROOT_DIR/dist/open-infinifactory-linux"

if [[ -n "${TARGET:-}" ]]; then
  TARGET_ARG=(--target "$TARGET")
  TARGET_DIR_SEGMENT="$TARGET/release"
fi

cd "$ROOT_DIR"
"$CARGO_BIN" build --release "${TARGET_ARG[@]}"

rm -rf "$DIST_DIR"
mkdir -p "$DIST_DIR"

cp "$ROOT_DIR/target/$TARGET_DIR_SEGMENT/$BIN_NAME" "$DIST_DIR/$BIN_NAME"
cp -R "$ROOT_DIR/assets" "$DIST_DIR/assets"

cat > "$DIST_DIR/$APP_NAME.desktop" <<DESKTOP
[Desktop Entry]
Type=Application
Name=$APP_NAME
Exec=$BIN_NAME
Terminal=false
Categories=Game;
DESKTOP

echo "$DIST_DIR"
