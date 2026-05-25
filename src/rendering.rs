use bevy::prelude::*;

use crate::blocks::{BlockData, BlockKind, BLOCK_SIZE};
use crate::world::{grid_to_world, WorldBlocks};

#[derive(Component)]
pub struct BlockEntity;

#[derive(Component)]
pub struct HoverMarker;

pub fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 4500.0,
            shadows_enabled: true,
            range: 40.0,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 9.0, 6.0),
        ..default()
    });

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 7000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, -0.6, 0.0)),
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
}

pub fn rebuild_world(
    commands: &mut Commands,
    world: &WorldBlocks,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    for (pos, data) in &world.blocks {
        spawn_block(commands, meshes, materials, *pos, *data);
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
    pos: IVec3,
    data: BlockData,
) {
    let block_mesh = meshes.add(Cuboid::new(BLOCK_SIZE, BLOCK_SIZE, BLOCK_SIZE));
    let mut material = StandardMaterial {
        base_color: data.kind.material(),
        perceptual_roughness: 0.82,
        ..default()
    };
    if data.kind == BlockKind::Glass {
        material.alpha_mode = AlphaMode::Blend;
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
        });
}
