use crate::game::state::UiPanelId;
pub use oif_sim::blocks::dirt::Dirt;

use bevy::prelude::{Color, Image};

use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::traits::{BlockRender, BlockUi, PlaceableBlock};
use crate::game::blocks::ColorSpecExt;
use crate::game::blocks::{BlockKind, rgb};

pub static BLOCK: BlockImpl<Dirt> = BlockImpl(Dirt);

mod texture;

impl BlockRender for Dirt {
    fn block_texture(&self) -> Option<Image> {
        Some(texture::image())
    }
}

impl PlaceableBlock for Dirt {
    fn item_slot_color(&self) -> Color {
        rgb(0.42, 0.26, 0.14).color()
    }
}

impl BlockUi for Dirt {
    fn ui_panel(&self) -> Option<UiPanelId> {
        None
    }
}

register_block!(BLOCK, BlockKind::Dirt, editable: true);
