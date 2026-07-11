use crate::blocks::adapter::BlockImpl;
use crate::blocks::basic::{BasicBlockDef, BasicBlockLayer};
use crate::blocks::{BlockKind, ColorSpec, MaterialKind, rgb};

pub struct BasicMaterial;

impl BasicBlockDef for BasicMaterial {
    const KIND: BlockKind = BlockKind::Material;
    const LAYER: BasicBlockLayer = BasicBlockLayer::Material(MaterialKind::Basic);
    const NAME_KEY: &'static str = "block.material";
    const SHORT_NAME_KEY: &'static str = "short.material";
    const COLOR: ColorSpec = rgb(0.82, 0.82, 0.86);
}

pub static BLOCK: BlockImpl<BasicMaterial> = BlockImpl(BasicMaterial);

impl crate::blocks::traits::BlockBehavior for BasicMaterial {}

register_block!(BLOCK, BlockKind::Material, editable: false);
