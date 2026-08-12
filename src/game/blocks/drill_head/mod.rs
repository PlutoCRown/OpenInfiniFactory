pub use oif_sim::blocks::drill_head::DrillHeadBlock;

use crate::game::blocks::BlockKind;
use crate::game::blocks::adapter::BlockImpl;

pub static BLOCK: BlockImpl<DrillHeadBlock> = BlockImpl(DrillHeadBlock);

mod render;

register_block!(BLOCK, BlockKind::DrillHead, editable: false);
