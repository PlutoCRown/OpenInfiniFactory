use super::{rgb, Block, BlockDefinition, BlockKind, FactoryBlock, MaterialMover};

pub struct LifterBlock;

pub static LIFTER: LifterBlock = LifterBlock;

impl Block for LifterBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Lifter
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.lifter",
            "short.lifter",
            rgb(0.25, 0.58, 0.72),
            rgb(0.18, 0.48, 0.62),
        )
    }

    fn is_directional(&self) -> bool {
        true
    }

    fn material_mover(&self, _facing: super::Facing) -> Option<MaterialMover> {
        Some(MaterialMover::Lifter)
    }
}

impl FactoryBlock for LifterBlock {}
