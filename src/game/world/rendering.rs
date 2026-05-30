use bevy::light::CascadeShadowConfigBuilder;
use bevy::prelude::*;
use std::collections::HashMap;

use crate::game::simulation::factory_activity::{
    FactoryActivity, FactoryStructureState, StructureKind,
};
use crate::game::systems::debug::DebugState;
use crate::game::world::animation::{
    AnimatedBlock, AnimatedPiston, AnimationEasing, AnimationTiming, BlockAnimation,
    BlockAnimationKind, PistonAnimation,
};
use crate::game::world::blocks::{
    BlockData, BlockModel, WeldConnectorBehavior, WireConnectorBehavior,
};
use crate::game::world::grid::{grid_to_world, WorldBlocks};
pub use crate::game::world::render_assets::{EditPreviewKind, WorldRenderAssets};
use crate::game::world::scene_material::SceneBlockMaterial;

#[derive(Component)]
pub struct BlockEntity {
    pub pos: IVec3,
}

#[derive(Component)]
pub struct HoverMarker;

#[derive(Resource, Default, Clone, Copy)]
pub struct HoverStructureBounds {
    pub bounds: Option<StructureBounds>,
}

#[derive(Clone, Copy)]
pub struct StructureBounds {
    pub kind: StructureKind,
    pub min: IVec3,
    pub max: IVec3,
}

#[derive(Component)]
pub struct PlacementPreview;

#[derive(Component)]
pub struct EditPreview;

#[derive(Component)]
pub struct PendingGeneratedPreview;

pub fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut scene_materials: ResMut<Assets<SceneBlockMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    commands.spawn((
        PointLight {
            intensity: 1100.0,
            shadows_enabled: true,
            range: 18.0,
            radius: 3.5,
            ..default()
        },
        Transform::from_xyz(3.5, 5.5, 4.5),
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 9500.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -1.05, -0.55, -0.28)),
        CascadeShadowConfigBuilder {
            num_cascades: 3,
            minimum_distance: 0.15,
            maximum_distance: 48.0,
            first_cascade_far_bound: 8.0,
            overlap_proportion: 0.16,
        }
        .build(),
    ));

    let render_assets = WorldRenderAssets::new(
        &mut meshes,
        &mut materials,
        &mut scene_materials,
        &mut images,
    );
    commands.insert_resource(render_assets);

    let marker_mesh = meshes.add(Cuboid::new(1.04, 1.04, 1.04));
    let marker_material = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 1.0, 1.0, 0.16),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });

    commands.spawn((
        Mesh3d(marker_mesh),
        MeshMaterial3d(marker_material),
        Visibility::Hidden,
        HoverMarker,
    ));

    let preview_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let preview_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.7, 0.95, 1.0, 0.34),
        alpha_mode: AlphaMode::Blend,
        perceptual_roughness: 0.92,
        reflectance: 0.0,
        ..default()
    });

    commands.spawn((
        Mesh3d(preview_mesh),
        MeshMaterial3d(preview_material),
        Visibility::Hidden,
        PlacementPreview,
    ));
}

pub fn rebuild_world(commands: &mut Commands, world: &WorldBlocks, assets: &WorldRenderAssets) {
    for (pos, data) in &world.blocks {
        spawn_block(commands, assets, world, *pos, *data);
    }
    for (pos, data) in &world.system_blocks {
        spawn_block(commands, assets, world, *pos, *data);
    }
}

pub fn rebuild_world_with_factory_activity_debug(
    commands: &mut Commands,
    world: &WorldBlocks,
    assets: &WorldRenderAssets,
    factory_structures: &FactoryStructureState,
) {
    for (pos, data) in &world.blocks {
        if data.kind.is_factory() {
            let material = match factory_structures.activity_at(*pos) {
                Some(FactoryActivity::Active) => assets.active_factory_debug_material(),
                _ => assets.inactive_factory_debug_material(),
            };
            spawn_debug_factory_block(commands, assets, *pos, material);
        } else {
            spawn_block(commands, assets, world, *pos, *data);
        }
    }
    for (pos, data) in &world.system_blocks {
        spawn_block(commands, assets, world, *pos, *data);
    }
}

pub fn rebuild_world_for_debug_state(
    commands: &mut Commands,
    world: &WorldBlocks,
    assets: &WorldRenderAssets,
    debug: &DebugState,
    factory_structures: &FactoryStructureState,
) {
    if debug.factory_activity {
        rebuild_world_with_factory_activity_debug(commands, world, assets, factory_structures);
    } else {
        rebuild_world(commands, world, assets);
    }
}

pub fn despawn_world(commands: &mut Commands, block_entities: &Query<Entity, With<BlockEntity>>) {
    for entity in block_entities {
        commands.entity(entity).despawn();
    }
}

fn spawn_debug_factory_block(
    commands: &mut Commands,
    assets: &WorldRenderAssets,
    pos: IVec3,
    material: Handle<StandardMaterial>,
) {
    commands.spawn((
        Mesh3d(assets.block.clone()),
        MeshMaterial3d(material),
        Transform::from_translation(grid_to_world(pos)),
        BlockEntity { pos },
    ));
}

pub fn despawn_edit_previews(commands: &mut Commands, previews: &Query<Entity, With<EditPreview>>) {
    for entity in previews {
        commands.entity(entity).despawn();
    }
}

pub fn despawn_pending_generated_previews(
    commands: &mut Commands,
    previews: &Query<Entity, With<PendingGeneratedPreview>>,
) {
    for entity in previews {
        commands.entity(entity).despawn();
    }
}

pub fn spawn_edit_preview(
    commands: &mut Commands,
    assets: &WorldRenderAssets,
    pos: IVec3,
    kind: EditPreviewKind,
) {
    commands.spawn((
        Mesh3d(assets.block.clone()),
        MeshMaterial3d(assets.edit_preview_material(kind)),
        Transform::from_translation(grid_to_world(pos)),
        EditPreview,
    ));
}

pub fn spawn_block_preview(
    commands: &mut Commands,
    assets: &WorldRenderAssets,
    world: &WorldBlocks,
    pos: IVec3,
    data: BlockData,
) {
    spawn_block_model(
        commands,
        assets,
        world,
        pos,
        data,
        assets.block_preview_material(data.kind),
        Some(EditPreview),
        None,
        None,
        AnimationTiming::edit(),
        false,
        false,
    );
}

pub fn spawn_block(
    commands: &mut Commands,
    assets: &WorldRenderAssets,
    world: &WorldBlocks,
    pos: IVec3,
    data: BlockData,
) {
    spawn_block_with_animation(commands, assets, world, pos, data, None);
}

pub fn spawn_block_with_animation(
    commands: &mut Commands,
    assets: &WorldRenderAssets,
    world: &WorldBlocks,
    pos: IVec3,
    data: BlockData,
    animation: Option<BlockAnimation>,
) {
    spawn_block_with_timed_animation(
        commands,
        assets,
        world,
        pos,
        data,
        animation,
        AnimationTiming::edit(),
    );
}

pub fn spawn_block_with_timed_animation(
    commands: &mut Commands,
    assets: &WorldRenderAssets,
    world: &WorldBlocks,
    pos: IVec3,
    data: BlockData,
    animation: Option<BlockAnimation>,
    timing: AnimationTiming,
) {
    spawn_block_model(
        commands,
        assets,
        world,
        pos,
        data,
        assets.block_material(data.kind),
        None,
        animation,
        None,
        timing,
        true,
        false,
    );
}

pub fn spawn_pending_generated_block(
    commands: &mut Commands,
    assets: &WorldRenderAssets,
    world: &WorldBlocks,
    pos: IVec3,
    data: BlockData,
    animation: Option<BlockAnimation>,
    timing: AnimationTiming,
) {
    spawn_block_model(
        commands,
        assets,
        world,
        pos,
        data,
        assets.block_material(data.kind),
        None,
        animation,
        None,
        timing,
        false,
        true,
    );
}

pub fn rebuild_world_with_animations(
    commands: &mut Commands,
    world: &WorldBlocks,
    assets: &WorldRenderAssets,
    animations: &HashMap<IVec3, BlockAnimation>,
) {
    rebuild_world_with_timed_animations(
        commands,
        world,
        assets,
        animations,
        AnimationTiming::edit(),
    );
}

pub fn rebuild_world_with_timed_animations(
    commands: &mut Commands,
    world: &WorldBlocks,
    assets: &WorldRenderAssets,
    animations: &HashMap<IVec3, BlockAnimation>,
    timing: AnimationTiming,
) {
    for (pos, data) in &world.blocks {
        spawn_block_model(
            commands,
            assets,
            world,
            *pos,
            *data,
            assets.block_material(data.kind),
            None,
            animations.get(pos).copied(),
            None,
            timing,
            true,
            false,
        );
    }
    for (pos, data) in &world.system_blocks {
        spawn_block_model(
            commands,
            assets,
            world,
            *pos,
            *data,
            assets.block_material(data.kind),
            None,
            None,
            None,
            timing,
            true,
            false,
        );
    }
}

pub fn rebuild_world_with_runtime_animations(
    commands: &mut Commands,
    world: &WorldBlocks,
    assets: &WorldRenderAssets,
    animations: &HashMap<IVec3, BlockAnimation>,
    piston_animations: &HashMap<IVec3, PistonAnimation>,
    timing: AnimationTiming,
) {
    for (pos, data) in &world.blocks {
        spawn_block_model(
            commands,
            assets,
            world,
            *pos,
            *data,
            assets.block_material(data.kind),
            None,
            animations.get(pos).copied(),
            piston_animations.get(pos).copied(),
            timing,
            true,
            false,
        );
    }
    for (pos, data) in &world.system_blocks {
        spawn_block_model(
            commands,
            assets,
            world,
            *pos,
            *data,
            assets.block_material(data.kind),
            None,
            None,
            None,
            timing,
            true,
            false,
        );
    }
}

pub fn rebuild_world_with_runtime_animations_for_debug_state(
    commands: &mut Commands,
    world: &WorldBlocks,
    assets: &WorldRenderAssets,
    animations: &HashMap<IVec3, BlockAnimation>,
    piston_animations: &HashMap<IVec3, PistonAnimation>,
    timing: AnimationTiming,
    debug: &DebugState,
    factory_structures: &FactoryStructureState,
) {
    if debug.factory_activity {
        rebuild_world_with_factory_activity_debug(commands, world, assets, factory_structures);
    } else {
        rebuild_world_with_runtime_animations(
            commands,
            world,
            assets,
            animations,
            piston_animations,
            timing,
        );
    }
}

fn spawn_block_model(
    commands: &mut Commands,
    assets: &WorldRenderAssets,
    world: &WorldBlocks,
    pos: IVec3,
    data: BlockData,
    material: Handle<StandardMaterial>,
    edit_preview: Option<EditPreview>,
    animation: Option<BlockAnimation>,
    piston_animation: Option<PistonAnimation>,
    timing: AnimationTiming,
    with_block_entity: bool,
    pending_generated_preview: bool,
) {
    let mut transform = Transform::from_translation(grid_to_world(pos));
    if let Some(animation) = animation {
        let progress = animation.progress.unwrap_or(0.0).clamp(0.0, 1.0);
        let eased = match timing.easing {
            AnimationEasing::Linear => progress,
            AnimationEasing::SmoothStep => progress * progress * (3.0 - 2.0 * progress),
        };
        transform.translation =
            grid_to_world(animation.from_pos).lerp(grid_to_world(animation.to_pos), eased);
        transform.rotation = Quat::from_rotation_y(animation.from_facing.yaw())
            .slerp(Quat::from_rotation_y(animation.to_facing.yaw()), eased);
        transform.scale = match animation.kind {
            BlockAnimationKind::Move => Vec3::ONE,
            BlockAnimationKind::SpawnScale => Vec3::splat(eased),
        };
    } else {
        transform.rotation = Quat::from_rotation_y(data.facing.yaw());
    }

    let mut entity = if data.kind == crate::game::world::blocks::BlockKind::Wire {
        commands.spawn((transform, Visibility::default()))
    } else if let Some(scene_material) = assets.scene_material(data.kind) {
        commands.spawn((
            Mesh3d(assets.block_mesh(data.kind)),
            MeshMaterial3d::<SceneBlockMaterial>(scene_material),
            transform,
        ))
    } else {
        commands.spawn((
            Mesh3d(assets.block_mesh(data.kind)),
            MeshMaterial3d(material.clone()),
            transform,
        ))
    };

    if with_block_entity {
        entity.insert(BlockEntity { pos });
    }

    if pending_generated_preview {
        entity.insert(PendingGeneratedPreview);
    }

    if let Some(edit_preview) = edit_preview {
        entity.insert(edit_preview);
    }

    if let Some(animation) = animation.filter(|_| !pending_generated_preview) {
        entity.insert(AnimatedBlock::new(animation, timing));
    }

    entity.with_children(|parent| {
        let mut model_root = parent.spawn((Transform::default(), Visibility::default()));
        if let Some(piston_animation) = piston_animation {
            model_root.insert(AnimatedPiston::new(piston_animation));
        }
        model_root.with_children(|parent| {
            spawn_model_parts(parent, assets, data);
        });

        let render_behavior = data.kind.render_behavior(data.facing);

        if render_behavior.goal_topper {
            parent.spawn((
                Mesh3d(assets.goal_top.clone()),
                MeshMaterial3d(assets.goal_top_material.clone()),
                Transform::from_xyz(0.0, 0.55, 0.0),
            ));
        }

        if let Some(weld_connector) = render_behavior.weld_connector {
            let offsets = match weld_connector {
                WeldConnectorBehavior::AllSides => signal_offsets().to_vec(),
                WeldConnectorBehavior::Offset(offset) => vec![offset],
            };
            for offset in offsets {
                let neighbor = pos + offset;
                if world
                    .blocks
                    .get(&neighbor)
                    .or_else(|| world.system_blocks.get(&neighbor))
                    .is_some_and(|block| weld_connects_to(block, -offset))
                {
                    let local_offset = local_connector_offset(data, offset);
                    parent.spawn((
                        Mesh3d(assets.connector_mesh(local_offset)),
                        MeshMaterial3d(assets.weld_connector_material.clone()),
                        Transform::from_translation(local_offset.as_vec3() * 0.34),
                    ));
                }
            }
        }

        if render_behavior.wire_connector.is_some() {
            let mut connected_offsets = Vec::new();
            for offset in signal_offsets() {
                let neighbor = pos + offset;
                if world
                    .blocks
                    .get(&neighbor)
                    .or_else(|| world.system_blocks.get(&neighbor))
                    .is_some_and(|block| wire_connects_to(block, -offset))
                {
                    connected_offsets.push(offset);
                    let local_offset = local_connector_offset(data, offset);
                    parent.spawn((
                        Mesh3d(assets.wire_connector_mesh(local_offset)),
                        MeshMaterial3d(assets.wire_connector_material.clone()),
                        Transform::from_translation(local_offset.as_vec3() * 0.34),
                    ));
                }
            }

            if data.kind == crate::game::world::blocks::BlockKind::Wire
                && connected_offsets.is_empty()
            {
                parent.spawn((Mesh3d(assets.wire_node_mesh()), MeshMaterial3d(material)));
            }
        }

        if data.kind.is_material() {
            for (face, mark) in world
                .material_face_marks
                .iter()
                .filter(|(face, _)| face.pos == pos)
            {
                parent.spawn((
                    Mesh3d(assets.face_mark.clone()),
                    MeshMaterial3d(assets.face_mark_material(mark.color)),
                    face_mark_transform(face.normal),
                ));
            }
        }
    });
}

fn face_mark_transform(normal: IVec3) -> Transform {
    let normal_vec = normal.as_vec3();
    let rotation = if normal.x != 0 {
        Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)
    } else if normal.z != 0 {
        Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)
    } else {
        Quat::IDENTITY
    };
    Transform {
        translation: normal_vec * 0.506,
        rotation,
        ..default()
    }
}

fn spawn_model_parts(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldRenderAssets,
    data: BlockData,
) {
    let parts = match data.kind.model() {
        BlockModel::Default => &[],
        BlockModel::Parts(parts) => parts,
        BlockModel::Asset { path: _, fallback } => fallback,
    };

    for part in parts {
        parent.spawn((
            Mesh3d(assets.model_mesh(part.mesh)),
            MeshMaterial3d(assets.model_material(part.material)),
            Transform {
                translation: model_vec3(part.translation),
                rotation: Quat::from_rotation_y(part.yaw_radians),
                scale: model_vec3(part.scale),
                ..default()
            },
        ));
    }
}

fn model_vec3(value: [f32; 3]) -> Vec3 {
    Vec3::new(value[0], value[1], value[2])
}

fn weld_connects_to(block: &BlockData, connector_from_block: IVec3) -> bool {
    match block.kind.render_behavior(block.facing).weld_connector {
        Some(WeldConnectorBehavior::AllSides) => true,
        Some(WeldConnectorBehavior::Offset(offset)) => connector_from_block == offset,
        None => false,
    }
}

fn local_connector_offset(data: BlockData, offset: IVec3) -> IVec3 {
    if data.kind.is_directional() {
        data.facing.inverse_rotate_offset(offset)
    } else {
        offset
    }
}

fn wire_connects_to(block: &BlockData, wire_from_block: IVec3) -> bool {
    match block.kind.render_behavior(block.facing).wire_connector {
        Some(WireConnectorBehavior::Wire) => true,
        Some(WireConnectorBehavior::Device { blocked_offset }) => wire_from_block != blocked_offset,
        None => false,
    }
}

fn signal_offsets() -> [IVec3; 6] {
    [
        IVec3::X,
        IVec3::NEG_X,
        IVec3::Y,
        IVec3::NEG_Y,
        IVec3::Z,
        IVec3::NEG_Z,
    ]
}
