use super::RotatorBlock;

use crate::game::blocks::traits::BlockMeta;
use crate::game::blocks::{BlockDefinition, BlockKind, rgb};

impl BlockMeta for RotatorBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Rotator
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.rotator",
            "short.rotator",
            rgb(0.48, 0.32, 0.72),
            rgb(0.42, 0.26, 0.64),
        )
    }

    fn alternate(&self) -> Option<BlockKind> {
        Some(BlockKind::CounterRotator)
    }
}
