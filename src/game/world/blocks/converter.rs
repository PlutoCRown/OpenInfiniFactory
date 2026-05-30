use super::{
    rgb, Block, BlockDefinition, BlockKind, BlockModel, BlockModelPart, EditableBlock,
    ModelMaterial, ModelMesh, SystemBlock,
};
use crate::game::world::grid::{BlockSettings, ConverterSettings};

const MODEL: &[BlockModelPart] = &[
    BlockModelPart::new(ModelMesh::Medium, ModelMaterial::System, [0.0, 0.38, 0.0]),
    BlockModelPart::new(
        ModelMesh::Small,
        ModelMaterial::TeleportIn,
        [-0.28, 0.54, 0.0],
    ),
    BlockModelPart::new(
        ModelMesh::Small,
        ModelMaterial::TeleportOut,
        [0.28, 0.54, 0.0],
    ),
    BlockModelPart::new(
        ModelMesh::RodX,
        ModelMaterial::SystemAccent,
        [0.0, 0.54, 0.0],
    )
    .scaled([0.62, 0.55, 0.55]),
];

pub struct ConverterBlock;

pub static CONVERTER: ConverterBlock = ConverterBlock;

impl Block for ConverterBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Converter
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::puzzle_system(
            self.id(),
            "block.converter",
            "short.converter",
            rgb(0.50, 0.36, 0.78),
            rgb(0.36, 0.24, 0.62),
        )
        .no_collision()
    }

    fn default_settings(&self, _pos: bevy::prelude::IVec3) -> Option<BlockSettings> {
        Some(BlockSettings::Converter(ConverterSettings::default()))
    }

    fn model(&self) -> BlockModel {
        BlockModel::Parts(MODEL)
    }
}

impl SystemBlock for ConverterBlock {}
impl EditableBlock for ConverterBlock {}
