use super::{
    rgb, Block, BlockDefinition, BlockKind, BlockModel, BlockModelPart, ModelMaterial, ModelMesh,
    RenderBehavior, SignalBehavior, WireConnectorBehavior,
};
use bevy::prelude::IVec3;

const MODEL: &[BlockModelPart] = &[
    BlockModelPart::new(ModelMesh::Medium, ModelMaterial::Signal, [0.0, 0.34, 0.0])
        .scaled([0.82, 0.82, 0.42]),
    BlockModelPart::new(ModelMesh::Small, ModelMaterial::Power, [0.0, 0.64, 0.0])
        .scaled([0.70, 0.70, 0.34]),
];

pub struct SwitchBlock;

pub static SWITCH: SwitchBlock = SwitchBlock;

impl Block for SwitchBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Switch
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.switch",
            "short.switch",
            rgb(0.90, 0.48, 0.18),
            rgb(0.78, 0.34, 0.12),
        )
    }

    fn signal_behavior(&self, _facing: super::Facing) -> Option<SignalBehavior> {
        Some(SignalBehavior::Detector {
            detection_pos: IVec3::ZERO,
        })
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
