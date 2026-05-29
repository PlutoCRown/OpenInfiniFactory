use super::{
    rgb, Block, BlockDefinition, BlockKind, BlockModel, BlockModelPart, FactoryBlock,
    ModelMaterial, ModelMesh, RenderBehavior, SignalBehavior, WireConnectorBehavior,
};

const MODEL: &[BlockModelPart] = &[
    BlockModelPart::new(ModelMesh::RodX, ModelMaterial::Wire, [0.0, 0.54, 0.0])
        .scaled([0.90, 0.70, 0.70]),
    BlockModelPart::new(ModelMesh::RodZ, ModelMaterial::Wire, [0.0, 0.54, 0.0])
        .scaled([0.70, 0.70, 0.90]),
    BlockModelPart::new(ModelMesh::Small, ModelMaterial::Power, [0.0, 0.58, 0.0]),
];

pub struct WireBlock;

pub static WIRE: WireBlock = WireBlock;

impl Block for WireBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Wire
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.wire",
            "short.wire",
            rgb(0.95, 0.72, 0.18),
            rgb(0.88, 0.62, 0.12),
        )
        .node()
    }

    fn signal_behavior(&self, _facing: super::Facing) -> Option<SignalBehavior> {
        Some(SignalBehavior::Wire)
    }

    fn render_behavior(&self, _facing: super::Facing) -> RenderBehavior {
        RenderBehavior {
            wire_connector: Some(WireConnectorBehavior::Wire),
            ..Default::default()
        }
    }

    fn model(&self) -> BlockModel {
        BlockModel::Parts(MODEL)
    }
}

impl FactoryBlock for WireBlock {}
