//! 传送块世界视觉：半透明螺旋立方体

use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;

use crate::game::world::render_assets::WorldRenderAssets;
use crate::game::world::rendering::portal_material::TeleportPortalVisual;
use crate::game::world::rendering::BlockIconRenderEntity;

/// 生成传送门立方体（螺旋材质）
pub fn spawn_teleport_visual(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldRenderAssets,
    pos: IVec3,
    icon_layer: Option<&RenderLayers>,
) {
    let mut entity = parent.spawn((
        Mesh3d(assets.portal_cube_mesh()),
        MeshMaterial3d(assets.portal_material_handle()),
        Transform::from_scale(Vec3::splat(0.98)),
        TeleportPortalVisual { pos },
    ));
    if let Some(layer) = icon_layer {
        entity.insert((layer.clone(), BlockIconRenderEntity));
    }
}
