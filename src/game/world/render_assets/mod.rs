//! 世界渲染资源：网格、材质与工厂/场景外观

mod factory;
mod materials;
mod meshes;
mod packs;

use std::collections::{HashMap, HashSet};

use bevy::prelude::*;

use crate::game::blocks::pusher::texture;
use crate::game::blocks::{
    BLOCK_SIZE, BlockKind, BlockPresent, BlockShape, ModelMaterial, ModelMesh, PaintMaterialId,
    all_blocks, paint_catalog, stamp_catalog, stamp_def,
};

pub use factory::{FactoryPartHandles, FactoryVisual};

/// 世界渲染用的网格与材质句柄集合
#[derive(Resource, Clone)]
pub struct WorldRenderAssets {
    pub(crate) block: Handle<Mesh>,
    node: Handle<Mesh>,
    wire_node: Handle<Mesh>,
    pub(crate) face_mark: Handle<Mesh>,
    /// 灯面板 mesh（factory_blocks/light_panel/model.glb，板心已烘焙）
    pub(crate) light_panel: Handle<Mesh>,
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

/// 编辑预览种类（删除 / 选区）
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
                    .map(|texture| materials::textured_block_material(kind, texture))
                    .unwrap_or_else(|| materials::block_material(kind));
                (kind, materials.add(material))
            })
            .collect();
        let preview_materials: HashMap<BlockKind, Handle<StandardMaterial>> = all_blocks()
            .into_iter()
            .map(|kind| {
                let texture = block_textures.get(&kind).cloned();
                (
                    kind,
                    materials.add(materials::preview_block_material(kind, texture, None)),
                )
            })
            .collect();
        // 场景 / 材料 / 印花：model.glb 或 texture.png → scene_materials 等 HashMap
        let mut scene_meshes = HashMap::new();
        let mut scene_face_uvs = HashMap::new();
        let mut scene_block_materials = HashMap::new();
        let mut block_materials = block_materials;
        let mut preview_materials = preview_materials;
        for kind in all_blocks().into_iter().filter(|kind| kind.is_scene()) {
            let Some(presentation) = scene_registry.get_kind(kind) else {
                continue;
            };
            packs::insert_configured_pack(
                kind,
                presentation.model_path.as_deref(),
                presentation.texture_path.as_deref(),
                None,
                kind.material(),
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
            packs::insert_configured_pack(
                kind,
                presentation.model_path.as_deref(),
                presentation.texture_path.as_deref(),
                presentation.normal_path.as_deref(),
                kind.material(),
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
            packs::insert_configured_pack(
                kind,
                presentation.model_path.as_deref(),
                presentation.texture_path.as_deref(),
                None,
                kind.material(),
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
            (ModelMaterial::ConveyorBase, materials::srgb_material(0.16, 0.18, 0.18)),
            (ModelMaterial::ConveyorBelt, materials::srgb_material(0.02, 0.02, 0.02)),
            (ModelMaterial::DrillTip, materials::srgb_material(0.82, 0.84, 0.82)),
            (ModelMaterial::Frame, materials::srgb_material(0.42, 0.44, 0.44)),
            (ModelMaterial::DarkFrame, materials::srgb_material(0.12, 0.13, 0.15)),
            (ModelMaterial::Belt, materials::srgb_material(0.86, 0.46, 0.14)),
            (
                ModelMaterial::BeltStripe,
                materials::emissive_material(1.0, 0.76, 0.28, 0.18, 0.10, 0.02),
            ),
            (
                ModelMaterial::WeldCore,
                materials::emissive_material(1.0, 0.22, 0.10, 0.22, 0.04, 0.02),
            ),
            (
                ModelMaterial::Welding,
                materials::emissive_material(0.18, 0.58, 1.0, 0.02, 0.12, 0.26),
            ),
            (
                ModelMaterial::Wire,
                materials::emissive_material(1.0, 0.88, 0.30, 0.20, 0.12, 0.02),
            ),
            (
                ModelMaterial::Signal,
                materials::emissive_material(0.12, 0.78, 1.0, 0.02, 0.18, 0.24),
            ),
            (
                ModelMaterial::Power,
                materials::emissive_material(1.0, 0.52, 0.20, 0.22, 0.08, 0.02),
            ),
            (
                ModelMaterial::DetectorBody,
                materials::block_material(BlockKind::Detector),
            ),
            (ModelMaterial::Pusher, materials::srgb_material(0.54, 0.56, 0.54)),
            (
                ModelMaterial::Platform,
                materials::textured_model_material(Color::WHITE, platform_texture.clone()),
            ),
            (
                ModelMaterial::PlatformBase,
                materials::block_material(BlockKind::Platform),
            ),
            (ModelMaterial::Wood, materials::srgb_material(0.72, 0.46, 0.22)),
            (
                ModelMaterial::WoodTexture,
                materials::textured_model_material(Color::WHITE, wood_texture),
            ),
            (
                ModelMaterial::BorderedWoodTexture,
                materials::textured_model_material(Color::WHITE, bordered_wood_texture),
            ),
            (
                ModelMaterial::StoneTexture,
                materials::textured_model_material(Color::WHITE, stone_texture),
            ),
            (
                ModelMaterial::Lift,
                materials::emissive_material(0.35, 0.82, 1.0, 0.03, 0.16, 0.22),
            ),
            (
                ModelMaterial::Rotation,
                materials::emissive_material(0.70, 0.36, 1.0, 0.11, 0.04, 0.20),
            ),
            (ModelMaterial::Drill, materials::srgb_material(0.06, 0.07, 0.08)),
            (
                ModelMaterial::Laser,
                materials::emissive_material(1.0, 0.10, 0.22, 0.35, 0.01, 0.04),
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
            (ModelMaterial::System, materials::srgb_material(0.35, 0.28, 0.48)),
            (
                ModelMaterial::SystemAccent,
                materials::emissive_material(0.72, 0.58, 1.0, 0.12, 0.08, 0.24),
            ),
            (
                ModelMaterial::TeleportIn,
                materials::emissive_material(0.18, 0.62, 1.0, 0.02, 0.10, 0.34),
            ),
            (
                ModelMaterial::TeleportOut,
                materials::emissive_material(1.0, 0.54, 0.18, 0.34, 0.10, 0.02),
            ),
            (ModelMaterial::SuctionCup, materials::srgb_material(0.82, 0.84, 0.82)),
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
                (*kind, materials.add(materials::preview_model_material(source)))
            })
            .collect();

        let factory_models = factory::load_factory_visuals(meshes, materials, images);
        let light_panel = factory::load_light_panel_mesh(meshes, materials, images);

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
            // 漆：零厚度面片（+Y 法线），spawn 时按附着法线旋转
            face_mark: meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(0.49))),
            light_panel,
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
            part_drill_tip: meshes.add(meshes::drill_tip_mesh(0.34, 1.0, 48)),
            part_large: meshes.add(Cuboid::new(0.72, 0.22, 0.72)),
            part_medium: meshes.add(Cuboid::new(0.44, 0.20, 0.44)),
            part_small: meshes.add(Cuboid::new(0.22, 0.22, 0.22)),
            part_plate: meshes.add(Cuboid::new(0.78, 0.06, 0.78)),
            // 告示竖板：薄在 Z，大面朝 ±Z（局部 +Z 贴宿主）
            part_sign_board: meshes.add(Cuboid::new(0.78, 0.72, 0.06)),
            part_rotator_base: meshes.add(Cuboid::new(1.0, 0.80, 1.0)),
            part_rotator_disk: meshes.add(Cylinder::new(0.40, 0.20).mesh().resolution(48)),
            part_rotator_ring: meshes.add(meshes::rotator_ring_mesh(0.50, 0.40, 0.20, 64)),
            part_rod_x: meshes.add(Cuboid::new(0.72, 0.12, 0.12)),
            part_rod_y: meshes.add(Cuboid::new(0.12, 0.72, 0.12)),
            part_rod_z: meshes.add(Cuboid::new(0.12, 0.12, 0.72)),
            // 镜子面片：000, 101, 111, 010
            part_mirror_face: meshes.add(meshes::thick_quad_mesh([
                [0, 0, 0],
                [1, 0, 1],
                [1, 1, 1],
                [0, 1, 0],
            ])),
            // 垂直镜子面片：000, 001, 111, 110
            part_vertical_mirror_face: meshes.add(meshes::thick_quad_mesh([
                [0, 0, 0],
                [0, 0, 1],
                [1, 1, 1],
                [1, 1, 0],
            ])),
            // 分光镜：x+y+z=0 六边形再烘焙 -180° yaw
            part_splitter_face: meshes.add(meshes::thick_splitter_hexagon_mesh()),
            // 吸盘：工作面在 -Z，顶点在格子中心
            part_suction_cup: meshes.add(meshes::suction_cup_pyramid_mesh()),
            part_pusher_body: meshes.add(meshes::cover_cuboid_mesh(Vec3::new(1.0, 1.0, 0.80))),
            part_pusher_head: meshes.add(meshes::cover_cuboid_mesh(Vec3::new(1.0, 1.0, 0.20))),
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
            // 未通电：黑；通电：白 + 轻度自发光（须 lit，Bloom 阈值约 10）；bias 用默认 0
            light_panel_material: materials.add(StandardMaterial {
                base_color: Color::BLACK,
                unlit: true,
                cull_mode: None,
                ..default()
            }),
            light_panel_lit_material: materials.add(StandardMaterial {
                base_color: Color::WHITE,
                emissive: LinearRgba::new(12.0, 12.0, 12.0, 1.0),
                perceptual_roughness: 1.0,
                metallic: 0.0,
                cull_mode: None,
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

    pub(crate) fn light_panel_mesh(&self) -> Handle<Mesh> {
        self.light_panel.clone()
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
