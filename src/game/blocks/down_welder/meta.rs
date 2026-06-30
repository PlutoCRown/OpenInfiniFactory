use super::DownWelderBlock;

use bevy::prelude::Color;

use crate::game::blocks::traits::{BlockMeta, PlaceableBlock};
use crate::game::blocks::{BlockDefinition, BlockKind, rgb};

impl BlockMeta for DownWelderBlock {
    fn id(&self) -> BlockKind {
        BlockKind::DownWelder
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.down_welder",
            "short.down_welder",
            rgb(0.14, 0.38, 0.74),
        )
    }

    fn alternate(&self) -> Option<BlockKind> {
        Some(BlockKind::Welder)
    }
}

impl PlaceableBlock for DownWelderBlock {
    fn item_slot_color(&self) -> Color {
        rgb(0.08, 0.26, 0.58).color()
    }
}
