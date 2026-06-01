use bevy::prelude::IVec3;

use super::{
    rgb, Block, BlockDefinition, BlockKind, BlockModel, BlockRenderAssets, Facing, MarkerBehavior,
    MaterialDestroyer, RenderBehavior, SignalBehavior,
};

mod definition;
mod render;
mod simulation;

pub struct DrillBlock;

pub static DRILL: DrillBlock = DrillBlock;

impl Block for DrillBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Drill
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

    fn material_destroyer(&self, facing: Facing) -> Option<MaterialDestroyer> {
        simulation::material_destroyer(self, facing)
    }

    fn factory_connection_blocker(&self, facing: Facing) -> Option<IVec3> {
        simulation::factory_connection_blocker(self, facing)
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
