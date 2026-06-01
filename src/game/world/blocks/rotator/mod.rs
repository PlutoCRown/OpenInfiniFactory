use super::{rgb, Block, BlockDefinition, BlockKind, BlockModel, BlockRenderAssets, MovementRule};

mod definition;
mod render;
mod simulation;

pub struct RotatorBlock;

pub static ROTATOR: RotatorBlock = RotatorBlock;

impl Block for RotatorBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Rotator
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
