use super::RollerBlock;

use crate::game::blocks::traits::BlockMeta;
use crate::game::blocks::{BlockDefinition, BlockKind, rgb};
use bevy::prelude::{IVec3};
use crate::game::world::grid::{BlockSettings, LabelerSettings};

impl BlockMeta for RollerBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Roller
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::puzzle_system(
            self.id(),
            "block.roller",
            "short.roller",
            rgb(0.18, 0.62, 0.78),
            rgb(0.10, 0.44, 0.60),
        )
        .no_collision()
    }

    fn default_settings(&self, _pos: IVec3) -> Option<BlockSettings> {
        Some(BlockSettings::Labeler(LabelerSettings::default()))
    }
}
