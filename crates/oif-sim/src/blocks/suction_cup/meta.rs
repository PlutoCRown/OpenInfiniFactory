use super::SuctionCupBlock;

use crate::blocks::traits::BlockMeta;
use crate::blocks::{rgb, BlockDefinition, BlockKind};

impl BlockMeta for SuctionCupBlock {
    fn id(&self) -> BlockKind {
        BlockKind::SuctionCup
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.suction_cup",
            "short.suction_cup",
            rgb(0.82, 0.84, 0.82),
        )
    }
}
