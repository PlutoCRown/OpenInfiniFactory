use super::PlatformBlock;

use crate::blocks::traits::BlockMeta;
use crate::blocks::{BlockDefinition, BlockKind, rgb};

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

