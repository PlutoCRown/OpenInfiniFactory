use super::{rgb, Block, BlockDefinition, BlockKind, SystemBlock};

pub struct BlockerHeadBlock;

pub static BLOCKER_HEAD: BlockerHeadBlock = BlockerHeadBlock;

impl Block for BlockerHeadBlock {
    fn id(&self) -> BlockKind {
        BlockKind::BlockerHead
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::marker(
            self.id(),
            "block.blocker_head",
            "short.blocker_head",
            rgb(0.70, 0.48, 0.28),
            rgb(0.58, 0.36, 0.18),
        )
    }

}

impl SystemBlock for BlockerHeadBlock {}
