use bevy::pbr::CascadeShadowConfigBuilder;
use bevy::prelude::*;

use crate::game::world::blocks::{BlockData, BlockKind, BLOCK_SIZE};
use crate::game::world::grid::{grid_to_world, WorldBlocks};

#[derive(Component)]
pub struct BlockEntity {
    pub pos: IVec3,
}

#[derive(Resource, Clone)]
pub struct WorldRenderAssets {
    block: Handle<Mesh>,
    node: Handle<Mesh>,
    arrow: Handle<Mesh>,
    arrow_nose: Handle<Mesh>,
    goal_top: Handle<Mesh>,
    connector_x: Handle<Mesh>,
    connector_y: Handle<Mesh>,
    connector_z: Handle<Mesh>,
    solid: Handle<StandardMaterial>,
    glass: Handle<StandardMaterial>,
    generator: Handle<StandardMaterial>,
    welder: Handle<StandardMaterial>,
    conveyor: Handle<StandardMaterial>,
    detector: Handle<StandardMaterial>,
    wire: Handle<StandardMaterial>,
    piston: Handle<StandardMaterial>,
    goal: Handle<StandardMaterial>,
    material: Handle<StandardMaterial>,
    weld_point_material: Handle<StandardMaterial>,
    wire_connector_material: Handle<StandardMaterial>,
    arrow_material: Handle<StandardMaterial>,
    arrow_nose_material: Handle<StandardMaterial>,
    goal_top_material: Handle<StandardMaterial>,
    weld_connector_material: Handle<StandardMaterial>,
}

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
            num_cascades: 3,
            minimum_distance: 0.15,
            maximum_distance: 48.0,
            first_cascade_far_bound: 8.0,
            overlap_proportion: 0.16,
        }
        .into(),
        ..default()
    });

    let render_assets = WorldRenderAssets::new(&mut meshes, &mut materials);
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

impl WorldRenderAssets {
    fn new(meshes: &mut Assets<Mesh>, materials: &mut Assets<StandardMaterial>) -> Self {
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
            glass: materials.add(block_material(BlockKind::Glass)),
            generator: materials.add(block_material(BlockKind::Generator)),
            welder: materials.add(block_material(BlockKind::Welder)),
            conveyor: materials.add(block_material(BlockKind::Conveyor)),
            detector: materials.add(block_material(BlockKind::Detector)),
            wire: materials.add(block_material(BlockKind::Wire)),
            piston: materials.add(block_material(BlockKind::Piston)),
            goal: materials.add(block_material(BlockKind::Goal)),
            material: materials.add(block_material(BlockKind::Material)),
            weld_point_material: materials.add(block_material(BlockKind::WeldPoint)),
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
        }
    }

    fn block_mesh(&self, kind: BlockKind) -> Handle<Mesh> {
        if matches!(kind, BlockKind::WeldPoint | BlockKind::Wire) {
            self.node.clone()
        } else {
            self.block.clone()
        }
    }

    fn block_material(&self, kind: BlockKind) -> Handle<StandardMaterial> {
        match kind {
            BlockKind::Solid => self.solid.clone(),
            BlockKind::Glass => self.glass.clone(),
            BlockKind::Generator => self.generator.clone(),
            BlockKind::Welder => self.welder.clone(),
            BlockKind::Conveyor => self.conveyor.clone(),
            BlockKind::Detector => self.detector.clone(),
            BlockKind::Wire => self.wire.clone(),
            BlockKind::Piston => self.piston.clone(),
            BlockKind::Goal => self.goal.clone(),
            BlockKind::Material => self.material.clone(),
            BlockKind::WeldPoint => self.weld_point_material.clone(),
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
    if matches!(kind, BlockKind::Glass | BlockKind::WeldPoint) {
        material.alpha_mode = AlphaMode::Blend;
        material.unlit = kind == BlockKind::WeldPoint;
    }
    material
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

impl WorldRenderAssets {
    fn connector_mesh(&self, offset: IVec3) -> Handle<Mesh> {
        if offset.x != 0 {
            self.connector_x.clone()
        } else if offset.y != 0 {
            self.connector_y.clone()
        } else {
            self.connector_z.clone()
        }
    }
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
