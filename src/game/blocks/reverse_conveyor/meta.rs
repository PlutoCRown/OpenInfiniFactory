use super::ReverseConveyorBlock;

use bevy::prelude::Color;

use crate::game::blocks::traits::{BlockMeta, PlaceableBlock};
use crate::game::blocks::{BlockDefinition, BlockKind, rgb};

impl BlockMeta for ReverseConveyorBlock {
    fn id(&self) -> BlockKind {
        BlockKind::ReverseConveyor
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.reverse_conveyor",
            "short.reverse_conveyor",
            rgb(0.86, 0.46, 0.14),
        )
    }

    fn alternate(&self) -> Option<BlockKind> {
        Some(BlockKind::Conveyor)
    }
}

impl PlaceableBlock for ReverseConveyorBlock {
    fn item_slot_color(&self) -> Color {
        rgb(0.70, 0.34, 0.08).color()
    }
}
