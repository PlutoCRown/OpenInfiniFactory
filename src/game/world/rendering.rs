use bevy::asset::RenderAssetUsages;
use bevy::camera::visibility::RenderLayers;
use bevy::camera::{RenderTarget, ScalingMode};
use bevy::light::CascadeShadowConfigBuilder;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;
use std::collections::{HashMap, HashSet};

use crate::game::blocks::{
    edit_blocks, BlockData, BlockKind, BlockModel, WeldConnectorBehavior, WireConnectorBehavior,
    PLAY_BLOCKS,
};
use crate::game::simulation::structure_state::{FactoryActivity, StructureKind, StructureState};
use crate::game::systems::debug::DebugState;
use crate::game::world::animation::{
    rotate_world_pos_y, AnimatedBlock, AnimatedPusher, AnimatedPusherRod, AnimationEasing,
    AnimationTiming, BlockAnimation, BlockAnimationKind, PusherAnimation, WeldSpark,
};
use crate::game::world::grid::{grid_to_world, WorldBlocks};
pub use crate::game::world::render_assets::{EditPreviewKind, WorldRenderAssets};

const ICON_TEXTURE_SIZE: u32 = 256;
const ICON_RENDER_LAYER: usize = 3;
const ICON_SPACING: f32 = 4.0;
const ICON_RENDER_FRAMES: u8 = 3;

pub fn block_face_highlight_transform(block_pos: IVec3, normal: IVec3) -> Transform {
    let normal = normal.as_vec3().normalize();
    Transform {
        translation: grid_to_world(block_pos) + normal * 0.501,
        rotation: Quat::from_rotation_arc(Vec3::Y, normal),
        scale: Vec3::ONE,
    }
}

#[derive(Component)]
pub struct BlockEntity {
    pub pos: IVec3,
}

#[derive(Component)]
pub struct GameplayScene;

#[derive(Resource, Default)]
pub struct BlockIconAssets {
    icons: HashMap<BlockKind, Handle<Image>>,
}

impl BlockIconAssets {
    pub fn get(&self, kind: BlockKind) -> Option<Handle<Image>> {
        self.icons.get(&kind).cloned()
    }
}

#[derive(Component)]
pub(crate) struct BlockIconRenderEntity;

#[derive(Component)]
pub(crate) struct BlockIconRenderRoot;

#[derive(Component)]
pub(crate) struct BlockIconRenderCamera;

#[derive(Resource)]
pub struct BlockIconRenderState {
    frames_remaining: u8,
}

#[derive(Component)]
pub struct HoverMarker;

#[derive(Component)]
pub struct AimFaceHighlight;

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
        GameplayScene,
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
        GameplayScene,
    ));

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
        Mesh3d(marker_mesh),
        MeshMaterial3d(marker_material),
        Visibility::Hidden,
        HoverMarker,
        GameplayScene,
    ));

    let face_mesh = meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(0.49)));
    let face_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.72, 0.92, 1.0, 0.72),
        emissive: LinearRgba::from(Color::srgb(0.35, 0.72, 1.0)),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        cull_mode: None,
        ..default()
    });

    commands.spawn((
        Mesh3d(face_mesh),
        MeshMaterial3d(face_material),
        Visibility::Hidden,
        AimFaceHighlight,
        GameplayScene,
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
        GameplayScene,
    ));
}

pub fn rebuild_world_on_enter(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    world: &WorldBlocks,
    render_assets: &WorldRenderAssets,
    debug: &DebugState,
    structure_state: &mut StructureState,
) {
    structure_state.clear();
    if debug.factory_activity {
        structure_state.rebuild_factory_for_debug(world);
    }
    rebuild_world_for_debug_state(
        commands,
        meshes,
        world,
        render_assets,
        debug,
        structure_state,
    );
}

pub fn teardown_playing_scene(commands: &mut Commands) {
    commands.remove_resource::<WorldRenderAssets>();
    commands.remove_resource::<BlockIconAssets>();
    commands.remove_resource::<BlockIconRenderState>();
}

pub fn setup_block_icons(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    assets: Res<WorldRenderAssets>,
) {
    let icon_layer = RenderLayers::layer(ICON_RENDER_LAYER);
    let mut icon_assets = BlockIconAssets::default();
    let icon_world = WorldBlocks::default();
    let icon_kinds = block_icon_kinds();

    commands.spawn((
        DirectionalLight {
            illuminance: 7800.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.85, -0.55, -0.25)),
        icon_layer.clone(),
        BlockIconRenderEntity,
        BlockIconRenderRoot,
    ));

    for (index, kind) in icon_kinds.into_iter().enumerate() {
        let image = Image::new_target_texture(
            ICON_TEXTURE_SIZE,
            ICON_TEXTURE_SIZE,
            TextureFormat::Rgba8Unorm,
            Some(TextureFormat::Rgba8UnormSrgb),
        );
        let image_handle = images.add(image);
        icon_assets.icons.insert(kind, image_handle.clone());

        let origin = Vec3::new(index as f32 * ICON_SPACING, -100.0, 0.0);
        spawn_block_icon_model(
            &mut commands,
            &mut meshes,
            &assets,
            &icon_world,
            kind,
            origin,
            &icon_layer,
        );

        commands.spawn((
            Camera3d::default(),
            Camera {
                order: -2,
                clear_color: Color::NONE.into(),
                ..default()
            },
            RenderTarget::Image(image_handle.into()),
            Projection::Orthographic(OrthographicProjection {
                scaling_mode: ScalingMode::Fixed {
                    width: 2.45,
                    height: 2.45,
                },
                ..OrthographicProjection::default_3d()
            }),
            Transform::from_translation(origin + Vec3::new(2.8, 2.2, 2.8))
                .looking_at(origin, Vec3::Y),
            AmbientLight {
                color: Color::WHITE,
                brightness: 520.0,
                ..default()
            },
            icon_layer.clone(),
            BlockIconRenderEntity,
            BlockIconRenderRoot,
            BlockIconRenderCamera,
        ));
    }

    commands.insert_resource(icon_assets);
    commands.insert_resource(BlockIconRenderState {
        frames_remaining: ICON_RENDER_FRAMES,
    });
}

fn block_icon_kinds() -> Vec<BlockKind> {
    let mut kinds = Vec::new();
    for kind in edit_blocks().into_iter().chain(PLAY_BLOCKS).chain([
        BlockKind::Material,
        BlockKind::IronMaterial,
        BlockKind::CopperMaterial,
    ]) {
        if !kinds.contains(&kind) {
            kinds.push(kind);
        }
    }
    kinds
}

pub fn retire_block_icon_renderers(
    mut commands: Commands,
    state: Option<ResMut<BlockIconRenderState>>,
    render_entities: Query<Entity, With<BlockIconRenderRoot>>,
    mut cameras: Query<&mut Camera, With<BlockIconRenderCamera>>,
) {
    let Some(mut state) = state else {
        return;
    };
    if state.frames_remaining > 0 {
        state.frames_remaining -= 1;
        return;
    }

    for mut camera in &mut cameras {
        camera.is_active = false;
    }
    for entity in &render_entities {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<BlockIconRenderState>();
}

fn spawn_block_icon_model(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    assets: &WorldRenderAssets,
    world: &WorldBlocks,
    kind: BlockKind,
    origin: Vec3,
    icon_layer: &RenderLayers,
) {
    let data = BlockData {
        kind,
        facing: crate::game::world::direction::Facing::South,
    };
    spawn_block_model(
        commands,
        meshes,
        assets,
        world,
        IVec3::ZERO,
        data,
        assets.block_material(data.kind),
        None,
        None,
        None,
        AnimationTiming::edit(),
        false,
        false,
        true,
        Some((origin - Vec3::splat(0.5), icon_layer)),
    );
}

pub fn rebuild_world(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    world: &WorldBlocks,
    assets: &WorldRenderAssets,
) {
    for (pos, data) in &world.blocks {
        spawn_block(commands, meshes, assets, world, *pos, *data);
    }
    for (pos, data) in &world.system_blocks {
        spawn_block(commands, meshes, assets, world, *pos, *data);
    }
}

pub fn rebuild_world_with_factory_activity_debug(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    world: &WorldBlocks,
    assets: &WorldRenderAssets,
    structure_state: &StructureState,
) {
    for (pos, data) in &world.blocks {
        if data.kind.is_factory() {
            let material = match structure_state.activity_at(*pos) {
                Some(FactoryActivity::Inactive) => assets.inactive_factory_debug_material(),
                _ => assets.active_factory_debug_material(),
            };
            spawn_debug_factory_block(commands, assets, *pos, material);
        } else {
            spawn_block(commands, meshes, assets, world, *pos, *data);
        }
    }
    for (pos, data) in &world.system_blocks {
        spawn_block(commands, meshes, assets, world, *pos, *data);
    }
}

pub fn rebuild_world_for_debug_state(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    world: &WorldBlocks,
    assets: &WorldRenderAssets,
    debug: &DebugState,
    structure_state: &StructureState,
) {
    if debug.factory_activity {
        rebuild_world_with_factory_activity_debug(commands, meshes, world, assets, structure_state);
    } else {
        rebuild_world(commands, meshes, world, assets);
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

pub fn spawn_weld_sparks(commands: &mut Commands, assets: &WorldRenderAssets, positions: &[IVec3]) {
    let velocities = [
        Vec3::new(1.60, 2.70, 0.42),
        Vec3::new(-1.44, 2.46, 0.76),
        Vec3::new(0.50, 2.86, -1.50),
        Vec3::new(-0.66, 2.28, -1.26),
        Vec3::new(1.18, 1.92, 1.34),
        Vec3::new(-1.26, 2.10, -0.34),
    ];

    for pos in positions {
        let origin = grid_to_world(*pos);
        for (index, velocity) in velocities.into_iter().enumerate() {
            let offset = Vec3::new(
                (index as f32 * 1.37).sin() * 0.20,
                0.04,
                (index as f32 * 2.11).cos() * 0.20,
            );
            commands.spawn((
                Mesh3d(assets.weld_spark.clone()),
                MeshMaterial3d(assets.weld_connector_material.clone()),
                Transform::from_translation(origin + offset),
                WeldSpark::new(velocity, 0.28),
            ));
        }
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
        Transform::from_translation(grid_to_world(pos)).with_scale(Vec3::splat(1.03)),
        EditPreview,
    ));
}

pub fn spawn_block_preview(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    assets: &WorldRenderAssets,
    world: &WorldBlocks,
    pos: IVec3,
    data: BlockData,
) {
    spawn_block_model(
        commands,
        meshes,
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
        true,
        None,
    );
}

pub fn spawn_block(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    assets: &WorldRenderAssets,
    world: &WorldBlocks,
    pos: IVec3,
    data: BlockData,
) {
    spawn_block_with_animation(commands, meshes, assets, world, pos, data, None);
}

pub fn spawn_block_with_animation(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    assets: &WorldRenderAssets,
    world: &WorldBlocks,
    pos: IVec3,
    data: BlockData,
    animation: Option<BlockAnimation>,
) {
    spawn_block_with_timed_animation(
        commands,
        meshes,
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
    meshes: &mut Assets<Mesh>,
    assets: &WorldRenderAssets,
    world: &WorldBlocks,
    pos: IVec3,
    data: BlockData,
    animation: Option<BlockAnimation>,
    timing: AnimationTiming,
) {
    spawn_block_model(
        commands,
        meshes,
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
        true,
        None,
    );
}

pub fn spawn_pending_generated_block(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    assets: &WorldRenderAssets,
    world: &WorldBlocks,
    pos: IVec3,
    data: BlockData,
    animation: Option<BlockAnimation>,
    timing: AnimationTiming,
) {
    spawn_block_model(
        commands,
        meshes,
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
        false,
        None,
    );
}

pub fn rebuild_world_with_animations(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    world: &WorldBlocks,
    assets: &WorldRenderAssets,
    animations: &HashMap<IVec3, BlockAnimation>,
) {
    rebuild_world_with_timed_animations(
        commands,
        meshes,
        world,
        assets,
        animations,
        AnimationTiming::edit(),
    );
}

pub fn rebuild_world_with_timed_animations(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    world: &WorldBlocks,
    assets: &WorldRenderAssets,
    animations: &HashMap<IVec3, BlockAnimation>,
    timing: AnimationTiming,
) {
    for (pos, data) in &world.blocks {
        spawn_block_model(
            commands,
            meshes,
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
            true,
            None,
        );
    }
    for (pos, data) in &world.system_blocks {
        spawn_block_model(
            commands,
            meshes,
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
            true,
            None,
        );
    }
}

pub fn rebuild_world_with_runtime_animations(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    world: &WorldBlocks,
    assets: &WorldRenderAssets,
    animations: &HashMap<IVec3, BlockAnimation>,
    pusher_animations: &HashMap<IVec3, PusherAnimation>,
    timing: AnimationTiming,
    powered_wires: &HashSet<IVec3>,
) {
    for (pos, data) in &world.blocks {
        let material = block_render_material(assets, *data, powered_wires.contains(pos));
        spawn_block_model(
            commands,
            meshes,
            assets,
            world,
            *pos,
            *data,
            material,
            None,
            animations.get(pos).copied(),
            pusher_animations.get(pos).copied(),
            timing,
            true,
            false,
            false,
            None,
        );
    }
    for (pos, data) in &world.system_blocks {
        spawn_block_model(
            commands,
            meshes,
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
            false,
            None,
        );
    }
}

pub fn rebuild_world_with_runtime_animations_for_debug_state(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    world: &WorldBlocks,
    assets: &WorldRenderAssets,
    animations: &HashMap<IVec3, BlockAnimation>,
    pusher_animations: &HashMap<IVec3, PusherAnimation>,
    timing: AnimationTiming,
    debug: &DebugState,
    structure_state: &StructureState,
    powered_wires: &HashSet<IVec3>,
) {
    if debug.factory_activity {
        rebuild_world_with_factory_activity_debug(commands, meshes, world, assets, structure_state);
    } else {
        rebuild_world_with_runtime_animations(
            commands,
            meshes,
            world,
            assets,
            animations,
            pusher_animations,
            timing,
            powered_wires,
        );
    }
}

fn block_render_material(
    assets: &WorldRenderAssets,
    data: BlockData,
    powered_wire: bool,
) -> Handle<StandardMaterial> {
    if powered_wire && data.kind == BlockKind::Wire {
        assets.active_wire_material.clone()
    } else {
        assets.block_material(data.kind)
    }
}

fn scene_block_mesh(pos: IVec3) -> Mesh {
    let min = Vec3::splat(-0.5);
    let max = Vec3::splat(0.5);
    let world_min = pos.as_vec3();
    let world_max = world_min + Vec3::ONE;
    let faces = [
        (
            [
                [min.x, min.y, max.z],
                [max.x, min.y, max.z],
                [max.x, max.y, max.z],
                [min.x, max.y, max.z],
            ],
            [0.0, 0.0, 1.0],
            [
                [world_min.x, world_min.y],
                [world_max.x, world_min.y],
                [world_max.x, world_max.y],
                [world_min.x, world_max.y],
            ],
        ),
        (
            [
                [max.x, min.y, min.z],
                [min.x, min.y, min.z],
                [min.x, max.y, min.z],
                [max.x, max.y, min.z],
            ],
            [0.0, 0.0, -1.0],
            [
                [world_max.x, world_min.y],
                [world_min.x, world_min.y],
                [world_min.x, world_max.y],
                [world_max.x, world_max.y],
            ],
        ),
        (
            [
                [max.x, min.y, max.z],
                [max.x, min.y, min.z],
                [max.x, max.y, min.z],
                [max.x, max.y, max.z],
            ],
            [1.0, 0.0, 0.0],
            [
                [world_max.z, world_min.y],
                [world_min.z, world_min.y],
                [world_min.z, world_max.y],
                [world_max.z, world_max.y],
            ],
        ),
        (
            [
                [min.x, min.y, min.z],
                [min.x, min.y, max.z],
                [min.x, max.y, max.z],
                [min.x, max.y, min.z],
            ],
            [-1.0, 0.0, 0.0],
            [
                [world_min.z, world_min.y],
                [world_max.z, world_min.y],
                [world_max.z, world_max.y],
                [world_min.z, world_max.y],
            ],
        ),
        (
            [
                [min.x, max.y, max.z],
                [max.x, max.y, max.z],
                [max.x, max.y, min.z],
                [min.x, max.y, min.z],
            ],
            [0.0, 1.0, 0.0],
            [
                [world_min.x, world_max.z],
                [world_max.x, world_max.z],
                [world_max.x, world_min.z],
                [world_min.x, world_min.z],
            ],
        ),
        (
            [
                [min.x, min.y, min.z],
                [max.x, min.y, min.z],
                [max.x, min.y, max.z],
                [min.x, min.y, max.z],
            ],
            [0.0, -1.0, 0.0],
            [
                [world_min.x, world_min.z],
                [world_max.x, world_min.z],
                [world_max.x, world_max.z],
                [world_min.x, world_max.z],
            ],
        ),
    ];

    let mut positions = Vec::with_capacity(24);
    let mut normals = Vec::with_capacity(24);
    let mut uvs = Vec::with_capacity(24);
    let mut indices = Vec::with_capacity(36);
    for (face_index, (face_positions, normal, face_uvs)) in faces.into_iter().enumerate() {
        let base = (face_index * 4) as u32;
        positions.extend_from_slice(&face_positions);
        normals.extend_from_slice(&[normal; 4]);
        uvs.extend_from_slice(&face_uvs);
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_indices(Indices::U32(indices))
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
}

fn render_rotation(data: BlockData, facing: crate::game::world::direction::Facing) -> Quat {
    if data.kind.is_directional() {
        Quat::from_rotation_y(facing.yaw())
    } else {
        Quat::IDENTITY
    }
}

fn spawn_block_model(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    assets: &WorldRenderAssets,
    world: &WorldBlocks,
    pos: IVec3,
    data: BlockData,
    material: Handle<StandardMaterial>,
    edit_preview: Option<EditPreview>,
    animation: Option<BlockAnimation>,
    pusher_animation: Option<PusherAnimation>,
    timing: AnimationTiming,
    with_block_entity: bool,
    pending_generated_preview: bool,
    show_generator_preview: bool,
    icon_render: Option<(Vec3, &RenderLayers)>,
) {
    let mut transform = Transform::from_translation(grid_to_world(pos));
    if let Some(animation) = animation {
        let progress = animation.progress.unwrap_or(0.0).clamp(0.0, 1.0);
        let eased = match timing.easing {
            AnimationEasing::Linear => progress,
            AnimationEasing::SmoothStep => progress * progress * (3.0 - 2.0 * progress),
        };
        transform.translation = match animation.kind {
            BlockAnimationKind::Rotate { pivot, clockwise } => {
                rotate_world_pos_y(grid_to_world(animation.from_pos), pivot, clockwise, eased)
            }
            _ => grid_to_world(animation.from_pos).lerp(grid_to_world(animation.to_pos), eased),
        };
        transform.rotation = render_rotation(data, animation.from_facing)
            .slerp(render_rotation(data, animation.to_facing), eased);
        transform.scale = match animation.kind {
            BlockAnimationKind::Move | BlockAnimationKind::Rotate { .. } => Vec3::ONE,
            BlockAnimationKind::SpawnScale => Vec3::splat(eased),
        };
    } else {
        transform.rotation = render_rotation(data, data.facing);
    }
    if let Some((origin, _)) = icon_render {
        transform.translation += origin;
    }

    let is_preview = edit_preview.is_some();
    let mut entity = if data.kind == crate::game::blocks::BlockKind::Wire
        || matches!(data.kind.model(), BlockModel::PartsOnly(_))
    {
        commands.spawn((transform, Visibility::default()))
    } else if data.kind == BlockKind::Platform {
        commands.spawn((
            Mesh3d(assets.block_mesh(data.kind)),
            MeshMaterial3d(if is_preview {
                assets.model_preview_material(crate::game::blocks::ModelMaterial::Platform)
            } else {
                assets.model_material(crate::game::blocks::ModelMaterial::Platform)
            }),
            transform,
        ))
    } else if let Some(scene_material) = assets.scene_material(data.kind) {
        let mesh = if icon_render.is_some() {
            assets.block_mesh(data.kind)
        } else {
            meshes.add(scene_block_mesh(pos))
        };
        commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(if is_preview {
                material.clone()
            } else {
                scene_material
            }),
            transform,
        ))
    } else {
        commands.spawn((
            Mesh3d(assets.block_mesh(data.kind)),
            MeshMaterial3d(material.clone()),
            transform,
        ))
    };

    if let Some((_, icon_layer)) = icon_render {
        entity.insert((
            icon_layer.clone(),
            BlockIconRenderEntity,
            BlockIconRenderRoot,
        ));
    }

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
        if let Some((_, icon_layer)) = icon_render {
            model_root.insert((icon_layer.clone(), BlockIconRenderEntity));
        }
        model_root.with_children(|parent| {
            spawn_model_parts(
                parent,
                assets,
                data,
                pusher_animation,
                icon_render.map(|(_, layer)| layer),
                is_preview,
            );
        });

        let render_behavior = data.kind.render_behavior(data.facing);

        if render_behavior.goal_topper {
            let mut child = parent.spawn((
                Mesh3d(assets.goal_top.clone()),
                MeshMaterial3d(assets.goal_top_material.clone()),
                Transform::from_xyz(0.0, 0.55, 0.0),
            ));
            if let Some((_, icon_layer)) = icon_render {
                child.insert((icon_layer.clone(), BlockIconRenderEntity));
            }
        }

        if let Some(weld_connector) = render_behavior.weld_connector {
            let offsets = match weld_connector {
                WeldConnectorBehavior::AllSides => signal_offsets().to_vec(),
                WeldConnectorBehavior::Offset(offset) => vec![offset],
            };
            for offset in offsets {
                let neighbor = pos + offset;
                if weld_neighbor_connects_to(world, neighbor, -offset) {
                    let local_offset = local_connector_offset(data, offset);
                    let mut child = parent.spawn((
                        Mesh3d(assets.connector_mesh(local_offset)),
                        MeshMaterial3d(assets.weld_connector_material.clone()),
                        Transform::from_translation(local_offset.as_vec3() * 0.225),
                    ));
                    if let Some((_, icon_layer)) = icon_render {
                        child.insert((icon_layer.clone(), BlockIconRenderEntity));
                    }
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
                    let mut child = parent.spawn((
                        Mesh3d(assets.wire_connector_mesh(local_offset)),
                        MeshMaterial3d(if data.kind == BlockKind::Wire {
                            material.clone()
                        } else {
                            assets.wire_connector_material.clone()
                        }),
                        Transform::from_translation(local_offset.as_vec3() * 0.174),
                    ));
                    if let Some((_, icon_layer)) = icon_render {
                        child.insert((icon_layer.clone(), BlockIconRenderEntity));
                    }
                }
            }

            if data.kind == crate::game::blocks::BlockKind::Wire && connected_offsets.is_empty() {
                let mut child =
                    parent.spawn((Mesh3d(assets.wire_node_mesh()), MeshMaterial3d(material)));
                if let Some((_, icon_layer)) = icon_render {
                    child.insert((icon_layer.clone(), BlockIconRenderEntity));
                }
            }
        }

        if data.kind.is_material() {
            for (face, mark) in world
                .material_face_marks
                .iter()
                .filter(|(face, _)| face.pos == pos)
            {
                let mut child = parent.spawn((
                    Mesh3d(assets.face_mark.clone()),
                    MeshMaterial3d(assets.face_mark_material(mark.color)),
                    face_mark_transform(face.normal),
                ));
                if let Some((_, icon_layer)) = icon_render {
                    child.insert((icon_layer.clone(), BlockIconRenderEntity));
                }
            }
        }

        if show_generator_preview && data.kind == BlockKind::Generator {
            spawn_generator_material_preview(
                parent,
                assets,
                world.generator_settings(pos).material,
                icon_render,
            );
        }
    });
}

fn spawn_generator_material_preview(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldRenderAssets,
    material: crate::game::blocks::MaterialKind,
    icon_render: Option<(Vec3, &RenderLayers)>,
) {
    let Some(kind) = BlockKind::material_block_kind(material) else {
        return;
    };

    let mut child = parent.spawn((
        Mesh3d(assets.block_mesh(kind)),
        MeshMaterial3d(assets.block_material(kind)),
        Transform {
            rotation: Quat::from_euler(
                EulerRot::XYZ,
                std::f32::consts::FRAC_PI_4,
                std::f32::consts::FRAC_PI_4,
                std::f32::consts::FRAC_PI_4,
            ),
            scale: Vec3::splat(0.38),
            ..default()
        },
    ));
    if let Some((_, icon_layer)) = icon_render {
        child.insert((icon_layer.clone(), BlockIconRenderEntity));
    }
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
    pusher_animation: Option<PusherAnimation>,
    icon_layer: Option<&RenderLayers>,
    preview: bool,
) {
    use crate::game::blocks::pusher::model::{
        pusher_rod_center_z, pusher_rod_length, ROD_BASE_LENGTH,
    };

    let parts = match data.kind.model() {
        BlockModel::Default => &[],
        BlockModel::Parts(parts) => parts,
        BlockModel::PartsOnly(parts) => parts,
    };
    let is_pusher_model = matches!(data.kind, BlockKind::Pusher | BlockKind::Blocker);

    for part in parts {
        let mut translation = model_vec3(part.translation);
        let mut scale = model_vec3(part.scale);

        if is_pusher_model {
            if part.mesh == crate::game::blocks::ModelMesh::PusherHead {
                if let Some(animation) = pusher_animation {
                    translation += Vec3::NEG_Z * animation.to_extension;
                }
            } else if part.mesh == crate::game::blocks::ModelMesh::RodZ {
                let extension = pusher_animation
                    .map(|animation| animation.to_extension)
                    .unwrap_or(0.0);
                let length = pusher_rod_length(extension);
                translation.z = pusher_rod_center_z(extension);
                scale = Vec3::new(scale.x, scale.y, length / ROD_BASE_LENGTH);
            }
        }

        let mut child = parent.spawn((
            Mesh3d(assets.model_mesh(part.mesh)),
            MeshMaterial3d(if preview {
                assets.model_preview_material(part.material)
            } else {
                assets.model_material(part.material)
            }),
            Transform {
                translation,
                rotation: Quat::from_rotation_y(part.yaw_radians),
                scale,
                ..default()
            },
        ));
        if let Some(icon_layer) = icon_layer {
            child.insert((icon_layer.clone(), BlockIconRenderEntity));
        }
        if part.mesh == crate::game::blocks::ModelMesh::PusherHead {
            if let Some(animation) = pusher_animation.filter(|animation| {
                animation.duration > 0.0 && animation.from_extension != animation.to_extension
            }) {
                child.insert(AnimatedPusher::new(animation, model_vec3(part.translation)));
            }
        } else if part.mesh == crate::game::blocks::ModelMesh::RodZ {
            if is_pusher_model {
                if let Some(animation) = pusher_animation.filter(|animation| {
                    animation.duration > 0.0 && animation.from_extension != animation.to_extension
                }) {
                    child.insert(AnimatedPusherRod::new(animation, model_vec3(part.scale)));
                }
            }
        }
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

fn weld_neighbor_connects_to(
    world: &WorldBlocks,
    neighbor: IVec3,
    connector_from_block: IVec3,
) -> bool {
    if let Some(block) = world.system_blocks.get(&neighbor) {
        return weld_connects_to(block, connector_from_block);
    }

    world
        .blocks
        .get(&neighbor)
        .is_some_and(|block| weld_connects_to(block, connector_from_block))
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
