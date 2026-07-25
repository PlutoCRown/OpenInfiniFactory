#!/usr/bin/env bash
# 批量重导工厂块 model.glb（含顶点色 AO）
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../../../.." && pwd)"
BLENDER="${BLENDER:-/Applications/Blender.app/Contents/MacOS/Blender}"
cd "$ROOT"

scripts=(
  tools/assets/models/factory/generate_suction_cup_glb.py
  tools/assets/models/factory/generate_lifter_glb.py
  tools/assets/models/factory/generate_welder_glb.py
  tools/assets/models/factory/generate_detector_glb.py
  tools/assets/models/factory/generate_pusher_glb.py
  tools/assets/models/factory/generate_drill_glb.py
  tools/assets/models/factory/generate_conveyor_glb.py
  tools/assets/models/factory/generate_rotator_glb.py
  tools/assets/models/factory/generate_wire_glb.py
  tools/assets/models/factory/generate_platform_glb.py
  tools/assets/models/factory/generate_optics_and_suction_glb.py
  tools/assets/models/factory/generate_light_panel_glb.py
  tools/assets/models/factory/generate_selection_glb.py
  tools/assets/models/factory/generate_selection_box_glb.py
)

for s in "${scripts[@]}"; do
  echo "======== $s ========"
  PYTHONDONTWRITEBYTECODE=1 "$BLENDER" --background --python "$s"
done

echo "Done. Optional icons: ./scripts/bake_factory_icons.sh --only lifter"
