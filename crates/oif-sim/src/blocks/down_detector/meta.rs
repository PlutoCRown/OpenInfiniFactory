use super::DownDetectorBlock;

use crate::blocks::traits::BlockMeta;
use crate::blocks::{BlockDefinition, BlockKind, rgb};

impl BlockMeta for DownDetectorBlock {
    fn id(&self) -> BlockKind {
        BlockKind::DownDetector
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.down_detector",
            "short.down_detector",
            "desc.down_detector",
            rgb(0.18, 0.52, 0.78),
        )
    }

    fn alternate(&self) -> Option<BlockKind> {
        Some(BlockKind::Detector)
    }
}

