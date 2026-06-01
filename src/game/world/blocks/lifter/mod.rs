use bevy::prelude::IVec3;

use super::{rgb, Block, BlockDefinition, BlockKind, BlockModel, BlockRenderAssets, MovementRule};

mod definition;
mod render;
mod simulation;

pub struct LifterBlock;

pub static LIFTER: LifterBlock = LifterBlock;

impl Block for LifterBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Lifter
    }

    fn definition(&self) -> BlockDefinition {
        definition::definition(self)
    }

    fn is_directional(&self) -> bool {
        simulation::is_directional(self)
    }

    fn movement_rule(&self, _facing: super::Facing) -> Option<MovementRule> {
        simulation::movement_rule(self, _facing)
    }

    fn factory_connection_blocker(&self, _facing: super::Facing) -> Option<IVec3> {
        simulation::factory_connection_blocker(self, _facing)
    }

    fn render_assets(&self) -> BlockRenderAssets {
        render::assets(self)
    }

    fn model(&self) -> BlockModel {
        render::model(self)
    }
}
