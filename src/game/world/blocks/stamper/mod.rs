use super::{
    edit_labeler, rgb, Block, BlockDefinition, BlockEditContext, BlockKind, BlockModel,
    BlockRenderAssets, EditableBlock, MaterialLabeler,
};
use crate::game::ui::{BlockEditAction, UiPanelId};
use crate::game::world::grid::{BlockSettings, LabelerSettings};

mod definition;
mod render;
mod simulation;
mod ui;

pub struct StamperBlock;

pub static STAMPER: StamperBlock = StamperBlock;

impl Block for StamperBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Stamper
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

    fn default_settings(&self, _pos: bevy::prelude::IVec3) -> Option<BlockSettings> {
        ui::default_settings(self, _pos)
    }

    fn render_assets(&self) -> BlockRenderAssets {
        render::assets(self)
    }

    fn model(&self) -> BlockModel {
        render::model(self)
    }
}
impl EditableBlock for StamperBlock {
    fn ui_panel(&self) -> Option<UiPanelId> {
        ui::ui_panel(self)
    }
    fn handle_edit_action(&self, ctx: &mut BlockEditContext, action: BlockEditAction) {
        ui::handle_edit_action(self, ctx, action)
    }
}
