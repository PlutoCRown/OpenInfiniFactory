//! 阴影代理材质：主相机看不见，仍写入平行光阴影

use bevy::pbr::{Material, MaterialPlugin};
use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;

/// 阴影代理材质插件
pub struct ShadowProxyMaterialPlugin;

impl Plugin for ShadowProxyMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<ShadowProxyMaterial>::default());
    }
}

/// 主视角 discard；阴影 pass 走引擎默认深度
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, Default)]
pub struct ShadowProxyMaterial {}

impl Material for ShadowProxyMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/shadow_proxy_material.wgsl".into()
    }

    fn enable_prepass() -> bool {
        false
    }

    fn enable_shadows() -> bool {
        true
    }
}
