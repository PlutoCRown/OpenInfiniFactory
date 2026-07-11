use crate::blocks::adapter::BlockImpl;
use crate::blocks::basic::{BasicBlockDef, BasicBlockLayer};
use crate::blocks::{rgb, BlockKind, ColorSpec, MaterialKind};

pub struct CopperMaterial;

impl BasicBlockDef for CopperMaterial {
    const KIND: BlockKind = BlockKind::CopperMaterial;
    const LAYER: BasicBlockLayer = BasicBlockLayer::Material(MaterialKind::Copper);
    const NAME_KEY: &'static str = "block.copper_material";
    const SHORT_NAME_KEY: &'static str = "short.copper_material";
    const DESCRIPTION_KEY: &'static str = "desc.copper_material";
    const COLOR: ColorSpec = rgb(0.78, 0.42, 0.22);
}

pub static BLOCK: BlockImpl<CopperMaterial> = BlockImpl(CopperMaterial);

impl crate::blocks::traits::BlockBehavior for CopperMaterial {}

register_block!(BLOCK, BlockKind::CopperMaterial, editable: false);
