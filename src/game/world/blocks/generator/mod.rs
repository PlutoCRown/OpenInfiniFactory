use super::{
    rgba, Block, BlockDefinition, BlockEditContext, BlockKind, EditableBlock, MaterialSource,
};
use crate::game::ui::{BlockEditAction, BlockPanelDropdown, UiPanelId};
use crate::game::world::blocks::SerializedBlockState;
use crate::game::world::grid::WorldBlocks;

mod definition;
mod simulation;
mod state;
pub(crate) mod ui;

pub(crate) use state::{
    set_settings as set_generator_settings, settings as generator_settings, GeneratorSettings,
};

pub struct GeneratorBlock;

pub static GENERATOR: GeneratorBlock = GeneratorBlock;

impl Block for GeneratorBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Generator
    }

    fn definition(&self) -> BlockDefinition {
        definition::definition(self)
    }

    fn material_source(&self, facing: super::Facing) -> Option<MaterialSource> {
        simulation::material_source(self, facing)
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
impl EditableBlock for GeneratorBlock {
    fn ui_panel(&self) -> Option<UiPanelId> {
        ui::ui_panel(self)
    }
    fn handle_edit_action(&self, ctx: &mut BlockEditContext, action: BlockEditAction) {
        ui::handle_edit_action(self, ctx, action)
    }
}
