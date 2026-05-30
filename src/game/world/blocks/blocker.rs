use super::{
    rgb, Block, BlockDefinition, BlockKind, BlockModel, BlockModelPart, Facing, FactoryBlock,
    MarkerBehavior, ModelMaterial, ModelMesh, RenderBehavior, SignalBehavior,
    WireConnectorBehavior,
};

const MODEL: &[BlockModelPart] = &[
    BlockModelPart::new(
        ModelMesh::PusherBody,
        ModelMaterial::StoneTexture,
        [0.0, 0.0, 0.10],
    ),
    BlockModelPart::new(
        ModelMesh::PusherHead,
        ModelMaterial::BorderedWoodTexture,
        [0.0, 0.0, -0.40],
    ),
];

pub struct BlockerBlock;

pub static BLOCKER: BlockerBlock = BlockerBlock;

impl Block for BlockerBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Blocker
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.blocker",
            "short.blocker",
            rgb(0.54, 0.56, 0.54),
            rgb(0.42, 0.44, 0.42),
        )
    }

    fn is_directional(&self) -> bool {
        true
    }

    fn marker_behavior(&self, facing: Facing) -> Option<MarkerBehavior> {
        Some(MarkerBehavior::BlockerHead {
            offset: facing.forward_ivec3(),
            facing,
        })
    }

    fn signal_behavior(&self, _facing: Facing) -> Option<SignalBehavior> {
        Some(SignalBehavior::PoweredDevice)
    }

    fn render_behavior(&self, facing: Facing) -> RenderBehavior {
        RenderBehavior {
            wire_connector: Some(WireConnectorBehavior::Device {
                blocked_offset: facing.forward_ivec3(),
            }),
            ..Default::default()
        }
    }

    fn model(&self) -> BlockModel {
        BlockModel::PartsOnly(MODEL)
    }

    fn alternate(&self) -> Option<BlockKind> {
        Some(BlockKind::Pusher)
    }
}

impl FactoryBlock for BlockerBlock {}
