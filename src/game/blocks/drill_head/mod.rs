pub use oif_sim::blocks::drill_head::DrillHeadBlock;

use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::BlockKind;

pub static BLOCK: BlockImpl<DrillHeadBlock> = BlockImpl(DrillHeadBlock);

mod render;

register_block!(BLOCK, BlockKind::DrillHead, editable: false);
