use crate::game::blocks::ColorSpecExt;
pub use oif_sim::blocks::lifter::LifterBlock;

use crate::game::blocks::BlockKind;
use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::rgb;
use crate::game::blocks::traits::PlaceableBlock;
use bevy::prelude::Color;

pub static BLOCK: BlockImpl<LifterBlock> = BlockImpl(LifterBlock);

mod render;

impl PlaceableBlock for LifterBlock {
    fn item_slot_color(&self) -> Color {
        rgb(0.18, 0.48, 0.62).color()
    }
}

register_block!(BLOCK, BlockKind::Lifter, editable: false, play: true);
