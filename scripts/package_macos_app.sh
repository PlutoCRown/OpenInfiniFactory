#!/usr/bin/env bash
set -euo pipefail

APP_NAME="OpenInfiniFactory"
BIN_NAME="oif"
CARGO_BIN="${CARGO:-cargo}"
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="$ROOT_DIR/dist"
APP_DIR="$DIST_DIR/$APP_NAME.app"
CONTENTS_DIR="$APP_DIR/Contents"
MACOS_DIR="$CONTENTS_DIR/MacOS"
RESOURCES_DIR="$CONTENTS_DIR/Resources"

cd "$ROOT_DIR"
"$CARGO_BIN" build --release -p open_infinifactory --bin "$BIN_NAME"

rm -rf "$APP_DIR"
mkdir -p "$MACOS_DIR" "$RESOURCES_DIR"

BIN_PATH="$ROOT_DIR/target/release/$BIN_NAME"
cp "$BIN_PATH" "$MACOS_DIR/$APP_NAME"
cp -R "$ROOT_DIR/assets" "$RESOURCES_DIR/assets"

# cargo strip=true 已去符号；再跑一遍系统 strip 兜底（幂等）
strip "$MACOS_DIR/$APP_NAME" 2>/dev/null || true
echo "binary: $(du -h "$MACOS_DIR/$APP_NAME" | awk '{print $1}') (was built with profile.release size opts)"

cat > "$CONTENTS_DIR/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleDevelopmentRegion</key>
  <string>en</string>
  <key>CFBundleDisplayName</key>
  <string>$APP_NAME</string>
  <key>CFBundleExecutable</key>
  <string>$APP_NAME</string>
  <key>CFBundleIdentifier</key>
  <string>dev.openinfinifactory.prototype</string>
  <key>CFBundleInfoDictionaryVersion</key>
  <string>6.0</string>
  <key>CFBundleName</key>
  <string>$APP_NAME</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleShortVersionString</key>
  <string>0.1.0</string>
  <key>CFBundleVersion</key>
  <string>0.1.0</string>
  <key>LSMinimumSystemVersion</key>
  <string>11.0</string>
  <key>NSHighResolutionCapable</key>
  <true/>
</dict>
</plist>
PLIST

echo "$APP_DIR"
