use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::BlockKind;
pub struct PlatformBlock;

pub static BLOCK: BlockImpl<PlatformBlock> = BlockImpl(PlatformBlock);

mod meta;

impl crate::game::blocks::traits::BlockBehavior for PlatformBlock {}
impl crate::game::blocks::traits::BlockRender for PlatformBlock {}

register_block!(BLOCK, BlockKind::Platform, editable: false, play: true);
