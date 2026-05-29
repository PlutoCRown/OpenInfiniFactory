use bevy::prelude::*;

use super::{
    rgb, Block, BlockDefinition, BlockKind, FactoryBlock, RenderBehavior, SignalBehavior,
    WireConnectorBehavior,
};

pub struct DownDetectorBlock;

pub static DOWN_DETECTOR: DownDetectorBlock = DownDetectorBlock;

impl Block for DownDetectorBlock {
    fn id(&self) -> BlockKind {
        BlockKind::DownDetector
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.down_detector",
            "short.down_detector",
            rgb(0.18, 0.52, 0.78),
            rgb(0.14, 0.40, 0.68),
        )
    }

    fn signal_behavior(&self, _facing: super::Facing) -> Option<SignalBehavior> {
        Some(SignalBehavior::Detector {
            detection_pos: IVec3::NEG_Y,
        })
    }

    fn render_behavior(&self, _facing: super::Facing) -> RenderBehavior {
        RenderBehavior {
            wire_connector: Some(WireConnectorBehavior::Device {
                blocked_offset: IVec3::NEG_Y,
            }),
            ..Default::default()
        }
    }

    fn alternate(&self) -> Option<BlockKind> {
        Some(BlockKind::Detector)
    }
}

impl FactoryBlock for DownDetectorBlock {}
