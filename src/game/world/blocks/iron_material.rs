use super::{rgb, Block, BlockDefinition, BlockKind, MaterialBlock, MaterialKind};

pub struct IronMaterialBlock;

pub static IRON_MATERIAL: IronMaterialBlock = IronMaterialBlock;

impl Block for IronMaterialBlock {
    fn id(&self) -> BlockKind {
        BlockKind::IronMaterial
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::material(
            self.id(),
            "block.iron_material",
            "short.iron_material",
            rgb(0.62, 0.64, 0.66),
            rgb(0.54, 0.56, 0.58),
        )
    }

    fn material_kind(&self) -> Option<MaterialKind> {
        Some(MaterialKind::Iron)
    }
}

impl MaterialBlock for IronMaterialBlock {}
