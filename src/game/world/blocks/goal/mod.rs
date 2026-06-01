use super::{
    rgb, Block, BlockDefinition, BlockEditContext, BlockKind, BlockModel, BlockRenderAssets,
    EditableBlock, RenderBehavior,
};
use crate::game::ui::{BlockEditAction, BlockPanelDropdown, UiPanelId};
use crate::game::world::blocks::SerializedBlockState;
use crate::game::world::grid::WorldBlocks;

mod definition;
mod render;
mod state;
mod ui;

pub use state::GoalSettings;

pub struct GoalBlock;

pub static GOAL: GoalBlock = GoalBlock;

impl Block for GoalBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Goal
    }

    fn definition(&self) -> BlockDefinition {
        definition::definition(self)
    }

    fn render_behavior(&self, _facing: super::Facing) -> RenderBehavior {
        render::render_behavior(self, _facing)
    }

    fn render_assets(&self) -> BlockRenderAssets {
        render::assets(self)
    }

    fn model(&self) -> BlockModel {
        render::model(self)
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
}
impl EditableBlock for GoalBlock {
    fn ui_panel(&self) -> Option<UiPanelId> {
        ui::ui_panel(self)
    }
    fn handle_edit_action(&self, ctx: &mut BlockEditContext, action: BlockEditAction) {
        ui::handle_edit_action(self, ctx, action)
    }
}
