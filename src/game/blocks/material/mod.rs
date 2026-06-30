use bevy::prelude::Image;

use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::basic::{BasicBlockDef, BasicBlockLayer};
use crate::game::blocks::{BlockKind, ColorSpec, MaterialKind, rgb};

pub struct BasicMaterial;

mod texture;

impl BasicBlockDef for BasicMaterial {
    const KIND: BlockKind = BlockKind::Material;
    const LAYER: BasicBlockLayer = BasicBlockLayer::Material(MaterialKind::Basic);
    const NAME_KEY: &'static str = "block.material";
    const SHORT_NAME_KEY: &'static str = "short.material";
    const COLOR: ColorSpec = rgb(0.82, 0.82, 0.86);
    const ITEM_SLOT_COLOR: ColorSpec = rgb(0.74, 0.74, 0.78);

    fn block_texture() -> Option<Image> {
        Some(texture::image())
    }
}

pub static BLOCK: BlockImpl<BasicMaterial> = BlockImpl(BasicMaterial);

impl crate::game::blocks::traits::BlockBehavior for BasicMaterial {}

register_block!(BLOCK, BlockKind::Material, editable: false);
