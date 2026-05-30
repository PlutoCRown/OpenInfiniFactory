use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;

#[derive(Asset, AsBindGroup, Clone, Debug, TypePath)]
pub struct SceneBlockMaterial {
    #[uniform(0)]
    pub base_color: LinearRgba,
    #[uniform(0)]
    pub accent_color: LinearRgba,
    #[uniform(0)]
    pub texture_kind: u32,
}

impl Material for SceneBlockMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/scene_block_material.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        if self.base_color.alpha < 1.0 {
            AlphaMode::Blend
        } else {
            AlphaMode::Opaque
        }
    }
}
