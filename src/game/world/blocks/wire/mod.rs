use super::{rgb, Block, BlockDefinition, BlockKind, RenderBehavior, SignalBehavior};

mod definition;
mod render;
mod simulation;

pub struct WireBlock;

pub static WIRE: WireBlock = WireBlock;

impl Block for WireBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Wire
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
}
