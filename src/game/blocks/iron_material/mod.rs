use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::basic::{BasicBlockDef, BasicBlockLayer};
use crate::game::blocks::{BlockKind, BlockTexture, ColorSpec, MaterialKind, rgb};
pub struct IronMaterial;

impl BasicBlockDef for IronMaterial {

    const KIND: BlockKind = BlockKind::IronMaterial;
    const LAYER: BasicBlockLayer = BasicBlockLayer::Material(MaterialKind::Iron);
    const NAME_KEY: &'static str = "block.iron_material";
    const SHORT_NAME_KEY: &'static str = "short.iron_material";
    const COLOR: ColorSpec = rgb(0.62, 0.64, 0.66);
    const SLOT_COLOR: ColorSpec = rgb(0.54, 0.56, 0.58);
    const TEXTURE: BlockTexture = BlockTexture::IronMaterial;
}

pub static BLOCK: BlockImpl<IronMaterial> = BlockImpl(IronMaterial);

impl crate::game::blocks::traits::BlockBehavior for IronMaterial {}
impl crate::game::blocks::traits::BlockRender for IronMaterial {}

register_block!(BLOCK, BlockKind::IronMaterial, editable: false);
