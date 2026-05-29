use super::{rgb, Block, BlockDefinition, BlockKind, MaterialBlock};

pub struct CopperMaterialBlock;

pub static COPPER_MATERIAL: CopperMaterialBlock = CopperMaterialBlock;

impl Block for CopperMaterialBlock {
    fn id(&self) -> BlockKind {
        BlockKind::CopperMaterial
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::material(
            self.id(),
            "block.copper_material",
            "short.copper_material",
            rgb(0.78, 0.42, 0.22),
            rgb(0.68, 0.34, 0.16),
        )
    }
}

impl MaterialBlock for CopperMaterialBlock {}
