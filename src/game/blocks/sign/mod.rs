use crate::game::blocks::ColorSpecExt;
pub use oif_sim::blocks::sign::SignBlock;

use bevy::prelude::Color;
use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::rgb;
use crate::game::blocks::traits::PlaceableBlock;
use crate::game::blocks::BlockKind;

pub static BLOCK: BlockImpl<SignBlock> = BlockImpl(SignBlock);

mod render;
mod ui;

impl PlaceableBlock for SignBlock {
    fn item_slot_color(&self) -> Color {
        rgb(0.72, 0.58, 0.32).color()
    }
}

register_block!(BLOCK, BlockKind::Sign, editable: true, play: true);
