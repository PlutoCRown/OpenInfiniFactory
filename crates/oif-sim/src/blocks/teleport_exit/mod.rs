use crate::blocks::adapter::BlockImpl;
use crate::blocks::traits::BlockBehavior;
use crate::blocks::{BlockKind, MaterialProcessor};

pub struct TeleportExitBlock;

pub static BLOCK: BlockImpl<TeleportExitBlock> = BlockImpl(TeleportExitBlock);

mod meta;

impl BlockBehavior for TeleportExitBlock {
    fn material_processor(&self) -> Option<MaterialProcessor> {
        Some(MaterialProcessor::TeleportExit)
    }
}

register_block!(BLOCK, BlockKind::TeleportExit, editable: true);
