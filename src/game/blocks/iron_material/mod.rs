pub use oif_sim::blocks::iron_material::IronMaterial;

use bevy::prelude::{Color, Image};

use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::traits::{BlockRender, BlockUi, PlaceableBlock};
use crate::game::state::UiPanelId;
use crate::game::blocks::ColorSpecExt;
use crate::game::blocks::{BlockKind, rgb};

pub static BLOCK: BlockImpl<IronMaterial> = BlockImpl(IronMaterial);

mod texture;

impl BlockRender for IronMaterial {
    fn block_texture(&self) -> Option<Image> {
        Some(texture::image())
    }
}

impl PlaceableBlock for IronMaterial {
    fn item_slot_color(&self) -> Color {
        rgb(0.54, 0.56, 0.58).color()
    }
}

impl BlockUi for IronMaterial {
    fn ui_panel(&self) -> Option<UiPanelId> {
        None
    }
}

register_block!(BLOCK, BlockKind::IronMaterial, editable: false);
