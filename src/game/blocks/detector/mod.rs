use crate::game::blocks::ColorSpecExt;
pub use oif_sim::blocks::detector::DetectorBlock;

use bevy::prelude::Color;
use crate::game::blocks::traits::PlaceableBlock;
use crate::game::blocks::rgb;
use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::BlockKind;

pub static BLOCK: BlockImpl<DetectorBlock> = BlockImpl(DetectorBlock);

mod render;

impl PlaceableBlock for DetectorBlock {
    fn item_slot_color(&self) -> Color {
        rgb(0.12, 0.34, 0.62).color()
    }
}

register_block!(BLOCK, BlockKind::Detector, editable: false, play: true);
