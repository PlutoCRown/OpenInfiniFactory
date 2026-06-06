use super::LifterBlock;

use crate::game::blocks::traits::BlockMeta;
use crate::game::blocks::{BlockDefinition, BlockKind, rgb, rgba};

impl BlockMeta for LifterBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Lifter
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.lifter",
            "short.lifter",
            rgb(0.25, 0.58, 0.72),
            rgb(0.18, 0.48, 0.62),
        )
    }
}
