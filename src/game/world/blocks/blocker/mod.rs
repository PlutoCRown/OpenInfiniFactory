use super::{
    rgb, Block, BlockDefinition, BlockKind, BlockModel, BlockRenderAssets, Facing, MarkerBehavior,
    MovementRule, RenderBehavior, SignalBehavior,
};

mod definition;
mod render;
mod simulation;

pub struct BlockerBlock;

pub static BLOCKER: BlockerBlock = BlockerBlock;

impl Block for BlockerBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Blocker
    }

    fn definition(&self) -> BlockDefinition {
        definition::definition(self)
    }

    fn is_directional(&self) -> bool {
        simulation::is_directional(self)
    }

    fn marker_behavior(&self, facing: Facing) -> Option<MarkerBehavior> {
        simulation::marker_behavior(self, facing)
    }

    fn movement_rule(&self, facing: Facing) -> Option<MovementRule> {
        simulation::movement_rule(self, facing)
    }

    fn signal_behavior(&self, _facing: Facing) -> Option<SignalBehavior> {
        simulation::signal_behavior(self, _facing)
    }

    fn render_behavior(&self, facing: Facing) -> RenderBehavior {
        render::render_behavior(self, facing)
    }

    fn render_assets(&self) -> BlockRenderAssets {
        render::assets(self)
    }

    fn model(&self) -> BlockModel {
        render::model(self)
    }

    fn alternate(&self) -> Option<BlockKind> {
        simulation::alternate(self)
    }
}
