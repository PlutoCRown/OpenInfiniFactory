use super::{rgb, Block, BlockDefinition, BlockKind, MaterialBlock, MaterialKind};

pub struct BasicMaterialBlock;

pub static MATERIAL: BasicMaterialBlock = BasicMaterialBlock;

impl Block for BasicMaterialBlock {
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

    fn material_kind(&self) -> Option<MaterialKind> {
        Some(MaterialKind::Basic)
    }
}

impl MaterialBlock for BasicMaterialBlock {}
