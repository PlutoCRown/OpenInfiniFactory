use bevy::prelude::Image;

use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::basic::{BasicBlockDef, BasicBlockLayer};
use crate::game::blocks::{rgb, BlockKind, ColorSpec, MaterialKind};

pub struct CopperMaterial;

mod texture;

impl BasicBlockDef for CopperMaterial {
    const KIND: BlockKind = BlockKind::CopperMaterial;
    const LAYER: BasicBlockLayer = BasicBlockLayer::Material(MaterialKind::Copper);
    const NAME_KEY: &'static str = "block.copper_material";
    const SHORT_NAME_KEY: &'static str = "short.copper_material";
    const COLOR: ColorSpec = rgb(0.78, 0.42, 0.22);
    const ITEM_SLOT_COLOR: ColorSpec = rgb(0.68, 0.34, 0.16);

    fn block_texture() -> Option<Image> {
        Some(texture::image())
    }
}

pub static BLOCK: BlockImpl<CopperMaterial> = BlockImpl(CopperMaterial);

impl crate::game::blocks::traits::BlockBehavior for CopperMaterial {}

register_block!(BLOCK, BlockKind::CopperMaterial, editable: false);
