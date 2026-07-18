use crate::blocks::adapter::BlockImpl;
use crate::blocks::basic::BasicBlockDef;
use crate::blocks::{rgba, BlockKind, ColorSpec, MaterialKind};

/// 玻璃材料：脆弱，运动冲突时碎裂
pub struct GlassMaterial;

impl BasicBlockDef for GlassMaterial {
    const KIND: BlockKind = BlockKind::GlassMaterial;
    const MATERIAL: MaterialKind = MaterialKind::Glass;
    const NAME_KEY: &'static str = "block.glass_material";
    const SHORT_NAME_KEY: &'static str = "short.glass_material";
    const DESCRIPTION_KEY: &'static str = "desc.glass_material";
    const COLOR: ColorSpec = rgba(0.72, 0.88, 0.94, 0.45);
}

pub static BLOCK: BlockImpl<GlassMaterial> = BlockImpl(GlassMaterial);

impl crate::blocks::traits::BlockBehavior for GlassMaterial {}

register_block!(BLOCK, BlockKind::GlassMaterial, editable: false);
