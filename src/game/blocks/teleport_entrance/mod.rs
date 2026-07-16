use crate::game::blocks::ColorSpecExt;
pub use oif_sim::blocks::teleport_entrance::TeleportEntranceBlock;

use bevy::prelude::Color;
use crate::game::blocks::traits::PlaceableBlock;
use crate::game::blocks::rgb;
use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::BlockKind;

pub static BLOCK: BlockImpl<TeleportEntranceBlock> = BlockImpl(TeleportEntranceBlock);

mod prompt;
mod render;
mod ui;

impl PlaceableBlock for TeleportEntranceBlock {
    fn item_slot_color(&self) -> Color {
        rgb(0.06, 0.42, 0.72).color()
    }
}

register_block!(BLOCK, BlockKind::TeleportEntrance, editable: true);
