use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::basic::{BasicBlockDef, BasicBlockLayer};
use crate::game::blocks::{BlockKind, BlockTexture, ColorSpec, MaterialKind, rgb};
pub struct BasicMaterial;

impl BasicBlockDef for BasicMaterial {

    const KIND: BlockKind = BlockKind::Material;
    const LAYER: BasicBlockLayer = BasicBlockLayer::Material(MaterialKind::Basic);
    const NAME_KEY: &'static str = "block.material";
    const SHORT_NAME_KEY: &'static str = "short.material";
    const COLOR: ColorSpec = rgb(0.82, 0.82, 0.86);
    const SLOT_COLOR: ColorSpec = rgb(0.74, 0.74, 0.78);
    const TEXTURE: BlockTexture = BlockTexture::Material;
}

pub static BLOCK: BlockImpl<BasicMaterial> = BlockImpl(BasicMaterial);

impl crate::game::blocks::traits::BlockBehavior for BasicMaterial {}
impl crate::game::blocks::traits::BlockRender for BasicMaterial {}

register_block!(BLOCK, BlockKind::Material, editable: false);
