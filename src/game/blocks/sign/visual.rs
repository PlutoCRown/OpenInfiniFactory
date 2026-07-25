//! 告示牌世界视觉：墙贴板 / 立杆板 + 正面 icon

use std::f32::consts::PI;

use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;

use crate::game::blocks::{BlockData, BlockKind, ModelMaterial, ModelMesh};
use crate::game::world::grid::{SignDisplay, WorldBlocks};
use crate::game::world::render_assets::WorldRenderAssets;
use crate::game::world::rendering::BlockIconRenderEntity;

/// 板高
const BOARD_H: f32 = 0.6;
/// 板厚（局部 Z）
const BOARD_T: f32 = 0.05;
/// 墙贴：板心贴宿主面内侧
const WALL_BOARD_Z: f32 = 0.5 - BOARD_T * 0.5;
/// 立杆：板心偏上，落在格中心
const STANDING_BOARD_Y: f32 = 0.12;
/// 杆网格基准高度（缩放用）
const POLE_MESH_H: f32 = 0.5;
/// 正面 icon 相对板面外偏（靠 depth_bias 叠层）
const ICON_OUTSET: f32 = 0.002;
/// 正面 icon 边长（正方形，贴在板上）
const ICON_SIZE: f32 = 0.84;

/// 生成告示牌板/杆/正面展示 icon（`model()` 为空，专由此处画）
pub fn spawn_sign_visual(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldRenderAssets,
    world: &WorldBlocks,
    pos: IVec3,
    data: BlockData,
    icon_layer: Option<&RenderLayers>,
    preview: bool,
) {
    let mount = sign_mount_normal(world, pos, data).unwrap_or(IVec3::Y);
    // bake/离屏 icon 一律立杆样式；世界里按贴面
    let standing = icon_layer.is_some() || mount.y != 0;
    let board_translation = if standing {
        Vec3::new(0.0, STANDING_BOARD_Y, 0.0)
    } else {
        Vec3::new(0.0, 0.0, WALL_BOARD_Z)
    };

    spawn_wood_part(
        parent,
        assets,
        ModelMesh::SignBoard,
        board_translation,
        Vec3::ONE,
        icon_layer,
        preview,
    );

    if standing {
        // 杆从格底撑到板底；天花板附着时朝上
        let board_bottom = STANDING_BOARD_Y - BOARD_H * 0.5;
        let (pole_y, pole_h) = if mount.y > 0 || icon_layer.is_some() {
            let bottom = -0.5;
            let h = (board_bottom - bottom).max(0.08);
            (bottom + h * 0.5, h)
        } else {
            let top = 0.5;
            let board_top = STANDING_BOARD_Y + BOARD_H * 0.5;
            let h = (top - board_top).max(0.08);
            (board_top + h * 0.5, h)
        };
        spawn_wood_part(
            parent,
            assets,
            ModelMesh::SignPole,
            Vec3::new(0.0, pole_y, 0.0),
            Vec3::new(1.0, pole_h / POLE_MESH_H, 1.0),
            icon_layer,
            preview,
        );
    }

    if preview {
        return;
    }
    let Some(display) = world.sign_settings(pos).display else {
        return;
    };
    let kind = match display {
        SignDisplay::Material(id) => BlockKind::material_block_kind(id),
        SignDisplay::Stamp(id) => BlockKind::stamp_block_kind(id),
    };
    let Some(material) = assets.sign_display_material(kind) else {
        return;
    };
    // Rectangle 在 XY、法线 +Z；Y 转 π 后正面朝局部 -Z，等比缩放避免拉长
    let icon_tf = Transform {
        translation: board_translation + Vec3::NEG_Z * (BOARD_T * 0.5 + ICON_OUTSET),
        rotation: Quat::from_rotation_y(PI),
        scale: Vec3::new(ICON_SIZE, ICON_SIZE, 1.0),
    };

    let mut child = parent.spawn((
        Mesh3d(assets.sign_icon_mesh()),
        MeshMaterial3d(material),
        icon_tf,
    ));
    if let Some(icon_layer) = icon_layer {
        child.insert((icon_layer.clone(), BlockIconRenderEntity));
    }
}

/// 生成带木材贴图的零件
fn spawn_wood_part(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldRenderAssets,
    mesh: ModelMesh,
    translation: Vec3,
    scale: Vec3,
    icon_layer: Option<&RenderLayers>,
    preview: bool,
) {
    let mut child = parent.spawn((
        Mesh3d(assets.model_mesh(mesh)),
        MeshMaterial3d(if preview {
            assets.model_preview_material(ModelMaterial::WoodTexture)
        } else {
            assets.model_material(ModelMaterial::WoodTexture)
        }),
        Transform {
            translation,
            scale,
            ..default()
        },
    ));
    if let Some(icon_layer) = icon_layer {
        child.insert((icon_layer.clone(), BlockIconRenderEntity));
    }
}

/// 推断告示贴面法线（优先附着表，否则与 rebuild 同序几何推断）
fn sign_mount_normal(world: &WorldBlocks, pos: IVec3, data: BlockData) -> Option<IVec3> {
    if !data.id.is_none() {
        if let Some(att) = world.factory_attachments.get(&data.id) {
            return Some(att.parent_face_normal);
        }
    }
    let candidates = [
        data.facing.forward_ivec3(),
        IVec3::Y,
        IVec3::X,
        IVec3::NEG_X,
        IVec3::Z,
        IVec3::NEG_Z,
        IVec3::NEG_Y,
    ];
    for normal in candidates {
        let host_pos = pos - normal;
        let Some(host) = world.blocks.get(&host_pos) else {
            continue;
        };
        if oif_sim::world::grid::WorldBlocks::host_face_accepts_sign(host, normal) {
            return Some(normal);
        }
    }
    None
}
