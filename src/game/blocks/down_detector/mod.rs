use crate::game::blocks::ColorSpecExt;
pub use oif_sim::blocks::down_detector::DownDetectorBlock;

use bevy::prelude::Color;
use crate::game::blocks::traits::PlaceableBlock;
use crate::game::blocks::rgb;
use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::BlockKind;

pub static BLOCK: BlockImpl<DownDetectorBlock> = BlockImpl(DownDetectorBlock);

mod render;

impl PlaceableBlock for DownDetectorBlock {
    fn item_slot_color(&self) -> Color {
        rgb(0.14, 0.40, 0.68).color()
    }
}

register_block!(BLOCK, BlockKind::DownDetector, editable: false, play: true);
