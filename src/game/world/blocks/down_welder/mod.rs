use bevy::prelude::*;

use super::{
    rgb, Block, BlockDefinition, BlockKind, BlockModel, Facing, MarkerBehavior, RenderBehavior,
};

mod definition;
mod render;
mod simulation;

pub struct DownWelderBlock;

pub static DOWN_WELDER: DownWelderBlock = DownWelderBlock;

impl Block for DownWelderBlock {
    fn id(&self) -> BlockKind {
        BlockKind::DownWelder
    }

    fn definition(&self) -> BlockDefinition {
        definition::definition(self)
    }

    fn marker_behavior(&self, _facing: Facing) -> Option<MarkerBehavior> {
        simulation::marker_behavior(self, _facing)
    }

    fn factory_connection_blocker(&self, _facing: Facing) -> Option<IVec3> {
        simulation::factory_connection_blocker(self, _facing)
    }

    fn render_behavior(&self, _facing: Facing) -> RenderBehavior {
        render::render_behavior(self, _facing)
    }

    fn model(&self) -> BlockModel {
        render::model(self)
    }

    fn alternate(&self) -> Option<BlockKind> {
        simulation::alternate(self)
    }
}
