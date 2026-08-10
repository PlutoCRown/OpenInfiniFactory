#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# 开发包使用增量友好的轻优化配置，并复用不影响功能验证的既有产物。
export DEBUG=1
export SKIP_ANDROID_ICONS="${SKIP_ANDROID_ICONS:-1}"
export USE_GRADLE_DAEMON="${USE_GRADLE_DAEMON:-1}"
export KEEP_ANDROID_DIST="${KEEP_ANDROID_DIST:-1}"
export ANDROID_APK_NAME="${ANDROID_APK_NAME:-OpenInfiniFactory-dev.apk}"
export CARGO_PROFILE_DEV_OPT_LEVEL="${CARGO_PROFILE_DEV_OPT_LEVEL:-1}"
export CARGO_PROFILE_DEV_DEBUG="${CARGO_PROFILE_DEV_DEBUG:-0}"

exec "$ROOT_DIR/scripts/package_android.sh"
