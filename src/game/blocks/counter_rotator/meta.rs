use super::CounterRotatorBlock;

use crate::game::blocks::traits::BlockMeta;
use crate::game::blocks::{BlockDefinition, BlockKind, rgb};

impl BlockMeta for CounterRotatorBlock {
    fn id(&self) -> BlockKind {
        BlockKind::CounterRotator
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.counter_rotator",
            "short.counter_rotator",
            rgb(0.62, 0.28, 0.78),
            rgb(0.54, 0.22, 0.68),
        )
    }

    fn alternate(&self) -> Option<BlockKind> {
        Some(BlockKind::Rotator)
    }
}
