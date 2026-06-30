use bevy::prelude::Image;

use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::basic::{BasicBlockDef, BasicBlockLayer};
use crate::game::blocks::{BlockKind, ColorSpec, rgb};

pub struct Stone;

mod texture;

impl BasicBlockDef for Stone {
    const KIND: BlockKind = BlockKind::Stone;
    const LAYER: BasicBlockLayer = BasicBlockLayer::Scene;
    const NAME_KEY: &'static str = "block.stone";
    const SHORT_NAME_KEY: &'static str = "short.stone";
    const COLOR: ColorSpec = rgb(0.43, 0.43, 0.42);
    const ITEM_SLOT_COLOR: ColorSpec = rgb(0.42, 0.42, 0.40);

    fn block_texture() -> Option<Image> {
        Some(texture::image())
    }
}

pub static BLOCK: BlockImpl<Stone> = BlockImpl(Stone);

impl crate::game::blocks::traits::BlockBehavior for Stone {}

register_block!(BLOCK, BlockKind::Stone, editable: true);
