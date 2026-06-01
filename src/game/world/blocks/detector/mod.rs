use bevy::prelude::IVec3;

use super::{
    rgb, Block, BlockDefinition, BlockKind, BlockModel, BlockRenderAssets, RenderBehavior,
    SignalBehavior,
};

mod definition;
mod render;
mod simulation;

pub struct DetectorBlock;

pub static DETECTOR: DetectorBlock = DetectorBlock;

impl Block for DetectorBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Detector
    }

    fn definition(&self) -> BlockDefinition {
        definition::definition(self)
    }

    fn is_directional(&self) -> bool {
        simulation::is_directional(self)
    }

    fn signal_behavior(&self, facing: super::Facing) -> Option<SignalBehavior> {
        simulation::signal_behavior(self, facing)
    }

    fn factory_connection_blocker(&self, facing: super::Facing) -> Option<IVec3> {
        simulation::factory_connection_blocker(self, facing)
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
