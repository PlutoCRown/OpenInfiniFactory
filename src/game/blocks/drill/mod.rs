use crate::game::blocks::ColorSpecExt;
pub use oif_sim::blocks::drill::DrillBlock;

use bevy::prelude::Color;
use crate::game::blocks::traits::PlaceableBlock;
use crate::game::blocks::rgb;
use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::BlockKind;

pub static BLOCK: BlockImpl<DrillBlock> = BlockImpl(DrillBlock);

mod render;

impl PlaceableBlock for DrillBlock {
    fn item_slot_color(&self) -> Color {
        rgb(0.24, 0.26, 0.30).color()
    }
}

register_block!(BLOCK, BlockKind::Drill, editable: false, play: true);
