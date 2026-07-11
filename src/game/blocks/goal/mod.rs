pub use oif_sim::blocks::goal::GoalBlock;

use bevy::prelude::Color;
use crate::game::blocks::traits::{BlockRender, PlaceableBlock};
use crate::game::blocks::ColorSpecExt;
use crate::game::blocks::{rgba};
use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::BlockKind;

pub static BLOCK: BlockImpl<GoalBlock> = BlockImpl(GoalBlock);

mod ui;

impl PlaceableBlock for GoalBlock {
    fn item_slot_color(&self) -> Color {
        rgba(0.24, 0.56, 0.30, 0.46).color()
    }
}

impl BlockRender for GoalBlock {}

register_block!(BLOCK, BlockKind::Goal, editable: true);
