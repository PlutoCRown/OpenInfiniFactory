# 工厂供电口几何常量与 bpy 建模（贴图烘焙 / 减面模型共用）
"""供电口：方块侧金属方板贴图（含双环+四螺丝）；电线端面圆环贴图（无螺丝）。

尺寸与活塞 / 传感器 / 电线对齐。改口时：先改本文件 → 重烘贴图 → 重导模型。
"""

from __future__ import annotations

import math
from dataclasses import dataclass
from pathlib import Path

import bpy
import bmesh
from mathutils import Euler, Vector

from common.bpy_util import (
    finish,
    load_image,
    make_mat,
    mesh_cube,
    mesh_cylinder,
    mesh_torus,
)
from common.paths import FACTORY_BLOCKS

# 烘焙源细分：圆环要够密，贴图上看不出多边形
BAKE_TORUS_MAJOR = 128
BAKE_TORUS_MINOR = 48
BAKE_CYL_VERTS = 64
BAKE_SCREW_VERTS = 24

POWER_PORT_TEX_ROOT = FACTORY_BLOCKS / "_shared" / "power_port"


@dataclass(frozen=True)
class BlockPortDims:
    """贴在方块侧面的供电口（金属方板 + 螺丝）。"""

    plate: float = 0.40
    plate_t: float = 0.03
    hole_r: float = 0.095
    hole_d: float = 0.035
    gold_major: float = 0.118
    gold_minor: float = 0.009
    outer_major: float = 0.145
    outer_minor: float = 0.007
    screw_r: float = 0.013
    screw_d: float = 0.013
    screw_off: float = 0.145


@dataclass(frozen=True)
class WirePortDims:
    """贴在电线臂横截面的圆环接口（无方板、无螺丝）。"""

    face_size: float = 0.30
    hole_r: float = 0.085
    hole_d: float = 0.045
    gold_major: float = 0.105
    gold_minor: float = 0.018
    outer_major: float = 0.122
    outer_minor: float = 0.014
    pad_r: float = 0.135
    pad_t: float = 0.008


BLOCK_PORT = BlockPortDims()
WIRE_PORT = WirePortDims()


def power_port_materials() -> dict[str, bpy.types.Material]:
    """与活塞 / 传感器供电口同一套 PBR 材质（烘焙源用）。"""
    return {
        "metal": make_mat(
            "PortMetal", (0.58, 0.60, 0.62, 1.0), metallic=0.90, roughness=0.24
        ),
        "gold": make_mat(
            "PortGold", (0.88, 0.68, 0.16, 1.0), metallic=0.95, roughness=0.20
        ),
        "dark": make_mat(
            "PortDark", (0.04, 0.04, 0.05, 1.0), metallic=0.40, roughness=0.42
        ),
        "orange": make_mat(
            "PortOrange", (0.92, 0.40, 0.06, 1.0), metallic=0.08, roughness=0.40
        ),
    }


def _add_screw_with_slot(
    prefix: str,
    center: Vector,
    dims: BlockPortDims,
    mat_dark: bpy.types.Material,
    mat_metal: bpy.types.Material,
    parts: list[bpy.types.Object],
) -> None:
    """暗色螺丝柱 + 浅十字槽（烘焙时更易看出螺丝）。"""
    body = mesh_cylinder(
        f"{prefix}_Body",
        dims.screw_r,
        dims.screw_d,
        center,
        verts=BAKE_SCREW_VERTS,
    )
    parts.append(finish(body, mat_dark))
    slot_z = center.z + dims.screw_d * 0.42
    slot_len = dims.screw_r * 1.55
    slot_w = dims.screw_r * 0.28
    slot_h = dims.screw_d * 0.35
    parts.append(
        finish(
            mesh_cube(
                f"{prefix}_SlotH",
                Vector((slot_len, slot_w, slot_h)),
                Vector((center.x, center.y, slot_z)),
            ),
            mat_metal,
        )
    )
    parts.append(
        finish(
            mesh_cube(
                f"{prefix}_SlotV",
                Vector((slot_w, slot_len, slot_h)),
                Vector((center.x, center.y, slot_z)),
            ),
            mat_metal,
        )
    )


def build_block_port_facing_pos_z(
    mats: dict[str, bpy.types.Material],
    *,
    dims: BlockPortDims = BLOCK_PORT,
    prefix: str = "BlockPort",
) -> list[bpy.types.Object]:
    """沿 +Z 朝向、中心在原点：高细分金属方板 + 环/螺丝（仅烘焙源）。"""
    parts: list[bpy.types.Object] = []
    metal, gold, dark = mats["metal"], mats["gold"], mats["dark"]
    plate_z = 0.0
    top = dims.plate_t * 0.5
    parts.append(
        finish(
            mesh_cube(
                f"{prefix}_Plate",
                Vector((dims.plate, dims.plate, dims.plate_t)),
                Vector((0, 0, plate_z)),
            ),
            metal,
        )
    )
    parts.append(
        finish(
            mesh_cylinder(
                f"{prefix}_Hole",
                dims.hole_r,
                dims.hole_d,
                Vector((0, 0, top + 0.002)),
                verts=BAKE_CYL_VERTS,
            ),
            dark,
        )
    )
    parts.append(
        finish(
            mesh_torus(
                f"{prefix}_Gold",
                dims.gold_major,
                dims.gold_minor,
                Vector((0, 0, top + 0.012)),
                major_segments=BAKE_TORUS_MAJOR,
                minor_segments=BAKE_TORUS_MINOR,
            ),
            gold,
        )
    )
    parts.append(
        finish(
            mesh_torus(
                f"{prefix}_Outer",
                dims.outer_major,
                dims.outer_minor,
                Vector((0, 0, top + 0.010)),
                major_segments=BAKE_TORUS_MAJOR,
                minor_segments=BAKE_TORUS_MINOR,
            ),
            metal,
        )
    )
    for i, (sx, sy) in enumerate(
        (
            (-dims.screw_off, -dims.screw_off),
            (dims.screw_off, -dims.screw_off),
            (-dims.screw_off, dims.screw_off),
            (dims.screw_off, dims.screw_off),
        )
    ):
        _add_screw_with_slot(
            f"{prefix}_S{i}",
            Vector((sx, sy, top + 0.006)),
            dims,
            dark,
            metal,
            parts,
        )
    return parts


def build_wire_port_facing_pos_z(
    mats: dict[str, bpy.types.Material],
    *,
    dims: WirePortDims = WIRE_PORT,
    prefix: str = "WirePort",
) -> list[bpy.types.Object]:
    """沿 +Z：橙色臂截面底 + 暗孔 + 内外金属环；无方板、无螺丝（仅烘焙源）。"""
    parts: list[bpy.types.Object] = []
    metal, gold, dark, orange = (
        mats["metal"],
        mats["gold"],
        mats["dark"],
        mats["orange"],
    )
    z0 = 0.0
    parts.append(
        finish(
            mesh_cube(
                f"{prefix}_Face",
                Vector((dims.face_size, dims.face_size, 0.006)),
                Vector((0, 0, z0 - 0.004)),
            ),
            orange,
        )
    )
    parts.append(
        finish(
            mesh_cylinder(
                f"{prefix}_Pad",
                dims.pad_r,
                dims.pad_t,
                Vector((0, 0, z0)),
                verts=BAKE_CYL_VERTS,
            ),
            metal,
        )
    )
    parts.append(
        finish(
            mesh_cylinder(
                f"{prefix}_Hole",
                dims.hole_r,
                dims.hole_d,
                Vector((0, 0, z0 + 0.002)),
                verts=BAKE_CYL_VERTS,
            ),
            dark,
        )
    )
    parts.append(
        finish(
            mesh_torus(
                f"{prefix}_Gold",
                dims.gold_major,
                dims.gold_minor,
                Vector((0, 0, z0 + 0.012)),
                major_segments=BAKE_TORUS_MAJOR,
                minor_segments=BAKE_TORUS_MINOR,
            ),
            gold,
        )
    )
    parts.append(
        finish(
            mesh_torus(
                f"{prefix}_Outer",
                dims.outer_major,
                dims.outer_minor,
                Vector((0, 0, z0 + 0.010)),
                major_segments=BAKE_TORUS_MAJOR,
                minor_segments=BAKE_TORUS_MINOR,
            ),
            metal,
        )
    )
    return parts


def bake_plane_size_for_block(dims: BlockPortDims = BLOCK_PORT) -> float:
    """方块接口烘焙平面边长（略大于金属板）。"""
    return dims.plate * 1.02


def bake_plane_size_for_wire(dims: WirePortDims = WIRE_PORT) -> float:
    """电线接口烘焙平面边长（臂横截面）。"""
    return dims.face_size


def power_port_tex_dir(kind: str) -> Path:
    """kind: 'block' | 'wire'。"""
    return POWER_PORT_TEX_ROOT / kind


def make_power_port_material(
    kind: str, *, name: str | None = None
) -> bpy.types.Material:
    """从已烘焙 PNG 建 Principled；albedo/normal/metallic/roughness 直连（便于 glTF 导出）。"""
    tex_dir = power_port_tex_dir(kind)
    albedo = load_image(tex_dir / "albedo.png")
    albedo.pack()
    normal = load_image(tex_dir / "normal.png", is_data=True)
    normal.pack()
    metallic = load_image(tex_dir / "metallic.png", is_data=True)
    metallic.pack()
    roughness = load_image(tex_dir / "roughness.png", is_data=True)
    roughness.pack()

    mat_name = name or (f"PowerPort_{kind}")
    mat = bpy.data.materials.new(name=mat_name)
    mat.use_nodes = True
    mat.blend_method = "OPAQUE"
    mat.use_backface_culling = True
    nt = mat.node_tree
    nt.nodes.clear()
    out = nt.nodes.new("ShaderNodeOutputMaterial")
    bsdf = nt.nodes.new("ShaderNodeBsdfPrincipled")
    bsdf.location = (360, 0)
    out.location = (620, 0)

    def tex_node(img: bpy.types.Image, y: float) -> bpy.types.Node:
        n = nt.nodes.new("ShaderNodeTexImage")
        n.image = img
        n.interpolation = "Smart"
        n.extension = "CLIP"
        n.location = (-420, y)
        return n

    n_alb = tex_node(albedo, 200)
    n_met = tex_node(metallic, -40)
    n_rgh = tex_node(roughness, -240)
    n_nrm = tex_node(normal, -440)

    nt.links.new(n_alb.outputs["Color"], bsdf.inputs["Base Color"])
    nt.links.new(n_met.outputs["Color"], bsdf.inputs["Metallic"])
    nt.links.new(n_rgh.outputs["Color"], bsdf.inputs["Roughness"])

    nmap = nt.nodes.new("ShaderNodeNormalMap")
    nmap.location = (40, -440)
    nmap.inputs["Strength"].default_value = 1.15
    nt.links.new(n_nrm.outputs["Color"], nmap.inputs["Color"])
    nt.links.new(nmap.outputs["Normal"], bsdf.inputs["Normal"])

    nt.links.new(bsdf.outputs["BSDF"], out.inputs["Surface"])
    return mat


def _axis_rotation(axis: str) -> Euler:
    """把局部 +Z 朝向转到目标轴（与旧 add_port 一致）。"""
    if axis == "+Z":
        return Euler((0, 0, 0))
    if axis == "-Z":
        return Euler((math.radians(180), 0, 0))
    if axis == "+X":
        return Euler((0, math.radians(90), 0))
    if axis == "-X":
        return Euler((0, math.radians(-90), 0))
    if axis == "-Y":
        return Euler((math.radians(90), 0, 0))
    if axis == "+Y":
        return Euler((math.radians(-90), 0, 0))
    raise ValueError(f"unknown axis: {axis}")


def mesh_uv_plate(
    name: str,
    size: float,
    thickness: float,
    loc: Vector,
    *,
    rot: Euler | None = None,
) -> bpy.types.Object:
    """薄板：有厚度；外表面（本地 +Z）UV 铺满贴图，侧面采金属板色。"""
    # 用扁立方体，避免 extrude 后再平移把厚度挤没
    mesh = bpy.data.meshes.new(name)
    bm = bmesh.new()
    bmesh.ops.create_cube(bm, size=1.0)
    bmesh.ops.scale(bm, vec=Vector((size, size, thickness)), verts=bm.verts)
    # 立方体默认中心在原点；外移使 +Z 外表面在 z=0，厚度朝 -Z（贴进机体）
    bmesh.ops.translate(bm, vec=Vector((0.0, 0.0, -thickness * 0.5)), verts=bm.verts)
    bm.to_mesh(mesh)
    bm.free()
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)

    uv = mesh.uv_layers.new(name="UVMap")
    # 侧面采贴图最外缘（烘焙平面略大于金属板，边缘近黑），恢复以前棱边偏暗的观感
    side_u, side_v = 0.0, 0.5
    for poly in mesh.polygons:
        n = poly.normal
        if n.z > 0.85:
            for li in poly.loop_indices:
                co = mesh.vertices[mesh.loops[li].vertex_index].co
                uv.data[li].uv = (co.x / size + 0.5, co.y / size + 0.5)
        elif n.z < -0.85:
            for li in poly.loop_indices:
                co = mesh.vertices[mesh.loops[li].vertex_index].co
                uv.data[li].uv = (co.x / size + 0.5, co.y / size + 0.5)
        else:
            for li in poly.loop_indices:
                uv.data[li].uv = (side_u, side_v)

    obj.location = loc
    if rot is not None:
        obj.rotation_euler = rot
    return obj


def add_textured_block_port(
    prefix: str,
    center: Vector,
    axis: str,
    mat: bpy.types.Material,
    parts: list[bpy.types.Object] | None = None,
    *,
    dims: BlockPortDims = BLOCK_PORT,
) -> bpy.types.Object:
    """低面供电口：金属方片 + 整图贴图。parts 非空则 append。"""
    obj = mesh_uv_plate(
        prefix,
        dims.plate,
        dims.plate_t,
        center,
        rot=_axis_rotation(axis),
    )
    finished = finish(obj, mat)
    if parts is not None:
        parts.append(finished)
    return finished
