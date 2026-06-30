use super::DownDetectorBlock;

use bevy::prelude::Color;

use crate::game::blocks::traits::{BlockMeta, PlaceableBlock};
use crate::game::blocks::{BlockDefinition, BlockKind, rgb};

impl BlockMeta for DownDetectorBlock {
    fn id(&self) -> BlockKind {
        BlockKind::DownDetector
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.down_detector",
            "short.down_detector",
            rgb(0.18, 0.52, 0.78),
        )
    }

    fn alternate(&self) -> Option<BlockKind> {
        Some(BlockKind::Detector)
    }
}

impl PlaceableBlock for DownDetectorBlock {
    fn item_slot_color(&self) -> Color {
        rgb(0.14, 0.40, 0.68).color()
    }
}
