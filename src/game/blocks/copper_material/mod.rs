pub use oif_sim::blocks::copper_material::CopperMaterial;

use bevy::prelude::{Color, Image};

use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::traits::{BlockRender, BlockUi, PlaceableBlock};
use crate::game::state::UiPanelId;
use crate::game::blocks::ColorSpecExt;
use crate::game::blocks::{BlockKind, rgb};

pub static BLOCK: BlockImpl<CopperMaterial> = BlockImpl(CopperMaterial);

mod texture;

impl BlockRender for CopperMaterial {
    fn block_texture(&self) -> Option<Image> {
        Some(texture::image())
    }
}

impl PlaceableBlock for CopperMaterial {
    fn item_slot_color(&self) -> Color {
        rgb(0.68, 0.34, 0.16).color()
    }
}

impl BlockUi for CopperMaterial {
    fn ui_panel(&self) -> Option<UiPanelId> {
        None
    }
}

register_block!(BLOCK, BlockKind::CopperMaterial, editable: false);
