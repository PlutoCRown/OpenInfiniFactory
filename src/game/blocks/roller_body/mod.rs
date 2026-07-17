pub use oif_sim::blocks::roller_body::RollerBodyBlock;

use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::BlockKind;

pub static BLOCK: BlockImpl<RollerBodyBlock> = BlockImpl(RollerBodyBlock);

mod render;

register_block!(BLOCK, BlockKind::RollerBody, editable: false);
