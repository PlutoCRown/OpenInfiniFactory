use super::TeleportBlock;

use crate::blocks::traits::BlockMeta;
use crate::blocks::{BlockDefinition, BlockKind, rgb};
use glam::IVec3;
use crate::world::grid::{BlockSettings, TeleportSettings};

impl BlockMeta for TeleportBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Teleport
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::puzzle_system(
            self.id(),
            "block.teleport",
            "short.teleport",
            "desc.teleport",
            rgb(0.55, 0.18, 0.92),
        )
        .no_collision()
    }

    fn default_settings(&self, pos: IVec3) -> Option<BlockSettings> {
        Some(BlockSettings::Teleport(TeleportSettings::unnamed(pos)))
    }
}
