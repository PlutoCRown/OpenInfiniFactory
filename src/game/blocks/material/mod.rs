pub use oif_sim::blocks::material::BasicMaterial;

use bevy::prelude::{Color, Image};

use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::traits::{BlockRender, BlockUi, PlaceableBlock};
use crate::game::state::UiPanelId;
use crate::game::blocks::ColorSpecExt;
use crate::game::blocks::{BlockKind, rgb};

pub static BLOCK: BlockImpl<BasicMaterial> = BlockImpl(BasicMaterial);

mod texture;

impl BlockRender for BasicMaterial {
    fn block_texture(&self) -> Option<Image> {
        Some(texture::image())
    }
}

impl PlaceableBlock for BasicMaterial {
    fn item_slot_color(&self) -> Color {
        rgb(0.74, 0.74, 0.78).color()
    }
}

impl BlockUi for BasicMaterial {
    fn ui_panel(&self) -> Option<UiPanelId> {
        None
    }
}

register_block!(BLOCK, BlockKind::Material, editable: false);
