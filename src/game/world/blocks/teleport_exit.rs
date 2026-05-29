use super::{rgb, Block, BlockDefinition, BlockKind, EditableBlock, SystemBlock};
use crate::game::world::grid::{BlockSettings, TeleportSettings};

pub struct TeleportExitBlock;

pub static TELEPORT_EXIT: TeleportExitBlock = TeleportExitBlock;

impl Block for TeleportExitBlock {
    fn id(&self) -> BlockKind {
        BlockKind::TeleportExit
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::marker(
            self.id(),
            "block.teleport_exit",
            "short.teleport_exit",
            rgb(0.72, 0.34, 0.96),
            rgb(0.50, 0.20, 0.74),
        )
        .no_collision()
    }

    fn default_settings(&self, pos: bevy::prelude::IVec3) -> Option<BlockSettings> {
        Some(BlockSettings::Teleport(TeleportSettings::unnamed(pos)))
    }
}

impl SystemBlock for TeleportExitBlock {}
impl EditableBlock for TeleportExitBlock {}
