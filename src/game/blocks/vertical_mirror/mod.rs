use crate::game::blocks::ColorSpecExt;
pub use oif_sim::blocks::vertical_mirror::VerticalMirrorBlock;

use bevy::prelude::Color;
use crate::game::blocks::traits::PlaceableBlock;
use crate::game::blocks::rgb;
use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::BlockKind;

pub static BLOCK: BlockImpl<VerticalMirrorBlock> = BlockImpl(VerticalMirrorBlock);

mod render;

impl PlaceableBlock for VerticalMirrorBlock {
    fn item_slot_color(&self) -> Color {
        rgb(1.0, 1.0, 1.0).color()
    }
}

register_block!(BLOCK, BlockKind::VerticalMirror, editable: false, play: true);
