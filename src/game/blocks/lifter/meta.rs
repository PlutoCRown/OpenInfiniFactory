use super::LifterBlock;

use bevy::prelude::Color;

use crate::game::blocks::traits::{BlockMeta, PlaceableBlock};
use crate::game::blocks::{BlockDefinition, BlockKind, rgb};

impl BlockMeta for LifterBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Lifter
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.lifter",
            "short.lifter",
            rgb(0.25, 0.58, 0.72),
        )
    }
}

impl PlaceableBlock for LifterBlock {
    fn item_slot_color(&self) -> Color {
        rgb(0.18, 0.48, 0.62).color()
    }
}
