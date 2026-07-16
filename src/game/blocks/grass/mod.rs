use crate::game::state::UiPanelId;
pub use oif_sim::blocks::grass::Grass;

use bevy::prelude::{Color, Image};

use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::traits::{BlockRender, BlockUi, PlaceableBlock};
use crate::game::blocks::ColorSpecExt;
use crate::game::blocks::{BlockKind, rgb};

pub static BLOCK: BlockImpl<Grass> = BlockImpl(Grass);

mod texture;

impl BlockRender for Grass {
    fn block_texture(&self) -> Option<Image> {
        Some(texture::image())
    }
}

impl PlaceableBlock for Grass {
    fn item_slot_color(&self) -> Color {
        rgb(0.27, 0.56, 0.20).color()
    }
}

impl BlockUi for Grass {
    fn ui_panel(&self) -> Option<UiPanelId> {
        None
    }
}

register_block!(BLOCK, BlockKind::Grass, editable: true);
