use crate::blocks::adapter::BlockImpl;
use crate::blocks::basic::{BasicBlockDef, BasicBlockLayer};
use crate::blocks::{BlockKind, ColorSpec, rgb};

pub struct Grass;

impl BasicBlockDef for Grass {
    const KIND: BlockKind = BlockKind::Grass;
    const LAYER: BasicBlockLayer = BasicBlockLayer::Scene;
    const NAME_KEY: &'static str = "block.grass";
    const SHORT_NAME_KEY: &'static str = "short.grass";
    const DESCRIPTION_KEY: &'static str = "desc.grass";
    const COLOR: ColorSpec = rgb(0.34, 0.62, 0.24);
}

pub static BLOCK: BlockImpl<Grass> = BlockImpl(Grass);

impl crate::blocks::traits::BlockBehavior for Grass {}

register_block!(BLOCK, BlockKind::Grass, editable: true);
