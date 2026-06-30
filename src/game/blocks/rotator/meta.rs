use super::RotatorBlock;

use bevy::prelude::Color;

use crate::game::blocks::traits::{BlockMeta, PlaceableBlock};
use crate::game::blocks::{BlockDefinition, BlockKind, rgb};

impl BlockMeta for RotatorBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Rotator
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.rotator",
            "short.rotator",
            rgb(0.48, 0.32, 0.72),
        )
    }

    fn alternate(&self) -> Option<BlockKind> {
        Some(BlockKind::CounterRotator)
    }
}

impl PlaceableBlock for RotatorBlock {
    fn item_slot_color(&self) -> Color {
        rgb(0.42, 0.26, 0.64).color()
    }
}
