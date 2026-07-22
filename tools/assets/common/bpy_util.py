# Blender bpy 建模/导出公共工具（供 models/** 脚本 import）
"""清空场景、建 mesh、布尔、材质、导出 GLB 等共用操作。

Blender --python 跑脚本时需把 tools/assets 加入 sys.path，例如：

    from pathlib import Path
    import sys
    _TOOLS = Path(__file__).resolve().parents[2]
    sys.path.insert(0, str(_TOOLS))
    from common.bpy_util import clear_scene, export_glb, ...
"""

from __future__ import annotations

import sys
from pathlib import Path

import bpy
import bmesh
from mathutils import Euler, Vector


def clear_scene() -> None:
    """清空当前场景里的物体与 data-block。"""
    bpy.ops.object.select_all(action="SELECT")
    bpy.ops.object.delete(use_global=False)
    for coll in (
        bpy.data.meshes,
        bpy.data.materials,
        bpy.data.images,
        bpy.data.cameras,
        bpy.data.lights,
    ):
        for block in list(coll):
            coll.remove(block)


def link(obj: bpy.types.Object) -> bpy.types.Object:
    """把物体链到当前 collection。"""
    bpy.context.collection.objects.link(obj)
    return obj


def set_active(obj: bpy.types.Object) -> None:
    """设为唯一选中且 active。"""
    bpy.ops.object.select_all(action="DESELECT")
    obj.select_set(True)
    bpy.context.view_layer.objects.active = obj


def apply_mat(obj: bpy.types.Object, mat: bpy.types.Material) -> None:
    """清空材质槽后挂上单个材质。"""
    obj.data.materials.clear()
    obj.data.materials.append(mat)


def apply_transforms(obj: bpy.types.Object) -> None:
    """把 loc/rot/scale 烘焙进 mesh。"""
    set_active(obj)
    bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)


def finish(obj: bpy.types.Object, mat: bpy.types.Material) -> bpy.types.Object:
    """赋材质并 apply transforms。"""
    apply_mat(obj, mat)
    apply_transforms(obj)
    return obj


def make_mat(
    name: str,
    color: tuple[float, float, float, float],
    *,
    metallic: float = 0.0,
    roughness: float = 0.55,
    texture: bpy.types.Image | None = None,
    alpha: float | None = None,
    emission: tuple[float, float, float] | None = None,
    emission_strength: float = 0.0,
    backface_culling: bool = False,
    blend_method: str = "OPAQUE",
    alpha_cutoff: float = 0.5,
) -> bpy.types.Material:
    """Principled BSDF 材质；可选贴图 / 透明 / 自发光。

    blend_method: OPAQUE / BLEND / CLIP（对应 glTF OPAQUE / BLEND / MASK）
    """
    mat = bpy.data.materials.new(name=name)
    mat.use_nodes = True
    mat.use_backface_culling = backface_culling
    nt = mat.node_tree
    nt.nodes.clear()
    out = nt.nodes.new("ShaderNodeOutputMaterial")
    bsdf = nt.nodes.new("ShaderNodeBsdfPrincipled")
    bsdf.location = (200, 0)
    out.location = (450, 0)
    if texture is not None:
        tex = nt.nodes.new("ShaderNodeTexImage")
        tex.image = texture
        tex.location = (-200, 0)
        tex.interpolation = "Closest"
        nt.links.new(tex.outputs["Color"], bsdf.inputs["Base Color"])
        if "Alpha" in tex.outputs and "Alpha" in bsdf.inputs:
            nt.links.new(tex.outputs["Alpha"], bsdf.inputs["Alpha"])
    else:
        col = color if alpha is None else (color[0], color[1], color[2], alpha)
        bsdf.inputs["Base Color"].default_value = col
    bsdf.inputs["Metallic"].default_value = metallic
    bsdf.inputs["Roughness"].default_value = roughness
    if alpha is not None and alpha < 1.0 and "Alpha" in bsdf.inputs:
        bsdf.inputs["Alpha"].default_value = alpha
        if blend_method == "OPAQUE":
            blend_method = "BLEND"
    if blend_method != "OPAQUE":
        mat.blend_method = blend_method
        if blend_method == "CLIP" and hasattr(mat, "alpha_threshold"):
            mat.alpha_threshold = alpha_cutoff
    if emission is not None:
        if "Emission Color" in bsdf.inputs:
            bsdf.inputs["Emission Color"].default_value = (*emission, 1.0)
            bsdf.inputs["Emission Strength"].default_value = emission_strength
        elif "Emission" in bsdf.inputs:
            bsdf.inputs["Emission"].default_value = (*emission, 1.0)
    nt.links.new(bsdf.outputs["BSDF"], out.inputs["Surface"])
    return mat


def mesh_cube(name: str, size: Vector, loc: Vector) -> bpy.types.Object:
    """单位立方体缩放后放到 loc（未 apply）。"""
    mesh = bpy.data.meshes.new(name)
    bm = bmesh.new()
    bmesh.ops.create_cube(bm, size=1.0)
    bmesh.ops.scale(bm, vec=size, verts=bm.verts)
    bm.to_mesh(mesh)
    bm.free()
    obj = bpy.data.objects.new(name, mesh)
    link(obj)
    obj.location = loc
    return obj


def mesh_cylinder(
    name: str,
    radius: float,
    depth: float,
    loc: Vector,
    *,
    rot: Euler | None = None,
    verts: int = 24,
    radius2: float | None = None,
) -> bpy.types.Object:
    """沿局部 Z 的圆柱/圆台（可用 rot 改朝向）。"""
    mesh = bpy.data.meshes.new(name)
    bm = bmesh.new()
    bmesh.ops.create_cone(
        bm,
        cap_ends=True,
        cap_tris=False,
        segments=verts,
        radius1=radius,
        radius2=radius if radius2 is None else radius2,
        depth=depth,
    )
    bm.to_mesh(mesh)
    bm.free()
    obj = bpy.data.objects.new(name, mesh)
    link(obj)
    obj.location = loc
    if rot is not None:
        obj.rotation_euler = rot
    return obj


def mesh_torus(
    name: str,
    major: float,
    minor: float,
    loc: Vector,
    *,
    rot: Euler | None = None,
) -> bpy.types.Object:
    """圆环原语。"""
    bpy.ops.mesh.primitive_torus_add(
        major_radius=major,
        minor_radius=minor,
        major_segments=36,
        minor_segments=12,
        location=loc,
        rotation=rot or Euler((0, 0, 0)),
    )
    obj = bpy.context.active_object
    obj.name = name
    return obj


def boolean_diff(target: bpy.types.Object, cutter: bpy.types.Object) -> None:
    """target 减去 cutter，并删除 cutter。"""
    apply_transforms(cutter)
    set_active(target)
    mod = target.modifiers.new("Bool", "BOOLEAN")
    mod.operation = "DIFFERENCE"
    mod.solver = "EXACT"
    mod.object = cutter
    bpy.ops.object.modifier_apply(modifier=mod.name)
    bpy.data.objects.remove(cutter, do_unlink=True)


def boolean_union(target: bpy.types.Object, other: bpy.types.Object) -> None:
    """target 并上 other，并删除 other。"""
    apply_transforms(other)
    set_active(target)
    mod = target.modifiers.new("BoolUnion", "BOOLEAN")
    mod.operation = "UNION"
    mod.solver = "EXACT"
    mod.object = other
    bpy.ops.object.modifier_apply(modifier=mod.name)
    bpy.data.objects.remove(other, do_unlink=True)


def join_objects(name: str, objs: list[bpy.types.Object]) -> bpy.types.Object:
    """把多个物体 join 成一个，命名为 name。"""
    assert objs
    for obj in objs:
        apply_transforms(obj)
    bpy.ops.object.select_all(action="DESELECT")
    for obj in objs:
        obj.select_set(True)
    bpy.context.view_layer.objects.active = objs[0]
    if len(objs) > 1:
        bpy.ops.object.join()
    joined = bpy.context.active_object
    joined.name = name
    if joined.data:
        joined.data.name = f"Mesh_{name}"
    print(f"  {name}: {len(objs)} parts", file=sys.stderr)
    return joined


def join_by_material() -> None:
    """按材质槽把场景中 mesh 合并（同材质一份，命名 Part_<材质名>）。"""
    by_mat: dict[str, list[bpy.types.Object]] = {}
    for obj in list(bpy.context.scene.objects):
        if obj.type != "MESH" or not obj.data.materials:
            continue
        apply_transforms(obj)
        by_mat.setdefault(obj.data.materials[0].name, []).append(obj)
    for mat_name, group in by_mat.items():
        bpy.ops.object.select_all(action="DESELECT")
        for obj in group:
            obj.select_set(True)
        bpy.context.view_layer.objects.active = group[0]
        if len(group) > 1:
            bpy.ops.object.join()
        joined = bpy.context.active_object
        joined.name = f"Part_{mat_name}"
        if joined.data:
            joined.data.name = f"Mesh_{mat_name}"
        print(f"  {mat_name}: {len(group)} parts", file=sys.stderr)


def export_glb(
    path: Path,
    *,
    export_tangents: bool = False,
    export_image_format: str = "AUTO",
) -> None:
    """导出选中 mesh 为 GLB（Y-up，与游戏一致）。"""
    path.parent.mkdir(parents=True, exist_ok=True)
    bpy.ops.object.select_all(action="DESELECT")
    for obj in bpy.context.scene.objects:
        if obj.type == "MESH":
            obj.select_set(True)
    kwargs = dict(
        filepath=str(path),
        export_format="GLB",
        use_selection=True,
        export_apply=True,
        export_texcoords=True,
        export_normals=True,
        export_materials="EXPORT",
        export_yup=True,
        export_image_format=export_image_format,
    )
    if export_tangents:
        kwargs["export_tangents"] = True
    bpy.ops.export_scene.gltf(**kwargs)
    print(f"Wrote {path}", file=sys.stderr)
