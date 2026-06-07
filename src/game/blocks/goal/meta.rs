use super::GoalBlock;

use crate::game::blocks::traits::BlockMeta;
use crate::game::blocks::{rgba, BlockDefinition, BlockKind};
use crate::game::world::grid::{BlockSettings, GoalSettings};
use bevy::prelude::IVec3;

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
            rgba(0.24, 0.56, 0.30, 0.46),
        )
        .no_collision()
        .transparent()
    }

    fn default_settings(&self, _pos: IVec3) -> Option<BlockSettings> {
        Some(BlockSettings::Goal(GoalSettings::default()))
    }
}
