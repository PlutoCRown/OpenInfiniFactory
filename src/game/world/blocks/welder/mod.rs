use bevy::prelude::IVec3;

use super::{
    rgb, Block, BlockDefinition, BlockKind, BlockModel, Facing, MarkerBehavior, RenderBehavior,
};

mod definition;
mod render;
mod simulation;

pub struct WelderBlock;

pub static WELDER: WelderBlock = WelderBlock;

impl Block for WelderBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Welder
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

    fn factory_connection_blocker(&self, facing: Facing) -> Option<IVec3> {
        simulation::factory_connection_blocker(self, facing)
    }

    fn render_behavior(&self, facing: Facing) -> RenderBehavior {
        render::render_behavior(self, facing)
    }

    fn model(&self) -> BlockModel {
        render::model(self)
    }

    fn alternate(&self) -> Option<BlockKind> {
        simulation::alternate(self)
    }
}
