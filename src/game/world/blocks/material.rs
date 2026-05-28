use super::{rgb, Block, BlockDefinition, BlockKind, SystemBlock};

pub struct MaterialBlock;

pub static MATERIAL: MaterialBlock = MaterialBlock;

impl Block for MaterialBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Material
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::material(
            self.id(),
            "block.material",
            "short.material",
            rgb(0.82, 0.82, 0.86),
            rgb(0.74, 0.74, 0.78),
        )
    }
}

impl SystemBlock for MaterialBlock {}
