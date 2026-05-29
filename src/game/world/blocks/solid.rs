use super::{
    rgb, Block, BlockDefinition, BlockKind, BlockModel, BlockModelPart, FactoryBlock,
    ModelMaterial, ModelMesh,
};

const MODEL: &[BlockModelPart] = &[
    BlockModelPart::new(ModelMesh::Plate, ModelMaterial::DarkFrame, [0.0, 0.54, 0.0]),
    BlockModelPart::new(ModelMesh::Plate, ModelMaterial::DarkFrame, [0.0, -0.54, 0.0]),
    BlockModelPart::new(ModelMesh::Small, ModelMaterial::Frame, [-0.28, 0.0, -0.28]),
    BlockModelPart::new(ModelMesh::Small, ModelMaterial::Frame, [0.28, 0.0, -0.28]),
    BlockModelPart::new(ModelMesh::Small, ModelMaterial::Frame, [-0.28, 0.0, 0.28]),
    BlockModelPart::new(ModelMesh::Small, ModelMaterial::Frame, [0.28, 0.0, 0.28]),
];

pub struct SolidBlock;

pub static SOLID: SolidBlock = SolidBlock;

impl Block for SolidBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Solid
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.solid",
            "short.solid",
            rgb(0.46, 0.48, 0.50),
            rgb(0.38, 0.39, 0.40),
        )
    }

    fn model(&self) -> BlockModel {
        BlockModel::Parts(MODEL)
    }
}

impl FactoryBlock for SolidBlock {}
