use crate::blocks::adapter::BlockImpl;
use crate::blocks::basic::{BasicBlockDef, BasicBlockLayer};
use crate::blocks::{rgb, BlockKind, ColorSpec, MaterialKind};

pub struct IronMaterial;

impl BasicBlockDef for IronMaterial {
    const KIND: BlockKind = BlockKind::IronMaterial;
    const LAYER: BasicBlockLayer = BasicBlockLayer::Material(MaterialKind::Iron);
    const NAME_KEY: &'static str = "block.iron_material";
    const SHORT_NAME_KEY: &'static str = "short.iron_material";
    const DESCRIPTION_KEY: &'static str = "desc.iron_material";
    const COLOR: ColorSpec = rgb(0.62, 0.64, 0.66);
}

pub static BLOCK: BlockImpl<IronMaterial> = BlockImpl(IronMaterial);

impl crate::blocks::traits::BlockBehavior for IronMaterial {}

register_block!(BLOCK, BlockKind::IronMaterial, editable: false);
