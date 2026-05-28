use std::collections::HashMap;

use bevy::prelude::*;

use crate::game::world::blocks::{BlockKind, BlockShape, ALL_BLOCKS, BLOCK_SIZE};
use crate::game::world::procedural_textures::{block_texture, ProceduralTexture};

#[derive(Resource, Clone)]
pub struct WorldRenderAssets {
    pub(crate) block: Handle<Mesh>,
    node: Handle<Mesh>,
    pub(crate) arrow: Handle<Mesh>,
    pub(crate) arrow_nose: Handle<Mesh>,
    pub(crate) goal_top: Handle<Mesh>,
    connector_x: Handle<Mesh>,
    connector_y: Handle<Mesh>,
    connector_z: Handle<Mesh>,
    block_materials: HashMap<BlockKind, Handle<StandardMaterial>>,
    preview_materials: HashMap<BlockKind, Handle<StandardMaterial>>,
    pub(crate) wire_connector_material: Handle<StandardMaterial>,
    pub(crate) arrow_material: Handle<StandardMaterial>,
    pub(crate) arrow_nose_material: Handle<StandardMaterial>,
    pub(crate) goal_top_material: Handle<StandardMaterial>,
    pub(crate) weld_connector_material: Handle<StandardMaterial>,
    delete_preview_material: Handle<StandardMaterial>,
    selection_preview_material: Handle<StandardMaterial>,
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
    ) -> Self {
        let grass_texture = images.add(block_texture(ProceduralTexture::Grass));
        let stone_texture = images.add(block_texture(ProceduralTexture::Stone));
        let dirt_texture = images.add(block_texture(ProceduralTexture::Dirt));
        let planks_texture = images.add(block_texture(ProceduralTexture::Planks));
        let textures = [
            (BlockKind::Grass, grass_texture.clone()),
            (BlockKind::Stone, stone_texture.clone()),
            (BlockKind::Dirt, dirt_texture.clone()),
            (BlockKind::Planks, planks_texture.clone()),
        ];
        let block_materials = ALL_BLOCKS
            .into_iter()
            .map(|kind| {
                let texture = textures
                    .iter()
                    .find_map(|(texture_kind, texture)| {
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
                let texture = textures
                    .iter()
                    .find_map(|(texture_kind, texture)| {
                        (*texture_kind == kind).then_some(texture.clone())
                    });
                (kind, materials.add(preview_block_material(kind, texture)))
            })
            .collect();

        Self {
            block: meshes.add(Cuboid::new(BLOCK_SIZE, BLOCK_SIZE, BLOCK_SIZE)),
            node: meshes.add(Cuboid::new(
                BLOCK_SIZE * 0.38,
                BLOCK_SIZE * 0.38,
                BLOCK_SIZE * 0.38,
            )),
            arrow: meshes.add(Cuboid::new(0.18, 0.08, 0.72)),
            arrow_nose: meshes.add(Cuboid::new(0.42, 0.10, 0.18)),
            goal_top: meshes.add(Cuboid::new(0.62, 0.08, 0.62)),
            connector_x: meshes.add(Cuboid::new(0.74, 0.10, 0.10)),
            connector_y: meshes.add(Cuboid::new(0.10, 0.74, 0.10)),
            connector_z: meshes.add(Cuboid::new(0.10, 0.10, 0.74)),
            block_materials,
            preview_materials,
            wire_connector_material: materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 0.88, 0.30),
                emissive: Color::srgb(0.20, 0.12, 0.02).into(),
                ..default()
            }),
            arrow_material: materials.add(StandardMaterial {
                base_color: Color::srgb(0.95, 0.95, 0.38),
                unlit: true,
                ..default()
            }),
            arrow_nose_material: materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 0.78, 0.25),
                unlit: true,
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
        }
    }

    pub(crate) fn block_mesh(&self, kind: BlockKind) -> Handle<Mesh> {
        match kind.shape() {
            BlockShape::Cube => self.block.clone(),
            BlockShape::Node => self.node.clone(),
        }
    }

    pub(crate) fn block_material(&self, kind: BlockKind) -> Handle<StandardMaterial> {
        self.block_materials
            .get(&kind)
            .expect("every block kind has a material")
            .clone()
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

    pub(crate) fn connector_mesh(&self, offset: IVec3) -> Handle<Mesh> {
        if offset.x != 0 {
            self.connector_x.clone()
        } else if offset.y != 0 {
            self.connector_y.clone()
        } else {
            self.connector_z.clone()
        }
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
