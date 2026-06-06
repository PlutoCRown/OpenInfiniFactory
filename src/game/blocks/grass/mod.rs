use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::basic::{BasicBlockDef, BasicBlockLayer};
use crate::game::blocks::{BlockKind, BlockTexture, ColorSpec, rgb};
pub struct Grass;

impl BasicBlockDef for Grass {

    const KIND: BlockKind = BlockKind::Grass;
    const LAYER: BasicBlockLayer = BasicBlockLayer::Scene;
    const NAME_KEY: &'static str = "block.grass";
    const SHORT_NAME_KEY: &'static str = "short.grass";
    const COLOR: ColorSpec = rgb(0.34, 0.62, 0.24);
    const SLOT_COLOR: ColorSpec = rgb(0.27, 0.56, 0.20);
    const TEXTURE: BlockTexture = BlockTexture::Grass;
}

pub static BLOCK: BlockImpl<Grass> = BlockImpl(Grass);

impl crate::game::blocks::traits::BlockBehavior for Grass {}
impl crate::game::blocks::traits::BlockRender for Grass {}

register_block!(BLOCK, BlockKind::Grass, editable: true);
