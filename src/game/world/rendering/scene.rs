use bevy::light::CascadeShadowConfigBuilder;
use bevy::prelude::*;

use super::components::{AimFaceHighlight, GameplayScene, HoverMarker, PlacementPreview};
use super::goal_ghost::GoalGhostMaterial;
use super::skybox::{spawn_sky_dome, transform_for_sun_direction, SkyMaterial};
use crate::game::world::render_assets::WorldRenderAssets;
use crate::shared::save::PuzzleLighting;

/// 初始化游玩场景灯光、渲染资源与准星/预览实体
pub fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut ghost_materials: ResMut<Assets<GoalGhostMaterial>>,
    mut sky_materials: ResMut<Assets<SkyMaterial>>,
    mut images: ResMut<Assets<Image>>,
    scene_registry: Res<crate::game::scene_blocks::SceneBlockRegistry>,
    material_registry: Res<crate::game::material_blocks::MaterialBlockRegistry>,
    stamp_registry: Res<crate::game::material_blocks::StampMaterialRegistry>,
    paint_registry: Res<crate::game::material_blocks::PaintMaterialRegistry>,
    config: Res<crate::shared::config::GameConfig>,
    lighting: Res<PuzzleLighting>,
) {
    commands.spawn((
        PointLight {
            intensity: 1100.0,
            // 点光阴影是立方体贴图，主线程 Vis/Lights 成本很高；场景以平行光阴影为主
            shadow_maps_enabled: false,
            range: 18.0,
            radius: 3.5,
            ..default()
        },
        Transform::from_xyz(3.5, 5.5, 4.5),
        GameplayScene,
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: lighting.illuminance,
            color: lighting.color,
            shadow_maps_enabled: config.shadows_enabled,
            ..default()
        },
        transform_for_sun_direction(lighting.direction),
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

    spawn_sky_dome(
        &mut commands,
        &mut meshes,
        &mut sky_materials,
        config.skybox_enabled,
        &lighting,
    );

    let mut render_assets = WorldRenderAssets::new(
        &mut meshes,
        &mut materials,
        &mut images,
        &scene_registry,
        &material_registry,
        &stamp_registry,
        &paint_registry,
    );
    render_assets.install_goal_ghost_materials(&materials, &mut ghost_materials, &mut images);
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
        base_color: Color::srgba(0.72, 0.92, 1.0, 0.10),
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

/// 退出游玩时卸掉渲染相关资源
pub fn teardown_playing_scene(commands: &mut Commands) {
    commands.remove_resource::<WorldRenderAssets>();
    commands.remove_resource::<super::components::BlockIconAssets>();
    commands.remove_resource::<super::components::BlockIconRenderState>();
}

/// 把配置里的阴影开关同步到场景平行光
pub fn sync_shadow_settings(
    config: Res<crate::shared::config::GameConfig>,
    mut lights: Query<&mut DirectionalLight, With<GameplayScene>>,
) {
    if !config.is_changed() {
        return;
    }
    for mut light in &mut lights {
        if light.shadow_maps_enabled != config.shadows_enabled {
            light.shadow_maps_enabled = config.shadows_enabled;
        }
    }
}

/// 把配置里的垂直同步开关同步到主窗口
pub fn sync_vsync_settings(
    config: Res<crate::shared::config::GameConfig>,
    mut windows: Query<&mut Window, With<bevy::window::PrimaryWindow>>,
) {
    if !config.is_changed() {
        return;
    }
    let mode = if config.vsync_enabled {
        bevy::window::PresentMode::AutoVsync
    } else {
        bevy::window::PresentMode::AutoNoVsync
    };
    for mut window in &mut windows {
        if window.present_mode != mode {
            window.present_mode = mode;
        }
    }
}
