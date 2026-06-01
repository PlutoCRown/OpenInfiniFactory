use bevy::prelude::*;

use super::{
    rgb, Block, BlockDefinition, BlockKind, BlockModel, BlockRenderAssets, RenderBehavior,
    SignalBehavior,
};

mod definition;
mod render;
mod simulation;

pub struct DownDetectorBlock;

pub static DOWN_DETECTOR: DownDetectorBlock = DownDetectorBlock;

impl Block for DownDetectorBlock {
    fn id(&self) -> BlockKind {
        BlockKind::DownDetector
    }

    fn definition(&self) -> BlockDefinition {
        definition::definition(self)
    }

    fn signal_behavior(&self, _facing: super::Facing) -> Option<SignalBehavior> {
        simulation::signal_behavior(self, _facing)
    }

    fn factory_connection_blocker(&self, _facing: super::Facing) -> Option<IVec3> {
        simulation::factory_connection_blocker(self, _facing)
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

    fn alternate(&self) -> Option<BlockKind> {
        simulation::alternate(self)
    }
}
