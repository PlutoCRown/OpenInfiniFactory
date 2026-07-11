use crate::blocks::adapter::BlockImpl;
use crate::blocks::basic::{BasicBlockDef, BasicBlockLayer};
use crate::blocks::{BlockKind, ColorSpec, rgb};

pub struct Planks;

impl BasicBlockDef for Planks {
    const KIND: BlockKind = BlockKind::Planks;
    const LAYER: BasicBlockLayer = BasicBlockLayer::Scene;
    const NAME_KEY: &'static str = "block.planks";
    const SHORT_NAME_KEY: &'static str = "short.planks";
    const COLOR: ColorSpec = rgb(0.66, 0.45, 0.25);
}

pub static BLOCK: BlockImpl<Planks> = BlockImpl(Planks);

impl crate::blocks::traits::BlockBehavior for Planks {}

register_block!(BLOCK, BlockKind::Planks, editable: true);
