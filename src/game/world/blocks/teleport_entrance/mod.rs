use super::{rgb, Block, BlockDefinition, BlockKind, BlockModel, BlockRenderAssets, EditableBlock};
use crate::game::ui::UiPanelId;
use crate::game::world::grid::{BlockSettings, TeleportSettings};

mod definition;
mod render;
mod ui;

pub struct TeleportEntranceBlock;

pub static TELEPORT_ENTRANCE: TeleportEntranceBlock = TeleportEntranceBlock;

impl Block for TeleportEntranceBlock {
    fn id(&self) -> BlockKind {
        BlockKind::TeleportEntrance
    }

    fn definition(&self) -> BlockDefinition {
        definition::definition(self)
    }

    fn default_settings(&self, pos: bevy::prelude::IVec3) -> Option<BlockSettings> {
        ui::default_settings(self, pos)
    }

    fn render_assets(&self) -> BlockRenderAssets {
        render::assets(self)
    }

    fn model(&self) -> BlockModel {
        render::model(self)
    }
}
impl EditableBlock for TeleportEntranceBlock {
    fn ui_panel(&self) -> Option<UiPanelId> {
        ui::ui_panel(self)
    }
}
