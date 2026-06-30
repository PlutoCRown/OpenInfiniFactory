use super::BlockerHeadBlock;

use crate::game::blocks::traits::BlockMeta;
use crate::game::blocks::{BlockDefinition, BlockKind, rgb};

impl BlockMeta for BlockerHeadBlock {
    fn id(&self) -> BlockKind {
        BlockKind::BlockerHead
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::marker(
            self.id(),
            "block.blocker_head",
            "short.blocker_head",
            rgb(0.54, 0.56, 0.54),
        )
    }
}
