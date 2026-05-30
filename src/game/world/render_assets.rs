use std::collections::HashMap;

use bevy::prelude::*;

use crate::game::world::blocks::{
    BlockKind, BlockShape, ModelMaterial, ModelMesh, StampColor, ALL_BLOCKS, BLOCK_SIZE,
};
use crate::game::world::procedural_textures::{block_texture, ProceduralTexture};
use crate::game::world::scene_material::SceneBlockMaterial;

#[derive(Resource, Clone)]
pub struct WorldRenderAssets {
    pub(crate) block: Handle<Mesh>,
    node: Handle<Mesh>,
    wire_node: Handle<Mesh>,
    pub(crate) goal_top: Handle<Mesh>,
    pub(crate) face_mark: Handle<Mesh>,
    pub(crate) weld_spark: Handle<Mesh>,
    connector_x: Handle<Mesh>,
    connector_y: Handle<Mesh>,
    connector_z: Handle<Mesh>,
    wire_connector_x: Handle<Mesh>,
    wire_connector_y: Handle<Mesh>,
    wire_connector_z: Handle<Mesh>,
    part_large: Handle<Mesh>,
    part_medium: Handle<Mesh>,
    part_small: Handle<Mesh>,
    part_plate: Handle<Mesh>,
    part_rod_x: Handle<Mesh>,
    part_rod_y: Handle<Mesh>,
    part_rod_z: Handle<Mesh>,
    part_pusher_body: Handle<Mesh>,
    part_pusher_head: Handle<Mesh>,
    block_materials: HashMap<BlockKind, Handle<StandardMaterial>>,
    preview_materials: HashMap<BlockKind, Handle<StandardMaterial>>,
    scene_materials: HashMap<BlockKind, Handle<SceneBlockMaterial>>,
    face_mark_materials: HashMap<StampColor, Handle<StandardMaterial>>,
    model_materials: HashMap<ModelMaterial, Handle<StandardMaterial>>,
    pub(crate) wire_connector_material: Handle<StandardMaterial>,
    pub(crate) active_wire_material: Handle<StandardMaterial>,
    pub(crate) goal_top_material: Handle<StandardMaterial>,
    pub(crate) weld_connector_material: Handle<StandardMaterial>,
    delete_preview_material: Handle<StandardMaterial>,
    selection_preview_material: Handle<StandardMaterial>,
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
        scene_materials: &mut Assets<SceneBlockMaterial>,
        images: &mut Assets<Image>,
    ) -> Self {
        let material_texture = images.add(block_texture(ProceduralTexture::Material));
        let iron_texture = images.add(block_texture(ProceduralTexture::IronMaterial));
        let copper_texture = images.add(block_texture(ProceduralTexture::CopperMaterial));
        let textures = [
            (BlockKind::Material, material_texture.clone()),
            (BlockKind::IronMaterial, iron_texture.clone()),
            (BlockKind::CopperMaterial, copper_texture.clone()),
        ];
        let block_materials = ALL_BLOCKS
            .into_iter()
            .map(|kind| {
                let texture = textures.iter().find_map(|(texture_kind, texture)| {
                    (*texture_kind == kind).then_some(texture.clone())
                });
                let material = texture
                    .map(|texture| textured_block_material(kind, texture))
                    .unwrap_or_else(|| block_material(kind));
                (kind, materials.add(material))
            })
            .collect();
        let preview_materials = ALL_BLOCKS
            .into_iter()
            .map(|kind| {
                let texture = textures.iter().find_map(|(texture_kind, texture)| {
                    (*texture_kind == kind).then_some(texture.clone())
                });
                (kind, materials.add(preview_block_material(kind, texture)))
            })
            .collect();
        let scene_block_materials = [
            (
                BlockKind::Grass,
                scene_block_material(
                    Color::srgb(0.28, 0.55, 0.20),
                    Color::srgb(0.46, 0.72, 0.28),
                    0,
                ),
            ),
            (
                BlockKind::Stone,
                scene_block_material(
                    Color::srgb(0.42, 0.43, 0.42),
                    Color::srgb(0.64, 0.65, 0.62),
                    1,
                ),
            ),
            (
                BlockKind::Dirt,
                scene_block_material(
                    Color::srgb(0.34, 0.22, 0.13),
                    Color::srgb(0.54, 0.36, 0.20),
                    2,
                ),
            ),
            (
                BlockKind::Planks,
                scene_block_material(
                    Color::srgb(0.54, 0.32, 0.13),
                    Color::srgb(0.82, 0.55, 0.28),
                    3,
                ),
            ),
        ]
        .into_iter()
        .map(|(kind, material)| (kind, scene_materials.add(material)))
        .collect();
        let face_mark_materials = StampColor::ALL
            .into_iter()
            .map(|color| {
                (
                    color,
                    materials.add(StandardMaterial {
                        base_color: color.color().with_alpha(0.82),
                        alpha_mode: AlphaMode::Blend,
                        unlit: true,
                        ..default()
                    }),
                )
            })
            .collect();
        let model_materials = [
            (ModelMaterial::Frame, srgb_material(0.42, 0.44, 0.44)),
            (ModelMaterial::DarkFrame, srgb_material(0.12, 0.13, 0.15)),
            (ModelMaterial::Belt, srgb_material(0.86, 0.46, 0.14)),
            (
                ModelMaterial::BeltStripe,
                emissive_material(1.0, 0.76, 0.28, 0.18, 0.10, 0.02),
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
            (ModelMaterial::Pusher, srgb_material(0.54, 0.56, 0.54)),
            (ModelMaterial::Wood, srgb_material(0.72, 0.46, 0.22)),
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
            (ModelMaterial::System, srgb_material(0.35, 0.28, 0.48)),
            (
                ModelMaterial::SystemAccent,
                emissive_material(0.72, 0.58, 1.0, 0.12, 0.08, 0.24),
            ),
            (
                ModelMaterial::Goal,
                emissive_material(0.55, 1.0, 0.36, 0.05, 0.22, 0.04),
            ),
            (
                ModelMaterial::TeleportIn,
                emissive_material(0.18, 0.62, 1.0, 0.02, 0.10, 0.34),
            ),
            (
                ModelMaterial::TeleportOut,
                emissive_material(1.0, 0.54, 0.18, 0.34, 0.10, 0.02),
            ),
        ]
        .into_iter()
        .map(|(kind, material)| (kind, materials.add(material)))
        .collect();

        Self {
            block: meshes.add(Cuboid::new(BLOCK_SIZE, BLOCK_SIZE, BLOCK_SIZE)),
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
            goal_top: meshes.add(Cuboid::new(0.62, 0.08, 0.62)),
            face_mark: meshes.add(Cuboid::new(0.72, 0.012, 0.72)),
            weld_spark: meshes.add(Cuboid::new(0.24, 0.24, 0.24)),
            connector_x: meshes.add(Cuboid::new(0.74, 0.10, 0.10)),
            connector_y: meshes.add(Cuboid::new(0.10, 0.74, 0.10)),
            connector_z: meshes.add(Cuboid::new(0.10, 0.10, 0.74)),
            wire_connector_x: meshes.add(Cuboid::new(0.74, 0.304, 0.304)),
            wire_connector_y: meshes.add(Cuboid::new(0.304, 0.74, 0.304)),
            wire_connector_z: meshes.add(Cuboid::new(0.304, 0.304, 0.74)),
            part_large: meshes.add(Cuboid::new(0.72, 0.22, 0.72)),
            part_medium: meshes.add(Cuboid::new(0.44, 0.20, 0.44)),
            part_small: meshes.add(Cuboid::new(0.22, 0.22, 0.22)),
            part_plate: meshes.add(Cuboid::new(0.78, 0.06, 0.78)),
            part_rod_x: meshes.add(Cuboid::new(0.72, 0.12, 0.12)),
            part_rod_y: meshes.add(Cuboid::new(0.12, 0.72, 0.12)),
            part_rod_z: meshes.add(Cuboid::new(0.12, 0.12, 0.72)),
            part_pusher_body: meshes.add(Cuboid::new(0.80, 0.80, 0.80)),
            part_pusher_head: meshes.add(Cuboid::new(0.82, 0.82, 0.20)),
            block_materials,
            preview_materials,
            scene_materials: scene_block_materials,
            face_mark_materials,
            model_materials,
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
            goal_top_material: materials.add(StandardMaterial {
                base_color: Color::srgb(0.75, 1.0, 0.55),
                emissive: Color::srgb(0.12, 0.28, 0.08).into(),
                ..default()
            }),
            weld_connector_material: materials.add(StandardMaterial {
                base_color: Color::srgba(1.0, 0.22, 0.10, 0.72),
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
            selection_preview_material: materials.add(StandardMaterial {
                base_color: Color::srgba(0.25, 0.95, 0.88, 0.34),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            }),
            active_factory_debug_material: materials.add(StandardMaterial {
                base_color: Color::srgb(0.12, 0.90, 0.22),
                unlit: true,
                ..default()
            }),
            inactive_factory_debug_material: materials.add(StandardMaterial {
                base_color: Color::srgb(0.95, 0.12, 0.08),
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

    pub(crate) fn active_factory_debug_material(&self) -> Handle<StandardMaterial> {
        self.active_factory_debug_material.clone()
    }

    pub(crate) fn inactive_factory_debug_material(&self) -> Handle<StandardMaterial> {
        self.inactive_factory_debug_material.clone()
    }

    pub(crate) fn edit_preview_material(&self, kind: EditPreviewKind) -> Handle<StandardMaterial> {
        match kind {
            EditPreviewKind::Delete => self.delete_preview_material.clone(),
            EditPreviewKind::Selection => self.selection_preview_material.clone(),
        }
    }

    pub(crate) fn block_preview_material(&self, kind: BlockKind) -> Handle<StandardMaterial> {
        self.preview_materials
            .get(&kind)
            .expect("every block kind has a preview material")
            .clone()
    }

    pub(crate) fn scene_material(&self, kind: BlockKind) -> Option<Handle<SceneBlockMaterial>> {
        self.scene_materials.get(&kind).cloned()
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

    pub(crate) fn face_mark_material(&self, color: StampColor) -> Handle<StandardMaterial> {
        self.face_mark_materials
            .get(&color)
            .expect("every stamp color has a material")
            .clone()
    }

    pub(crate) fn model_mesh(&self, mesh: ModelMesh) -> Handle<Mesh> {
        match mesh {
            ModelMesh::Large => self.part_large.clone(),
            ModelMesh::Medium => self.part_medium.clone(),
            ModelMesh::Small => self.part_small.clone(),
            ModelMesh::Plate => self.part_plate.clone(),
            ModelMesh::RodX => self.part_rod_x.clone(),
            ModelMesh::RodY => self.part_rod_y.clone(),
            ModelMesh::RodZ => self.part_rod_z.clone(),
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

fn preview_block_material(kind: BlockKind, texture: Option<Handle<Image>>) -> StandardMaterial {
    StandardMaterial {
        base_color: kind.material().with_alpha(0.46),
        base_color_texture: texture,
        alpha_mode: AlphaMode::Blend,
        perceptual_roughness: 0.94,
        reflectance: 0.08,
        ..default()
    }
}

fn scene_block_material(
    base_color: Color,
    accent_color: Color,
    texture_kind: u32,
) -> SceneBlockMaterial {
    SceneBlockMaterial {
        base_color: base_color.to_linear(),
        accent_color: accent_color.to_linear(),
        texture_kind,
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

fn emissive_material(r: f32, g: f32, b: f32, er: f32, eg: f32, eb: f32) -> StandardMaterial {
    StandardMaterial {
        base_color: Color::srgb(r, g, b),
        emissive: Color::srgb(er, eg, eb).into(),
        perceptual_roughness: 0.72,
        reflectance: 0.10,
        ..default()
    }
}
