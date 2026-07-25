# 工厂 GLB：顶点色 AO 烘焙（无 UV，避免贴图串面）
"""把 Cycles Ambient Occlusion 烘到 Color Attribute（导出为 glTF COLOR_0）。

Bevy StandardMaterial 会把顶点色乘进 base_color，凹槽自遮挡变深，且不依赖 UV 布局。
短射线 + strength 软混，避免整块发黑。

Blender 5 的 glTF 导出：顶点色必须接到材质节点树里才会写出 COLOR_0。
"""

from __future__ import annotations

import sys

import bpy


def bake_vertex_ao(
    *,
    samples: int = 64,
    max_ray_distance: float = 0.22,
    strength: float = 0.42,
    attr_name: str = "Col",
) -> None:
    """对场景内全部 Mesh 烘 AO → 顶点色，并按 strength 软混向白色。"""
    meshes = [obj for obj in bpy.context.scene.objects if obj.type == "MESH"]
    if not meshes:
        print("ao_bake: no mesh objects", file=sys.stderr)
        return

    if bpy.context.scene.world is None:
        bpy.context.scene.world = bpy.data.worlds.new("World")

    for obj in meshes:
        mesh = obj.data
        existing = mesh.color_attributes.get(attr_name)
        if existing is not None:
            mesh.color_attributes.remove(existing)
        attr = mesh.color_attributes.new(
            name=attr_name, type="BYTE_COLOR", domain="CORNER"
        )
        mesh.color_attributes.active_color = attr

    scene = bpy.context.scene
    prev_engine = scene.render.engine
    prev_device = scene.cycles.device
    prev_samples = scene.cycles.samples

    scene.render.engine = "CYCLES"
    scene.cycles.device = "CPU"
    scene.cycles.samples = samples
    scene.cycles.bake_type = "AO"
    bake = scene.render.bake
    bake.target = "VERTEX_COLORS"
    bake.max_ray_distance = max_ray_distance
    bake.use_clear = True

    bpy.ops.object.select_all(action="DESELECT")
    for obj in meshes:
        obj.select_set(True)
    bpy.context.view_layer.objects.active = meshes[0]

    print(
        f"ao_bake: vertex AO samples={samples} ray={max_ray_distance} "
        f"strength={strength} meshes={len(meshes)}",
        file=sys.stderr,
    )
    bpy.ops.object.bake(type="AO")

    # 软混：v = 1 - (1 - ao) * strength，保留凹槽对比但不压死亮面
    for obj in meshes:
        attr = obj.data.color_attributes.get(attr_name)
        if attr is None:
            continue
        for loop in attr.data:
            ao = float(loop.color[0])
            v = 1.0 - (1.0 - ao) * strength
            loop.color = (v, v, v, 1.0)

    _wire_vertex_colors_for_gltf_export(attr_name)

    # 只保留 AO 这一层，避免导出多余 COLOR_n（全白占位）
    for obj in meshes:
        mesh = obj.data
        for name in [a.name for a in mesh.color_attributes if a.name != attr_name]:
            mesh.color_attributes.remove(mesh.color_attributes[name])
        if mesh.color_attributes.get(attr_name) is not None:
            mesh.color_attributes.active_color = mesh.color_attributes[attr_name]

    scene.render.engine = prev_engine
    scene.cycles.device = prev_device
    scene.cycles.samples = prev_samples
    print("ao_bake: vertex AO done", file=sys.stderr)


def _wire_vertex_colors_for_gltf_export(attr_name: str) -> None:
    """把 Color Attribute 乘进 Base Color，否则 Blender 5 不导出 COLOR_0。"""
    wired = 0
    for mat in bpy.data.materials:
        if not mat.use_nodes or mat.node_tree is None:
            continue
        nt = mat.node_tree
        if any(
            n.type == "ATTRIBUTE" and getattr(n, "attribute_name", "") == attr_name
            for n in nt.nodes
        ):
            wired += 1
            continue
        bsdf = next((n for n in nt.nodes if n.type == "BSDF_PRINCIPLED"), None)
        if bsdf is None:
            continue
        base_in = bsdf.inputs.get("Base Color")
        if base_in is None:
            continue

        attrn = nt.nodes.new("ShaderNodeAttribute")
        attrn.attribute_name = attr_name
        attrn.location = (-450.0, 180.0)

        mul = nt.nodes.new("ShaderNodeMix")
        mul.data_type = "RGBA"
        mul.blend_type = "MULTIPLY"
        mul.inputs["Factor"].default_value = 1.0
        mul.location = (-220.0, 80.0)

        if base_in.is_linked:
            from_sock = base_in.links[0].from_socket
            nt.links.remove(base_in.links[0])
            nt.links.new(from_sock, mul.inputs["A"])
        else:
            rgb = nt.nodes.new("ShaderNodeRGB")
            rgb.outputs[0].default_value = tuple(base_in.default_value)
            rgb.location = (-450.0, 0.0)
            base_in.default_value = (1.0, 1.0, 1.0, 1.0)
            nt.links.new(rgb.outputs[0], mul.inputs["A"])

        nt.links.new(attrn.outputs["Color"], mul.inputs["B"])
        nt.links.new(mul.outputs["Result"], base_in)
        wired += 1

    print(f"ao_bake: wired vertex color into {wired} materials", file=sys.stderr)
