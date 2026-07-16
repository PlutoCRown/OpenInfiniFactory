use crate::game::state::UiPanelId;
pub use oif_sim::blocks::stone::Stone;

use bevy::prelude::{Color, Image};

use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::traits::{BlockRender, BlockUi, PlaceableBlock};
use crate::game::blocks::ColorSpecExt;
use crate::game::blocks::{BlockKind, rgb};

pub static BLOCK: BlockImpl<Stone> = BlockImpl(Stone);

mod texture;

impl BlockRender for Stone {
    fn block_texture(&self) -> Option<Image> {
        Some(texture::image())
    }
}

impl PlaceableBlock for Stone {
    fn item_slot_color(&self) -> Color {
        rgb(0.42, 0.42, 0.40).color()
    }
}

impl BlockUi for Stone {
    fn ui_panel(&self) -> Option<UiPanelId> {
        None
    }
}

register_block!(BLOCK, BlockKind::Stone, editable: true);
