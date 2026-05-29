use super::{
    rgb, Block, BlockDefinition, BlockKind, BlockModel, BlockModelPart, Facing, FactoryBlock,
    MarkerBehavior, ModelMaterial, ModelMesh, RenderBehavior, SignalBehavior,
    WireConnectorBehavior,
};

const MODEL: &[BlockModelPart] = &[
    BlockModelPart::new(ModelMesh::Large, ModelMaterial::Frame, [0.0, 0.38, 0.08]),
    BlockModelPart::new(ModelMesh::RodZ, ModelMaterial::DarkFrame, [0.0, 0.40, -0.34])
        .scaled([1.15, 1.15, 0.50]),
    BlockModelPart::new(ModelMesh::Plate, ModelMaterial::Power, [0.0, 0.40, -0.54])
        .scaled([0.74, 0.86, 0.44]),
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
            rgb(0.58, 0.40, 0.24),
            rgb(0.50, 0.32, 0.20),
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
        BlockModel::Parts(MODEL)
    }

    fn alternate(&self) -> Option<BlockKind> {
        Some(BlockKind::Piston)
    }
}

impl FactoryBlock for BlockerBlock {}
