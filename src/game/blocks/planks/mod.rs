use crate::game::state::UiPanelId;
pub use oif_sim::blocks::planks::Planks;

use bevy::prelude::{Color, Image};

use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::traits::{BlockRender, BlockUi, PlaceableBlock};
use crate::game::blocks::ColorSpecExt;
use crate::game::blocks::{BlockKind, rgb};

pub static BLOCK: BlockImpl<Planks> = BlockImpl(Planks);

mod texture;

impl BlockRender for Planks {
    fn block_texture(&self) -> Option<Image> {
        Some(texture::image())
    }
}

impl PlaceableBlock for Planks {
    fn item_slot_color(&self) -> Color {
        rgb(0.62, 0.40, 0.20).color()
    }
}

impl BlockUi for Planks {
    fn ui_panel(&self) -> Option<UiPanelId> {
        None
    }
}

register_block!(BLOCK, BlockKind::Planks, editable: true);
