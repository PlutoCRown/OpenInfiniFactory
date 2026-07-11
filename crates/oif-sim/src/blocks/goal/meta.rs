use super::GoalBlock;

use crate::blocks::traits::BlockMeta;
use crate::blocks::{rgba, BlockDefinition, BlockKind};
use crate::world::grid::{BlockSettings, GoalSettings};
use glam::IVec3;

impl BlockMeta for GoalBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Goal
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::puzzle_system(
            self.id(),
            "block.goal",
            "short.goal",
            rgba(0.35, 0.72, 0.42, 0.30),
        )
        .no_collision()
        .transparent()
    }

    fn default_settings(&self, _pos: IVec3) -> Option<BlockSettings> {
        Some(BlockSettings::Goal(GoalSettings::default()))
    }
}

