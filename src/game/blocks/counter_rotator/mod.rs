use crate::game::blocks::ColorSpecExt;
pub use oif_sim::blocks::counter_rotator::CounterRotatorBlock;

use bevy::prelude::Color;
use crate::game::blocks::traits::PlaceableBlock;
use crate::game::blocks::rgb;
use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::BlockKind;

pub static BLOCK: BlockImpl<CounterRotatorBlock> = BlockImpl(CounterRotatorBlock);

mod render;

impl PlaceableBlock for CounterRotatorBlock {
    fn item_slot_color(&self) -> Color {
        rgb(0.54, 0.22, 0.68).color()
    }
}

register_block!(BLOCK, BlockKind::CounterRotator, editable: false, play: true);
