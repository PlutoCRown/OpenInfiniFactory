use super::{
    rgb, Block, BlockDefinition, BlockKind, FactoryBlock, RenderBehavior, SignalBehavior,
    WireConnectorBehavior,
};

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
        .directional()
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
}

impl FactoryBlock for DetectorBlock {}
