use bevy::pbr::CascadeShadowConfigBuilder;
use bevy::prelude::*;
use std::collections::HashMap;

use crate::game::world::animation::{AnimatedBlock, AnimationTiming, BlockAnimation};
use crate::game::world::blocks::{BlockData, WeldConnectorBehavior, WireConnectorBehavior};
use crate::game::world::direction::Facing;
use crate::game::world::grid::{grid_to_world, WorldBlocks};
pub use crate::game::world::render_assets::{EditPreviewKind, WorldRenderAssets};

#[derive(Component)]
pub struct BlockEntity {
    pub pos: IVec3,
}

#[derive(Component)]
pub struct HoverMarker;

#[derive(Component)]
pub struct PlacementPreview;

#[derive(Component)]
pub struct EditPreview;

pub fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1100.0,
            shadows_enabled: true,
            range: 18.0,
            radius: 3.5,
            ..default()
        },
        transform: Transform::from_xyz(3.5, 5.5, 4.5),
        ..default()
    });

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 9500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -1.05, -0.55, -0.28)),
        cascade_shadow_config: CascadeShadowConfigBuilder {
            num_cascades: 3,
            minimum_distance: 0.15,
            maximum_distance: 48.0,
            first_cascade_far_bound: 8.0,
            overlap_proportion: 0.16,
        }
        .into(),
        ..default()
    });

    let render_assets = WorldRenderAssets::new(&mut meshes, &mut materials, &mut images);
    commands.insert_resource(render_assets);

    let marker_mesh = meshes.add(Cuboid::new(1.04, 1.04, 1.04));
    let marker_material = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 1.0, 1.0, 0.16),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });

    commands.spawn((
        PbrBundle {
            mesh: marker_mesh,
            material: marker_material,
            visibility: Visibility::Hidden,
            ..default()
        },
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
        PbrBundle {
            mesh: preview_mesh,
            material: preview_material,
            visibility: Visibility::Hidden,
            ..default()
        },
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

pub fn despawn_world(commands: &mut Commands, block_entities: &Query<Entity, With<BlockEntity>>) {
    for entity in block_entities {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn despawn_edit_previews(commands: &mut Commands, previews: &Query<Entity, With<EditPreview>>) {
    for entity in previews {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn spawn_edit_preview(
    commands: &mut Commands,
    assets: &WorldRenderAssets,
    pos: IVec3,
    kind: EditPreviewKind,
) {
    commands.spawn((
        PbrBundle {
            mesh: assets.block.clone(),
            material: assets.edit_preview_material(kind),
            transform: Transform::from_translation(grid_to_world(pos)),
            ..default()
        },
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
        AnimationTiming::edit(),
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
        timing,
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
            timing,
            true,
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
            timing,
            true,
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
    timing: AnimationTiming,
    with_block_entity: bool,
) {
    let mut transform = Transform::from_translation(grid_to_world(pos));
    if animation.is_none() {
        transform.rotation = Quat::from_rotation_y(data.facing.yaw());
    }

    let mut entity = commands.spawn(PbrBundle {
        mesh: assets.block_mesh(data.kind),
        material,
        transform,
        ..default()
    });

    if with_block_entity {
        entity.insert(BlockEntity { pos });
    }

    if let Some(edit_preview) = edit_preview {
        entity.insert(edit_preview);
    }

    if let Some(animation) = animation {
        entity.insert(AnimatedBlock::new(animation, timing));
    }

    entity.with_children(|parent| {
        if data.kind.is_directional() {
            let forward = Facing::North.forward();
            parent.spawn(PbrBundle {
                mesh: assets.arrow.clone(),
                material: assets.arrow_material.clone(),
                transform: Transform {
                    translation: forward * 0.05 + Vec3::Y * 0.54,
                    ..default()
                },
                ..default()
            });

            parent.spawn(PbrBundle {
                mesh: assets.arrow_nose.clone(),
                material: assets.arrow_nose_material.clone(),
                transform: Transform {
                    translation: forward * 0.42 + Vec3::Y * 0.56,
                    ..default()
                },
                ..default()
            });
        }

        let render_behavior = data.kind.render_behavior(data.facing);

        if render_behavior.goal_topper {
            parent.spawn(PbrBundle {
                mesh: assets.goal_top.clone(),
                material: assets.goal_top_material.clone(),
                transform: Transform::from_xyz(0.0, 0.55, 0.0),
                ..default()
            });
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
                    parent.spawn(PbrBundle {
                        mesh: assets.connector_mesh(local_offset),
                        material: assets.weld_connector_material.clone(),
                        transform: Transform::from_translation(local_offset.as_vec3() * 0.34),
                        ..default()
                    });
                }
            }
        }

        if render_behavior.wire_connector.is_some() {
            for offset in signal_offsets() {
                let neighbor = pos + offset;
                if world
                    .blocks
                    .get(&neighbor)
                    .or_else(|| world.system_blocks.get(&neighbor))
                    .is_some_and(|block| wire_connects_to(block, -offset))
                {
                    let local_offset = local_connector_offset(data, offset);
                    parent.spawn(PbrBundle {
                        mesh: assets.connector_mesh(local_offset),
                        material: assets.wire_connector_material.clone(),
                        transform: Transform::from_translation(local_offset.as_vec3() * 0.34),
                        ..default()
                    });
                }
            }
        }

        if data.kind.is_material() {
            for (face, mark) in world
                .material_face_marks
                .iter()
                .filter(|(face, _)| face.pos == pos)
            {
                parent.spawn(PbrBundle {
                    mesh: assets.face_mark.clone(),
                    material: assets.face_mark_material(mark.color),
                    transform: face_mark_transform(face.normal),
                    ..default()
                });
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
        Some(WireConnectorBehavior::Device { blocked_offset }) => {
            wire_from_block != blocked_offset
        }
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
