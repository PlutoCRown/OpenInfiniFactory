# 资源外观生成工具（按功能划分）

"""OpenInfiniFactory 资源生成入口。

按「做什么」分目录：

common/ 共用：路径、bpy、PNG、GLB 换皮
models/factory/ 工厂块 → model.glb（Blender bpy）
models/material/ 材料块 → model.glb（bpy）
models/scene/ 场景块 → model.glb（bpy）
models/stamp/ 印花薄板 → model.glb（bpy）
textures/ 只写 texture.png / normal.png（PIL）

约定：

- 新建/大改 model.glb → models/ 下用 bpy
- 只写贴图 → textures/ 下用 PIL
- icon.png → ./scripts/bake_scene_icons.sh，禁止手绘

示例：
/Applications/Blender.app/Contents/MacOS/Blender --background \\
--python tools/assets/models/factory/generate_pusher_glb.py

/Applications/Blender.app/Contents/MacOS/Blender --background \\
--python tools/assets/models/scene/generate_quartz.py

python3 tools/assets/textures/generate_pixel_textures.py
"""
