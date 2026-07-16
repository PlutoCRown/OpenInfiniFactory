use crate::blocks::adapter::BlockImpl;
use crate::blocks::BlockKind;

// 水平镜子，可在模拟中反射水平激光
pub struct MirrorBlock;

pub static BLOCK: BlockImpl<MirrorBlock> = BlockImpl(MirrorBlock);

mod behavior;
mod meta;

register_block!(BLOCK, BlockKind::Mirror, editable: false, play: true);
