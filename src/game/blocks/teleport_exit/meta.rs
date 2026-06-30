use super::TeleportExitBlock;

use bevy::prelude::Color;

use crate::game::blocks::traits::{BlockMeta, PlaceableBlock};
use crate::game::blocks::{BlockDefinition, BlockKind, rgb};
use bevy::prelude::{IVec3};
use crate::game::world::grid::{BlockSettings, TeleportSettings};

impl BlockMeta for TeleportExitBlock {
    fn id(&self) -> BlockKind {
        BlockKind::TeleportExit
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::puzzle_system(
            self.id(),
            "block.teleport_exit",
            "short.teleport_exit",
            rgb(0.72, 0.34, 0.96),
        )
        .no_collision()
    }

    fn default_settings(&self, pos: IVec3) -> Option<BlockSettings> {
        Some(BlockSettings::Teleport(TeleportSettings::unnamed(pos)))
    }
}

impl PlaceableBlock for TeleportExitBlock {
    fn item_slot_color(&self) -> Color {
        rgb(0.50, 0.20, 0.74).color()
    }
}
