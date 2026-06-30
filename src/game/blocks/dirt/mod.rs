use bevy::prelude::Image;

use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::basic::{BasicBlockDef, BasicBlockLayer};
use crate::game::blocks::{BlockKind, ColorSpec, rgb};

pub struct Dirt;

mod texture;

impl BasicBlockDef for Dirt {
    const KIND: BlockKind = BlockKind::Dirt;
    const LAYER: BasicBlockLayer = BasicBlockLayer::Scene;
    const NAME_KEY: &'static str = "block.dirt";
    const SHORT_NAME_KEY: &'static str = "short.dirt";
    const COLOR: ColorSpec = rgb(0.40, 0.27, 0.16);
    const ITEM_SLOT_COLOR: ColorSpec = rgb(0.42, 0.26, 0.14);

    fn block_texture() -> Option<Image> {
        Some(texture::image())
    }
}

pub static BLOCK: BlockImpl<Dirt> = BlockImpl(Dirt);

impl crate::game::blocks::traits::BlockBehavior for Dirt {}

register_block!(BLOCK, BlockKind::Dirt, editable: true);
