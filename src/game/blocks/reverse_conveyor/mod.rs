use crate::game::blocks::ColorSpecExt;
pub use oif_sim::blocks::reverse_conveyor::ReverseConveyorBlock;

use bevy::prelude::Color;
use crate::game::blocks::traits::PlaceableBlock;
use crate::game::blocks::rgb;
use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::BlockKind;

pub static BLOCK: BlockImpl<ReverseConveyorBlock> = BlockImpl(ReverseConveyorBlock);

mod render;

impl PlaceableBlock for ReverseConveyorBlock {
    fn item_slot_color(&self) -> Color {
        rgb(0.70, 0.34, 0.08).color()
    }
}

register_block!(BLOCK, BlockKind::ReverseConveyor, editable: false, play: true);
