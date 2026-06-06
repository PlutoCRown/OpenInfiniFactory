use super::DetectorBlock;

use crate::game::blocks::traits::BlockMeta;
use crate::game::blocks::{BlockDefinition, BlockKind, rgb, rgba};

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
            rgb(0.12, 0.34, 0.62),
        )
    }

    fn alternate(&self) -> Option<BlockKind> {
        Some(BlockKind::DownDetector)
    }
}
