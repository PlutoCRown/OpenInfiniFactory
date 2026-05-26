use bevy::prelude::*;

use crate::game::world::blocks::{BlockKind, BLOCK_SIZE};
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
    solid: Handle<StandardMaterial>,
    grass: Handle<StandardMaterial>,
    stone: Handle<StandardMaterial>,
    dirt: Handle<StandardMaterial>,
    planks: Handle<StandardMaterial>,
    glass: Handle<StandardMaterial>,
    generator: Handle<StandardMaterial>,
    welder: Handle<StandardMaterial>,
    conveyor: Handle<StandardMaterial>,
    detector: Handle<StandardMaterial>,
    wire: Handle<StandardMaterial>,
    piston: Handle<StandardMaterial>,
    lifter: Handle<StandardMaterial>,
    rotator: Handle<StandardMaterial>,
    blocker: Handle<StandardMaterial>,
    drill: Handle<StandardMaterial>,
    laser: Handle<StandardMaterial>,
    goal: Handle<StandardMaterial>,
    material: Handle<StandardMaterial>,
    weld_point_material: Handle<StandardMaterial>,
    blocker_head: Handle<StandardMaterial>,
    drill_head: Handle<StandardMaterial>,
    pub(crate) wire_connector_material: Handle<StandardMaterial>,
    pub(crate) arrow_material: Handle<StandardMaterial>,
    pub(crate) arrow_nose_material: Handle<StandardMaterial>,
    pub(crate) goal_top_material: Handle<StandardMaterial>,
    pub(crate) weld_connector_material: Handle<StandardMaterial>,
    place_preview_material: Handle<StandardMaterial>,
    delete_preview_material: Handle<StandardMaterial>,
    selection_preview_material: Handle<StandardMaterial>,
}

pub enum EditPreviewKind {
    Place,
    Delete,
    Selection,
}

impl WorldRenderAssets {
    pub(crate) fn new(
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
        images: &mut Assets<Image>,
    ) -> Self {
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
            solid: materials.add(block_material(BlockKind::Solid)),
            grass: materials.add(textured_block_material(
                BlockKind::Grass,
                images.add(block_texture(ProceduralTexture::Grass)),
            )),
            stone: materials.add(textured_block_material(
                BlockKind::Stone,
                images.add(block_texture(ProceduralTexture::Stone)),
            )),
            dirt: materials.add(textured_block_material(
                BlockKind::Dirt,
                images.add(block_texture(ProceduralTexture::Dirt)),
            )),
            planks: materials.add(textured_block_material(
                BlockKind::Planks,
                images.add(block_texture(ProceduralTexture::Planks)),
            )),
            glass: materials.add(block_material(BlockKind::Glass)),
            generator: materials.add(block_material(BlockKind::Generator)),
            welder: materials.add(block_material(BlockKind::Welder)),
            conveyor: materials.add(block_material(BlockKind::Conveyor)),
            detector: materials.add(block_material(BlockKind::Detector)),
            wire: materials.add(block_material(BlockKind::Wire)),
            piston: materials.add(block_material(BlockKind::Piston)),
            lifter: materials.add(block_material(BlockKind::Lifter)),
            rotator: materials.add(block_material(BlockKind::Rotator)),
            blocker: materials.add(block_material(BlockKind::Blocker)),
            drill: materials.add(block_material(BlockKind::Drill)),
            laser: materials.add(block_material(BlockKind::Laser)),
            goal: materials.add(block_material(BlockKind::Goal)),
            material: materials.add(block_material(BlockKind::Material)),
            weld_point_material: materials.add(block_material(BlockKind::WeldPoint)),
            blocker_head: materials.add(block_material(BlockKind::BlockerHead)),
            drill_head: materials.add(block_material(BlockKind::DrillHead)),
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
            place_preview_material: materials.add(StandardMaterial {
                base_color: Color::srgba(0.55, 0.92, 1.0, 0.36),
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
        if matches!(
            kind,
            BlockKind::WeldPoint | BlockKind::Wire | BlockKind::DrillHead
        ) {
            self.node.clone()
        } else {
            self.block.clone()
        }
    }

    pub(crate) fn block_material(&self, kind: BlockKind) -> Handle<StandardMaterial> {
        match kind {
            BlockKind::Solid => self.solid.clone(),
            BlockKind::Grass => self.grass.clone(),
            BlockKind::Stone => self.stone.clone(),
            BlockKind::Dirt => self.dirt.clone(),
            BlockKind::Planks => self.planks.clone(),
            BlockKind::Glass => self.glass.clone(),
            BlockKind::Generator => self.generator.clone(),
            BlockKind::Welder => self.welder.clone(),
            BlockKind::Conveyor => self.conveyor.clone(),
            BlockKind::Detector => self.detector.clone(),
            BlockKind::Wire => self.wire.clone(),
            BlockKind::Piston => self.piston.clone(),
            BlockKind::Lifter => self.lifter.clone(),
            BlockKind::Rotator => self.rotator.clone(),
            BlockKind::Blocker => self.blocker.clone(),
            BlockKind::Drill => self.drill.clone(),
            BlockKind::Laser => self.laser.clone(),
            BlockKind::Goal => self.goal.clone(),
            BlockKind::Material => self.material.clone(),
            BlockKind::WeldPoint => self.weld_point_material.clone(),
            BlockKind::BlockerHead => self.blocker_head.clone(),
            BlockKind::DrillHead => self.drill_head.clone(),
        }
    }

    pub(crate) fn edit_preview_material(&self, kind: EditPreviewKind) -> Handle<StandardMaterial> {
        match kind {
            EditPreviewKind::Place => self.place_preview_material.clone(),
            EditPreviewKind::Delete => self.delete_preview_material.clone(),
            EditPreviewKind::Selection => self.selection_preview_material.clone(),
        }
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
    if matches!(
        kind,
        BlockKind::Glass | BlockKind::WeldPoint | BlockKind::DrillHead
    ) {
        material.alpha_mode = AlphaMode::Blend;
        material.unlit = matches!(kind, BlockKind::WeldPoint | BlockKind::DrillHead);
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
