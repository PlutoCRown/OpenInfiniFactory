//! 验收器游玩态：半透明目标材料 + 自下而上扫光

use bevy::asset::RenderAssetUsages;
use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::pbr::{Material, MaterialPipeline, MaterialPipelineKey, MaterialPlugin};
use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::{
    AsBindGroup, Extent3d, RenderPipelineDescriptor, ShaderType, SpecializedMeshPipelineError,
    TextureDimension, TextureFormat,
};
use bevy::shader::ShaderRef;

/// 验收器幽灵材质插件
pub struct GoalGhostPlugin;

impl Plugin for GoalGhostPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<GoalGhostMaterial>::default());
        super::goal_visual_sync::register_goal_visual_systems(app);
    }
}

/// 与 WGSL `GoalGhostUniform` 对齐
#[derive(Clone, Copy, Debug, ShaderType)]
pub struct GoalGhostUniform {
    pub base_color: Vec4,
    pub sweep_color: Vec4,
    /// x: 扫光速度（圈/秒）
    pub params: Vec4,
}

/// 半透明目标材料 + 扫光
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct GoalGhostMaterial {
    #[uniform(0)]
    pub uniform: GoalGhostUniform,
    #[texture(1)]
    #[sampler(2)]
    pub base_color_texture: Handle<Image>,
}

impl Material for GoalGhostMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/goal_ghost_material.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }

    fn depth_bias(&self) -> f32 {
        super::depth_bias::GOAL_GHOST
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
        if let Some(depth) = &mut descriptor.depth_stencil {
            depth.depth_write_enabled = Some(false);
        }
        Ok(())
    }
}

/// 由标准材质生成验收器幽灵材质
pub(crate) fn goal_ghost_from_standard(
    standard: &StandardMaterial,
    white: &Handle<Image>,
) -> GoalGhostMaterial {
    let c = standard.base_color.to_linear();
    GoalGhostMaterial {
        uniform: GoalGhostUniform {
            base_color: Vec4::new(c.red, c.green, c.blue, 0.42),
            // 柔和高光，避免过曝发白
            sweep_color: Vec4::new(0.22, 0.26, 0.30, 1.0),
            // x: 世界单位/秒；y: 约 2.2 格一道光；z: 光带半宽（相对周期）
            params: Vec4::new(0.35, 2.2, 0.07, 0.0),
        },
        base_color_texture: standard
            .base_color_texture
            .clone()
            .unwrap_or_else(|| white.clone()),
    }
}

/// 1×1 白贴图，无材料贴图时回退
pub(crate) fn white_pixel_image() -> Image {
    Image::new_fill(
        Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[255, 255, 255, 255],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}
