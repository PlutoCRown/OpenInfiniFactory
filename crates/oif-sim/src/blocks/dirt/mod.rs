use crate::blocks::adapter::BlockImpl;
use crate::blocks::basic::{BasicBlockDef, BasicBlockLayer};
use crate::blocks::{BlockKind, ColorSpec, rgb};

pub struct Dirt;

impl BasicBlockDef for Dirt {
    const KIND: BlockKind = BlockKind::Dirt;
    const LAYER: BasicBlockLayer = BasicBlockLayer::Scene;
    const NAME_KEY: &'static str = "block.dirt";
    const SHORT_NAME_KEY: &'static str = "short.dirt";
    const DESCRIPTION_KEY: &'static str = "desc.dirt";
    const COLOR: ColorSpec = rgb(0.40, 0.27, 0.16);
}

pub static BLOCK: BlockImpl<Dirt> = BlockImpl(Dirt);

impl crate::blocks::traits::BlockBehavior for Dirt {}

register_block!(BLOCK, BlockKind::Dirt, editable: true);
