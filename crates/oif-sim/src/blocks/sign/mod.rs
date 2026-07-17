use crate::blocks::adapter::BlockImpl;
use crate::blocks::BlockKind;

pub struct SignBlock;

pub static BLOCK: BlockImpl<SignBlock> = BlockImpl(SignBlock);

mod meta;
mod behavior;

register_block!(BLOCK, BlockKind::Sign, editable: false, play: true);
