use super::DetectorBlock;

use bevy::prelude::Color;

use crate::game::blocks::traits::{BlockMeta, PlaceableBlock};
use crate::game::blocks::{BlockDefinition, BlockKind, rgb};

impl BlockMeta for DetectorBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Detector
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.detector",
            "short.detector",
            rgb(0.15, 0.45, 0.72),
        )
    }

    fn alternate(&self) -> Option<BlockKind> {
        Some(BlockKind::DownDetector)
    }
}

impl PlaceableBlock for DetectorBlock {
    fn item_slot_color(&self) -> Color {
        rgb(0.12, 0.34, 0.62).color()
    }
}
