use super::{
    edit_labeler, rgb, Block, BlockDefinition, BlockEditContext, BlockKind, BlockModel,
    BlockRenderAssets, EditableBlock, MaterialLabeler,
};
use crate::game::ui::{BlockEditAction, UiPanelId};
use crate::game::world::blocks::SerializedBlockState;
use crate::game::world::grid::WorldBlocks;

mod definition;
mod render;
mod simulation;
mod state;
pub(crate) mod ui;

pub(crate) use state::{
    set_settings as set_roller_settings, settings as roller_settings, RollerSettings,
};

pub struct RollerBlock;

pub static ROLLER: RollerBlock = RollerBlock;

impl Block for RollerBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Roller
    }

    fn definition(&self) -> BlockDefinition {
        definition::definition(self)
    }

    fn is_directional(&self) -> bool {
        simulation::is_directional(self)
    }

    fn material_labeler(&self, facing: super::Facing) -> Option<MaterialLabeler> {
        simulation::material_labeler(self, facing)
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
impl EditableBlock for RollerBlock {
    fn ui_panel(&self) -> Option<UiPanelId> {
        ui::ui_panel(self)
    }
    fn handle_edit_action(&self, ctx: &mut BlockEditContext, action: BlockEditAction) {
        ui::handle_edit_action(self, ctx, action)
    }
}
