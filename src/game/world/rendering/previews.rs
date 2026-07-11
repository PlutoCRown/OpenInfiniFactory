use bevy::prelude::*;

use super::components::{EditPreview, PendingGeneratedPreview};
use super::spawn::spawn_block_model;
use crate::game::blocks::BlockData;
use crate::game::world::animation::AnimationTiming;
use crate::game::world::grid::{grid_to_world, WorldBlocks};
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

/// 生成删除选区包围盒预览
pub fn spawn_delete_bounds_preview(
    commands: &mut Commands,
    assets: &WorldRenderAssets,
    min: IVec3,
    max: IVec3,
) {
    let center = (grid_to_world(min) + grid_to_world(max)) * 0.5;
    let size = (max - min + IVec3::ONE).as_vec3() * 1.1;
    commands.spawn((
        Mesh3d(assets.block.clone()),
        MeshMaterial3d(assets.edit_preview_material(EditPreviewKind::Delete)),
        Transform::from_translation(center).with_scale(size),
        EditPreview,
    ));
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
