#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ANDROID_DIR="$ROOT_DIR/mobile/android"
DIST_DIR="$ROOT_DIR/dist/android"
TARGET="${TARGET:-aarch64-linux-android}"
JNI_DIR="$ANDROID_DIR/app/src/main/jniLibs"

export ANDROID_HOME="${ANDROID_HOME:-$HOME/Library/Android/sdk}"
export ANDROID_NDK_HOME="${ANDROID_NDK_HOME:-$ANDROID_HOME/ndk/27.0.12077973}"
export JAVA_HOME="${JAVA_HOME:-/Applications/Android Studio.app/Contents/jbr/Contents/Home}"
export PATH="$JAVA_HOME/bin:$PATH"

# DEBUG=1 时做 debug 构建（保留符号，APK 自动设 debuggable=true）
if [ "${DEBUG:-0}" = "1" ]; then
  CARGO_FLAGS=""
  GRADLE_TASK="assembleDebug"
  APK_PATH="debug/app-debug.apk"
else
  CARGO_FLAGS="--release"
  GRADLE_TASK="assembleRelease"
  APK_PATH="release/app-release.apk"
fi

# 1. 用 cargo-ndk 编译 .so 到 jniLibs
echo "==> Building .so with cargo-ndk..."
cargo ndk -t "$TARGET" -P 26 -o "$JNI_DIR" build $CARGO_FLAGS

# 2. 用 Gradle 打包 APK
echo "==> Building APK with Gradle..."
cd "$ANDROID_DIR"
./gradlew "$GRADLE_TASK" --no-daemon

# 3. 收集产物
rm -rf "$DIST_DIR"
mkdir -p "$DIST_DIR"
cp "$ANDROID_DIR/app/build/outputs/apk/$APK_PATH" "$DIST_DIR/OpenInfiniFactory.apk"

echo "$DIST_DIR/OpenInfiniFactory.apk"
