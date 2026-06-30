use super::CounterRotatorBlock;

use bevy::prelude::Color;

use crate::game::blocks::traits::{BlockMeta, PlaceableBlock};
use crate::game::blocks::{BlockDefinition, BlockKind, rgb};

impl BlockMeta for CounterRotatorBlock {
    fn id(&self) -> BlockKind {
        BlockKind::CounterRotator
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.counter_rotator",
            "short.counter_rotator",
            rgb(0.62, 0.28, 0.78),
        )
    }

    fn alternate(&self) -> Option<BlockKind> {
        Some(BlockKind::Rotator)
    }
}

impl PlaceableBlock for CounterRotatorBlock {
    fn item_slot_color(&self) -> Color {
        rgb(0.54, 0.22, 0.68).color()
    }
}
