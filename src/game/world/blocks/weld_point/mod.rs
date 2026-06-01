use super::{
    rgb, rgba, Block, BlockDefinition, BlockKind, BlockModel, BlockRenderAssets, RenderBehavior,
    WeldBehavior,
};

mod definition;
mod render;
mod simulation;

pub struct WeldPointBlock;

pub static WELD_POINT: WeldPointBlock = WeldPointBlock;

impl Block for WeldPointBlock {
    fn id(&self) -> BlockKind {
        BlockKind::WeldPoint
    }

    fn definition(&self) -> BlockDefinition {
        definition::definition(self)
    }

    fn render_behavior(&self, _facing: super::Facing) -> RenderBehavior {
        render::render_behavior(self, _facing)
    }

    fn weld_behavior(&self) -> Option<WeldBehavior> {
        simulation::weld_behavior(self)
    }

    fn render_assets(&self) -> BlockRenderAssets {
        render::assets(self)
    }

    fn model(&self) -> BlockModel {
        render::model(self)
    }
}
