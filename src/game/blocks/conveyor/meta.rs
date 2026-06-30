use super::ConveyorBlock;

use bevy::prelude::Color;

use crate::game::blocks::traits::{BlockMeta, PlaceableBlock};
use crate::game::blocks::{BlockDefinition, BlockKind, rgb};

impl BlockMeta for ConveyorBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Conveyor
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.conveyor",
            "short.conveyor",
            rgb(0.86, 0.46, 0.14),
        )
    }

    fn alternate(&self) -> Option<BlockKind> {
        Some(BlockKind::ReverseConveyor)
    }
}

impl PlaceableBlock for ConveyorBlock {
    fn item_slot_color(&self) -> Color {
        rgb(0.70, 0.34, 0.08).color()
    }
}
