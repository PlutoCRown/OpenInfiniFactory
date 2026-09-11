//! 世界编辑上下文（权威簇 C2，内嵌 C1）

use bevy::prelude::*;

use crate::game::edit_history::EditHistory;
use crate::game::simulation::structure_state::StructureState;
use crate::game::world::grid::WorldBlocks;
use crate::game::world::rendering::BlockEntity;

use super::scene_render::SceneRenderMut;

/// 格子编辑：世界 + 历史 + 实体查询 + 场景渲染内核
pub struct WorldEditScene<'a, 'b, 's> {
    pub scene: SceneRenderMut<'a, 'b, 's>,
    pub world: &'a mut WorldBlocks,
    pub structure_state: &'a mut StructureState,
    pub edit_history: &'a mut EditHistory,
    pub block_entities: &'a Query<'a, 's, (Entity, &'static BlockEntity)>,
}
