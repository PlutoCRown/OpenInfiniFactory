use bevy::pbr::CascadeShadowConfigBuilder;
use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::render::texture::ImageSampler;

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
    wire_connector_material: Handle<StandardMaterial>,
    arrow_material: Handle<StandardMaterial>,
    arrow_nose_material: Handle<StandardMaterial>,
    goal_top_material: Handle<StandardMaterial>,
    weld_connector_material: Handle<StandardMaterial>,
    place_preview_material: Handle<StandardMaterial>,
    delete_preview_material: Handle<StandardMaterial>,
    selection_preview_material: Handle<StandardMaterial>,
}

#[derive(Component)]
pub struct HoverMarker;

#[derive(Component)]
pub struct PlacementPreview;

#[derive(Component)]
pub struct EditPreview;

pub enum EditPreviewKind {
    Place,
    Delete,
    Selection,
}

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

impl WorldRenderAssets {
    fn new(
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
                images.add(procedural_block_texture(ProceduralTexture::Grass)),
            )),
            stone: materials.add(textured_block_material(
                BlockKind::Stone,
                images.add(procedural_block_texture(ProceduralTexture::Stone)),
            )),
            dirt: materials.add(textured_block_material(
                BlockKind::Dirt,
                images.add(procedural_block_texture(ProceduralTexture::Dirt)),
            )),
            planks: materials.add(textured_block_material(
                BlockKind::Planks,
                images.add(procedural_block_texture(ProceduralTexture::Planks)),
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

    fn block_mesh(&self, kind: BlockKind) -> Handle<Mesh> {
        if matches!(
            kind,
            BlockKind::WeldPoint | BlockKind::Wire | BlockKind::DrillHead
        ) {
            self.node.clone()
        } else {
            self.block.clone()
        }
    }

    fn block_material(&self, kind: BlockKind) -> Handle<StandardMaterial> {
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

    fn edit_preview_material(&self, kind: EditPreviewKind) -> Handle<StandardMaterial> {
        match kind {
            EditPreviewKind::Place => self.place_preview_material.clone(),
            EditPreviewKind::Delete => self.delete_preview_material.clone(),
            EditPreviewKind::Selection => self.selection_preview_material.clone(),
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

#[derive(Clone, Copy)]
enum ProceduralTexture {
    Grass,
    Stone,
    Dirt,
    Planks,
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

fn procedural_block_texture(kind: ProceduralTexture) -> Image {
    const SIZE: u32 = 32;
    let mut data = Vec::with_capacity((SIZE * SIZE * 4) as usize);

    for y in 0..SIZE {
        for x in 0..SIZE {
            let [r, g, b] = match kind {
                ProceduralTexture::Grass => grass_pixel(x, y),
                ProceduralTexture::Stone => stone_pixel(x, y),
                ProceduralTexture::Dirt => dirt_pixel(x, y),
                ProceduralTexture::Planks => planks_pixel(x, y),
            };
            data.extend_from_slice(&[r, g, b, 255]);
        }
    }

    let mut image = Image::new(
        Extent3d {
            width: SIZE,
            height: SIZE,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::nearest();
    image
}

fn grass_pixel(x: u32, y: u32) -> [u8; 3] {
    let noise = texture_noise(x, y, 17);
    if y < 7 {
        let blade = ((x * 5 + y * 11 + noise as u32) % 13) < 4;
        if blade {
            shade([66, 135, 42], noise, 24)
        } else {
            shade([82, 154, 48], noise, 18)
        }
    } else {
        let root = ((x + y * 3 + noise as u32) % 17) < 3;
        if root {
            shade([78, 48, 26], noise, 18)
        } else {
            shade([118, 79, 43], noise, 22)
        }
    }
}

fn stone_pixel(x: u32, y: u32) -> [u8; 3] {
    let noise = texture_noise(x, y, 41);
    let crack = ((x * 3 + y * 7 + noise as u32) % 23) == 0 || (x + y * 2) % 29 == 0;
    if crack {
        shade([70, 70, 68], noise, 10)
    } else {
        shade([122, 123, 119], noise, 26)
    }
}

fn dirt_pixel(x: u32, y: u32) -> [u8; 3] {
    let noise = texture_noise(x, y, 73);
    let pebble = ((x * 13 + y * 5 + noise as u32) % 19) < 2;
    if pebble {
        shade([96, 73, 52], noise, 16)
    } else {
        shade([111, 72, 39], noise, 24)
    }
}

fn planks_pixel(x: u32, y: u32) -> [u8; 3] {
    let noise = texture_noise(x, y, 109);
    let seam = y % 8 == 0 || x % 16 == 0;
    let grain = ((x * 7 + noise as u32) % 11) < 3;
    if seam {
        shade([86, 52, 25], noise, 10)
    } else if grain {
        shade([154, 104, 55], noise, 18)
    } else {
        shade([178, 121, 65], noise, 20)
    }
}

fn texture_noise(x: u32, y: u32, seed: u32) -> u8 {
    let mut value = x
        .wrapping_mul(73_856_093)
        .wrapping_add(y.wrapping_mul(19_349_663))
        .wrapping_add(seed.wrapping_mul(83_492_791));
    value ^= value >> 13;
    value = value.wrapping_mul(1_274_126_177);
    ((value ^ (value >> 16)) & 0xff) as u8
}

fn shade(base: [u8; 3], noise: u8, amount: i16) -> [u8; 3] {
    let delta = (noise as i16 - 128) * amount / 128;
    [
        (base[0] as i16 + delta).clamp(0, 255) as u8,
        (base[1] as i16 + delta).clamp(0, 255) as u8,
        (base[2] as i16 + delta).clamp(0, 255) as u8,
    ]
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
