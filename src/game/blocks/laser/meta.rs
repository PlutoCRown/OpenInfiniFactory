use super::LaserBlock;

use bevy::prelude::Color;

use crate::game::blocks::traits::{BlockMeta, PlaceableBlock};
use crate::game::blocks::{BlockDefinition, BlockKind, rgb};

impl BlockMeta for LaserBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Laser
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.laser",
            "short.laser",
            rgb(0.85, 0.20, 0.34),
        )
    }

    fn alternate(&self) -> Option<BlockKind> {
        Some(BlockKind::Drill)
    }
}

impl PlaceableBlock for LaserBlock {
    fn item_slot_color(&self) -> Color {
        rgb(0.72, 0.12, 0.26).color()
    }
}
