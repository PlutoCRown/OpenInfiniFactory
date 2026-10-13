//! 将会话的关卡光照同步到灯光、环境反射和天空。

use super::components::GameplayScene;
use super::procedural_sky::ProceduralSky;
use crate::game::cameras::GameplayCamera;
use crate::shared::config::GameConfig;
use crate::shared::save::PuzzleLighting;
use bevy::light::{EnvironmentMapLight, GlobalAmbientLight, Skybox};
use bevy::prelude::*;

/// 贴图天空亮度为零时使用统一默认值。
pub(super) fn image_skybox_brightness(lighting: &PuzzleLighting) -> f32 {
    if lighting.skybox_brightness > 0.0 {
        lighting.skybox_brightness
    } else {
        1000.0
    }
}

/// 与场景平行光一致的默认旋转
pub fn sunlight_rotation() -> Quat {
    Quat::from_euler(EulerRot::XYZ, -1.05, -0.55, -0.28)
}

/// 由方向向量得到平行光 Transform（局部 +Z 指向太阳）
pub fn transform_for_sun_direction(dir: Option<Vec3>) -> Transform {
    match dir {
        Some(dir) if dir != Vec3::ZERO => {
            Transform::from_rotation(Quat::from_rotation_arc(Vec3::Z, dir.normalize()))
        }
        _ => Transform::from_rotation(sunlight_rotation()),
    }
}

/// 解析后的太阳方向（含默认）。
pub fn resolved_sun_direction(dir: Option<Vec3>) -> Vec3 {
    match dir {
        Some(dir) if dir != Vec3::ZERO => dir.normalize(),
        _ => (sunlight_rotation() * Vec3::Z).normalize(),
    }
}

/// 环境贴图强度：给金属镜面反射，略低于环境光以免整景过曝
pub fn environment_map_intensity(lighting: &PuzzleLighting) -> f32 {
    (lighting.ambient_brightness * 0.6).clamp(100.0, 450.0)
}

/// 简易半球环境贴图（金属 PBR 需要可反射的环境，否则金属度几乎看不见）
pub fn environment_map_light(
    images: &mut Assets<Image>,
    lighting: &PuzzleLighting,
) -> EnvironmentMapLight {
    let mut light = EnvironmentMapLight::hemispherical_gradient(
        images,
        Color::srgb(0.42, 0.58, 0.95),
        lighting.ambient_color,
        Color::srgb(0.30, 0.28, 0.26),
    );
    light.intensity = environment_map_intensity(lighting);
    light
}

/// 把 PuzzleLighting 同步到平行光、环境光、贴图天空亮度与程序化天空
pub(super) fn sync_puzzle_lighting(
    lighting: Res<PuzzleLighting>,
    config: Res<GameConfig>,
    mut lights: Query<(&mut Transform, &mut DirectionalLight), With<GameplayScene>>,
    mut skyboxes: Query<&mut Skybox, With<GameplayCamera>>,
    mut env_maps: Query<&mut EnvironmentMapLight, With<GameplayCamera>>,
    mut procedural_skies: Query<&mut ProceduralSky>,
    mut ambient: ResMut<GlobalAmbientLight>,
) {
    if !lighting.is_changed() && !config.is_changed() {
        return;
    }
    let tf = transform_for_sun_direction(lighting.direction);
    for (mut light_tf, mut light) in &mut lights {
        *light_tf = tf;
        light.illuminance = lighting.illuminance;
        light.color = lighting.color;
        light.shadow_maps_enabled = config.shadows_enabled;
    }
    for mut skybox in &mut skyboxes {
        skybox.brightness = image_skybox_brightness(&lighting);
    }
    for mut env in &mut env_maps {
        env.intensity = environment_map_intensity(&lighting);
    }
    for mut sky in &mut procedural_skies {
        let time = sky.time;
        *sky = ProceduralSky::from_lighting(&lighting);
        sky.time = time;
    }
    ambient.color = lighting.ambient_color;
    ambient.brightness = lighting.ambient_brightness;
}
