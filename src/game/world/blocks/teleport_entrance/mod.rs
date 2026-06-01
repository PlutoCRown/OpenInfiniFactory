use super::{rgb, Block, BlockDefinition, BlockKind, BlockModel, BlockRenderAssets, EditableBlock};
use crate::game::ui::{BlockPanelDropdown, TeleportAction, UiPanelId};
use crate::game::world::blocks::SerializedBlockState;
use crate::game::world::grid::WorldBlocks;

mod definition;
mod render;
pub mod state;
pub(crate) mod ui;

pub use state::TeleportSettings;

pub struct TeleportEntranceBlock;

pub static TELEPORT_ENTRANCE: TeleportEntranceBlock = TeleportEntranceBlock;

impl Block for TeleportEntranceBlock {
    fn id(&self) -> BlockKind {
        BlockKind::TeleportEntrance
    }

    fn definition(&self) -> BlockDefinition {
        definition::definition(self)
    }

    fn default_state(
        &self,
        pos: bevy::prelude::IVec3,
        world: &WorldBlocks,
    ) -> Option<SerializedBlockState> {
        state::default_state(self.id(), pos, world)
    }

    fn normalize_state(
        &self,
        state: &SerializedBlockState,
        pos: bevy::prelude::IVec3,
    ) -> Option<SerializedBlockState> {
        state::normalize_state(state, pos)
    }

    fn on_removed(&self, pos: bevy::prelude::IVec3, world: &mut WorldBlocks) {
        state::clear_pair_references(pos, world);
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
