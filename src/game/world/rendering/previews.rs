use bevy::prelude::*;

use super::components::{EditPreview, PendingGeneratedPreview};
use super::spawn::spawn_block_model;
use crate::game::blocks::BlockData;
use crate::game::world::animation::AnimationTiming;
use crate::game::world::grid::{WorldBlocks, grid_to_world};
use crate::game::world::render_assets::{EditPreviewKind, WorldRenderAssets};

/// 清除所有编辑预览实体
pub fn despawn_edit_previews(commands: &mut Commands, previews: &Query<Entity, With<EditPreview>>) {
    for entity in previews {
        commands.entity(entity).despawn();
    }
}

/// 清除所有待生成预览实体
pub fn despawn_pending_generated_previews(
    commands: &mut Commands,
    previews: &Query<Entity, With<PendingGeneratedPreview>>,
) {
    for entity in previews {
        commands.entity(entity).despawn();
    }
}

/// 生成单格编辑预览（放置/删除色块）
pub fn spawn_edit_preview(
    commands: &mut Commands,
    assets: &WorldRenderAssets,
    pos: IVec3,
    kind: EditPreviewKind,
) {
    commands.spawn((
        Mesh3d(assets.block.clone()),
        MeshMaterial3d(assets.edit_preview_material(kind)),
        Transform::from_translation(grid_to_world(pos)).with_scale(Vec3::splat(1.03)),
        EditPreview,
    ));
}

/// 生成删除选区包围盒预览（半透明红填充 + 12 条红边，无角块）
pub fn spawn_delete_bounds_preview(
    commands: &mut Commands,
    assets: &WorldRenderAssets,
    min: IVec3,
    max: IVec3,
) {
    let center = (grid_to_world(min) + grid_to_world(max)) * 0.5;
    let size = (max - min + IVec3::ONE).as_vec3();
    let half = size * 0.5;
    let mesh = assets.block.clone();
    let fill = assets.selection_invalid_fill_material();
    let edge_mat = assets.selection_invalid_edge_material();
    let edge_t = 0.034;

    commands.spawn((
        Mesh3d(mesh.clone()),
        MeshMaterial3d(fill),
        Transform::from_translation(center).with_scale(size),
        EditPreview,
    ));

    let (x0, x1) = (center.x - half.x, center.x + half.x);
    let (y0, y1) = (center.y - half.y, center.y + half.y);
    let (z0, z1) = (center.z - half.z, center.z + half.z);

    for (pos, scale) in [
        (
            Vec3::new(center.x, y0, z0),
            Vec3::new(size.x, edge_t, edge_t),
        ),
        (
            Vec3::new(center.x, y0, z1),
            Vec3::new(size.x, edge_t, edge_t),
        ),
        (
            Vec3::new(center.x, y1, z0),
            Vec3::new(size.x, edge_t, edge_t),
        ),
        (
            Vec3::new(center.x, y1, z1),
            Vec3::new(size.x, edge_t, edge_t),
        ),
        (
            Vec3::new(x0, center.y, z0),
            Vec3::new(edge_t, size.y, edge_t),
        ),
        (
            Vec3::new(x0, center.y, z1),
            Vec3::new(edge_t, size.y, edge_t),
        ),
        (
            Vec3::new(x1, center.y, z0),
            Vec3::new(edge_t, size.y, edge_t),
        ),
        (
            Vec3::new(x1, center.y, z1),
            Vec3::new(edge_t, size.y, edge_t),
        ),
        (
            Vec3::new(x0, y0, center.z),
            Vec3::new(edge_t, edge_t, size.z),
        ),
        (
            Vec3::new(x0, y1, center.z),
            Vec3::new(edge_t, edge_t, size.z),
        ),
        (
            Vec3::new(x1, y0, center.z),
            Vec3::new(edge_t, edge_t, size.z),
        ),
        (
            Vec3::new(x1, y1, center.z),
            Vec3::new(edge_t, edge_t, size.z),
        ),
    ] {
        commands.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(edge_mat.clone()),
            Transform::from_translation(pos).with_scale(scale),
            EditPreview,
        ));
    }
}

/// 选区包围盒：半透明填充；可选固定尺寸边线/角块；valid=false 时整框变红
pub fn spawn_selection_bounds_preview(
    commands: &mut Commands,
    assets: &WorldRenderAssets,
    min: IVec3,
    max: IVec3,
    include_frame: bool,
    valid: bool,
) {
    let center = (grid_to_world(min) + grid_to_world(max)) * 0.5;
    let size = (max - min + IVec3::ONE).as_vec3();
    let half = size * 0.5;
    let fill = if valid {
        assets.selection_fill_material()
    } else {
        assets.selection_invalid_fill_material()
    };

    commands.spawn((
        Mesh3d(assets.block.clone()),
        MeshMaterial3d(fill),
        Transform::from_translation(center).with_scale(size),
        EditPreview,
    ));

    if !include_frame {
        return;
    }

    let edge_t = 0.034;
    let corner_s = 0.088;
    let edge_mat = if valid {
        assets.selection_edge_material()
    } else {
        assets.selection_invalid_edge_material()
    };
    let mesh = assets.block.clone();

    let (x0, x1) = (center.x - half.x, center.x + half.x);
    let (y0, y1) = (center.y - half.y, center.y + half.y);
    let (z0, z1) = (center.z - half.z, center.z + half.z);

    for (pos, scale) in [
        (
            Vec3::new(center.x, y0, z0),
            Vec3::new(size.x, edge_t, edge_t),
        ),
        (
            Vec3::new(center.x, y0, z1),
            Vec3::new(size.x, edge_t, edge_t),
        ),
        (
            Vec3::new(center.x, y1, z0),
            Vec3::new(size.x, edge_t, edge_t),
        ),
        (
            Vec3::new(center.x, y1, z1),
            Vec3::new(size.x, edge_t, edge_t),
        ),
        (
            Vec3::new(x0, center.y, z0),
            Vec3::new(edge_t, size.y, edge_t),
        ),
        (
            Vec3::new(x0, center.y, z1),
            Vec3::new(edge_t, size.y, edge_t),
        ),
        (
            Vec3::new(x1, center.y, z0),
            Vec3::new(edge_t, size.y, edge_t),
        ),
        (
            Vec3::new(x1, center.y, z1),
            Vec3::new(edge_t, size.y, edge_t),
        ),
        (
            Vec3::new(x0, y0, center.z),
            Vec3::new(edge_t, edge_t, size.z),
        ),
        (
            Vec3::new(x0, y1, center.z),
            Vec3::new(edge_t, edge_t, size.z),
        ),
        (
            Vec3::new(x1, y0, center.z),
            Vec3::new(edge_t, edge_t, size.z),
        ),
        (
            Vec3::new(x1, y1, center.z),
            Vec3::new(edge_t, edge_t, size.z),
        ),
    ] {
        commands.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(edge_mat.clone()),
            Transform::from_translation(pos).with_scale(scale),
            EditPreview,
        ));
    }

    for x in [x0, x1] {
        for y in [y0, y1] {
            for z in [z0, z1] {
                commands.spawn((
                    Mesh3d(mesh.clone()),
                    MeshMaterial3d(edge_mat.clone()),
                    Transform::from_translation(Vec3::new(x, y, z))
                        .with_scale(Vec3::splat(corner_s)),
                    EditPreview,
                ));
            }
        }
    }
}

/// 生成带完整模型的放置预览方块
pub fn spawn_block_preview(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    assets: &WorldRenderAssets,
    world: &WorldBlocks,
    pos: IVec3,
    data: BlockData,
) {
    spawn_block_model(
        commands,
        meshes,
        assets,
        world,
        pos,
        data,
        assets.block_preview_material(data.kind),
        Some(EditPreview),
        None,
        None,
        AnimationTiming::edit(),
        false,
        false,
        true,
        None,
        None,
        None,
    );
}
