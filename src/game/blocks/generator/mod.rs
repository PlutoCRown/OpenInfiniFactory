pub use oif_sim::blocks::generator::GeneratorBlock;

use bevy::prelude::Color;
use crate::game::blocks::traits::{BlockRender, PlaceableBlock};
use crate::game::blocks::ColorSpecExt;
use crate::game::blocks::{rgba};
use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::BlockKind;

pub static BLOCK: BlockImpl<GeneratorBlock> = BlockImpl(GeneratorBlock);

mod ui;

impl PlaceableBlock for GeneratorBlock {
    fn item_slot_color(&self) -> Color {
        rgba(0.32, 0.48, 0.82, 0.46).color()
    }
}

impl BlockRender for GeneratorBlock {}

register_block!(BLOCK, BlockKind::Generator, editable: true);
