use crate::blocks::adapter::BlockImpl;
use crate::blocks::BlockKind;

// 分光镜，同时具有水平与垂直镜面的反射能力
pub struct SplitterBlock;

pub static BLOCK: BlockImpl<SplitterBlock> = BlockImpl(SplitterBlock);

mod behavior;
mod meta;

register_block!(BLOCK, BlockKind::Splitter, editable: false, play: true);
