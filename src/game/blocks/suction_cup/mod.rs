use crate::game::blocks::ColorSpecExt;
pub use oif_sim::blocks::suction_cup::SuctionCupBlock;

use bevy::prelude::Color;
use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::rgb;
use crate::game::blocks::traits::PlaceableBlock;
use crate::game::blocks::BlockKind;

pub static BLOCK: BlockImpl<SuctionCupBlock> = BlockImpl(SuctionCupBlock);

mod render;

impl PlaceableBlock for SuctionCupBlock {
    fn item_slot_color(&self) -> Color {
        rgb(0.82, 0.84, 0.82).color()
    }
}

register_block!(BLOCK, BlockKind::SuctionCup, editable: false, play: true);
