pub use oif_sim::blocks::weld_point::WeldPointBlock;

use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::BlockKind;

pub static BLOCK: BlockImpl<WeldPointBlock> = BlockImpl(WeldPointBlock);

mod render;

register_block!(BLOCK, BlockKind::WeldPoint, editable: false);
