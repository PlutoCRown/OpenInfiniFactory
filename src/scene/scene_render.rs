//! 场景渲染可变上下文（权威簇 C1）

use bevy::prelude::*;

use crate::game::world::rendering::{SceneChunkMeshes, WorldRenderAssets};

use super::entity_index::BlockEntityIndex;

/// 编辑刷新与回合视觉共用的渲染可变内核
pub struct SceneRenderMut<'a, 'b, 's> {
    pub commands: &'a mut Commands<'b, 's>,
    pub meshes: &'a mut Assets<Mesh>,
    pub render_assets: &'a WorldRenderAssets,
    pub block_index: &'a mut BlockEntityIndex,
    pub scene_chunks: &'a mut SceneChunkMeshes,
}
