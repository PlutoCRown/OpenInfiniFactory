use bevy::prelude::*;

use super::{rgb, Block, BlockDefinition, BlockKind, BlockModel, BlockRenderAssets, MovementRule};

mod definition;
mod render;
mod simulation;

pub struct ReverseConveyorBlock;

pub static REVERSE_CONVEYOR: ReverseConveyorBlock = ReverseConveyorBlock;

impl Block for ReverseConveyorBlock {
    fn id(&self) -> BlockKind {
        BlockKind::ReverseConveyor
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

    fn factory_connection_blocker(&self, _facing: super::Facing) -> Option<IVec3> {
        simulation::factory_connection_blocker(self, _facing)
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
