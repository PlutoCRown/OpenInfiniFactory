use crate::game::blocks::ColorSpecExt;
pub use oif_sim::blocks::teleport_exit::TeleportExitBlock;

use bevy::prelude::Color;
use crate::game::blocks::traits::PlaceableBlock;
use crate::game::blocks::rgb;
use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::BlockKind;

pub static BLOCK: BlockImpl<TeleportExitBlock> = BlockImpl(TeleportExitBlock);

mod render;
mod ui;

impl PlaceableBlock for TeleportExitBlock {
    fn item_slot_color(&self) -> Color {
        rgb(0.50, 0.20, 0.74).color()
    }
}

register_block!(BLOCK, BlockKind::TeleportExit, editable: true);
