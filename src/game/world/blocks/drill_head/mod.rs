use super::{rgb, Block, BlockDefinition, BlockKind, BlockModel, MaterialDestroyer};

mod definition;
mod render;
mod simulation;

pub struct DrillHeadBlock;

pub static DRILL_HEAD: DrillHeadBlock = DrillHeadBlock;

impl Block for DrillHeadBlock {
    fn id(&self) -> BlockKind {
        BlockKind::DrillHead
    }

    fn definition(&self) -> BlockDefinition {
        definition::definition(self)
    }

    fn material_destroyer(&self, _facing: super::Facing) -> Option<MaterialDestroyer> {
        simulation::material_destroyer(self, _facing)
    }

    fn model(&self) -> BlockModel {
        render::model(self)
    }
}
