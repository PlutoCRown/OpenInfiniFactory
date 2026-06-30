use super::WireBlock;

use bevy::prelude::Color;

use crate::game::blocks::traits::{BlockMeta, PlaceableBlock};
use crate::game::blocks::{BlockDefinition, BlockKind, rgb};

impl BlockMeta for WireBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Wire
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.wire",
            "short.wire",
            rgb(0.95, 0.72, 0.18),
        )
        .node()
    }
}

impl PlaceableBlock for WireBlock {
    fn item_slot_color(&self) -> Color {
        rgb(0.88, 0.62, 0.12).color()
    }
}
