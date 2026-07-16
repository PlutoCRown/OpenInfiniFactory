use crate::blocks::adapter::BlockImpl;
use crate::blocks::BlockKind;

// 垂直镜子，可在模拟中反射竖直/水平激光
pub struct VerticalMirrorBlock;

pub static BLOCK: BlockImpl<VerticalMirrorBlock> = BlockImpl(VerticalMirrorBlock);

mod behavior;
mod meta;

register_block!(BLOCK, BlockKind::VerticalMirror, editable: false, play: true);
