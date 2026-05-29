use super::{rgb, Block, BlockDefinition, BlockKind, EditableBlock, SystemBlock};
use crate::game::world::grid::{BlockSettings, TeleportSettings};

pub struct TeleportEntranceBlock;

pub static TELEPORT_ENTRANCE: TeleportEntranceBlock = TeleportEntranceBlock;

impl Block for TeleportEntranceBlock {
    fn id(&self) -> BlockKind {
        BlockKind::TeleportEntrance
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::marker(
            self.id(),
            "block.teleport_entrance",
            "short.teleport_entrance",
            rgb(0.12, 0.62, 0.92),
            rgb(0.06, 0.42, 0.72),
        )
        .no_collision()
    }

    fn default_settings(&self, pos: bevy::prelude::IVec3) -> Option<BlockSettings> {
        Some(BlockSettings::Teleport(TeleportSettings::unnamed(pos)))
    }
}

impl SystemBlock for TeleportEntranceBlock {}
impl EditableBlock for TeleportEntranceBlock {}
