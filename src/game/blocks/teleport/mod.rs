use crate::game::blocks::ColorSpecExt;
pub use oif_sim::blocks::teleport::TeleportBlock;

use bevy::prelude::Color;
use crate::game::blocks::traits::PlaceableBlock;
use crate::game::blocks::rgb;
use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::BlockKind;

pub static BLOCK: BlockImpl<TeleportBlock> = BlockImpl(TeleportBlock);

mod prompt;
mod render;
mod ui;
pub mod visual;

impl PlaceableBlock for TeleportBlock {
    fn item_slot_color(&self) -> Color {
        rgb(0.42, 0.12, 0.72).color()
    }
}

register_block!(BLOCK, BlockKind::Teleport, editable: true);
