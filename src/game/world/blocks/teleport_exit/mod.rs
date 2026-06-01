use super::{rgb, Block, BlockDefinition, BlockKind, BlockModel, BlockRenderAssets, EditableBlock};
use crate::game::ui::UiPanelId;
use crate::game::world::blocks::teleport_entrance::state;
use crate::game::world::blocks::SerializedBlockState;
use crate::game::world::grid::WorldBlocks;

mod definition;
mod render;
pub(crate) mod ui;

pub struct TeleportExitBlock;

pub static TELEPORT_EXIT: TeleportExitBlock = TeleportExitBlock;

impl Block for TeleportExitBlock {
    fn id(&self) -> BlockKind {
        BlockKind::TeleportExit
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
impl EditableBlock for TeleportExitBlock {
    fn ui_panel(&self) -> Option<UiPanelId> {
        ui::ui_panel(self)
    }
}
