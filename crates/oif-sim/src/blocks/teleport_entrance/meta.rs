use super::TeleportEntranceBlock;

use crate::blocks::traits::BlockMeta;
use crate::blocks::{BlockDefinition, BlockKind, rgb};
use glam::IVec3;
use crate::world::grid::{BlockSettings, TeleportSettings};

impl BlockMeta for TeleportEntranceBlock {
    fn id(&self) -> BlockKind {
        BlockKind::TeleportEntrance
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::puzzle_system(
            self.id(),
            "block.teleport_entrance",
            "short.teleport_entrance",
            "desc.teleport_entrance",
            rgb(0.12, 0.62, 0.92),
        )
        .no_collision()
    }

    fn default_settings(&self, pos: IVec3) -> Option<BlockSettings> {
        Some(BlockSettings::Teleport(TeleportSettings::unnamed(pos)))
    }
}

