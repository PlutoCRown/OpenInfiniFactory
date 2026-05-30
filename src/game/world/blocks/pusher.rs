use super::{
    rgb, Block, BlockDefinition, BlockKind, BlockModel, BlockModelPart, FactoryBlock,
    ModelMaterial, ModelMesh, MovementRule, RenderBehavior, SignalBehavior, WireConnectorBehavior,
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

pub struct PusherBlock;

pub static PUSHER: PusherBlock = PusherBlock;

impl Block for PusherBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Pusher
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.pusher",
            "short.pusher",
            rgb(0.54, 0.56, 0.54),
            rgb(0.42, 0.44, 0.42),
        )
    }

    fn is_directional(&self) -> bool {
        true
    }

    fn movement_rule(&self, facing: super::Facing) -> Option<MovementRule> {
        Some(MovementRule::PoweredTranslate {
            source: facing.forward_ivec3(),
            offset: facing.forward_ivec3(),
        })
    }

    fn signal_behavior(&self, _facing: super::Facing) -> Option<SignalBehavior> {
        Some(SignalBehavior::PoweredDevice)
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
        BlockModel::PartsOnly(MODEL)
    }

    fn alternate(&self) -> Option<BlockKind> {
        Some(BlockKind::Blocker)
    }
}

impl FactoryBlock for PusherBlock {}
