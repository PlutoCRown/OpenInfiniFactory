use crate::game::blocks::ColorSpecExt;
pub use oif_sim::blocks::roller::RollerBlock;

use crate::game::blocks::BlockKind;
use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::rgb;
use crate::game::blocks::traits::PlaceableBlock;
use bevy::prelude::Color;

pub static BLOCK: BlockImpl<RollerBlock> = BlockImpl(RollerBlock);

mod render;
mod ui;

impl PlaceableBlock for RollerBlock {
    fn item_slot_color(&self) -> Color {
        rgb(0.10, 0.44, 0.60).color()
    }
}

register_block!(BLOCK, BlockKind::Roller, editable: true);
