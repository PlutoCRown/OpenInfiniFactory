use super::DrillBlock;

use crate::game::blocks::traits::BlockMeta;
use crate::game::blocks::{BlockDefinition, BlockKind, rgb, rgba};

impl BlockMeta for DrillBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Drill
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.drill",
            "short.drill",
            rgb(0.32, 0.36, 0.40),
            rgb(0.24, 0.26, 0.30),
        )
    }

    fn alternate(&self) -> Option<BlockKind> {
        Some(BlockKind::Laser)
    }
}
