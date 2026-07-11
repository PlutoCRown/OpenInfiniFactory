use crate::game::blocks::ColorSpecExt;
pub use oif_sim::blocks::down_welder::DownWelderBlock;

use bevy::prelude::Color;
use crate::game::blocks::traits::PlaceableBlock;
use crate::game::blocks::rgb;
use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::BlockKind;

pub static BLOCK: BlockImpl<DownWelderBlock> = BlockImpl(DownWelderBlock);

mod render;

impl PlaceableBlock for DownWelderBlock {
    fn item_slot_color(&self) -> Color {
        rgb(0.08, 0.26, 0.58).color()
    }
}

register_block!(BLOCK, BlockKind::DownWelder, editable: false, play: true);
