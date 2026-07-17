use crate::blocks::adapter::BlockImpl;
use crate::blocks::basic::{BasicBlockDef, BasicBlockLayer};
use crate::blocks::{rgb, BlockKind, ColorSpec, MaterialKind};

/// 印花材料：占格附着面片，有向、不可焊接
pub struct StampMaterial;

impl BasicBlockDef for StampMaterial {
    const KIND: BlockKind = BlockKind::StampMaterial;
    const LAYER: BasicBlockLayer = BasicBlockLayer::Material(MaterialKind::Stamp);
    const NAME_KEY: &'static str = "block.stamp_material";
    const SHORT_NAME_KEY: &'static str = "short.stamp_material";
    const DESCRIPTION_KEY: &'static str = "desc.stamp_material";
    const COLOR: ColorSpec = rgb(0.95, 0.12, 0.10);
}

pub static BLOCK: BlockImpl<StampMaterial> = BlockImpl(StampMaterial);

impl crate::blocks::traits::BlockBehavior for StampMaterial {}

register_block!(BLOCK, BlockKind::StampMaterial, editable: false);
