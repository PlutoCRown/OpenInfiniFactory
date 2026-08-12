use crate::game::blocks::ColorSpecExt;
pub use oif_sim::blocks::welder::WelderBlock;

use crate::game::blocks::BlockKind;
use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::rgb;
use crate::game::blocks::traits::PlaceableBlock;
use bevy::prelude::Color;

pub static BLOCK: BlockImpl<WelderBlock> = BlockImpl(WelderBlock);

mod render;

impl PlaceableBlock for WelderBlock {
    fn item_slot_color(&self) -> Color {
        rgb(0.08, 0.26, 0.58).color()
    }
}

register_block!(BLOCK, BlockKind::Welder, editable: false, play: true);
