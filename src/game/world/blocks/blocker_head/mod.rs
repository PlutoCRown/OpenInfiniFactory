use super::{rgb, Block, BlockDefinition, BlockKind, BlockModel};

mod definition;
mod render;

pub struct BlockerHeadBlock;

pub static BLOCKER_HEAD: BlockerHeadBlock = BlockerHeadBlock;

impl Block for BlockerHeadBlock {
    fn id(&self) -> BlockKind {
        BlockKind::BlockerHead
    }

    fn definition(&self) -> BlockDefinition {
        definition::definition(self)
    }

    fn model(&self) -> BlockModel {
        render::model(self)
    }
}
