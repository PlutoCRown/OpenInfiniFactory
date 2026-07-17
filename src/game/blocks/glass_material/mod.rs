pub use oif_sim::blocks::glass_material::GlassMaterial;

use bevy::prelude::{Color, Image};

use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::traits::{BlockRender, BlockUi, PlaceableBlock};
use crate::game::state::UiPanelId;
use crate::game::blocks::ColorSpecExt;
use crate::game::blocks::{BlockKind, rgba};

pub static BLOCK: BlockImpl<GlassMaterial> = BlockImpl(GlassMaterial);

mod texture;

impl BlockRender for GlassMaterial {
    fn block_texture(&self) -> Option<Image> {
        Some(texture::image())
    }
}

impl PlaceableBlock for GlassMaterial {
    fn item_slot_color(&self) -> Color {
        rgba(0.72, 0.88, 0.94, 0.45).color()
    }
}

impl BlockUi for GlassMaterial {
    fn ui_panel(&self) -> Option<UiPanelId> {
        None
    }
}

register_block!(BLOCK, BlockKind::GlassMaterial, editable: false);
