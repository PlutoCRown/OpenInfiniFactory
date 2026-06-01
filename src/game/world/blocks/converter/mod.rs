use super::{
    rgb, Block, BlockDefinition, BlockEditContext, BlockKind, BlockModel, BlockRenderAssets,
    EditableBlock,
};
use crate::game::ui::{BlockEditAction, BlockPanelDropdown, UiPanelId};
use crate::game::world::blocks::SerializedBlockState;
use crate::game::world::grid::WorldBlocks;

mod definition;
mod render;
mod state;
mod ui;

pub use state::{ConverterMode, ConverterSettings};

pub struct ConverterBlock;

pub static CONVERTER: ConverterBlock = ConverterBlock;

impl Block for ConverterBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Converter
    }

    fn definition(&self) -> BlockDefinition {
        definition::definition(self)
    }

    fn default_state(
        &self,
        pos: bevy::prelude::IVec3,
        world: &WorldBlocks,
    ) -> Option<SerializedBlockState> {
        state::default_state(pos, world)
    }

    fn normalize_state(
        &self,
        state: &SerializedBlockState,
        pos: bevy::prelude::IVec3,
    ) -> Option<SerializedBlockState> {
        state::normalize_state(state, pos)
    }

    fn render_assets(&self) -> BlockRenderAssets {
        render::assets(self)
    }

    fn model(&self) -> BlockModel {
        render::model(self)
    }
}
impl EditableBlock for ConverterBlock {
    fn ui_panel(&self) -> Option<UiPanelId> {
        ui::ui_panel(self)
    }
    fn handle_edit_action(&self, ctx: &mut BlockEditContext, action: BlockEditAction) {
        ui::handle_edit_action(self, ctx, action)
    }
}
