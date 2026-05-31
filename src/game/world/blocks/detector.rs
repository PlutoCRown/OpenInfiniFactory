use super::{
    rgb, Block, BlockDefinition, BlockKind, BlockModel, BlockModelPart, ModelMaterial, ModelMesh,
    RenderBehavior, SignalBehavior, WireConnectorBehavior,
};

const MODEL: &[BlockModelPart] = &[
    BlockModelPart::new(ModelMesh::Medium, ModelMaterial::Signal, [0.0, 0.52, 0.0]),
    BlockModelPart::new(ModelMesh::RodZ, ModelMaterial::Signal, [0.0, 0.38, -0.34])
        .scaled([0.72, 0.72, 0.55]),
    BlockModelPart::new(ModelMesh::Small, ModelMaterial::Power, [0.0, 0.38, -0.52]),
];

pub struct DetectorBlock;

pub static DETECTOR: DetectorBlock = DetectorBlock;

impl Block for DetectorBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Detector
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.detector",
            "short.detector",
            rgb(0.15, 0.45, 0.72),
            rgb(0.12, 0.34, 0.62),
        )
    }

    fn is_directional(&self) -> bool {
        true
    }

    fn signal_behavior(&self, facing: super::Facing) -> Option<SignalBehavior> {
        Some(SignalBehavior::Detector {
            detection_pos: facing.forward_ivec3(),
        })
    }

    fn render_behavior(&self, facing: super::Facing) -> RenderBehavior {
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
        Some(BlockKind::DownDetector)
    }
}
