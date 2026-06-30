use super::TeleportEntranceBlock;

use bevy::prelude::Color;

use crate::game::blocks::traits::{BlockMeta, PlaceableBlock};
use crate::game::blocks::{BlockDefinition, BlockKind, rgb};
use bevy::prelude::{IVec3};
use crate::game::world::grid::{BlockSettings, TeleportSettings};

impl BlockMeta for TeleportEntranceBlock {
    fn id(&self) -> BlockKind {
        BlockKind::TeleportEntrance
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::puzzle_system(
            self.id(),
            "block.teleport_entrance",
            "short.teleport_entrance",
            rgb(0.12, 0.62, 0.92),
        )
        .no_collision()
    }

    fn default_settings(&self, pos: IVec3) -> Option<BlockSettings> {
        Some(BlockSettings::Teleport(TeleportSettings::unnamed(pos)))
    }
}

impl PlaceableBlock for TeleportEntranceBlock {
    fn item_slot_color(&self) -> Color {
        rgb(0.06, 0.42, 0.72).color()
    }
}
