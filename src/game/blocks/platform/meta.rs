use super::PlatformBlock;

use bevy::prelude::Color;

use crate::game::blocks::traits::{BlockMeta, PlaceableBlock};
use crate::game::blocks::{BlockDefinition, BlockKind, rgb};

impl BlockMeta for PlatformBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Platform
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.platform",
            "short.platform",
            rgb(0.36, 0.47, 0.58),
        )
    }
}

impl PlaceableBlock for PlatformBlock {
    fn item_slot_color(&self) -> Color {
        rgb(0.28, 0.38, 0.48).color()
    }
}
