use super::{rgb, Block, BlockDefinition, BlockKind, BlockModel, BlockModelPart, ModelMaterial, ModelMesh};

const MODEL: &[BlockModelPart] = &[BlockModelPart::new(
    ModelMesh::PusherHead,
    ModelMaterial::BorderedWoodTexture,
    [0.0, 0.0, 0.0],
)];

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
            rgb(0.54, 0.56, 0.54),
            rgb(0.42, 0.44, 0.42),
        )
    }

    fn model(&self) -> BlockModel {
        BlockModel::Parts(MODEL)
    }
}
