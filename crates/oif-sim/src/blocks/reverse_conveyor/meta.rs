use super::ReverseConveyorBlock;

use crate::blocks::traits::BlockMeta;
use crate::blocks::{BlockDefinition, BlockKind, rgb};

impl BlockMeta for ReverseConveyorBlock {
    fn id(&self) -> BlockKind {
        BlockKind::ReverseConveyor
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.reverse_conveyor",
            "short.reverse_conveyor",
            "desc.reverse_conveyor",
            rgb(0.86, 0.46, 0.14),
        )
    }

    fn alternate(&self) -> Option<BlockKind> {
        Some(BlockKind::Conveyor)
    }

    fn alternate_flip_facing(&self) -> bool {
        true
    }
}

