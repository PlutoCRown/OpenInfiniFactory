use crate::game::blocks::ColorSpecExt;
pub use oif_sim::blocks::rotator::RotatorBlock;

use bevy::prelude::Color;
use crate::game::blocks::traits::PlaceableBlock;
use crate::game::blocks::rgb;
use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::BlockKind;

pub static BLOCK: BlockImpl<RotatorBlock> = BlockImpl(RotatorBlock);

mod render;

impl PlaceableBlock for RotatorBlock {
    fn item_slot_color(&self) -> Color {
        rgb(0.42, 0.26, 0.64).color()
    }
}

register_block!(BLOCK, BlockKind::Rotator, editable: false, play: true);
