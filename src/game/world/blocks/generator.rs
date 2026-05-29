use super::{
    rgb, Block, BlockDefinition, BlockKind, BlockModel, BlockModelPart, EditableBlock,
    MaterialSource, ModelMaterial, ModelMesh, SystemBlock,
};
use crate::game::world::grid::{BlockSettings, GeneratorSettings};

const MODEL: &[BlockModelPart] = &[
    BlockModelPart::new(ModelMesh::Medium, ModelMaterial::System, [0.0, 0.40, 0.0]),
    BlockModelPart::new(ModelMesh::Small, ModelMaterial::SystemAccent, [-0.22, 0.56, 0.0]),
    BlockModelPart::new(ModelMesh::Small, ModelMaterial::SystemAccent, [0.22, 0.56, 0.0]),
    BlockModelPart::new(ModelMesh::Plate, ModelMaterial::SystemAccent, [0.0, 0.20, 0.0])
        .scaled([0.56, 0.70, 0.56]),
];

pub struct GeneratorBlock;

pub static GENERATOR: GeneratorBlock = GeneratorBlock;

impl Block for GeneratorBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Generator
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::marker(
            self.id(),
            "block.generator",
            "short.generator",
            rgb(0.52, 0.30, 0.68),
            rgb(0.42, 0.20, 0.56),
        )
        .no_collision()
    }

    fn material_source(&self, facing: super::Facing) -> Option<MaterialSource> {
        let _ = facing;
        Some(MaterialSource::Generator)
    }

    fn default_settings(&self, _pos: bevy::prelude::IVec3) -> Option<BlockSettings> {
        Some(BlockSettings::Generator(GeneratorSettings::default()))
    }

    fn model(&self) -> BlockModel {
        BlockModel::Parts(MODEL)
    }
}

impl SystemBlock for GeneratorBlock {}
impl EditableBlock for GeneratorBlock {}
