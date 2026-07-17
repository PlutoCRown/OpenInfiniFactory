pub use oif_sim::blocks::stamper_body::StamperBodyBlock;

use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::BlockKind;

pub static BLOCK: BlockImpl<StamperBodyBlock> = BlockImpl(StamperBodyBlock);

mod render;

register_block!(BLOCK, BlockKind::StamperBody, editable: false);
