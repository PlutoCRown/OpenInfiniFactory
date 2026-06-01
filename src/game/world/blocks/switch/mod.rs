use super::{
    rgb, Block, BlockDefinition, BlockKind, BlockModel, BlockRenderAssets, RenderBehavior,
    SignalBehavior,
};

mod definition;
mod render;
mod simulation;

pub struct SwitchBlock;

pub static SWITCH: SwitchBlock = SwitchBlock;

impl Block for SwitchBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Switch
    }

    fn definition(&self) -> BlockDefinition {
        definition::definition(self)
    }

    fn signal_behavior(&self, _facing: super::Facing) -> Option<SignalBehavior> {
        simulation::signal_behavior(self, _facing)
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
}
