pub use oif_sim::blocks::weld_point::WeldPointBlock;

use crate::game::blocks::BlockKind;
use crate::game::blocks::adapter::BlockImpl;

pub static BLOCK: BlockImpl<WeldPointBlock> = BlockImpl(WeldPointBlock);

mod render;

register_block!(BLOCK, BlockKind::WeldPoint, editable: false);
