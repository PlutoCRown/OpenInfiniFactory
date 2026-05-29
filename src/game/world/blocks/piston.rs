use super::{
    rgb, Block, BlockDefinition, BlockKind, BlockModel, BlockModelPart, FactoryBlock,
    ModelMaterial, ModelMesh, MovementRule, RenderBehavior, SignalBehavior, WireConnectorBehavior,
};

const MODEL: &[BlockModelPart] = &[
    BlockModelPart::new(ModelMesh::Large, ModelMaterial::Piston, [0.0, 0.42, 0.05]),
    BlockModelPart::new(ModelMesh::RodZ, ModelMaterial::Piston, [0.0, 0.42, -0.30])
        .scaled([0.82, 0.82, 0.70]),
    BlockModelPart::new(ModelMesh::Plate, ModelMaterial::DarkFrame, [0.0, 0.42, -0.54])
        .scaled([0.68, 0.75, 0.45]),
];

pub struct PistonBlock;

pub static PISTON: PistonBlock = PistonBlock;

impl Block for PistonBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Piston
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.piston",
            "short.piston",
            rgb(0.78, 0.55, 0.28),
            rgb(0.66, 0.43, 0.20),
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
        BlockModel::Parts(MODEL)
    }

    fn alternate(&self) -> Option<BlockKind> {
        Some(BlockKind::Blocker)
    }
}

impl FactoryBlock for PistonBlock {}
