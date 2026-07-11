use crate::blocks::adapter::BlockImpl;
use crate::blocks::traits::BlockBehavior;
use crate::blocks::{BlockKind, MaterialProcessor};

pub struct TeleportEntranceBlock;

pub static BLOCK: BlockImpl<TeleportEntranceBlock> = BlockImpl(TeleportEntranceBlock);

mod meta;

impl BlockBehavior for TeleportEntranceBlock {
    fn material_processor(&self) -> Option<MaterialProcessor> {
        Some(MaterialProcessor::TeleportEntrance)
    }
}

register_block!(BLOCK, BlockKind::TeleportEntrance, editable: true);
