use super::DrillBlock;

use crate::blocks::traits::BlockMeta;
use crate::blocks::{BlockDefinition, BlockKind, rgb};

impl BlockMeta for DrillBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Drill
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.drill",
            "short.drill",
            "desc.drill",
            rgb(0.32, 0.36, 0.40),
        )
    }

    fn alternate(&self) -> Option<BlockKind> {
        Some(BlockKind::Laser)
    }
}

