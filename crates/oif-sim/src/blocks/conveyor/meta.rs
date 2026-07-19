use super::ConveyorBlock;

use crate::blocks::traits::BlockMeta;
use crate::blocks::{BlockDefinition, BlockKind, rgb};

impl BlockMeta for ConveyorBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Conveyor
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.conveyor",
            "short.conveyor",
            "desc.conveyor",
            rgb(0.86, 0.46, 0.14),
        )
    }

    fn alternate(&self) -> Option<BlockKind> {
        Some(BlockKind::ReverseConveyor)
    }

    fn alternate_flip_facing(&self) -> bool {
        true
    }
}

