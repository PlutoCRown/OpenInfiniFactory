use crate::game::blocks::ColorSpecExt;
pub use oif_sim::blocks::stamper::StamperBlock;

use bevy::prelude::Color;
use crate::game::blocks::traits::PlaceableBlock;
use crate::game::blocks::rgb;
use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::BlockKind;

pub static BLOCK: BlockImpl<StamperBlock> = BlockImpl(StamperBlock);

mod render;
mod ui;

impl PlaceableBlock for StamperBlock {
    fn item_slot_color(&self) -> Color {
        rgb(0.64, 0.14, 0.42).color()
    }
}

register_block!(BLOCK, BlockKind::Stamper, editable: true);
