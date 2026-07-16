use crate::game::blocks::ColorSpecExt;
pub use oif_sim::blocks::mirror::MirrorBlock;

use bevy::prelude::Color;
use crate::game::blocks::traits::PlaceableBlock;
use crate::game::blocks::rgb;
use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::BlockKind;

pub static BLOCK: BlockImpl<MirrorBlock> = BlockImpl(MirrorBlock);

mod render;

impl PlaceableBlock for MirrorBlock {
    fn item_slot_color(&self) -> Color {
        rgb(1.0, 1.0, 1.0).color()
    }
}

register_block!(BLOCK, BlockKind::Mirror, editable: false, play: true);
