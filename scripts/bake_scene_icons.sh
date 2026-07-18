#!/usr/bin/env bash
# 更新场景方块 UI 图标：改完 model.glb 后跑这个
# 用法:
#   ./scripts/bake_scene_icons.sh
#   ./scripts/bake_scene_icons.sh --size 64 --output icon_64.png
#   ./scripts/bake_scene_icons.sh --only grass
set -euo pipefail
cd "$(dirname "$0")/.."
exec cargo run --features native-tools --bin bake_scene_icons -- "$@"
