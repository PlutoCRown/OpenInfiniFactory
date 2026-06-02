use bevy::asset::RenderAssetUsages;
use bevy::camera::visibility::RenderLayers;
use bevy::camera::{RenderTarget, ScalingMode};
use bevy::light::CascadeShadowConfigBuilder;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;
use std::collections::{HashMap, HashSet};

use crate::game::simulation::factory_activity::{
    FactoryActivity, FactoryStructureState, StructureKind,
};
use crate::game::systems::debug::DebugState;
use crate::game::world::animation::{
    rotate_world_pos_y, AnimatedBlock, AnimatedPusher, AnimationEasing, AnimationTiming,
    BlockAnimation, BlockAnimationKind, PusherAnimation, WeldSpark,
};
use crate::game::world::blocks::{
    edit_blocks, generator_settings, local_connection_offset, six_way_offsets,
    wire_connector_render_plan, BlockData, BlockKind, BlockModel, BlockRenderSpec,
    WeldConnectorBehavior, PLAY_BLOCKS,
};
use crate::game::world::grid::{grid_to_world, WorldBlocks};
pub use crate::game::world::render_manager::WorldRenderManager;
use crate::game::world::selection_overlays::debug::{
    ActiveFactoryDebugOverlay, InactiveFactoryDebugOverlay, MaterialDebugOverlay,
};
use crate::game::world::selection_overlays::placement::PlacementOverlay;
use crate::game::world::selection_overlays::SelectionOverlayDefinition;

const ICON_TEXTURE_SIZE: u32 = 256;
const ICON_RENDER_LAYER: usize = 3;
const ICON_SPACING: f32 = 4.0;
const ICON_RENDER_FRAMES: u8 = 3;

#[derive(Component)]
pub struct BlockEntity {
    pub pos: IVec3,
}

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
pub struct SelectionOverlay;

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
            shadow_maps_enabled: true,
            range: 18.0,
            radius: 3.5,
            ..default()
        },
        Transform::from_xyz(3.5, 5.5, 4.5),
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 9500.0,
            shadow_maps_enabled: true,
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

    let render_manager = WorldRenderManager::new(&mut meshes, &mut materials, &mut images);

    let marker_mesh = meshes.add(Cuboid::new(0.92, 0.018, 0.92));
    let marker_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.78, 0.96, 1.0, 0.46),
        emissive: Color::srgb(0.04, 0.22, 0.26).into(),
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
    let preview_material = render_manager.selection_overlay_material::<PlacementOverlay>();

    commands.spawn((
        Mesh3d(preview_mesh),
        MeshMaterial3d(preview_material),
        Visibility::Hidden,
        PlacementPreview,
    ));

    commands.insert_resource(render_manager);
}

pub fn setup_block_icons(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    assets: Res<WorldRenderManager>,
) {
    let icon_layer = RenderLayers::layer(ICON_RENDER_LAYER);
    let mut icon_assets = BlockIconAssets::default();
    let icon_world = WorldBlocks::default();
    let icon_kinds = block_icon_kinds();

    commands.spawn((
        DirectionalLight {
            illuminance: 7800.0,
            shadow_maps_enabled: false,
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
    assets: &WorldRenderManager,
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
    assets: &WorldRenderManager,
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
    assets: &WorldRenderManager,
    factory_structures: &FactoryStructureState,
) {
    for (pos, data) in &world.blocks {
        if data.kind.is_factory() {
            let material = match factory_structures.activity_at(*pos) {
                Some(FactoryActivity::Inactive) => {
                    assets.selection_overlay_material::<InactiveFactoryDebugOverlay>()
                }
                _ => assets.selection_overlay_material::<ActiveFactoryDebugOverlay>(),
            };
            spawn_debug_factory_block(commands, assets, *pos, material);
        } else if data.kind.is_material() {
            spawn_debug_factory_block(
                commands,
                assets,
                *pos,
                assets.selection_overlay_material::<MaterialDebugOverlay>(),
            );
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
    assets: &WorldRenderManager,
    debug: &DebugState,
    factory_structures: &FactoryStructureState,
) {
    if debug.factory_activity {
        rebuild_world_with_factory_activity_debug(
            commands,
            meshes,
            world,
            assets,
            factory_structures,
        );
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
    assets: &WorldRenderManager,
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

pub fn despawn_selection_overlays(
    commands: &mut Commands,
    previews: &Query<Entity, With<SelectionOverlay>>,
) {
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

pub fn spawn_weld_sparks(
    commands: &mut Commands,
    assets: &WorldRenderManager,
    positions: &[IVec3],
) {
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

pub fn spawn_selection_overlay<T: SelectionOverlayDefinition>(
    commands: &mut Commands,
    assets: &WorldRenderManager,
    pos: IVec3,
) {
    commands.spawn((
        Mesh3d(assets.block.clone()),
        MeshMaterial3d(assets.selection_overlay_material::<T>()),
        Transform::from_translation(grid_to_world(pos)).with_scale(Vec3::splat(1.03)),
        SelectionOverlay,
    ));
}

pub fn spawn_block_preview(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    assets: &WorldRenderManager,
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
        Some(SelectionOverlay),
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
    assets: &WorldRenderManager,
    world: &WorldBlocks,
    pos: IVec3,
    data: BlockData,
) {
    spawn_block_with_animation(commands, meshes, assets, world, pos, data, None);
}

pub fn spawn_block_with_animation(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    assets: &WorldRenderManager,
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
    assets: &WorldRenderManager,
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
    assets: &WorldRenderManager,
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
    assets: &WorldRenderManager,
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
    assets: &WorldRenderManager,
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
    assets: &WorldRenderManager,
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
    assets: &WorldRenderManager,
    animations: &HashMap<IVec3, BlockAnimation>,
    pusher_animations: &HashMap<IVec3, PusherAnimation>,
    timing: AnimationTiming,
    debug: &DebugState,
    factory_structures: &FactoryStructureState,
    powered_wires: &HashSet<IVec3>,
) {
    if debug.factory_activity {
        rebuild_world_with_factory_activity_debug(
            commands,
            meshes,
            world,
            assets,
            factory_structures,
        );
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
    assets: &WorldRenderManager,
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
    assets: &WorldRenderManager,
    world: &WorldBlocks,
    pos: IVec3,
    data: BlockData,
    material: Handle<StandardMaterial>,
    selection_overlay: Option<SelectionOverlay>,
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

    let render_spec = assets.block_render_spec(data.kind, data.facing);

    let mut entity = if matches!(render_spec.model, BlockModel::PartsOnly(_)) {
        commands.spawn((transform, Visibility::default()))
    } else if data.kind == BlockKind::Platform {
        commands.spawn((
            Mesh3d(assets.block_mesh_for_spec(render_spec)),
            MeshMaterial3d(
                assets.model_material(crate::game::world::blocks::ModelMaterial::Platform),
            ),
            transform,
        ))
    } else if let Some(scene_material) = assets.scene_material(data.kind) {
        let mesh = if icon_render.is_some() {
            assets.block_mesh(data.kind)
        } else {
            meshes.add(scene_block_mesh(pos))
        };
        commands.spawn((Mesh3d(mesh), MeshMaterial3d(scene_material), transform))
    } else {
        commands.spawn((
            Mesh3d(assets.block_mesh_for_spec(render_spec)),
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

    let is_selection_overlay = selection_overlay.is_some();
    if let Some(selection_overlay) = selection_overlay {
        entity.insert(selection_overlay);
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
                render_spec,
                is_selection_overlay
                    .then(|| material.clone())
                    .filter(|_| icon_render.is_none()),
                pusher_animation,
                icon_render.map(|(_, layer)| layer),
            );
        });

        let render_behavior = render_spec.behavior;

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
                if weld_neighbor_connects_to(assets, world, neighbor, -offset) {
                    let local_offset = local_connection_offset(data, offset);
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

        if let Some(wire_plan) = wire_connector_render_plan(data, pos, world) {
            for local_offset in wire_plan.local_connector_offsets {
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

            if wire_plan.isolated_wire_node {
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
                generator_settings(&world, pos).material,
                icon_render,
            );
        }
    });
}

fn spawn_generator_material_preview(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldRenderManager,
    material: crate::game::world::blocks::MaterialKind,
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
    assets: &WorldRenderManager,
    render_spec: BlockRenderSpec,
    override_material: Option<Handle<StandardMaterial>>,
    pusher_animation: Option<PusherAnimation>,
    icon_layer: Option<&RenderLayers>,
) {
    let parts = match render_spec.model {
        BlockModel::Default => &[],
        BlockModel::Parts(parts) => parts,
        BlockModel::PartsOnly(parts) => parts,
    };

    for part in parts {
        let mut child = parent.spawn((
            Mesh3d(assets.model_mesh(part.mesh)),
            MeshMaterial3d(
                override_material
                    .clone()
                    .unwrap_or_else(|| assets.model_material(part.material)),
            ),
            Transform {
                translation: model_vec3(part.translation),
                rotation: Quat::from_rotation_y(part.yaw_radians),
                scale: model_vec3(part.scale),
                ..default()
            },
        ));
        if let Some(icon_layer) = icon_layer {
            child.insert((icon_layer.clone(), BlockIconRenderEntity));
        }
        if matches!(
            part.mesh,
            crate::game::world::blocks::ModelMesh::PusherHead
                | crate::game::world::blocks::ModelMesh::RodZ
        ) {
            if let Some(pusher_animation) = pusher_animation {
                child.insert(AnimatedPusher::new(
                    pusher_animation,
                    model_vec3(part.translation),
                ));
            }
        }
    }
}

fn model_vec3(value: [f32; 3]) -> Vec3 {
    Vec3::new(value[0], value[1], value[2])
}

fn weld_connects_to(
    render_manager: &WorldRenderManager,
    block: &BlockData,
    connector_from_block: IVec3,
) -> bool {
    match render_manager
        .block_render_spec(block.kind, block.facing)
        .behavior
        .weld_connector
    {
        Some(WeldConnectorBehavior::AllSides) => true,
        Some(WeldConnectorBehavior::Offset(offset)) => connector_from_block == offset,
        None => false,
    }
}

fn weld_neighbor_connects_to(
    render_manager: &WorldRenderManager,
    world: &WorldBlocks,
    neighbor: IVec3,
    connector_from_block: IVec3,
) -> bool {
    if let Some(block) = world.system_blocks.get(&neighbor) {
        return weld_connects_to(render_manager, block, connector_from_block);
    }

    world
        .blocks
        .get(&neighbor)
        .is_some_and(|block| weld_connects_to(render_manager, block, connector_from_block))
}

fn signal_offsets() -> [IVec3; 6] {
    six_way_offsets()
}
