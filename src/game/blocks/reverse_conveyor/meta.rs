use super::ReverseConveyorBlock;

use crate::game::blocks::traits::BlockMeta;
use crate::game::blocks::{BlockDefinition, BlockKind, rgb, rgba};

impl BlockMeta for ReverseConveyorBlock {
    fn id(&self) -> BlockKind {
        BlockKind::ReverseConveyor
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.reverse_conveyor",
            "short.reverse_conveyor",
            rgb(0.86, 0.46, 0.14),
            rgb(0.70, 0.34, 0.08),
        )
    }

    fn alternate(&self) -> Option<BlockKind> {
        Some(BlockKind::Conveyor)
    }
}
