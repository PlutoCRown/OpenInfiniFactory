//! 世界渲染资源：网格、材质与工厂/场景外观

mod factory;
mod materials;
mod packs;

use std::collections::{HashMap, HashSet};

use bevy::prelude::*;

use crate::game::blocks::{
    BLOCK_SIZE, BlockKind, BlockPresent, BlockShape, PaintMaterialId, all_blocks, paint_catalog,
    stamp_catalog, stamp_def,
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
    part_sign_board: Handle<Mesh>,
    part_sign_pole: Handle<Mesh>,
    /// 告示正面 icon 四边形（XY，法线 +Z）
    part_sign_icon: Handle<Mesh>,
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
    /// 传送门流动材质（模板；实体 spawn 后会 uniquify）
    portal_material: Handle<crate::game::world::rendering::PortalMaterial>,
    /// 传送门立方体网格
    portal_cube_mesh: Handle<Mesh>,
    face_mark_materials: HashMap<PaintMaterialId, Handle<StandardMaterial>>,
    /// 告示牌正面展示用的方块 icon 材质（depth_bias = PAINT）
    sign_display_materials: HashMap<BlockKind, Handle<StandardMaterial>>,
    /// 灯面板未通电材质
    pub(crate) light_panel_material: Handle<StandardMaterial>,
    /// 灯面板通电发光材质
    pub(crate) light_panel_lit_material: Handle<StandardMaterial>,
    /// 告示牌木材材质
    sign_wood_material: Handle<StandardMaterial>,
    /// 告示牌木材预览材质
    sign_wood_preview_material: Handle<StandardMaterial>,
    pub(crate) wire_connector_material: Handle<StandardMaterial>,
    pub(crate) active_wire_material: Handle<StandardMaterial>,
    /// 抬升器顶盘：模拟期微弱白自发光（通电关闭时切回 GLB 原材质）
    pub(crate) lifter_disk_lit_material: Handle<StandardMaterial>,
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
        // 告示等用浅色白桦纹（勿用深色 oak 材料贴图）
        let wood_texture = images.add(crate::game::world::procedural_textures::from_fn(
            crate::game::world::procedural_textures::birch_wood_pixel,
        ));
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
        let sign_wood_source = materials::textured_model_material(Color::WHITE, wood_texture);
        let sign_wood_material = materials.add(sign_wood_source.clone());
        let sign_wood_preview_material =
            materials.add(materials::preview_model_material(sign_wood_source));

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
            // 告示竖板：宽 1、高 0.6、薄在 Z（局部 +Z 贴宿主）
            part_sign_board: meshes.add(Cuboid::new(1.0, 0.6, 0.05)),
            part_sign_pole: meshes.add(Cuboid::new(0.1, 0.5, 0.1)),
            part_sign_icon: meshes.add(Rectangle::new(1.0, 1.0)),
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
            portal_material: Handle::default(),
            portal_cube_mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
            face_mark_materials,
            sign_display_materials: HashMap::new(),
            // 未通电：黑；通电：白 + 轻度自发光（须 lit，Bloom 阈值约 5）；bias 用默认 0
            light_panel_material: materials.add(StandardMaterial {
                base_color: Color::BLACK,
                unlit: true,
                cull_mode: None,
                ..default()
            }),
            light_panel_lit_material: materials.add(StandardMaterial {
                base_color: Color::WHITE,
                emissive: LinearRgba::new(6.0, 6.0, 6.0, 1.0),
                perceptual_roughness: 1.0,
                metallic: 0.0,
                cull_mode: None,
                ..default()
            }),
            sign_wood_material,
            sign_wood_preview_material,
            wire_connector_material: materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 0.88, 0.30),
                emissive: Color::srgb(0.10, 0.06, 0.01).into(),
                ..default()
            }),
            active_wire_material: materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 0.08, 0.04),
                emissive: Color::srgb(0.17, 0.01, 0.005).into(),
                ..default()
            }),
            // 抬升顶盘：弱白自发光（须 lit，强度低于电线通电条）
            lifter_disk_lit_material: materials.add(StandardMaterial {
                base_color: Color::WHITE,
                emissive: LinearRgba::new(2.2, 2.2, 2.2, 1.0),
                perceptual_roughness: 0.5,
                metallic: 0.08,
                ..default()
            }),
            // 白杆 + 黄自发光（须 lit：unlit 会丢掉 emissive，Bloom 看不到）
            weld_connector_material: materials.add(StandardMaterial {
                base_color: Color::WHITE,
                emissive: LinearRgba::new(11.0, 7.0, 0.3, 1.0),
                perceptual_roughness: 1.0,
                metallic: 0.0,
                ..default()
            }),
            weld_burst_material: materials.add(StandardMaterial {
                base_color: Color::WHITE,
                emissive: LinearRgba::new(11.0, 7.0, 0.3, 1.0),
                perceptual_roughness: 1.0,
                metallic: 0.0,
                cull_mode: None,
                ..default()
            }),
            laser_beam_material: materials.add(StandardMaterial {
                base_color: Color::srgba(1.0, 0.12, 0.26, 0.92),
                emissive: LinearRgba::new(0.275, 0.01, 0.05, 1.0),
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

    /// 安装传送门流动材质
    pub(crate) fn install_portal_material(
        &mut self,
        portal_materials: &mut Assets<crate::game::world::rendering::PortalMaterial>,
    ) {
        use crate::game::world::rendering::portal_material::default_portal_material;
        self.portal_material = portal_materials.add(default_portal_material());
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

    pub(crate) fn portal_material_handle(
        &self,
    ) -> Handle<crate::game::world::rendering::PortalMaterial> {
        self.portal_material.clone()
    }

    pub(crate) fn portal_cube_mesh(&self) -> Handle<Mesh> {
        self.portal_cube_mesh.clone()
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

    /// 告示正面 icon 网格（单位正方形）
    pub(crate) fn sign_icon_mesh(&self) -> Handle<Mesh> {
        self.part_sign_icon.clone()
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

    /// 告示牌正面 icon 材质（未烘焙则无）
    pub(crate) fn sign_display_material(
        &self,
        kind: BlockKind,
    ) -> Option<Handle<StandardMaterial>> {
        self.sign_display_materials.get(&kind).cloned()
    }

    /// 写入告示展示 icon 材质（由 setup_block_icons 填充）
    pub(crate) fn insert_sign_display_material(
        &mut self,
        kind: BlockKind,
        material: Handle<StandardMaterial>,
    ) {
        self.sign_display_materials.insert(kind, material);
    }

    pub(crate) fn sign_mesh(&self, mesh: crate::game::blocks::SignMesh) -> Handle<Mesh> {
        match mesh {
            crate::game::blocks::SignMesh::Board => self.part_sign_board.clone(),
            crate::game::blocks::SignMesh::Pole => self.part_sign_pole.clone(),
        }
    }
    pub(crate) fn sign_wood_material(&self) -> Handle<StandardMaterial> {
        self.sign_wood_material.clone()
    }
    pub(crate) fn sign_wood_preview_material(&self) -> Handle<StandardMaterial> {
        self.sign_wood_preview_material.clone()
    }
}
