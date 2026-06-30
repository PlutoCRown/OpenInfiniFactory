use super::DrillBlock;

use bevy::prelude::Color;

use crate::game::blocks::traits::{BlockMeta, PlaceableBlock};
use crate::game::blocks::{BlockDefinition, BlockKind, rgb};

impl BlockMeta for DrillBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Drill
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.drill",
            "short.drill",
            rgb(0.32, 0.36, 0.40),
        )
    }

    fn alternate(&self) -> Option<BlockKind> {
        Some(BlockKind::Laser)
    }
}

impl PlaceableBlock for DrillBlock {
    fn item_slot_color(&self) -> Color {
        rgb(0.24, 0.26, 0.30).color()
    }
}
