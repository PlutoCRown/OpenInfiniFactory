use bevy::pbr::CascadeShadowConfigBuilder;
use bevy::prelude::*;

use crate::game::world::blocks::{BlockData, BlockKind, BLOCK_SIZE};
use crate::game::world::grid::{grid_to_world, WorldBlocks};

#[derive(Component)]
pub struct BlockEntity;

#[derive(Component)]
pub struct HoverMarker;

#[derive(Component)]
pub struct PlacementPreview;

pub fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
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
            num_cascades: 4,
            minimum_distance: 0.15,
            maximum_distance: 60.0,
            first_cascade_far_bound: 8.0,
            overlap_proportion: 0.18,
        }
        .into(),
        ..default()
    });

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

pub fn rebuild_world(
    commands: &mut Commands,
    world: &WorldBlocks,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    for (pos, data) in &world.blocks {
        spawn_block(commands, meshes, materials, world, *pos, *data);
    }
}

pub fn despawn_world(commands: &mut Commands, block_entities: &Query<Entity, With<BlockEntity>>) {
    for entity in block_entities {
        commands.entity(entity).despawn_recursive();
    }
}

fn spawn_block(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    world: &WorldBlocks,
    pos: IVec3,
    data: BlockData,
) {
    let size = if data.kind == BlockKind::WeldPoint {
        BLOCK_SIZE * 0.38
    } else {
        BLOCK_SIZE
    };
    let block_mesh = meshes.add(Cuboid::new(size, size, size));
    let mut material = StandardMaterial {
        base_color: data.kind.material(),
        perceptual_roughness: 0.88,
        reflectance: 0.18,
        ..default()
    };
    if matches!(data.kind, BlockKind::Glass | BlockKind::WeldPoint) {
        material.alpha_mode = AlphaMode::Blend;
        material.unlit = data.kind == BlockKind::WeldPoint;
    }
    let block_material = materials.add(material);

    commands
        .spawn((
            PbrBundle {
                mesh: block_mesh,
                material: block_material,
                transform: Transform::from_translation(grid_to_world(pos)),
                ..default()
            },
            BlockEntity,
        ))
        .with_children(|parent| {
            if data.kind.is_directional() {
                let forward = data.facing.forward();
                let arrow_mesh = meshes.add(Cuboid::new(0.18, 0.08, 0.72));
                let arrow_material = materials.add(StandardMaterial {
                    base_color: Color::srgb(0.95, 0.95, 0.38),
                    unlit: true,
                    ..default()
                });
                parent.spawn(PbrBundle {
                    mesh: arrow_mesh,
                    material: arrow_material,
                    transform: Transform {
                        translation: forward * 0.05 + Vec3::Y * 0.54,
                        rotation: Quat::from_rotation_y(data.facing.yaw()),
                        ..default()
                    },
                    ..default()
                });

                let nose_mesh = meshes.add(Cuboid::new(0.42, 0.10, 0.18));
                let nose_material = materials.add(StandardMaterial {
                    base_color: Color::srgb(1.0, 0.78, 0.25),
                    unlit: true,
                    ..default()
                });
                parent.spawn(PbrBundle {
                    mesh: nose_mesh,
                    material: nose_material,
                    transform: Transform {
                        translation: forward * 0.42 + Vec3::Y * 0.56,
                        rotation: Quat::from_rotation_y(data.facing.yaw()),
                        ..default()
                    },
                    ..default()
                });
            }

            if data.kind == BlockKind::Goal {
                let top_mesh = meshes.add(Cuboid::new(0.62, 0.08, 0.62));
                let top_material = materials.add(StandardMaterial {
                    base_color: Color::srgb(0.75, 1.0, 0.55),
                    emissive: Color::srgb(0.12, 0.28, 0.08).into(),
                    ..default()
                });
                parent.spawn(PbrBundle {
                    mesh: top_mesh,
                    material: top_material,
                    transform: Transform::from_xyz(0.0, 0.55, 0.0),
                    ..default()
                });
            }

            if data.kind == BlockKind::WeldPoint {
                for offset in [IVec3::X, IVec3::NEG_X, IVec3::Z, IVec3::NEG_Z] {
                    let neighbor = pos + offset;
                    if world
                        .blocks
                        .get(&neighbor)
                        .is_some_and(|block| block.kind == BlockKind::WeldPoint)
                    {
                        let connector_mesh = if offset.x != 0 {
                            meshes.add(Cuboid::new(0.72, 0.08, 0.08))
                        } else {
                            meshes.add(Cuboid::new(0.08, 0.08, 0.72))
                        };
                        let connector_material = materials.add(StandardMaterial {
                            base_color: Color::srgba(1.0, 0.22, 0.10, 0.72),
                            alpha_mode: AlphaMode::Blend,
                            unlit: true,
                            ..default()
                        });
                        parent.spawn(PbrBundle {
                            mesh: connector_mesh,
                            material: connector_material,
                            transform: Transform::from_translation(offset.as_vec3() * 0.36),
                            ..default()
                        });
                    }
                }
            }
        });
}
