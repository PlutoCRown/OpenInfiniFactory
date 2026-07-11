pub use oif_sim::blocks::platform::PlatformBlock;

use bevy::prelude::{Color, Image};

use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::traits::{BlockRender, PlaceableBlock};
use crate::game::blocks::ColorSpecExt;
use crate::game::blocks::{BlockKind, rgb};

pub static BLOCK: BlockImpl<PlatformBlock> = BlockImpl(PlatformBlock);

mod texture;

impl BlockRender for PlatformBlock {
    fn block_texture(&self) -> Option<Image> {
        Some(texture::image())
    }
}

impl PlaceableBlock for PlatformBlock {
    fn item_slot_color(&self) -> Color {
        rgb(0.28, 0.38, 0.48).color()
    }
}


register_block!(BLOCK, BlockKind::Platform, editable: false, play: true);
