//! 天空盒：存档贴图优先，否则启用相机上的低分辨率体积云通道。

use std::sync::Arc;

use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::light::Skybox;
use bevy::prelude::*;
use bevy::render::render_resource::{TextureViewDescriptor, TextureViewDimension};

use super::procedural_sky::{ProceduralSky, ProceduralSkyPlugin};
use crate::game::cameras::{GameplayCamera, MENU_CLEAR};
use crate::shared::config::GameConfig;
use crate::shared::persistent_storage;
use crate::shared::save::{PuzzleLighting, SaveSlot, SaveState};
use crate::shared::save_format::SKYBOX_FILE;

/// 根据存档贴图和设置选择相机的天空通道。
pub struct SkyboxPlugin;

impl Plugin for SkyboxPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ProceduralSkyPlugin).add_systems(
            Update,
            (sync_skybox_settings, sync_puzzle_image_skybox)
                .chain()
                .in_set(crate::game::schedule::GameSet::View)
                .before(super::lighting::sync_puzzle_lighting),
        );
    }
}

/// 当前已应用到相机的贴图天空（puzzle 名）
#[derive(Default)]
struct AppliedImageSkybox {
    puzzle: Option<String>,
    source: Option<Arc<[u8]>>,
    camera: Option<Entity>,
    enabled: Option<bool>,
}

/// 配置开关时更新相机清屏色
fn sync_skybox_settings(
    config: Res<GameConfig>,
    mut cameras: Query<&mut Camera, With<GameplayCamera>>,
) {
    if !config.is_changed() {
        return;
    }
    let clear = if config.skybox_enabled {
        ClearColorConfig::Custom(Color::BLACK)
    } else {
        ClearColorConfig::Custom(MENU_CLEAR)
    };
    for mut camera in &mut cameras {
        camera.clear_color = clear;
    }
}

/// 存档贴图优先；关闭天空或贴图可用时移除程序化通道。
fn sync_puzzle_image_skybox(
    save_state: Res<SaveState>,
    config: Res<GameConfig>,
    lighting: Res<PuzzleLighting>,
    mut images: ResMut<Assets<Image>>,
    mut commands: Commands,
    cameras: Query<Entity, With<GameplayCamera>>,
    mut applied: Local<AppliedImageSkybox>,
) {
    let puzzle = save_state.current.as_ref().map(|slot| slot.puzzle.clone());
    let want_sky = config.skybox_enabled;
    let skybox_brightness = super::lighting::image_skybox_brightness(&lighting);

    let Ok(camera) = cameras.single() else {
        *applied = AppliedImageSkybox::default();
        return;
    };
    let source = want_sky
        .then(|| {
            puzzle.as_ref().and_then(|name| {
                persistent_storage::read_save_asset(
                    &SaveSlot::puzzle(name).storage_path(),
                    SKYBOX_FILE,
                )
            })
        })
        .flatten();
    let same_source = match (&applied.source, &source) {
        (Some(old), Some(new)) => Arc::ptr_eq(old, new),
        (None, None) => true,
        _ => false,
    };
    if applied.camera == Some(camera)
        && applied.enabled == Some(want_sky)
        && applied.puzzle == puzzle
        && same_source
    {
        return;
    }
    // 失败也记录输入身份，内容更新前不逐帧重试损坏图片。
    applied.camera = Some(camera);
    applied.enabled = Some(want_sky);
    applied.puzzle = puzzle;
    applied.source = source.clone();
    let mut image_visible = false;
    if let Some(bytes) = source {
        match horizontal_cross_png_to_cubemap(&bytes) {
            Ok(image) => {
                commands.entity(camera).insert(Skybox {
                    image: Some(images.add(image)),
                    brightness: skybox_brightness,
                    ..default()
                });
                image_visible = true;
            }
            Err(err) => bevy::log::warn!("skybox.png load failed: {err}"),
        }
    }
    if !image_visible {
        commands.entity(camera).remove::<Skybox>();
    }
    if want_sky && !image_visible {
        commands
            .entity(camera)
            .insert(ProceduralSky::from_lighting(&lighting));
    } else {
        commands.entity(camera).remove::<ProceduralSky>();
    }
}

/// 水平十字 PNG（4×3）→ 竖直六面立方体贴图 Image
fn horizontal_cross_png_to_cubemap(png_bytes: &[u8]) -> Result<Image, String> {
    let rgba = image::load_from_memory(png_bytes)
        .map_err(|e| e.to_string())?
        .to_rgba8();
    let width = rgba.width();
    let height = rgba.height();
    if width % 4 != 0 || height % 3 != 0 || width / 4 != height / 3 {
        return Err(format!(
            "expected 4×3 horizontal cross cubemap, got {width}×{height}"
        ));
    }
    let face = width / 4;
    // 十字格 (col, row)：中行 -X +Z +X -Z；上 +Y；下 -Y
    // 竖直堆叠顺序：+X -X +Y -Y +Z -Z（Bevy / wgpu）
    let faces = [(2u32, 1), (0, 1), (1, 0), (1, 2), (1, 1), (3, 1)];
    let mut stacked = image::RgbaImage::new(face, face * 6);
    for (layer, &(col, row)) in faces.iter().enumerate() {
        let x0 = col * face;
        let y0 = row * face;
        for y in 0..face {
            for x in 0..face {
                stacked.put_pixel(x, layer as u32 * face + y, *rgba.get_pixel(x0 + x, y0 + y));
            }
        }
    }

    let mut image = Image::from_dynamic(
        image::DynamicImage::ImageRgba8(stacked),
        true,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::linear();
    image
        .reinterpret_stacked_2d_as_array(6)
        .map_err(|e| format!("{e:?}"))?;
    image.texture_view_descriptor = Some(TextureViewDescriptor {
        dimension: Some(TextureViewDimension::Cube),
        ..default()
    });
    Ok(image)
}

/// 验证天空开关和相机重建时的兜底通道生命周期。
#[cfg(test)]
mod tests {
    use super::*;

    /// 菜单开关与会话光照在同帧落地后，真实天空插件和光照同步再读取它们。
    #[test]
    fn menu_and_session_changes_reach_rendering_in_the_same_frame() {
        use super::super::components::GameplayScene;
        use crate::game::schedule::{GameSchedulePlugin, GameSet};
        let mut app = App::new();
        app.add_plugins(GameSchedulePlugin)
            .init_resource::<Time>()
            .init_resource::<GameConfig>()
            .init_resource::<SaveState>()
            .init_resource::<PuzzleLighting>()
            .init_resource::<Assets<Image>>()
            .init_resource::<bevy::light::GlobalAmbientLight>()
            .add_plugins(SkyboxPlugin)
            .add_systems(
                Update,
                super::super::lighting::sync_puzzle_lighting.in_set(GameSet::View),
            )
            .add_systems(
                Update,
                (|mut config: ResMut<GameConfig>| {
                    config.skybox_enabled = false;
                    config.shadows_enabled = false;
                })
                .in_set(GameSet::Menus),
            )
            .add_systems(
                Update,
                (|mut commands: Commands| {
                    commands.insert_resource(PuzzleLighting {
                        direction: Some(Vec3::Y),
                        illuminance: 1234.0,
                        ..default()
                    });
                })
                .in_set(GameSet::Session),
            );
        let camera = app
            .world_mut()
            .spawn((Camera::default(), GameplayCamera))
            .id();
        let light = app
            .world_mut()
            .spawn((
                DirectionalLight::default(),
                Transform::default(),
                GameplayScene,
            ))
            .id();
        app.update();
        assert!(app.world().get::<ProceduralSky>(camera).is_none());
        assert!(
            matches!(app.world().get::<Camera>(camera).unwrap().clear_color,
            ClearColorConfig::Custom(color) if color == MENU_CLEAR)
        );
        let light_data = app.world().get::<DirectionalLight>(light).unwrap();
        assert!(!light_data.shadow_maps_enabled);
        assert_eq!(light_data.illuminance, 1234.0);
        let sun = app.world().get::<Transform>(light).unwrap().rotation * Vec3::Z;
        assert!(sun.abs_diff_eq(Vec3::Y, 0.0001));
    }

    /// 无渲染世界时仍能按输入正确装卸相机上的程序化天空。
    #[test]
    fn procedural_fallback_tracks_toggle_and_recreated_camera() {
        let mut app = App::new();
        app.init_resource::<GameConfig>()
            .init_resource::<SaveState>()
            .init_resource::<PuzzleLighting>()
            .init_resource::<Assets<Image>>()
            .add_systems(Update, sync_puzzle_image_skybox);
        app.world_mut().resource_mut::<GameConfig>().skybox_enabled = true;
        let camera = app.world_mut().spawn(GameplayCamera).id();
        app.update();
        assert!(app.world().get::<ProceduralSky>(camera).is_some());
        app.world_mut().resource_mut::<GameConfig>().skybox_enabled = false;
        app.update();
        assert!(app.world().get::<ProceduralSky>(camera).is_none());
        app.world_mut().resource_mut::<GameConfig>().skybox_enabled = true;
        app.update();
        assert!(app.world().get::<ProceduralSky>(camera).is_some());
        app.world_mut().despawn(camera);
        let replacement = app.world_mut().spawn(GameplayCamera).id();
        app.update();
        assert!(app.world().get::<ProceduralSky>(replacement).is_some());
    }
}
