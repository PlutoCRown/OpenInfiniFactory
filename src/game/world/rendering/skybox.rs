//! 天空盒：参考 Shadertoy Enscape Cube（4dSBDt）的天空与体积云

use bevy::camera::Camera3d;
use bevy::light::NotShadowCaster;
use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::pbr::{Material, MaterialPipeline, MaterialPipelineKey, MaterialPlugin};
use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::{
    AsBindGroup, CompareFunction, Face, RenderPipelineDescriptor, ShaderType,
    SpecializedMeshPipelineError,
};
use bevy::shader::ShaderRef;

use super::components::GameplayScene;
use crate::game::cameras::{GameplayCamera, MENU_CLEAR};
use crate::shared::config::GameConfig;

/// 天空穹顶半边长
const SKY_HALF_EXTENT: f32 = 800.0;

/// 天空盒材质与跟随相机
pub struct SkyboxPlugin;

impl Plugin for SkyboxPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<SkyMaterial>::default())
            .add_systems(Update, (sync_sky_to_camera, sync_skybox_settings));
    }
}

/// 跟随游玩相机的天空立方体
#[derive(Component)]
struct SkyDome;

/// 与场景平行光一致的旋转
pub fn sunlight_rotation() -> Quat {
    Quat::from_euler(EulerRot::XYZ, -1.05, -0.55, -0.28)
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
pub(crate) struct SkyMaterial {
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
) {
    let sun_dir = (sunlight_rotation() * Vec3::Z).normalize();
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

/// 把配置里的天空盒开关同步到穹顶可见性与相机清屏色
pub fn sync_skybox_settings(
    config: Res<GameConfig>,
    mut sky: Query<&mut Visibility, With<SkyDome>>,
    mut cameras: Query<&mut Camera, With<GameplayCamera>>,
) {
    if !config.is_changed() {
        return;
    }
    let visibility = if config.skybox_enabled {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    let clear = if config.skybox_enabled {
        ClearColorConfig::Custom(Color::BLACK)
    } else {
        ClearColorConfig::Custom(MENU_CLEAR)
    };
    for mut vis in &mut sky {
        *vis = visibility;
    }
    for mut camera in &mut cameras {
        camera.clear_color = clear;
    }
}
