# 仓库根路径与资源目录约定
"""资源生成脚本共用的路径常量。"""

from __future__ import annotations

from pathlib import Path

# tools/assets/common/paths.py → 仓库根
REPO_ROOT = Path(__file__).resolve().parents[3]

ASSETS = REPO_ROOT / "assets"
FACTORY_BLOCKS = ASSETS / "factory_blocks"
MATERIAL_BLOCKS = ASSETS / "material_blocks"
SCENE_BLOCKS = ASSETS / "scene_blocks"
STAMP_MATERIALS = ASSETS / "stamp_materials"
PAINT_MATERIALS = ASSETS / "paint_materials"
