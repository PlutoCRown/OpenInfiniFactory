use bevy::prelude::Image;

use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::basic::{BasicBlockDef, BasicBlockLayer};
use crate::game::blocks::{BlockKind, ColorSpec, rgb};

pub struct Grass;

mod texture;

impl BasicBlockDef for Grass {
    const KIND: BlockKind = BlockKind::Grass;
    const LAYER: BasicBlockLayer = BasicBlockLayer::Scene;
    const NAME_KEY: &'static str = "block.grass";
    const SHORT_NAME_KEY: &'static str = "short.grass";
    const COLOR: ColorSpec = rgb(0.34, 0.62, 0.24);
    const ITEM_SLOT_COLOR: ColorSpec = rgb(0.27, 0.56, 0.20);

    fn block_texture() -> Option<Image> {
        Some(texture::image())
    }
}

pub static BLOCK: BlockImpl<Grass> = BlockImpl(Grass);

impl crate::game::blocks::traits::BlockBehavior for Grass {}

register_block!(BLOCK, BlockKind::Grass, editable: true);
