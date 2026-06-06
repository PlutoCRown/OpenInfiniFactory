use super::DownDetectorBlock;

use crate::game::blocks::traits::BlockMeta;
use crate::game::blocks::{BlockDefinition, BlockKind, rgb, rgba};

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
            rgb(0.14, 0.40, 0.68),
        )
    }

    fn alternate(&self) -> Option<BlockKind> {
        Some(BlockKind::Detector)
    }
}
