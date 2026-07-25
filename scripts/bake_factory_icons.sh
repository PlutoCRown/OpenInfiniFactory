#!/usr/bin/env bash
# 更新工厂/系统方块与选区工具 UI 图标：改完工厂外观后跑这个
# 用法:
#   ./scripts/bake_factory_icons.sh
#   ./scripts/bake_factory_icons.sh --only conveyor
#   ./scripts/bake_factory_icons.sh --only light_panel
#   ./scripts/bake_factory_icons.sh --only selection
set -euo pipefail
cd "$(dirname "$0")/.."
exec cargo run --features native-tools --bin bake_scene_icons -- --factory-only "$@"
