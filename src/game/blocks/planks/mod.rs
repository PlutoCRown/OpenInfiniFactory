use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::basic::{BasicBlockDef, BasicBlockLayer};
use crate::game::blocks::{BlockKind, BlockTexture, ColorSpec, MaterialKind, rgb};
pub struct Planks;

impl BasicBlockDef for Planks {

    const KIND: BlockKind = BlockKind::Planks;
    const LAYER: BasicBlockLayer = BasicBlockLayer::Scene;
    const NAME_KEY: &'static str = "block.planks";
    const SHORT_NAME_KEY: &'static str = "short.planks";
    const COLOR: ColorSpec = rgb(0.66, 0.45, 0.25);
    const SLOT_COLOR: ColorSpec = rgb(0.62, 0.40, 0.20);
    const TEXTURE: BlockTexture = BlockTexture::Wood;
}

pub static BLOCK: BlockImpl<Planks> = BlockImpl(Planks);

impl crate::game::blocks::traits::BlockBehavior for Planks {}
impl crate::game::blocks::traits::BlockRender for Planks {}

register_block!(BLOCK, BlockKind::Planks, editable: true);
