use crate::game::blocks::ColorSpecExt;
pub use oif_sim::blocks::pusher::PusherBlock;

use bevy::prelude::Color;
use crate::game::blocks::traits::PlaceableBlock;
use crate::game::blocks::rgb;
use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::BlockKind;

pub static BLOCK: BlockImpl<PusherBlock> = BlockImpl(PusherBlock);

pub mod model;
pub mod texture;
mod render;

impl PlaceableBlock for PusherBlock {
    fn item_slot_color(&self) -> Color {
        rgb(0.42, 0.44, 0.42).color()
    }
}

register_block!(BLOCK, BlockKind::Pusher, editable: false, play: true);
