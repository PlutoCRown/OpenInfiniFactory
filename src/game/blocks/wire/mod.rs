use crate::game::blocks::ColorSpecExt;
pub use oif_sim::blocks::wire::WireBlock;

use bevy::prelude::Color;
use crate::game::blocks::traits::PlaceableBlock;
use crate::game::blocks::rgb;
use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::BlockKind;

pub static BLOCK: BlockImpl<WireBlock> = BlockImpl(WireBlock);

mod render;

impl PlaceableBlock for WireBlock {
    fn item_slot_color(&self) -> Color {
        rgb(0.88, 0.62, 0.12).color()
    }
}

register_block!(BLOCK, BlockKind::Wire, editable: false, play: true);
