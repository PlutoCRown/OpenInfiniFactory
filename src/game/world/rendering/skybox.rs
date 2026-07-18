//! 天空盒：默认程序化 WGSL；存档有 skybox.png（水平十字）时用 Bevy Skybox 覆盖

use bevy::asset::RenderAssetUsages;
use bevy::camera::Camera3d;
use bevy::image::ImageSampler;
use bevy::light::{NotShadowCaster, Skybox};
use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::pbr::{Material, MaterialPipeline, MaterialPipelineKey, MaterialPlugin};
use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::{
    AsBindGroup, CompareFunction, Face, RenderPipelineDescriptor, ShaderType,
    SpecializedMeshPipelineError, TextureViewDescriptor, TextureViewDimension,
};
use bevy::shader::ShaderRef;

use super::components::GameplayScene;
use crate::game::cameras::{GameplayCamera, MENU_CLEAR};
use crate::shared::config::GameConfig;
use crate::shared::save::{PuzzleLighting, SaveSlot, SaveState};
use crate::shared::save_format::SKYBOX_FILE;
use crate::shared::persistent_storage;
use bevy::light::GlobalAmbientLight;

/// 天空穹顶半边长
const SKY_HALF_EXTENT: f32 = 800.0;

/// 贴图天空盒默认亮度（meta 未指定时）
const DEFAULT_IMAGE_SKYBOX_BRIGHTNESS: f32 = 1000.0;

/// 天空盒材质与跟随相机
pub struct SkyboxPlugin;

impl Plugin for SkyboxPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PuzzleLighting>()
            .add_plugins(MaterialPlugin::<SkyMaterial>::default())
            .add_systems(
                Update,
                (
                    sync_sky_to_camera,
                    sync_skybox_settings,
                    sync_puzzle_image_skybox,
                    sync_puzzle_lighting,
                ),
            );
    }
}

/// 跟随游玩相机的天空立方体
#[derive(Component)]
struct SkyDome;

/// 当前已应用到相机的贴图天空（puzzle 名）
#[derive(Default)]
struct AppliedImageSkybox {
    puzzle: Option<String>,
}

/// 与场景平行光一致的默认旋转
pub fn sunlight_rotation() -> Quat {
    Quat::from_euler(EulerRot::XYZ, -1.05, -0.55, -0.28)
}

/// 由方向向量得到平行光 Transform（局部 +Z 对齐光线前进方向）
pub fn transform_for_sun_direction(dir: Option<Vec3>) -> Transform {
    match dir {
        Some(dir) if dir != Vec3::ZERO => {
            Transform::from_rotation(Quat::from_rotation_arc(Vec3::Z, dir.normalize()))
        }
        _ => Transform::from_rotation(sunlight_rotation()),
    }
}

/// 解析后的光线前进方向（含默认）
pub fn resolved_sun_direction(dir: Option<Vec3>) -> Vec3 {
    match dir {
        Some(dir) if dir != Vec3::ZERO => dir.normalize(),
        _ => (sunlight_rotation() * Vec3::Z).normalize(),
    }
}

/// 着色器天空参数
#[derive(Clone, Copy, Debug, ShaderType)]
struct SkyUniform {
    sun_dir: Vec3,
    /// 线性曝光，配合场景色调映射（勿用 Skybox 的 cd/m² 大数）
    exposure: f32,
}

/// 天空盒自定义材质
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct SkyMaterial {
    #[uniform(0)]
    params: SkyUniform,
}

impl Material for SkyMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/sky_material.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Opaque
    }

    fn enable_prepass() -> bool {
        false
    }

    fn enable_shadows() -> bool {
        false
    }

    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = Some(Face::Front);
        if let Some(depth) = &mut descriptor.depth_stencil {
            depth.depth_compare = Some(CompareFunction::GreaterEqual);
            depth.depth_write_enabled = Some(false);
        }
        Ok(())
    }
}

/// 生成天空立方体（默认可见性由配置决定）
pub fn spawn_sky_dome(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<SkyMaterial>,
    enabled: bool,
    lighting: &PuzzleLighting,
) {
    let sun_dir = resolved_sun_direction(lighting.direction);
    let material = materials.add(SkyMaterial {
        params: SkyUniform {
            sun_dir,
            exposure: 0.055,
        },
    });

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::from_length(SKY_HALF_EXTENT * 2.0))),
        MeshMaterial3d(material),
        Transform::default(),
        if enabled {
            Visibility::Visible
        } else {
            Visibility::Hidden
        },
        NotShadowCaster,
        SkyDome,
        GameplayScene,
    ));
}

/// 天空中心跟随游玩相机
fn sync_sky_to_camera(
    camera: Query<&Transform, (With<GameplayCamera>, With<Camera3d>, Without<SkyDome>)>,
    mut sky: Query<&mut Transform, With<SkyDome>>,
) {
    let Ok(camera_tf) = camera.single() else {
        return;
    };
    for mut sky_tf in &mut sky {
        sky_tf.translation = camera_tf.translation;
    }
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

/// 存档 skybox.png → Bevy Skybox；没有则回退程序化穹顶
fn sync_puzzle_image_skybox(
    save_state: Res<SaveState>,
    config: Res<GameConfig>,
    lighting: Res<PuzzleLighting>,
    mut images: ResMut<Assets<Image>>,
    mut commands: Commands,
    cameras: Query<Entity, With<GameplayCamera>>,
    mut sky_domes: Query<&mut Visibility, With<SkyDome>>,
    mut applied: Local<AppliedImageSkybox>,
) {
    let puzzle = save_state.current.as_ref().map(|slot| slot.puzzle.clone());
    let want_sky = config.skybox_enabled;
    let skybox_brightness = if lighting.skybox_brightness > 0.0 {
        lighting.skybox_brightness
    } else {
        DEFAULT_IMAGE_SKYBOX_BRIGHTNESS
    };

    let Ok(camera) = cameras.single() else {
        applied.puzzle = None;
        return;
    };

    if !want_sky {
        if applied.puzzle.is_some() {
            commands.entity(camera).remove::<Skybox>();
            applied.puzzle = None;
        }
        for mut vis in &mut sky_domes {
            *vis = Visibility::Hidden;
        }
        return;
    }

    let image_bytes = puzzle.as_ref().and_then(|name| {
        let path = SaveSlot::puzzle(name).storage_path();
        persistent_storage::read_save_bytes(&path, SKYBOX_FILE)
    });

    match image_bytes {
        Some(bytes) => {
            if applied.puzzle == puzzle {
                for mut vis in &mut sky_domes {
                    *vis = Visibility::Hidden;
                }
                return;
            }
            match horizontal_cross_png_to_cubemap(&bytes) {
                Ok(image) => {
                    let handle = images.add(image);
                    commands.entity(camera).insert(Skybox {
                        image: Some(handle),
                        brightness: skybox_brightness,
                        ..default()
                    });
                    for mut vis in &mut sky_domes {
                        *vis = Visibility::Hidden;
                    }
                    applied.puzzle = puzzle;
                }
                Err(err) => {
                    bevy::log::warn!("skybox.png load failed: {err}");
                    commands.entity(camera).remove::<Skybox>();
                    for mut vis in &mut sky_domes {
                        *vis = Visibility::Visible;
                    }
                    applied.puzzle = None;
                }
            }
        }
        None => {
            if applied.puzzle.take().is_some() {
                commands.entity(camera).remove::<Skybox>();
            }
            for mut vis in &mut sky_domes {
                *vis = Visibility::Visible;
            }
        }
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

/// 把 PuzzleLighting 同步到平行光、环境光、贴图天空亮度与程序化天空
fn sync_puzzle_lighting(
    lighting: Res<PuzzleLighting>,
    config: Res<GameConfig>,
    mut lights: Query<
        (&mut Transform, &mut DirectionalLight),
        With<GameplayScene>,
    >,
    mut skyboxes: Query<&mut Skybox, With<GameplayCamera>>,
    sky_meshes: Query<&MeshMaterial3d<SkyMaterial>, With<SkyDome>>,
    mut sky_materials: ResMut<Assets<SkyMaterial>>,
    mut ambient: ResMut<GlobalAmbientLight>,
) {
    if !lighting.is_changed() && !config.is_changed() {
        return;
    }
    let tf = transform_for_sun_direction(lighting.direction);
    let dir = resolved_sun_direction(lighting.direction);
    for (mut light_tf, mut light) in &mut lights {
        *light_tf = tf;
        light.illuminance = lighting.illuminance;
        light.color = lighting.color;
        light.shadow_maps_enabled = config.shadows_enabled;
    }
    for mut skybox in &mut skyboxes {
        skybox.brightness = lighting.skybox_brightness;
    }
    for mat in &sky_meshes {
        if let Some(mut material) = sky_materials.get_mut(&mat.0) {
            material.params.sun_dir = dir;
        }
    }
    ambient.color = lighting.ambient_color;
    ambient.brightness = lighting.ambient_brightness;
}
