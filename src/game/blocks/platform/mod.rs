use bevy::prelude::Image;

use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::traits::BlockRender;
use crate::game::blocks::BlockKind;

pub struct PlatformBlock;

pub static BLOCK: BlockImpl<PlatformBlock> = BlockImpl(PlatformBlock);

mod meta;
mod texture;

impl crate::game::blocks::traits::BlockBehavior for PlatformBlock {}

impl BlockRender for PlatformBlock {
    fn block_texture(&self) -> Option<Image> {
        Some(texture::image())
    }
}

register_block!(BLOCK, BlockKind::Platform, editable: false, play: true);
