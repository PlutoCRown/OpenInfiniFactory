use crate::blocks::adapter::BlockImpl;
use crate::blocks::traits::BlockBehavior;
use crate::blocks::{BlockKind, MaterialProcessor};

pub struct TeleportBlock;

pub static BLOCK: BlockImpl<TeleportBlock> = BlockImpl(TeleportBlock);

mod meta;

impl BlockBehavior for TeleportBlock {
    fn material_processor(&self) -> Option<MaterialProcessor> {
        Some(MaterialProcessor::Teleport)
    }
}

register_block!(BLOCK, BlockKind::Teleport, editable: true);
