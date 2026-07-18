#!/usr/bin/env bash
# 更新场景/材料方块 UI 图标：改完外观后跑这个
# 用法:
#   ./scripts/bake_scene_icons.sh
#   ./scripts/bake_scene_icons.sh --materials-only
#   ./scripts/bake_scene_icons.sh --scene-only
#   ./scripts/bake_scene_icons.sh --only iron
#   ./scripts/bake_scene_icons.sh --size 64 --output icon_64.png
set -euo pipefail
cd "$(dirname "$0")/.."
exec cargo run --features native-tools --bin bake_scene_icons -- "$@"
