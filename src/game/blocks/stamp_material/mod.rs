pub use oif_sim::blocks::stamp_material::StampMaterial;

use bevy::prelude::{Color, Image};

use crate::game::blocks::adapter::BlockImpl;
use crate::game::blocks::traits::{BlockRender, BlockUi, PlaceableBlock};
use crate::game::blocks::{BlockKind, BlockModel, ColorSpecExt, rgb};
use crate::game::state::UiPanelId;

pub static BLOCK: BlockImpl<StampMaterial> = BlockImpl(StampMaterial);

impl BlockRender for StampMaterial {
    fn model(&self) -> BlockModel {
        // 无实体立方体：面片由 stamp_face_colors 挂在印花/宿主表面上
        BlockModel::PartsOnly(&[])
    }

    fn block_texture(&self) -> Option<Image> {
        None
    }
}

impl PlaceableBlock for StampMaterial {
    fn item_slot_color(&self) -> Color {
        rgb(0.95, 0.12, 0.10).color()
    }
}

impl BlockUi for StampMaterial {
    fn ui_panel(&self) -> Option<UiPanelId> {
        None
    }
}

register_block!(BLOCK, BlockKind::StampMaterial, editable: false);
