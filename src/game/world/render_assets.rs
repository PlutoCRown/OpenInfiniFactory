use std::collections::{HashMap, HashSet};
use std::path::Path;

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

use crate::game::blocks::BlockPresent;
use crate::game::blocks::pusher::texture;
use crate::game::blocks::{
    BLOCK_SIZE, BlockKind, BlockShape, ModelMaterial, ModelMesh, PaintMaterialId, all_blocks,
    paint_catalog, stamp_catalog, stamp_def,
};
use crate::game::scene_blocks::{FactoryGltfPart, load_factory_glb};

/// 工厂 GLB 单个可渲染零件（含编辑预览材质）
#[derive(Clone)]
pub struct FactoryPartHandles {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
    pub preview_material: Handle<StandardMaterial>,
}

/// 工厂块从 assets/factory_blocks 加载的外观
#[derive(Clone)]
pub enum FactoryVisual {
    /// 静态整块零件
    Static {
        parts: Vec<FactoryPartHandles>,
        /// 相对方块朝向的额外旋转（DownWelder 俯仰、反向传送带翻底等）
        local_rotation: Quat,
    },
    /// 钻头：Body 静、Head 模拟时绕前进轴转
    Drill {
        body: Vec<FactoryPartHandles>,
        head: Vec<FactoryPartHandles>,
    },
    /// 活塞 / 拦截器：Body 静、Stage 半行程、Head 全行程
    Pusher {
        body: Vec<FactoryPartHandles>,
        stage: Vec<FactoryPartHandles>,
        head: Vec<FactoryPartHandles>,
    },
    /// 电线六向臂：索引对齐 signal_neighbor_offsets；power 为通电凹槽灯
    Wire {
        faces: [Vec<FactoryPartHandles>; 6],
        power: [Vec<FactoryPartHandles>; 6],
    },
}

#[derive(Resource, Clone)]
pub struct WorldRenderAssets {
    pub(crate) block: Handle<Mesh>,
    node: Handle<Mesh>,
    wire_node: Handle<Mesh>,
    pub(crate) face_mark: Handle<Mesh>,
    /// 生成器/验收预览用：厚 0.1、心在原点的印花板（贴面后整板外凸）
    stamp_embed_plate: Handle<Mesh>,
    pub(crate) weld_spark: Handle<Mesh>,
    /// 焊接扩散粒子薄方片（局部 XY，法线 +Z）
    pub(crate) weld_burst_quad: Handle<Mesh>,
    connector_x: Handle<Mesh>,
    connector_y: Handle<Mesh>,
    connector_z: Handle<Mesh>,
    wire_connector_x: Handle<Mesh>,
    wire_connector_y: Handle<Mesh>,
    wire_connector_z: Handle<Mesh>,
    part_conveyor_base: Handle<Mesh>,
    part_conveyor_belt: Handle<Mesh>,
    part_drill_body: Handle<Mesh>,
    part_drill_tip: Handle<Mesh>,
    part_large: Handle<Mesh>,
    part_medium: Handle<Mesh>,
    part_small: Handle<Mesh>,
    part_plate: Handle<Mesh>,
    part_sign_board: Handle<Mesh>,
    part_rotator_base: Handle<Mesh>,
    part_rotator_disk: Handle<Mesh>,
    part_rotator_ring: Handle<Mesh>,
    part_rod_x: Handle<Mesh>,
    part_rod_y: Handle<Mesh>,
    part_rod_z: Handle<Mesh>,
    part_mirror_face: Handle<Mesh>,
    part_vertical_mirror_face: Handle<Mesh>,
    part_splitter_face: Handle<Mesh>,
    part_suction_cup: Handle<Mesh>,
    part_pusher_body: Handle<Mesh>,
    part_pusher_head: Handle<Mesh>,
    block_materials: HashMap<BlockKind, Handle<StandardMaterial>>,
    /// 破坏碎片公告板材质（双面 unlit，采样方块贴图）
    break_debris_materials: HashMap<BlockKind, Handle<StandardMaterial>>,
    preview_materials: HashMap<BlockKind, Handle<StandardMaterial>>,
    /// 场景块材质（从 model.glb 加载）
    scene_materials: HashMap<BlockKind, Handle<StandardMaterial>>,
    /// 场景块网格（从 model.glb 加载，图标/非立方体用）
    scene_meshes: HashMap<BlockKind, Handle<Mesh>>,
    /// 立方体 24 顶点 UV 模板（世界 AO 网格复用 glb UV）
    scene_face_uvs: HashMap<BlockKind, [[f32; 2]; 24]>,
    /// 合并 mesh 时可作为实心遮挡邻面的场景种类（不透明立方体）
    scene_face_occluders: HashSet<BlockKind>,
    /// 工厂块 GLB 外观（无则回退程序化零件）
    factory_models: HashMap<BlockKind, FactoryVisual>,
    /// 当前是否用游玩态验收器外观（由 BuilderMode 驱动）
    pub(crate) goal_play_visual: bool,
    /// 是否已完成首次与 BuilderMode 对齐（避免 OnEnter 前误重建）
    pub(crate) goal_play_visual_initialized: bool,
    /// 游玩态验收器幽灵材质（按目标材料种类）
    goal_ghost_materials:
        HashMap<BlockKind, Handle<crate::game::world::rendering::GoalGhostMaterial>>,
    face_mark_materials: HashMap<PaintMaterialId, Handle<StandardMaterial>>,
    /// 灯面板未通电材质
    pub(crate) light_panel_material: Handle<StandardMaterial>,
    /// 灯面板通电发光材质
    pub(crate) light_panel_lit_material: Handle<StandardMaterial>,
    model_materials: HashMap<ModelMaterial, Handle<StandardMaterial>>,
    model_preview_materials: HashMap<ModelMaterial, Handle<StandardMaterial>>,
    pub(crate) wire_connector_material: Handle<StandardMaterial>,
    pub(crate) active_wire_material: Handle<StandardMaterial>,
    pub(crate) weld_connector_material: Handle<StandardMaterial>,
    /// 焊接扩散粒子：白底 + 黄自发光（与焊点同风格，须 lit）
    pub(crate) weld_burst_material: Handle<StandardMaterial>,
    pub(crate) laser_beam_material: Handle<StandardMaterial>,
    delete_preview_material: Handle<StandardMaterial>,
    /// 选区包围盒半透明填充
    selection_fill_material: Handle<StandardMaterial>,
    /// 选区包围盒边线 / 角块
    selection_edge_material: Handle<StandardMaterial>,
    /// 选区不可放置时的填充
    selection_invalid_fill_material: Handle<StandardMaterial>,
    /// 选区不可放置时的边线 / 角块
    selection_invalid_edge_material: Handle<StandardMaterial>,
    active_factory_debug_material: Handle<StandardMaterial>,
    inactive_factory_debug_material: Handle<StandardMaterial>,
}

pub enum EditPreviewKind {
    Delete,
    Selection,
}

impl WorldRenderAssets {
    pub(crate) fn new(
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
        images: &mut Assets<Image>,
        scene_registry: &crate::game::scene_blocks::SceneBlockRegistry,
        material_registry: &crate::game::material_blocks::MaterialBlockRegistry,
        stamp_registry: &crate::game::material_blocks::StampMaterialRegistry,
        paint_registry: &crate::game::material_blocks::PaintMaterialRegistry,
    ) -> Self {
        let block_textures: HashMap<_, _> = all_blocks()
            .into_iter()
            .filter_map(|kind| kind.block_texture().map(|image| (kind, images.add(image))))
            .collect();
        let platform_texture = block_textures
            .get(&BlockKind::Platform)
            .expect("platform defines a texture")
            .clone();
        let stone_texture = images.add(crate::game::world::procedural_textures::from_fn(|x, y| {
            crate::game::world::procedural_textures::material_pixel(x, y, [124, 128, 132], 89)
        }));
        let wood_texture = images.add(crate::game::world::procedural_textures::from_fn(
            crate::game::world::procedural_textures::wood_pixel,
        ));
        let bordered_wood_texture = images.add(texture::bordered_wood());
        let block_materials: HashMap<BlockKind, Handle<StandardMaterial>> = all_blocks()
            .into_iter()
            .map(|kind| {
                let texture = block_textures.get(&kind).cloned();
                let material = texture
                    .map(|texture| textured_block_material(kind, texture))
                    .unwrap_or_else(|| block_material(kind));
                (kind, materials.add(material))
            })
            .collect();
        let preview_materials: HashMap<BlockKind, Handle<StandardMaterial>> = all_blocks()
            .into_iter()
            .map(|kind| {
                let texture = block_textures.get(&kind).cloned();
                (
                    kind,
                    materials.add(preview_block_material(kind, texture, None)),
                )
            })
            .collect();
        // 场景 / 材料 / 印花：model.glb 或 texture.png → scene_materials 等 HashMap
        let mut scene_meshes = HashMap::new();
        let mut scene_face_uvs = HashMap::new();
        let mut scene_block_materials = HashMap::new();
        let mut block_materials = block_materials;
        let mut preview_materials = preview_materials;
        let stamp_plate = meshes.add(stamp_plate_mesh());
        for kind in all_blocks().into_iter().filter(|kind| kind.is_scene()) {
            let Some(presentation) = scene_registry.get_kind(kind) else {
                continue;
            };
            insert_configured_pack(
                kind,
                presentation.model_path.as_deref(),
                presentation.texture_path.as_deref(),
                None,
                kind.material(),
                None,
                meshes,
                materials,
                images,
                &mut scene_meshes,
                &mut scene_face_uvs,
                &mut scene_block_materials,
                &mut block_materials,
                &mut preview_materials,
            );
        }
        for presentation in material_registry.ordered() {
            let kind = BlockKind::Material(presentation.id);
            insert_configured_pack(
                kind,
                presentation.model_path.as_deref(),
                presentation.texture_path.as_deref(),
                presentation.normal_path.as_deref(),
                kind.material(),
                None,
                meshes,
                materials,
                images,
                &mut scene_meshes,
                &mut scene_face_uvs,
                &mut scene_block_materials,
                &mut block_materials,
                &mut preview_materials,
            );
        }
        for presentation in stamp_registry.ordered() {
            let kind = BlockKind::Stamp(presentation.id);
            insert_configured_pack(
                kind,
                presentation.model_path.as_deref(),
                presentation.texture_path.as_deref(),
                None,
                kind.material(),
                Some(&stamp_plate),
                meshes,
                materials,
                images,
                &mut scene_meshes,
                &mut scene_face_uvs,
                &mut scene_block_materials,
                &mut block_materials,
                &mut preview_materials,
            );
        }
        // 兜底场景/材料：MC 式洋红黑棋盘（不进 all_blocks，需显式注册）
        {
            use crate::game::blocks::{fallback_material_id, fallback_scene_id};
            let texture =
                images.add(crate::game::world::procedural_textures::missing_texture_image());
            let material = materials.add(StandardMaterial {
                base_color: Color::WHITE,
                base_color_texture: Some(texture.clone()),
                perceptual_roughness: 0.94,
                reflectance: 0.10,
                ..default()
            });
            let preview = materials.add(StandardMaterial {
                base_color: Color::WHITE.with_alpha(0.46),
                base_color_texture: Some(texture),
                alpha_mode: AlphaMode::Blend,
                perceptual_roughness: 0.94,
                reflectance: 0.08,
                ..default()
            });
            for kind in [
                BlockKind::Material(fallback_material_id()),
                BlockKind::Scene(fallback_scene_id()),
            ] {
                scene_block_materials.insert(kind, material.clone());
                block_materials.insert(kind, material.clone());
                preview_materials.insert(kind, preview.clone());
            }
        }
        // 材料/印花破坏碎片：从方块材质抽出贴图做双面 unlit
        let mut break_debris_materials = HashMap::new();
        for (kind, handle) in &block_materials {
            if !(kind.is_material() || matches!(kind, BlockKind::Stamp(_))) {
                continue;
            }
            let Some(source) = materials.get(handle) else {
                continue;
            };
            break_debris_materials.insert(
                *kind,
                materials.add(StandardMaterial {
                    base_color: if source.base_color_texture.is_some() {
                        Color::WHITE
                    } else {
                        source.base_color
                    },
                    base_color_texture: source.base_color_texture.clone(),
                    unlit: true,
                    cull_mode: None,
                    alpha_mode: AlphaMode::Mask(0.2),
                    ..default()
                }),
            );
        }
        // 不透明立方体才遮挡邻面；玻璃等 Blend、异形 GLB 不进此集合
        let mut scene_face_occluders = HashSet::new();
        for (kind, handle) in &scene_block_materials {
            if !kind.is_scene() || !kind.has_collision() {
                continue;
            }
            if !scene_face_uvs.contains_key(kind) && scene_meshes.contains_key(kind) {
                continue;
            }
            let opaque = materials
                .get(handle)
                .is_none_or(|mat| matches!(mat.alpha_mode, AlphaMode::Opaque));
            if opaque {
                scene_face_occluders.insert(*kind);
            }
        }
        let face_mark_materials = paint_catalog()
            .iter()
            .map(|(id, def)| {
                let textured = paint_registry
                    .get(id)
                    .and_then(|p| crate::game::scene_blocks::load_icon_png(&p.texture_path, images))
                    .map(|texture| StandardMaterial {
                        base_color: Color::WHITE,
                        base_color_texture: Some(texture),
                        emissive: LinearRgba::new(0.08, 0.08, 0.08, 1.0),
                        unlit: true,
                        cull_mode: None,
                        depth_bias: crate::game::world::rendering::depth_bias::PAINT,
                        ..default()
                    });
                let material = textured.unwrap_or_else(|| {
                    let base = if let Some(stamp_id) = stamp_catalog().id_by_string(&def.string_id)
                    {
                        let c = stamp_def(stamp_id).color;
                        Color::srgba(c.r, c.g, c.b, c.a)
                    } else {
                        Color::srgb(0.95, 0.12, 0.10)
                    };
                    StandardMaterial {
                        base_color: base,
                        emissive: base.to_linear() * 0.35,
                        unlit: true,
                        cull_mode: None,
                        depth_bias: crate::game::world::rendering::depth_bias::PAINT,
                        ..default()
                    }
                });
                (id, materials.add(material))
            })
            .collect();
        let model_materials = [
            (ModelMaterial::ConveyorBase, srgb_material(0.16, 0.18, 0.18)),
            (ModelMaterial::ConveyorBelt, srgb_material(0.02, 0.02, 0.02)),
            (ModelMaterial::DrillTip, srgb_material(0.82, 0.84, 0.82)),
            (ModelMaterial::Frame, srgb_material(0.42, 0.44, 0.44)),
            (ModelMaterial::DarkFrame, srgb_material(0.12, 0.13, 0.15)),
            (ModelMaterial::Belt, srgb_material(0.86, 0.46, 0.14)),
            (
                ModelMaterial::BeltStripe,
                emissive_material(1.0, 0.76, 0.28, 0.18, 0.10, 0.02),
            ),
            (
                ModelMaterial::WeldCore,
                emissive_material(1.0, 0.22, 0.10, 0.22, 0.04, 0.02),
            ),
            (
                ModelMaterial::Welding,
                emissive_material(0.18, 0.58, 1.0, 0.02, 0.12, 0.26),
            ),
            (
                ModelMaterial::Wire,
                emissive_material(1.0, 0.88, 0.30, 0.20, 0.12, 0.02),
            ),
            (
                ModelMaterial::Signal,
                emissive_material(0.12, 0.78, 1.0, 0.02, 0.18, 0.24),
            ),
            (
                ModelMaterial::Power,
                emissive_material(1.0, 0.52, 0.20, 0.22, 0.08, 0.02),
            ),
            (
                ModelMaterial::DetectorBody,
                block_material(BlockKind::Detector),
            ),
            (ModelMaterial::Pusher, srgb_material(0.54, 0.56, 0.54)),
            (
                ModelMaterial::Platform,
                textured_model_material(Color::WHITE, platform_texture.clone()),
            ),
            (
                ModelMaterial::PlatformBase,
                block_material(BlockKind::Platform),
            ),
            (ModelMaterial::Wood, srgb_material(0.72, 0.46, 0.22)),
            (
                ModelMaterial::WoodTexture,
                textured_model_material(Color::WHITE, wood_texture),
            ),
            (
                ModelMaterial::BorderedWoodTexture,
                textured_model_material(Color::WHITE, bordered_wood_texture),
            ),
            (
                ModelMaterial::StoneTexture,
                textured_model_material(Color::WHITE, stone_texture),
            ),
            (
                ModelMaterial::Lift,
                emissive_material(0.35, 0.82, 1.0, 0.03, 0.16, 0.22),
            ),
            (
                ModelMaterial::Rotation,
                emissive_material(0.70, 0.36, 1.0, 0.11, 0.04, 0.20),
            ),
            (ModelMaterial::Drill, srgb_material(0.06, 0.07, 0.08)),
            (
                ModelMaterial::Laser,
                emissive_material(1.0, 0.10, 0.22, 0.35, 0.01, 0.04),
            ),
            (
                ModelMaterial::Mirror,
                StandardMaterial {
                    base_color: Color::srgb(0.45, 0.88, 1.0),
                    emissive: LinearRgba::new(0.10, 0.22, 0.30, 1.0),
                    alpha_mode: AlphaMode::Blend,
                    perceptual_roughness: 0.72,
                    reflectance: 0.10,
                    cull_mode: None,
                    ..default()
                },
            ),
            (ModelMaterial::System, srgb_material(0.35, 0.28, 0.48)),
            (
                ModelMaterial::SystemAccent,
                emissive_material(0.72, 0.58, 1.0, 0.12, 0.08, 0.24),
            ),
            (
                ModelMaterial::TeleportIn,
                emissive_material(0.18, 0.62, 1.0, 0.02, 0.10, 0.34),
            ),
            (
                ModelMaterial::TeleportOut,
                emissive_material(1.0, 0.54, 0.18, 0.34, 0.10, 0.02),
            ),
            (ModelMaterial::SuctionCup, srgb_material(0.82, 0.84, 0.82)),
        ]
        .into_iter()
        .map(|(kind, material)| (kind, materials.add(material)))
        .collect::<HashMap<_, _>>();
        let model_preview_materials = model_materials
            .iter()
            .map(|(kind, handle)| {
                let source = materials
                    .get(handle)
                    .expect("model material exists")
                    .clone();
                (*kind, materials.add(preview_model_material(source)))
            })
            .collect();

        let factory_models = load_factory_visuals(meshes, materials, images);

        Self {
            block: {
                let mut mesh = Mesh::from(Cuboid::new(BLOCK_SIZE, BLOCK_SIZE, BLOCK_SIZE));
                // 无 GLB、靠 texture/normal.png 的立方体需要切线
                mesh.generate_tangents()
                    .expect("unit cube should generate tangents");
                meshes.add(mesh)
            },
            node: meshes.add(Cuboid::new(
                BLOCK_SIZE * 0.38,
                BLOCK_SIZE * 0.38,
                BLOCK_SIZE * 0.38,
            )),
            wire_node: meshes.add(Cuboid::new(
                BLOCK_SIZE * 0.304,
                BLOCK_SIZE * 0.304,
                BLOCK_SIZE * 0.304,
            )),
            // 漆/灯面板：零厚度面片（+Y 法线），spawn 时按附着法线旋转
            face_mark: meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(0.49))),
            // 预览印花：厚 0.1 居中；spawn 时平移到面外 0.55，整板外凸 0.1
            stamp_embed_plate: meshes.add(Cuboid::new(0.9, 0.9, 0.1)),
            weld_spark: meshes.add(Cuboid::new(0.24, 0.24, 0.24)),
            weld_burst_quad: meshes.add(Rectangle::new(1.0, 1.0)),
            connector_x: meshes.add(Cuboid::new(0.55, 0.045, 0.045)),
            connector_y: meshes.add(Cuboid::new(0.045, 0.55, 0.045)),
            connector_z: meshes.add(Cuboid::new(0.045, 0.045, 0.55)),
            wire_connector_x: meshes.add(Cuboid::new(0.652, 0.304, 0.304)),
            wire_connector_y: meshes.add(Cuboid::new(0.304, 0.652, 0.304)),
            wire_connector_z: meshes.add(Cuboid::new(0.304, 0.304, 0.652)),
            part_conveyor_base: meshes.add(Cuboid::new(1.0, 0.90, 1.0)),
            part_conveyor_belt: meshes.add(Cuboid::new(0.90, 0.10, 1.0)),
            part_drill_body: meshes.add(Cuboid::new(1.0, 1.0, 0.80)),
            part_drill_tip: meshes.add(drill_tip_mesh(0.34, 1.0, 48)),
            part_large: meshes.add(Cuboid::new(0.72, 0.22, 0.72)),
            part_medium: meshes.add(Cuboid::new(0.44, 0.20, 0.44)),
            part_small: meshes.add(Cuboid::new(0.22, 0.22, 0.22)),
            part_plate: meshes.add(Cuboid::new(0.78, 0.06, 0.78)),
            // 告示竖板：薄在 Z，大面朝 ±Z（局部 +Z 贴宿主）
            part_sign_board: meshes.add(Cuboid::new(0.78, 0.72, 0.06)),
            part_rotator_base: meshes.add(Cuboid::new(1.0, 0.80, 1.0)),
            part_rotator_disk: meshes.add(Cylinder::new(0.40, 0.20).mesh().resolution(48)),
            part_rotator_ring: meshes.add(rotator_ring_mesh(0.50, 0.40, 0.20, 64)),
            part_rod_x: meshes.add(Cuboid::new(0.72, 0.12, 0.12)),
            part_rod_y: meshes.add(Cuboid::new(0.12, 0.72, 0.12)),
            part_rod_z: meshes.add(Cuboid::new(0.12, 0.12, 0.72)),
            // 镜子面片：000, 101, 111, 010
            part_mirror_face: meshes.add(thick_quad_mesh([
                [0, 0, 0],
                [1, 0, 1],
                [1, 1, 1],
                [0, 1, 0],
            ])),
            // 垂直镜子面片：000, 001, 111, 110
            part_vertical_mirror_face: meshes.add(thick_quad_mesh([
                [0, 0, 0],
                [0, 0, 1],
                [1, 1, 1],
                [1, 1, 0],
            ])),
            // 分光镜：x+y+z=0 六边形再烘焙 -180° yaw
            part_splitter_face: meshes.add(thick_splitter_hexagon_mesh()),
            // 吸盘：工作面在 -Z，顶点在格子中心
            part_suction_cup: meshes.add(suction_cup_pyramid_mesh()),
            part_pusher_body: meshes.add(cover_cuboid_mesh(Vec3::new(1.0, 1.0, 0.80))),
            part_pusher_head: meshes.add(cover_cuboid_mesh(Vec3::new(1.0, 1.0, 0.20))),
            block_materials,
            break_debris_materials,
            preview_materials,
            scene_materials: scene_block_materials,
            scene_meshes,
            scene_face_uvs,
            scene_face_occluders,
            factory_models,
            goal_play_visual: false,
            goal_play_visual_initialized: false,
            goal_ghost_materials: HashMap::new(),
            face_mark_materials,
            light_panel_material: materials.add(StandardMaterial {
                base_color: Color::srgb(0.55, 0.58, 0.62),
                unlit: true,
                cull_mode: None,
                depth_bias: crate::game::world::rendering::depth_bias::PAINT,
                ..default()
            }),
            light_panel_lit_material: materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 0.92, 0.45),
                emissive: Color::srgb(0.55, 0.42, 0.08).into(),
                unlit: true,
                cull_mode: None,
                depth_bias: crate::game::world::rendering::depth_bias::PAINT,
                ..default()
            }),
            model_materials,
            model_preview_materials,
            wire_connector_material: materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 0.88, 0.30),
                emissive: Color::srgb(0.20, 0.12, 0.02).into(),
                ..default()
            }),
            active_wire_material: materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 0.08, 0.04),
                emissive: Color::srgb(0.34, 0.02, 0.01).into(),
                ..default()
            }),
            // 白杆 + 黄自发光（须 lit：unlit 会丢掉 emissive，Bloom 看不到）
            weld_connector_material: materials.add(StandardMaterial {
                base_color: Color::WHITE,
                emissive: LinearRgba::new(22.0, 14.0, 0.6, 1.0),
                perceptual_roughness: 1.0,
                metallic: 0.0,
                ..default()
            }),
            weld_burst_material: materials.add(StandardMaterial {
                base_color: Color::WHITE,
                emissive: LinearRgba::new(22.0, 14.0, 0.6, 1.0),
                perceptual_roughness: 1.0,
                metallic: 0.0,
                cull_mode: None,
                ..default()
            }),
            laser_beam_material: materials.add(StandardMaterial {
                base_color: Color::srgba(1.0, 0.12, 0.26, 0.92),
                emissive: LinearRgba::new(0.55, 0.02, 0.10, 1.0),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            }),
            delete_preview_material: materials.add(StandardMaterial {
                base_color: Color::srgba(1.0, 0.08, 0.04, 0.38),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            }),
            // 配色对齐 assets/factory_blocks/selection_box/face_albedo.png
            selection_fill_material: materials.add(StandardMaterial {
                base_color: Color::srgba(150.0 / 255.0, 210.0 / 255.0, 205.0 / 255.0, 58.0 / 255.0),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                cull_mode: None,
                depth_bias: crate::game::world::rendering::depth_bias::OVERLAY,
                ..default()
            }),
            selection_edge_material: materials.add(StandardMaterial {
                base_color: Color::srgb(180.0 / 255.0, 240.0 / 255.0, 225.0 / 255.0),
                emissive: LinearRgba::new(0.35, 0.55, 0.48, 1.0),
                alpha_mode: AlphaMode::Opaque,
                unlit: true,
                depth_bias: crate::game::world::rendering::depth_bias::OVERLAY,
                ..default()
            }),
            selection_invalid_fill_material: materials.add(StandardMaterial {
                base_color: Color::srgba(1.0, 0.12, 0.08, 0.38),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                cull_mode: None,
                depth_bias: crate::game::world::rendering::depth_bias::OVERLAY,
                ..default()
            }),
            selection_invalid_edge_material: materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 0.28, 0.18),
                emissive: LinearRgba::new(0.55, 0.08, 0.04, 1.0),
                alpha_mode: AlphaMode::Opaque,
                unlit: true,
                depth_bias: crate::game::world::rendering::depth_bias::OVERLAY,
                ..default()
            }),
            active_factory_debug_material: materials.add(StandardMaterial {
                base_color: Color::srgba(0.12, 0.90, 0.22, 0.38),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            }),
            inactive_factory_debug_material: materials.add(StandardMaterial {
                base_color: Color::srgba(0.95, 0.12, 0.08, 0.38),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            }),
        }
    }

    pub(crate) fn block_mesh(&self, kind: BlockKind) -> Handle<Mesh> {
        match kind.shape() {
            BlockShape::Cube => self.block.clone(),
            BlockShape::Node => self.node.clone(),
        }
    }

    pub(crate) fn wire_node_mesh(&self) -> Handle<Mesh> {
        self.wire_node.clone()
    }

    pub(crate) fn block_material(&self, kind: BlockKind) -> Handle<StandardMaterial> {
        self.block_materials
            .get(&kind)
            .expect("every block kind has a material")
            .clone()
    }

    /// 破坏碎片材质；无专用贴图时回退方块材质
    pub(crate) fn break_debris_material(&self, kind: BlockKind) -> Handle<StandardMaterial> {
        self.break_debris_materials
            .get(&kind)
            .cloned()
            .unwrap_or_else(|| self.block_material(kind))
    }

    pub(crate) fn active_factory_debug_material(&self) -> Handle<StandardMaterial> {
        self.active_factory_debug_material.clone()
    }

    pub(crate) fn inactive_factory_debug_material(&self) -> Handle<StandardMaterial> {
        self.inactive_factory_debug_material.clone()
    }

    pub(crate) fn edit_preview_material(&self, kind: EditPreviewKind) -> Handle<StandardMaterial> {
        match kind {
            EditPreviewKind::Delete => self.delete_preview_material.clone(),
            EditPreviewKind::Selection => self.selection_fill_material.clone(),
        }
    }

    pub(crate) fn selection_fill_material(&self) -> Handle<StandardMaterial> {
        self.selection_fill_material.clone()
    }

    pub(crate) fn selection_edge_material(&self) -> Handle<StandardMaterial> {
        self.selection_edge_material.clone()
    }

    pub(crate) fn selection_invalid_fill_material(&self) -> Handle<StandardMaterial> {
        self.selection_invalid_fill_material.clone()
    }

    pub(crate) fn selection_invalid_edge_material(&self) -> Handle<StandardMaterial> {
        self.selection_invalid_edge_material.clone()
    }

    pub(crate) fn block_preview_material(&self, kind: BlockKind) -> Handle<StandardMaterial> {
        self.preview_materials
            .get(&kind)
            .expect("every block kind has a preview material")
            .clone()
    }

    /// 游玩态验收器是否显示幽灵材料外观
    pub(crate) fn use_goal_play_visual(&self) -> bool {
        self.goal_play_visual
    }

    pub(crate) fn set_goal_play_visual(&mut self, play: bool) {
        self.goal_play_visual = play;
        self.goal_play_visual_initialized = true;
    }

    /// 填入游玩态验收器幽灵材质
    pub(crate) fn install_goal_ghost_materials(
        &mut self,
        standard_materials: &Assets<StandardMaterial>,
        ghost_materials: &mut Assets<crate::game::world::rendering::GoalGhostMaterial>,
        images: &mut Assets<Image>,
    ) {
        use crate::game::world::rendering::goal_ghost::{
            goal_ghost_from_standard, white_pixel_image,
        };
        let white = images.add(white_pixel_image());
        let mut map = HashMap::new();
        for (kind, handle) in &self.block_materials {
            if !kind.is_material() {
                continue;
            }
            let Some(standard) = standard_materials.get(handle) else {
                continue;
            };
            map.insert(
                *kind,
                ghost_materials.add(goal_ghost_from_standard(standard, &white)),
            );
        }
        self.goal_ghost_materials = map;
    }

    pub(crate) fn goal_ghost_material(
        &self,
        kind: BlockKind,
    ) -> Option<Handle<crate::game::world::rendering::GoalGhostMaterial>> {
        self.goal_ghost_materials.get(&kind).cloned()
    }

    pub(crate) fn goal_ghost_mesh(&self, kind: BlockKind) -> Handle<Mesh> {
        self.scene_mesh(kind)
            .unwrap_or_else(|| self.block_mesh(kind))
    }

    pub(crate) fn scene_material(&self, kind: BlockKind) -> Option<Handle<StandardMaterial>> {
        self.scene_materials.get(&kind).cloned()
    }

    pub(crate) fn scene_mesh(&self, kind: BlockKind) -> Option<Handle<Mesh>> {
        self.scene_meshes.get(&kind).cloned()
    }

    pub(crate) fn scene_face_uvs(&self, kind: BlockKind) -> Option<&[[f32; 2]; 24]> {
        self.scene_face_uvs.get(&kind)
    }

    /// 合并场景 mesh 时邻格是否按实心遮挡此面（玻璃/异形块为 false）
    pub(crate) fn scene_occludes_faces(&self, kind: BlockKind) -> bool {
        self.scene_face_occluders.contains(&kind)
    }

    /// 工厂 GLB 外观；无则走程序化零件
    pub(crate) fn factory_visual(&self, kind: BlockKind) -> Option<&FactoryVisual> {
        self.factory_models.get(&kind)
    }

    pub(crate) fn connector_mesh(&self, offset: IVec3) -> Handle<Mesh> {
        if offset.x != 0 {
            self.connector_x.clone()
        } else if offset.y != 0 {
            self.connector_y.clone()
        } else {
            self.connector_z.clone()
        }
    }

    pub(crate) fn wire_connector_mesh(&self, offset: IVec3) -> Handle<Mesh> {
        if offset.x != 0 {
            self.wire_connector_x.clone()
        } else if offset.y != 0 {
            self.wire_connector_y.clone()
        } else {
            self.wire_connector_z.clone()
        }
    }

    pub(crate) fn face_mark_mesh(&self, _normal: IVec3) -> Handle<Mesh> {
        self.face_mark.clone()
    }

    /// 预览用外凸印花板（厚 0.1，局部 +Z 朝外）
    pub(crate) fn stamp_embed_plate(&self) -> Handle<Mesh> {
        self.stamp_embed_plate.clone()
    }

    pub(crate) fn face_mark_material(&self, paint: PaintMaterialId) -> Handle<StandardMaterial> {
        self.face_mark_materials
            .get(&paint)
            .expect("every paint material has a face mark material")
            .clone()
    }

    pub(crate) fn model_mesh(&self, mesh: ModelMesh) -> Handle<Mesh> {
        match mesh {
            ModelMesh::ConveyorBase => self.part_conveyor_base.clone(),
            ModelMesh::ConveyorBelt => self.part_conveyor_belt.clone(),
            ModelMesh::DrillBody => self.part_drill_body.clone(),
            ModelMesh::DrillTip => self.part_drill_tip.clone(),
            ModelMesh::Large => self.part_large.clone(),
            ModelMesh::Medium => self.part_medium.clone(),
            ModelMesh::Small => self.part_small.clone(),
            ModelMesh::Plate => self.part_plate.clone(),
            ModelMesh::SignBoard => self.part_sign_board.clone(),
            ModelMesh::RotatorBase => self.part_rotator_base.clone(),
            ModelMesh::RotatorDisk => self.part_rotator_disk.clone(),
            ModelMesh::RotatorRing => self.part_rotator_ring.clone(),
            ModelMesh::RodX => self.part_rod_x.clone(),
            ModelMesh::RodY => self.part_rod_y.clone(),
            ModelMesh::RodZ => self.part_rod_z.clone(),
            ModelMesh::MirrorFace => self.part_mirror_face.clone(),
            ModelMesh::VerticalMirrorFace => self.part_vertical_mirror_face.clone(),
            ModelMesh::SplitterFace => self.part_splitter_face.clone(),
            ModelMesh::SuctionCup => self.part_suction_cup.clone(),
            ModelMesh::PusherBody => self.part_pusher_body.clone(),
            ModelMesh::PusherHead => self.part_pusher_head.clone(),
        }
    }

    pub(crate) fn model_material(&self, material: ModelMaterial) -> Handle<StandardMaterial> {
        self.model_materials
            .get(&material)
            .expect("every model material exists")
            .clone()
    }

    pub(crate) fn model_preview_material(
        &self,
        material: ModelMaterial,
    ) -> Handle<StandardMaterial> {
        self.model_preview_materials
            .get(&material)
            .expect("every model material has a preview material")
            .clone()
    }
}

fn block_material(kind: BlockKind) -> StandardMaterial {
    let mut material = StandardMaterial {
        base_color: kind.material(),
        perceptual_roughness: 0.88,
        reflectance: 0.18,
        ..default()
    };
    if kind.is_transparent() {
        material.alpha_mode = AlphaMode::Blend;
        material.unlit = kind.is_generated_marker();
    }
    material
}

fn textured_block_material(kind: BlockKind, texture: Handle<Image>) -> StandardMaterial {
    StandardMaterial {
        base_color: kind.material(),
        base_color_texture: Some(texture),
        perceptual_roughness: 0.94,
        reflectance: 0.10,
        ..default()
    }
}

fn preview_block_material(
    kind: BlockKind,
    texture: Option<Handle<Image>>,
    normal: Option<Handle<Image>>,
) -> StandardMaterial {
    StandardMaterial {
        base_color: kind.material().with_alpha(0.46),
        base_color_texture: texture,
        normal_map_texture: normal,
        alpha_mode: AlphaMode::Blend,
        perceptual_roughness: 0.94,
        reflectance: 0.08,
        ..default()
    }
}

fn preview_model_material(material: StandardMaterial) -> StandardMaterial {
    // 不用 Blend：多零件 GLB 每帧重建时透明排序会闪；Opaque 幽灵色更稳
    let c = material.base_color.to_srgba();
    StandardMaterial {
        base_color: Color::srgba(
            c.red * 0.55 + 0.28,
            c.green * 0.55 + 0.30,
            c.blue * 0.55 + 0.34,
            1.0,
        ),
        base_color_texture: material.base_color_texture,
        normal_map_texture: material.normal_map_texture,
        emissive: material.emissive * 0.25,
        metallic: material.metallic * 0.35,
        perceptual_roughness: material.perceptual_roughness.max(0.75),
        reflectance: material.reflectance,
        alpha_mode: AlphaMode::Opaque,
        cull_mode: material.cull_mode,
        unlit: false,
        ..default()
    }
}

fn scene_color_material(base_color: Color) -> StandardMaterial {
    StandardMaterial {
        base_color,
        perceptual_roughness: 0.96,
        reflectance: 0.08,
        ..default()
    }
}

/// 印花无 GLB 时的默认薄板（局部 +Z 朝宿主；厚 0.1，板心 +0.45 → 贴宿主面外凸 0.1）
fn stamp_plate_mesh() -> Mesh {
    let mut mesh = Mesh::from(Cuboid::new(0.78, 0.72, 0.1));
    if let Some(positions) = mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION) {
        if let bevy::mesh::VertexAttributeValues::Float32x3(values) = positions {
            for v in values.iter_mut() {
                v[2] += 0.45;
            }
        }
    }
    mesh
}

/// 扫描 factory_blocks 目录，按种类装成 FactoryVisual
fn load_factory_visuals(
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    images: &mut Assets<Image>,
) -> HashMap<BlockKind, FactoryVisual> {
    use std::f32::consts::{FRAC_PI_2, PI};
    use std::path::PathBuf;

    let root = PathBuf::from(crate::shared::platform::asset_path()).join("factory_blocks");
    let mut map = HashMap::new();

    // 反向传送带：绕局部 X 翻 180°，顶面到底面
    let static_dirs: &[(BlockKind, &str, Quat)] = &[
        (BlockKind::Platform, "platform", Quat::IDENTITY),
        (BlockKind::Conveyor, "conveyor", Quat::IDENTITY),
        (
            BlockKind::ReverseConveyor,
            "conveyor",
            Quat::from_rotation_x(PI),
        ),
        (BlockKind::Rotator, "rotator", Quat::IDENTITY),
        (BlockKind::CounterRotator, "counter_rotator", Quat::IDENTITY),
        (BlockKind::Detector, "detector", Quat::IDENTITY),
        (BlockKind::DownDetector, "detector", Quat::IDENTITY),
        (BlockKind::Lifter, "lifter", Quat::IDENTITY),
        (BlockKind::Welder, "welder", Quat::IDENTITY),
        (
            BlockKind::DownWelder,
            "welder",
            Quat::from_rotation_x(-FRAC_PI_2),
        ),
        (BlockKind::Laser, "laser", Quat::IDENTITY),
        (BlockKind::Mirror, "mirror", Quat::IDENTITY),
        (BlockKind::VerticalMirror, "vertical_mirror", Quat::IDENTITY),
        (BlockKind::Splitter, "splitter", Quat::IDENTITY),
        (BlockKind::SuctionCup, "suction_cup", Quat::IDENTITY),
    ];
    for &(kind, dir, local_rotation) in static_dirs {
        let path = root.join(dir).join("model.glb");
        match load_factory_glb(&path, meshes, materials, images) {
            Ok(raw) => {
                map.insert(
                    kind,
                    FactoryVisual::Static {
                        parts: factory_parts_with_preview(raw, materials),
                        local_rotation,
                    },
                );
            }
            Err(err) => {
                bevy::log::warn!("factory glb load failed ({}): {err}", kind.name_key());
            }
        }
    }

    let drill_path = root.join("drill").join("model.glb");
    match load_factory_glb(&drill_path, meshes, materials, images) {
        Ok(raw) => match split_drill_parts(raw, materials) {
            Some(visual) => {
                map.insert(BlockKind::Drill, visual);
            }
            None => {
                bevy::log::warn!("factory drill glb missing Body/Head");
            }
        },
        Err(err) => {
            bevy::log::warn!("factory glb load failed (drill): {err}");
        }
    }

    for &(kind, dir) in &[
        (BlockKind::Pusher, "pusher"),
        (BlockKind::Blocker, "blocker"),
    ] {
        let path = root.join(dir).join("model.glb");
        match load_factory_glb(&path, meshes, materials, images) {
            Ok(raw) => match split_pusher_parts(raw, materials) {
                Some(visual) => {
                    map.insert(kind, visual);
                }
                None => {
                    bevy::log::warn!(
                        "factory pusher glb missing Body/Stage/Head ({})",
                        kind.name_key()
                    );
                }
            },
            Err(err) => {
                bevy::log::warn!("factory glb load failed ({}): {err}", kind.name_key());
            }
        }
    }

    let wire_path = root.join("wire").join("model.glb");
    match load_factory_glb(&wire_path, meshes, materials, images) {
        Ok(raw) => match split_wire_faces(raw, materials) {
            Some(visual) => {
                map.insert(BlockKind::Wire, visual);
            }
            None => {
                bevy::log::warn!("factory wire glb missing Pos/Neg face groups");
            }
        },
        Err(err) => {
            bevy::log::warn!("factory glb load failed (wire): {err}");
        }
    }

    map
}

/// 给工厂零件补半透明预览材质
fn factory_parts_with_preview(
    raw: Vec<FactoryGltfPart>,
    materials: &mut Assets<StandardMaterial>,
) -> Vec<FactoryPartHandles> {
    raw.into_iter()
        .map(|part| {
            let preview_material = materials
                .get(&part.material)
                .cloned()
                .map(|source| materials.add(preview_model_material(source)))
                .unwrap_or_else(|| part.material.clone());
            FactoryPartHandles {
                mesh: part.mesh,
                material: part.material,
                preview_material,
            }
        })
        .collect()
}

/// 按 Body / Head 拆钻头零件
fn split_drill_parts(
    raw: Vec<FactoryGltfPart>,
    materials: &mut Assets<StandardMaterial>,
) -> Option<FactoryVisual> {
    let mut body = Vec::new();
    let mut head = Vec::new();
    for part in raw {
        let group = part.group.as_deref().unwrap_or("");
        let handles = {
            let preview_material = materials
                .get(&part.material)
                .cloned()
                .map(|source| materials.add(preview_model_material(source)))
                .unwrap_or_else(|| part.material.clone());
            FactoryPartHandles {
                mesh: part.mesh,
                material: part.material,
                preview_material,
            }
        };
        match group {
            "Head" => head.push(handles),
            _ => body.push(handles),
        }
    }
    if body.is_empty() || head.is_empty() {
        return None;
    }
    Some(FactoryVisual::Drill { body, head })
}

/// 按 Body / Stage / Head 拆活塞零件
fn split_pusher_parts(
    raw: Vec<FactoryGltfPart>,
    materials: &mut Assets<StandardMaterial>,
) -> Option<FactoryVisual> {
    let mut body = Vec::new();
    let mut stage = Vec::new();
    let mut head = Vec::new();
    for part in raw {
        let group = part.group.as_deref().unwrap_or("");
        let handles = {
            let preview_material = materials
                .get(&part.material)
                .cloned()
                .map(|source| materials.add(preview_model_material(source)))
                .unwrap_or_else(|| part.material.clone());
            FactoryPartHandles {
                mesh: part.mesh,
                material: part.material,
                preview_material,
            }
        };
        match group {
            "Body" => body.push(handles),
            "Stage" => stage.push(handles),
            "Head" => head.push(handles),
            _ => body.push(handles),
        }
    }
    if body.is_empty() || head.is_empty() {
        return None;
    }
    Some(FactoryVisual::Pusher { body, stage, head })
}

/// 按 PosX…NegZ / PosX_Power… 拆电线六向臂与通电条
fn split_wire_faces(
    raw: Vec<FactoryGltfPart>,
    materials: &mut Assets<StandardMaterial>,
) -> Option<FactoryVisual> {
    let mut faces: [Vec<FactoryPartHandles>; 6] = Default::default();
    let mut power: [Vec<FactoryPartHandles>; 6] = Default::default();
    let mut any = false;
    for part in raw {
        let group = part.group.as_deref().unwrap_or("");
        let (is_power, face_name) = match group.strip_suffix("_Power") {
            Some(face) => (true, face),
            None => (false, group),
        };
        let Some(index) = wire_face_index(Some(face_name)) else {
            continue;
        };
        any = true;
        let mut handles = {
            let preview_material = materials
                .get(&part.material)
                .cloned()
                .map(|source| materials.add(preview_model_material(source)))
                .unwrap_or_else(|| part.material.clone());
            FactoryPartHandles {
                mesh: part.mesh,
                material: part.material,
                preview_material,
            }
        };
        // 通电条：白自发光（须 lit，否则 emissive 不进 HDR，Bloom 无效）
        if is_power {
            handles.material = materials.add(StandardMaterial {
                base_color: Color::WHITE,
                emissive: LinearRgba::new(16.0, 16.0, 16.0, 1.0),
                perceptual_roughness: 1.0,
                metallic: 0.0,
                ..default()
            });
            handles.preview_material = handles.material.clone();
            power[index].push(handles);
        } else {
            faces[index].push(handles);
        }
    }
    if !any {
        return None;
    }
    Some(FactoryVisual::Wire { faces, power })
}

/// 电线节点名 → signal_neighbor_offsets 下标
fn wire_face_index(group: Option<&str>) -> Option<usize> {
    match group? {
        "PosX" => Some(0),
        "NegX" => Some(1),
        "PosY" => Some(2),
        "NegY" => Some(3),
        "PosZ" => Some(4),
        "NegZ" => Some(5),
        _ => None,
    }
}

/// 把 model.glb 或 texture.png（可选 normal.png）装进 scene_* / block_materials
/// 有 model.glb 时只走 GLB（不读外部贴图）；无模型的纯立方体才用 texture/normal.png
/// `stamp_plate`：印花在无 GLB 走贴图/纯色 fallback 时用的薄板网格
fn insert_configured_pack(
    kind: BlockKind,
    model_path: Option<&Path>,
    texture_path: Option<&Path>,
    normal_path: Option<&Path>,
    fallback_color: Color,
    stamp_plate: Option<&Handle<Mesh>>,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    images: &mut Assets<Image>,
    scene_meshes: &mut HashMap<BlockKind, Handle<Mesh>>,
    scene_face_uvs: &mut HashMap<BlockKind, [[f32; 2]; 24]>,
    scene_block_materials: &mut HashMap<BlockKind, Handle<StandardMaterial>>,
    block_materials: &mut HashMap<BlockKind, Handle<StandardMaterial>>,
    preview_materials: &mut HashMap<BlockKind, Handle<StandardMaterial>>,
) {
    if let Some(model_path) = model_path {
        match crate::game::scene_blocks::load_scene_glb(model_path, meshes, materials, images) {
            Ok(loaded) => {
                if let Some(uvs) = loaded.face_uvs {
                    scene_face_uvs.insert(kind, uvs);
                }
                scene_meshes.insert(kind, loaded.mesh);
                if let Some(base) = materials.get(&loaded.material).cloned() {
                    preview_materials.insert(kind, materials.add(preview_model_material(base)));
                }
                scene_block_materials.insert(kind, loaded.material.clone());
                block_materials.insert(kind, loaded.material);
                return;
            }
            Err(err) => {
                bevy::log::error!(
                    "configured pack glb load failed ({}): {err}",
                    kind.name_key()
                );
            }
        }
    }

    if let Some(texture_path) = texture_path {
        if let Some(texture) =
            crate::game::scene_blocks::load_block_texture_png(texture_path, images)
        {
            let normal = normal_path.and_then(|path| {
                let handle = crate::game::scene_blocks::load_block_normal_png(path, images);
                if handle.is_none() {
                    bevy::log::error!("configured pack normal load failed: {}", path.display());
                }
                handle
            });
            let mut material = StandardMaterial {
                base_color: Color::WHITE,
                base_color_texture: Some(texture.clone()),
                normal_map_texture: normal.clone(),
                perceptual_roughness: 0.94,
                reflectance: 0.10,
                ..default()
            };
            if kind.is_transparent() {
                material.alpha_mode = AlphaMode::Blend;
            }
            let material = materials.add(material);
            preview_materials.insert(
                kind,
                materials.add(preview_block_material(kind, Some(texture), normal)),
            );
            scene_block_materials.insert(kind, material.clone());
            block_materials.insert(kind, material);
            if let Some(plate) = stamp_plate {
                scene_meshes.insert(kind, plate.clone());
            }
            return;
        }
        bevy::log::error!(
            "configured pack texture load failed: {}",
            texture_path.display()
        );
    }

    let fallback = materials.add(scene_color_material(fallback_color));
    scene_block_materials.insert(kind, fallback.clone());
    block_materials.insert(kind, fallback);
    if let Some(plate) = stamp_plate {
        scene_meshes.insert(kind, plate.clone());
    }
}

fn srgb_material(r: f32, g: f32, b: f32) -> StandardMaterial {
    StandardMaterial {
        base_color: Color::srgb(r, g, b),
        perceptual_roughness: 0.82,
        reflectance: 0.16,
        ..default()
    }
}

fn textured_model_material(base_color: Color, texture: Handle<Image>) -> StandardMaterial {
    StandardMaterial {
        base_color,
        base_color_texture: Some(texture),
        perceptual_roughness: 0.90,
        reflectance: 0.12,
        ..default()
    }
}

fn block_corner(v: [u8; 3]) -> Vec3 {
    Vec3::new(v[0] as f32 - 0.5, v[1] as f32 - 0.5, v[2] as f32 - 0.5)
}

/// 吸盘四棱锥：工作面为局部 -Z 的 1×1 面，顶点在格子中心
fn suction_cup_pyramid_mesh() -> Mesh {
    let base = [
        block_corner([0, 0, 0]),
        block_corner([1, 0, 0]),
        block_corner([1, 1, 0]),
        block_corner([0, 1, 0]),
    ];
    let apex = Vec3::ZERO;

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    // 工作面（外法线 -Z）
    push_face(
        &mut positions,
        &mut normals,
        &mut uvs,
        &mut indices,
        &base,
        Vec3::NEG_Z,
        &[[0, 3, 2], [0, 2, 1]],
    );

    // 四个侧面：从底边到顶点
    for i in 0..4 {
        let next = (i + 1) % 4;
        let tri = [base[i], base[next], apex];
        let normal = (tri[1] - tri[0]).cross(tri[2] - tri[0]).normalize_or_zero();
        push_face(
            &mut positions,
            &mut normals,
            &mut uvs,
            &mut indices,
            &tri,
            normal,
            &[[0, 1, 2]],
        );
    }

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(Indices::U32(indices))
}

const MIRROR_FACE_THICKNESS: f32 = 0.06;

fn thick_quad_mesh(corners: [[u8; 3]; 4]) -> Mesh {
    let vertices = corners.map(block_corner);
    thick_face_mesh(&vertices, &[[0, 1, 2], [0, 2, 3]], MIRROR_FACE_THICKNESS)
}

// 分光镜六边形：先取过中心的 x+y+z=0 切面，再把 -180° yaw 烘焙进顶点
fn thick_splitter_hexagon_mesh() -> Mesh {
    let yaw = Quat::from_rotation_y(-std::f32::consts::PI);
    // x+y+z=0 与立方体相交的六个边中点，绕向使法线朝向 (1,1,1)
    let vertices = [
        Vec3::new(0.5, -0.5, 0.0),
        Vec3::new(0.5, 0.0, -0.5),
        Vec3::new(0.0, 0.5, -0.5),
        Vec3::new(-0.5, 0.5, 0.0),
        Vec3::new(-0.5, 0.0, 0.5),
        Vec3::new(0.0, -0.5, 0.5),
    ]
    .map(|vertex| yaw * vertex);
    thick_face_mesh(
        &vertices,
        &[[0, 1, 2], [0, 2, 3], [0, 3, 4], [0, 4, 5]],
        MIRROR_FACE_THICKNESS,
    )
}

fn thick_face_mesh(vertices: &[Vec3], front_triangles: &[[usize; 3]], thickness: f32) -> Mesh {
    let normal = face_normal(vertices, front_triangles[0]);
    let back: Vec<Vec3> = vertices
        .iter()
        .map(|vertex| *vertex - normal * thickness)
        .collect();

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    push_face(
        &mut positions,
        &mut normals,
        &mut uvs,
        &mut indices,
        vertices,
        normal,
        front_triangles,
    );
    let back_triangles: Vec<[usize; 3]> = front_triangles
        .iter()
        .map(|triangle| [triangle[0], triangle[2], triangle[1]])
        .collect();
    push_face(
        &mut positions,
        &mut normals,
        &mut uvs,
        &mut indices,
        &back,
        -normal,
        &back_triangles,
    );

    for index in 0..vertices.len() {
        let next = (index + 1) % vertices.len();
        push_side_quad(
            &mut positions,
            &mut normals,
            &mut uvs,
            &mut indices,
            vertices[index],
            vertices[next],
            back[next],
            back[index],
        );
    }

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(Indices::U32(indices))
}

fn face_normal(vertices: &[Vec3], triangle: [usize; 3]) -> Vec3 {
    (vertices[triangle[1]] - vertices[triangle[0]])
        .cross(vertices[triangle[2]] - vertices[triangle[0]])
        .normalize_or_zero()
}

fn push_face(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
    vertices: &[Vec3],
    normal: Vec3,
    triangles: &[[usize; 3]],
) {
    let base = positions.len() as u32;
    for vertex in vertices {
        positions.push([vertex.x, vertex.y, vertex.z]);
        normals.push([normal.x, normal.y, normal.z]);
        uvs.push([vertex.x + 0.5, vertex.y + 0.5]);
    }
    for triangle in triangles {
        indices.extend(triangle.iter().map(|index| base + *index as u32));
    }
}

fn push_side_quad(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
    front_a: Vec3,
    front_b: Vec3,
    back_b: Vec3,
    back_a: Vec3,
) {
    let side_normal = (front_b - front_a)
        .cross(back_a - front_a)
        .normalize_or_zero();
    let base = positions.len() as u32;
    for vertex in [front_a, front_b, back_b, back_a] {
        positions.push([vertex.x, vertex.y, vertex.z]);
        normals.push([side_normal.x, side_normal.y, side_normal.z]);
        uvs.push([vertex.x + 0.5, vertex.z + 0.5]);
    }
    indices.extend([base, base + 1, base + 2, base, base + 2, base + 3]);
}

fn cover_cuboid_mesh(size: Vec3) -> Mesh {
    let min = -size * 0.5;
    let max = size * 0.5;
    let faces = [
        (
            [
                [min.x, min.y, max.z],
                [max.x, min.y, max.z],
                [max.x, max.y, max.z],
                [min.x, max.y, max.z],
            ],
            [0.0, 0.0, 1.0],
            size.x,
            size.y,
        ),
        (
            [
                [min.x, max.y, min.z],
                [max.x, max.y, min.z],
                [max.x, min.y, min.z],
                [min.x, min.y, min.z],
            ],
            [0.0, 0.0, -1.0],
            size.x,
            size.y,
        ),
        (
            [
                [max.x, min.y, min.z],
                [max.x, max.y, min.z],
                [max.x, max.y, max.z],
                [max.x, min.y, max.z],
            ],
            [1.0, 0.0, 0.0],
            size.z,
            size.y,
        ),
        (
            [
                [min.x, min.y, max.z],
                [min.x, max.y, max.z],
                [min.x, max.y, min.z],
                [min.x, min.y, min.z],
            ],
            [-1.0, 0.0, 0.0],
            size.z,
            size.y,
        ),
        (
            [
                [max.x, max.y, min.z],
                [min.x, max.y, min.z],
                [min.x, max.y, max.z],
                [max.x, max.y, max.z],
            ],
            [0.0, 1.0, 0.0],
            size.x,
            size.z,
        ),
        (
            [
                [max.x, min.y, max.z],
                [min.x, min.y, max.z],
                [min.x, min.y, min.z],
                [max.x, min.y, min.z],
            ],
            [0.0, -1.0, 0.0],
            size.x,
            size.z,
        ),
    ];

    let mut positions = Vec::with_capacity(24);
    let mut normals = Vec::with_capacity(24);
    let mut uvs = Vec::with_capacity(24);
    let mut indices = Vec::with_capacity(36);

    for (face_index, (face_positions, normal, width, height)) in faces.into_iter().enumerate() {
        let base = face_index as u32 * 4;
        positions.extend_from_slice(&face_positions);
        normals.extend_from_slice(&[normal; 4]);
        uvs.extend_from_slice(&cover_uvs(width, height));
        indices.extend_from_slice(&[base, base + 1, base + 2, base + 2, base + 3, base]);
    }

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(Indices::U32(indices))
}

fn cover_uvs(width: f32, height: f32) -> [[f32; 2]; 4] {
    let min_side = width.min(height).max(f32::EPSILON);
    let u_span = width / min_side;
    let v_span = height / min_side;
    let u0 = 0.5 - u_span * 0.5;
    let u1 = 0.5 + u_span * 0.5;
    let v0 = 0.5 - v_span * 0.5;
    let v1 = 0.5 + v_span * 0.5;
    [[u0, v0], [u1, v0], [u1, v1], [u0, v1]]
}

fn emissive_material(r: f32, g: f32, b: f32, er: f32, eg: f32, eb: f32) -> StandardMaterial {
    StandardMaterial {
        base_color: Color::srgb(r, g, b),
        emissive: Color::srgb(er, eg, eb).into(),
        perceptual_roughness: 0.72,
        reflectance: 0.10,
        ..default()
    }
}

fn rotator_ring_mesh(outer_radius: f32, inner_radius: f32, height: f32, segments: u32) -> Mesh {
    let half_height = height * 0.5;
    let mut positions = Vec::with_capacity((segments * 8) as usize);
    let mut normals = Vec::with_capacity((segments * 8) as usize);
    let mut uvs = Vec::with_capacity((segments * 8) as usize);
    let mut indices = Vec::with_capacity((segments * 24) as usize);

    for i in 0..segments {
        let angle = i as f32 / segments as f32 * std::f32::consts::TAU;
        let (sin, cos) = angle.sin_cos();
        let outer = [cos * outer_radius, sin * outer_radius];
        let inner = [cos * inner_radius, sin * inner_radius];

        positions.extend_from_slice(&[
            [outer[0], half_height, outer[1]],
            [inner[0], half_height, inner[1]],
            [outer[0], -half_height, outer[1]],
            [inner[0], -half_height, inner[1]],
            [outer[0], half_height, outer[1]],
            [outer[0], -half_height, outer[1]],
            [inner[0], half_height, inner[1]],
            [inner[0], -half_height, inner[1]],
        ]);
        normals.extend_from_slice(&[
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, -1.0, 0.0],
            [0.0, -1.0, 0.0],
            [cos, 0.0, sin],
            [cos, 0.0, sin],
            [-cos, 0.0, -sin],
            [-cos, 0.0, -sin],
        ]);
        let u = i as f32 / segments as f32;
        uvs.extend_from_slice(&[
            [u, 1.0],
            [u, 0.0],
            [u, 1.0],
            [u, 0.0],
            [u, 1.0],
            [u, 0.0],
            [u, 1.0],
            [u, 0.0],
        ]);
    }

    for i in 0..segments {
        let next = (i + 1) % segments;
        let a = i * 8;
        let b = next * 8;
        indices.extend_from_slice(&[
            a,
            a + 1,
            b,
            a + 1,
            b + 1,
            b,
            a + 2,
            b + 2,
            a + 3,
            a + 3,
            b + 2,
            b + 3,
            a + 4,
            b + 4,
            a + 5,
            a + 5,
            b + 4,
            b + 5,
            a + 6,
            a + 7,
            b + 6,
            a + 7,
            b + 7,
            b + 6,
        ]);
    }

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_indices(Indices::U32(indices))
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
}

fn drill_tip_mesh(radius: f32, length: f32, segments: u32) -> Mesh {
    let base_z = 0.0;
    let tip_z = -length;
    let mut positions = Vec::with_capacity((segments * 2 + 2) as usize);
    let mut normals = Vec::with_capacity((segments * 2 + 2) as usize);
    let mut uvs = Vec::with_capacity((segments * 2 + 2) as usize);
    let mut indices = Vec::with_capacity((segments * 6) as usize);

    positions.push([0.0, 0.0, tip_z]);
    normals.push([0.0, 0.0, -1.0]);
    uvs.push([0.5, 0.5]);

    for i in 0..segments {
        let angle = i as f32 / segments as f32 * std::f32::consts::TAU;
        let (sin, cos) = angle.sin_cos();
        let normal = Vec3::new(cos, sin, radius / length).normalize();
        positions.push([cos * radius, sin * radius, base_z]);
        normals.push(normal.to_array());
        uvs.push([i as f32 / segments as f32, 1.0]);
    }

    let base_center = positions.len() as u32;
    positions.push([0.0, 0.0, base_z]);
    normals.push([0.0, 0.0, 1.0]);
    uvs.push([0.5, 0.5]);

    let base_start = positions.len() as u32;
    for i in 0..segments {
        let angle = i as f32 / segments as f32 * std::f32::consts::TAU;
        let (sin, cos) = angle.sin_cos();
        positions.push([cos * radius, sin * radius, base_z]);
        normals.push([0.0, 0.0, 1.0]);
        uvs.push([0.5 + cos * 0.5, 0.5 + sin * 0.5]);
    }

    for i in 0..segments {
        let next = if i + 1 == segments { 0 } else { i + 1 };
        indices.extend_from_slice(&[0, 1 + i, 1 + next]);
        indices.extend_from_slice(&[base_center, base_start + next, base_start + i]);
    }

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_indices(Indices::U32(indices))
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
}
