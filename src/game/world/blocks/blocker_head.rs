use super::{
    rgb, Block, BlockDefinition, BlockKind, BlockModel, BlockModelPart, ModelMaterial, ModelMesh,
};

const MODEL: &[BlockModelPart] =
    &[
        BlockModelPart::new(ModelMesh::Plate, ModelMaterial::Power, [0.0, 0.0, 0.0])
            .scaled([0.70, 1.15, 0.70]),
    ];

pub struct BlockerHeadBlock;

pub static BLOCKER_HEAD: BlockerHeadBlock = BlockerHeadBlock;

impl Block for BlockerHeadBlock {
    fn id(&self) -> BlockKind {
        BlockKind::BlockerHead
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::marker(
            self.id(),
            "block.blocker_head",
            "short.blocker_head",
            rgb(0.70, 0.48, 0.28),
            rgb(0.58, 0.36, 0.18),
        )
    }

    fn model(&self) -> BlockModel {
        BlockModel::Parts(MODEL)
    }
}
