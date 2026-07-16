use crate::game::blocks::ColorSpecExt;
pub use oif_sim::blocks::laser::LaserBlock;

use bevy::prelude::Color;
use crate::game::blocks::traits::PlaceableBlock;
use crate::game::blocks::rgb;
use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::BlockKind;

pub static BLOCK: BlockImpl<LaserBlock> = BlockImpl(LaserBlock);

mod render;

impl PlaceableBlock for LaserBlock {
    fn item_slot_color(&self) -> Color {
        rgb(0.72, 0.12, 0.26).color()
    }
}

register_block!(BLOCK, BlockKind::Laser, editable: false, play: true);
