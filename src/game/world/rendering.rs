use bevy::pbr::CascadeShadowConfigBuilder;
use bevy::prelude::*;

use crate::game::world::blocks::{BlockData, BlockKind};
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

pub fn spawn_block(
    commands: &mut Commands,
    assets: &WorldRenderAssets,
    world: &WorldBlocks,
    pos: IVec3,
    data: BlockData,
) {
    commands
        .spawn((
            PbrBundle {
                mesh: assets.block_mesh(data.kind),
                material: assets.block_material(data.kind),
                transform: Transform::from_translation(grid_to_world(pos)),
                ..default()
            },
            BlockEntity { pos },
        ))
        .with_children(|parent| {
            if data.kind.is_directional() {
                let forward = data.facing.forward();
                parent.spawn(PbrBundle {
                    mesh: assets.arrow.clone(),
                    material: assets.arrow_material.clone(),
                    transform: Transform {
                        translation: forward * 0.05 + Vec3::Y * 0.54,
                        rotation: Quat::from_rotation_y(data.facing.yaw()),
                        ..default()
                    },
                    ..default()
                });

                parent.spawn(PbrBundle {
                    mesh: assets.arrow_nose.clone(),
                    material: assets.arrow_nose_material.clone(),
                    transform: Transform {
                        translation: forward * 0.42 + Vec3::Y * 0.56,
                        rotation: Quat::from_rotation_y(data.facing.yaw()),
                        ..default()
                    },
                    ..default()
                });
            }

            if data.kind == BlockKind::Goal {
                parent.spawn(PbrBundle {
                    mesh: assets.goal_top.clone(),
                    material: assets.goal_top_material.clone(),
                    transform: Transform::from_xyz(0.0, 0.55, 0.0),
                    ..default()
                });
            }

            if data.kind == BlockKind::WeldPoint {
                for offset in signal_offsets() {
                    let neighbor = pos + offset;
                    if world
                        .blocks
                        .get(&neighbor)
                        .is_some_and(|block| weld_point_connects_to(block, -offset))
                    {
                        parent.spawn(PbrBundle {
                            mesh: assets.connector_mesh(offset),
                            material: assets.weld_connector_material.clone(),
                            transform: Transform::from_translation(offset.as_vec3() * 0.34),
                            ..default()
                        });
                    }
                }
            }

            if data.kind == BlockKind::Wire {
                for offset in signal_offsets() {
                    let neighbor = pos + offset;
                    if world
                        .blocks
                        .get(&neighbor)
                        .is_some_and(|block| wire_connects_to(block, -offset))
                    {
                        parent.spawn(PbrBundle {
                            mesh: assets.connector_mesh(offset),
                            material: assets.wire_connector_material.clone(),
                            transform: Transform::from_translation(offset.as_vec3() * 0.34),
                            ..default()
                        });
                    }
                }
            }
        });
}

fn weld_point_connects_to(block: &BlockData, connector_from_block: IVec3) -> bool {
    match block.kind {
        BlockKind::WeldPoint => true,
        BlockKind::Welder => connector_from_block == block.facing.forward_ivec3(),
        _ => false,
    }
}

fn wire_connects_to(block: &BlockData, wire_from_block: IVec3) -> bool {
    match block.kind {
        BlockKind::Wire => true,
        BlockKind::Detector => wire_from_block != block.facing.forward_ivec3(),
        BlockKind::Piston => wire_from_block != block.facing.forward_ivec3(),
        BlockKind::Blocker => wire_from_block != block.facing.forward_ivec3(),
        BlockKind::Laser => wire_from_block != block.facing.forward_ivec3(),
        _ => false,
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
