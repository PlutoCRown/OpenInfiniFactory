//! MC 风格传送门：半透明紫红立方体 + 中心螺旋流动；传送成功时自发光闪一下

use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::pbr::{Material, MaterialPipeline, MaterialPipelineKey, MaterialPlugin};
use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::{
    AsBindGroup, RenderPipelineDescriptor, ShaderType, SpecializedMeshPipelineError,
};
use bevy::shader::ShaderRef;
use std::collections::HashSet;

/// 传送门材质与闪烁同步插件
pub struct PortalMaterialPlugin;

impl Plugin for PortalMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PortalFlashQueue>()
            .add_plugins(MaterialPlugin::<PortalMaterial>::default())
            .add_systems(
                Update,
                (
                    uniquify_portal_materials,
                    apply_portal_flash_queue,
                    decay_portal_flash,
                )
                    .chain(),
            );
    }
}

/// 待闪烁的传送口格（表现层写入，本插件消费）
#[derive(Resource, Default)]
pub struct PortalFlashQueue {
    pub positions: Vec<IVec3>,
}

/// 与 WGSL `PortalUniform` 对齐
#[derive(Clone, Copy, Debug, ShaderType)]
pub struct PortalUniform {
    pub base_color: Vec4,
    pub flow_color: Vec4,
    /// x: 转速；y: 径向密度；z: 臂数；w: 闪烁
    pub params: Vec4,
}

/// 半透明螺旋传送门
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct PortalMaterial {
    #[uniform(0)]
    pub uniform: PortalUniform,
}

/// 传送门视觉实体（pos 用于定位闪烁）
#[derive(Component, Clone, Copy, Debug)]
pub struct TeleportPortalVisual {
    pub pos: IVec3,
}

impl Material for PortalMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/portal_material.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }

    fn depth_bias(&self) -> f32 {
        super::depth_bias::PORTAL
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
        // 与玻璃一样只渲正面（默认 back-face cull）
        if let Some(depth) = &mut descriptor.depth_stencil {
            depth.depth_write_enabled = Some(false);
            depth.bias.constant = super::depth_bias::PORTAL as i32;
        }
        Ok(())
    }
}

/// 默认深紫传送门色（比先前更深、螺旋更疏）
pub fn default_portal_material() -> PortalMaterial {
    PortalMaterial {
        uniform: PortalUniform {
            base_color: Vec4::new(0.22, 0.02, 0.42, 0.62),
            flow_color: Vec4::new(0.55, 0.12, 0.85, 0.95),
            // 转速 / 径向密度（疏） / 螺旋臂数 / 闪烁
            params: Vec4::new(0.55, 2.2, 2.0, 0.0),
        },
    }
}

/// 把共享模板换成每扇门独立材质，才能单独闪
fn uniquify_portal_materials(
    mut commands: Commands,
    mut materials: ResMut<Assets<PortalMaterial>>,
    assets: Option<Res<crate::game::world::render_assets::WorldRenderAssets>>,
    added: Query<(Entity, &MeshMaterial3d<PortalMaterial>), Added<TeleportPortalVisual>>,
) {
    let Some(assets) = assets else {
        return;
    };
    let shared_id = assets.portal_material_handle().id();
    let Some(template) = materials.get(&assets.portal_material_handle()).cloned() else {
        return;
    };
    for (entity, handle) in &added {
        if handle.id() != shared_id {
            continue;
        }
        let unique = materials.add(template.clone());
        commands.entity(entity).insert(MeshMaterial3d(unique));
    }
}

/// 消费闪烁队列，点亮对应传送口
fn apply_portal_flash_queue(
    mut queue: ResMut<PortalFlashQueue>,
    portals: Query<(&TeleportPortalVisual, &MeshMaterial3d<PortalMaterial>)>,
    mut materials: ResMut<Assets<PortalMaterial>>,
) {
    if queue.positions.is_empty() {
        return;
    }
    let targets: HashSet<IVec3> = queue.positions.drain(..).collect();
    for (visual, handle) in &portals {
        if !targets.contains(&visual.pos) {
            continue;
        }
        if let Some(mut material) = materials.get_mut(handle) {
            material.uniform.params.w = 1.0;
        }
    }
}

/// 闪烁强度随时间衰减
fn decay_portal_flash(
    time: Res<Time>,
    portals: Query<&MeshMaterial3d<PortalMaterial>, With<TeleportPortalVisual>>,
    mut materials: ResMut<Assets<PortalMaterial>>,
) {
    let step = time.delta_secs() * 2.8;
    for handle in &portals {
        let Some(mut material) = materials.get_mut(handle) else {
            continue;
        };
        let flash = material.uniform.params.w;
        if flash > 0.0 {
            material.uniform.params.w = (flash - step).max(0.0);
        }
    }
}
