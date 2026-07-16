use super::DetectorBlock;

use crate::blocks::traits::BlockMeta;
use crate::blocks::{BlockDefinition, BlockKind, rgb};

impl BlockMeta for DetectorBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Detector
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.detector",
            "short.detector",
            "desc.detector",
            rgb(0.15, 0.45, 0.72),
        )
    }

    fn alternate(&self) -> Option<BlockKind> {
        Some(BlockKind::DownDetector)
    }
}

