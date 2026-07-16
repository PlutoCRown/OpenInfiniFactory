use crate::game::blocks::ColorSpecExt;
pub use oif_sim::blocks::converter::ConverterBlock;

use bevy::prelude::Color;
use crate::game::blocks::traits::PlaceableBlock;
use crate::game::blocks::rgb;
use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::BlockKind;

pub static BLOCK: BlockImpl<ConverterBlock> = BlockImpl(ConverterBlock);

mod render;
mod ui;

impl PlaceableBlock for ConverterBlock {
    fn item_slot_color(&self) -> Color {
        rgb(0.36, 0.24, 0.62).color()
    }
}

register_block!(BLOCK, BlockKind::Converter, editable: true);
