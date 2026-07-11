use crate::blocks::adapter::BlockImpl;
use crate::blocks::basic::{BasicBlockDef, BasicBlockLayer};
use crate::blocks::{BlockKind, ColorSpec, rgb};

pub struct Stone;

impl BasicBlockDef for Stone {
    const KIND: BlockKind = BlockKind::Stone;
    const LAYER: BasicBlockLayer = BasicBlockLayer::Scene;
    const NAME_KEY: &'static str = "block.stone";
    const SHORT_NAME_KEY: &'static str = "short.stone";
    const COLOR: ColorSpec = rgb(0.43, 0.43, 0.42);
}

pub static BLOCK: BlockImpl<Stone> = BlockImpl(Stone);

impl crate::blocks::traits::BlockBehavior for Stone {}

register_block!(BLOCK, BlockKind::Stone, editable: true);
