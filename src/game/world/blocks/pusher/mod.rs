use super::{
    rgb, Block, BlockDefinition, BlockKind, BlockModel, BlockRenderAssets, MovementRule,
    RenderBehavior, SignalBehavior,
};

mod definition;
mod render;
mod simulation;

pub struct PusherBlock;

pub static PUSHER: PusherBlock = PusherBlock;

impl Block for PusherBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Pusher
    }

    fn definition(&self) -> BlockDefinition {
        definition::definition(self)
    }

    fn is_directional(&self) -> bool {
        simulation::is_directional(self)
    }

    fn movement_rule(&self, facing: super::Facing) -> Option<MovementRule> {
        simulation::movement_rule(self, facing)
    }

    fn signal_behavior(&self, _facing: super::Facing) -> Option<SignalBehavior> {
        simulation::signal_behavior(self, _facing)
    }

    fn render_behavior(&self, facing: super::Facing) -> RenderBehavior {
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
