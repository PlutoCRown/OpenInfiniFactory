use super::{
    rgba, Block, BlockDefinition, BlockEditContext, BlockKind, EditableBlock, MaterialSource,
};
use crate::game::ui::{BlockEditAction, BlockPanelDropdown, UiPanelId};
use crate::game::world::grid::{BlockSettings, GeneratorSettings};

mod definition;
mod simulation;
mod ui;

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

    fn default_settings(&self, _pos: bevy::prelude::IVec3) -> Option<BlockSettings> {
        ui::default_settings(self, _pos)
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
