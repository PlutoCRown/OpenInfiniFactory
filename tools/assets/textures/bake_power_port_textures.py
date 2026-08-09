"""烘焙两种供电口 PBR 贴图（方块侧板 / 电线横截面）。

输出（可供预览；后续减面模型会引用同一目录）：
  assets/factory_blocks/_shared/power_port/block/
    albedo.png normal.png ao.png roughness.png metallic.png height.png
  assets/factory_blocks/_shared/power_port/wire/
    同上

几何与材质来自 common.power_port_geom（与活塞/传感器/电线脚本共用常量）。

用法：
  /Applications/Blender.app/Contents/MacOS/Blender --background \\
    --python tools/assets/textures/bake_power_port_textures.py
"""

from __future__ import annotations

from pathlib import Path
import sys

_TOOLS = Path(__file__).resolve().parents[1]
if str(_TOOLS) not in sys.path:
    sys.path.insert(0, str(_TOOLS))

from common.paths import REPO_ROOT
from common.bpy_util import clear_scene, finish, set_active
from common.power_port_geom import (
    bake_plane_size_for_block,
    bake_plane_size_for_wire,
    build_block_port_facing_pos_z,
    build_wire_port_facing_pos_z,
    power_port_materials,
)

import bpy

OUT_ROOT = REPO_ROOT / "assets" / "factory_blocks" / "_shared" / "power_port"
TEX_RES = 1024
CAGE_EXTRUSION = 0.08
BAKE_MARGIN = 4
SAMPLES = 128


def _clear_geometry() -> None:
    """清空物体/网格/材质，保留已烘焙的 Image（clear_scene 会删图）。"""
    bpy.ops.object.select_all(action="SELECT")
    bpy.ops.object.delete(use_global=False)
    for coll in (
        bpy.data.meshes,
        bpy.data.materials,
        bpy.data.cameras,
        bpy.data.lights,
    ):
        for block in list(coll):
            coll.remove(block)


def _ensure_cycles() -> None:
    scene = bpy.context.scene
    scene.render.engine = "CYCLES"
    scene.cycles.device = "CPU"
    scene.cycles.samples = SAMPLES
    if scene.world is None:
        scene.world = bpy.data.worlds.new("World")
    scene.world.use_nodes = True
    nt = scene.world.node_tree
    nt.nodes.clear()
    bg = nt.nodes.new("ShaderNodeBackground")
    bg.inputs["Color"].default_value = (0.8, 0.8, 0.8, 1.0)
    bg.inputs["Strength"].default_value = 1.0
    out = nt.nodes.new("ShaderNodeOutputWorld")
    nt.links.new(bg.outputs["Background"], out.inputs["Surface"])


def _new_image(name: str, *, alpha: bool = True) -> bpy.types.Image:
    img = bpy.data.images.new(
        name, width=TEX_RES, height=TEX_RES, alpha=alpha, float_buffer=False
    )
    img.colorspace_settings.name = "sRGB" if "albedo" in name.lower() else "Non-Color"
    return img


def _make_bake_plane(size: float, fill_mat: bpy.types.Material) -> bpy.types.Object:
    """XY 平面，中心原点，法线 +Z；自带 0–1 UV。"""
    bpy.ops.mesh.primitive_plane_add(size=size, location=(0, 0, -0.002))
    plane = bpy.context.active_object
    plane.name = "BakePlane"
    finish(plane, fill_mat)
    set_active(plane)
    bpy.ops.object.mode_set(mode="EDIT")
    bpy.ops.uv.unwrap(method="ANGLE_BASED", margin=0.001)
    bpy.ops.object.mode_set(mode="OBJECT")
    return plane


def _attach_bake_image(plane: bpy.types.Object, image: bpy.types.Image) -> None:
    """把 Image Texture 挂到平面材质并设为 active，供 bake 写入。"""
    mat = plane.data.materials[0]
    mat.use_nodes = True
    nt = mat.node_tree
    for node in list(nt.nodes):
        if node.type == "TEX_IMAGE":
            nt.nodes.remove(node)
    tex = nt.nodes.new("ShaderNodeTexImage")
    tex.image = image
    tex.select = True
    nt.nodes.active = tex


def _select_sources_active_plane(
    sources: list[bpy.types.Object], plane: bpy.types.Object
) -> None:
    bpy.ops.object.select_all(action="DESELECT")
    for obj in sources:
        obj.select_set(True)
    plane.select_set(True)
    bpy.context.view_layer.objects.active = plane


def _bake(
    sources: list[bpy.types.Object],
    plane: bpy.types.Object,
    image: bpy.types.Image,
    bake_type: str,
    *,
    pass_filter: set[str] | None = None,
) -> None:
    _attach_bake_image(plane, image)
    _select_sources_active_plane(sources, plane)
    scene = bpy.context.scene
    scene.cycles.bake_type = bake_type
    bake = scene.render.bake
    bake.use_selected_to_active = True
    bake.cage_extrusion = CAGE_EXTRUSION
    bake.margin = BAKE_MARGIN
    bake.use_clear = True
    kwargs: dict = {"type": bake_type, "use_clear": True}
    if pass_filter is not None:
        kwargs["pass_filter"] = pass_filter
    print(f"  bake {bake_type} → {image.name}…", file=sys.stderr)
    bpy.ops.object.bake(**kwargs)


def _save_image(image: bpy.types.Image, path: Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    image.filepath_raw = str(path)
    image.file_format = "PNG"
    image.save()
    print(f"  wrote {path}", file=sys.stderr)


def _bake_basecolor_as_emit(
    sources: list[bpy.types.Object],
    plane: bpy.types.Object,
    image: bpy.types.Image,
) -> None:
    """Principled Base Color → Emission，烘出无光照污染的 albedo。"""
    for obj in sources:
        for slot in obj.material_slots:
            mat = slot.material
            if mat is None or not mat.use_nodes:
                continue
            nt = mat.node_tree
            principled = next(
                (n for n in nt.nodes if n.type == "BSDF_PRINCIPLED"), None
            )
            out = next((n for n in nt.nodes if n.type == "OUTPUT_MATERIAL"), None)
            if principled is None or out is None:
                continue
            for link in list(out.inputs["Surface"].links):
                nt.links.remove(link)
            emit = nt.nodes.new("ShaderNodeEmission")
            emit.inputs["Strength"].default_value = 1.0
            base = principled.inputs["Base Color"]
            if base.is_linked:
                nt.links.new(base.links[0].from_socket, emit.inputs["Color"])
            else:
                emit.inputs["Color"].default_value = tuple(base.default_value)
            nt.links.new(emit.outputs["Emission"], out.inputs["Surface"])
    _bake(sources, plane, image, "EMIT")


def _bake_metallic_as_emit(
    sources: list[bpy.types.Object],
    plane: bpy.types.Object,
    image: bpy.types.Image,
) -> None:
    """把各源材质 Principled Metallic 接到 Emission，烘 EMIT 得金属度图。"""
    for obj in sources:
        for slot in obj.material_slots:
            mat = slot.material
            if mat is None or not mat.use_nodes:
                continue
            nt = mat.node_tree
            principled = next(
                (n for n in nt.nodes if n.type == "BSDF_PRINCIPLED"), None
            )
            out = next((n for n in nt.nodes if n.type == "OUTPUT_MATERIAL"), None)
            if principled is None or out is None:
                continue
            for link in list(out.inputs["Surface"].links):
                nt.links.remove(link)
            emit = nt.nodes.new("ShaderNodeEmission")
            emit.inputs["Strength"].default_value = 1.0
            comb = nt.nodes.new("ShaderNodeCombineColor")
            metal_in = principled.inputs["Metallic"]
            if metal_in.is_linked:
                src = metal_in.links[0].from_socket
                nt.links.new(src, comb.inputs["Red"])
                nt.links.new(src, comb.inputs["Green"])
                nt.links.new(src, comb.inputs["Blue"])
            else:
                m = float(metal_in.default_value)
                comb.inputs["Red"].default_value = m
                comb.inputs["Green"].default_value = m
                comb.inputs["Blue"].default_value = m
            nt.links.new(comb.outputs["Color"], emit.inputs["Color"])
            nt.links.new(emit.outputs["Emission"], out.inputs["Surface"])

    _bake(sources, plane, image, "EMIT")


def _bake_height_as_emit(
    sources: list[bpy.types.Object],
    plane: bpy.types.Object,
    image: bpy.types.Image,
    *,
    z_min: float,
    z_max: float,
) -> None:
    """用 Geometry Position.Z 归一化烘高度图。"""
    for obj in sources:
        for slot in obj.material_slots:
            mat = slot.material
            if mat is None or not mat.use_nodes:
                continue
            nt = mat.node_tree
            principled = next(
                (n for n in nt.nodes if n.type == "BSDF_PRINCIPLED"), None
            )
            out = next((n for n in nt.nodes if n.type == "OUTPUT_MATERIAL"), None)
            if principled is None or out is None:
                continue
            for link in list(out.inputs["Surface"].links):
                nt.links.remove(link)
            geom = nt.nodes.new("ShaderNodeNewGeometry")
            sep = nt.nodes.new("ShaderNodeSeparateXYZ")
            nt.links.new(geom.outputs["Position"], sep.inputs["Vector"])
            mapr = nt.nodes.new("ShaderNodeMapRange")
            mapr.inputs["From Min"].default_value = z_min
            mapr.inputs["From Max"].default_value = z_max
            mapr.inputs["To Min"].default_value = 0.0
            mapr.inputs["To Max"].default_value = 1.0
            nt.links.new(sep.outputs["Z"], mapr.inputs["Value"])
            emit = nt.nodes.new("ShaderNodeEmission")
            comb = nt.nodes.new("ShaderNodeCombineColor")
            nt.links.new(mapr.outputs["Result"], comb.inputs["Red"])
            nt.links.new(mapr.outputs["Result"], comb.inputs["Green"])
            nt.links.new(mapr.outputs["Result"], comb.inputs["Blue"])
            nt.links.new(comb.outputs["Color"], emit.inputs["Color"])
            nt.links.new(emit.outputs["Emission"], out.inputs["Surface"])

    _bake(sources, plane, image, "EMIT")


def bake_variant(
    *,
    name: str,
    out_dir: Path,
    sources_builder,
    plane_size: float,
    height_z_min: float,
    height_z_max: float,
) -> None:
    """每种贴图独立建场景并立刻落盘，避免 clear 误删 Image。"""
    print(f"=== {name} ===", file=sys.stderr)
    out_dir.mkdir(parents=True, exist_ok=True)
    fill_key = "orange" if name == "wire" else "metal"

    def rebuild() -> tuple[list[bpy.types.Object], bpy.types.Object]:
        clear_scene()
        _ensure_cycles()
        mats = power_port_materials()
        sources = sources_builder(mats)
        plane = _make_bake_plane(plane_size, mats[fill_key])
        return sources, plane

    # Normal / AO / Roughness（需完整 Principled）
    sources, plane = rebuild()
    img = _new_image(f"{name}_normal", alpha=False)
    _bake(sources, plane, img, "NORMAL")
    _save_image(img, out_dir / "normal.png")

    sources, plane = rebuild()
    img = _new_image(f"{name}_ao", alpha=False)
    _bake(sources, plane, img, "AO")
    _save_image(img, out_dir / "ao.png")

    sources, plane = rebuild()
    img = _new_image(f"{name}_roughness", alpha=False)
    _bake(sources, plane, img, "ROUGHNESS")
    _save_image(img, out_dir / "roughness.png")

    sources, plane = rebuild()
    img = _new_image(f"{name}_albedo", alpha=True)
    _bake_basecolor_as_emit(sources, plane, img)
    _save_image(img, out_dir / "albedo.png")

    sources, plane = rebuild()
    img = _new_image(f"{name}_metallic", alpha=False)
    _bake_metallic_as_emit(sources, plane, img)
    _save_image(img, out_dir / "metallic.png")

    sources, plane = rebuild()
    img = _new_image(f"{name}_height", alpha=False)
    _bake_height_as_emit(sources, plane, img, z_min=height_z_min, z_max=height_z_max)
    _save_image(img, out_dir / "height.png")


def main() -> None:
    OUT_ROOT.mkdir(parents=True, exist_ok=True)
    bake_variant(
        name="block",
        out_dir=OUT_ROOT / "block",
        sources_builder=build_block_port_facing_pos_z,
        plane_size=bake_plane_size_for_block(),
        height_z_min=-0.02,
        height_z_max=0.05,
    )
    bake_variant(
        name="wire",
        out_dir=OUT_ROOT / "wire",
        sources_builder=build_wire_port_facing_pos_z,
        plane_size=bake_plane_size_for_wire(),
        height_z_min=-0.02,
        height_z_max=0.04,
    )
    print(f"Done. Preview under {OUT_ROOT}", file=sys.stderr)


if __name__ == "__main__":
    main()
