use super::{
    rgb, Block, BlockDefinition, BlockKind, BlockModel, BlockModelPart, Facing, FactoryBlock,
    MarkerBehavior, MaterialDestroyer, ModelMaterial, ModelMesh, RenderBehavior, SignalBehavior,
    WireConnectorBehavior,
};

const MODEL: &[BlockModelPart] = &[
    BlockModelPart::new(ModelMesh::Medium, ModelMaterial::Frame, [0.0, 0.42, 0.08]),
    BlockModelPart::new(ModelMesh::RodZ, ModelMaterial::Drill, [0.0, 0.42, -0.30])
        .scaled([0.65, 0.65, 0.68]),
    BlockModelPart::new(ModelMesh::Small, ModelMaterial::Drill, [-0.16, 0.42, -0.52]),
    BlockModelPart::new(ModelMesh::Small, ModelMaterial::Drill, [0.16, 0.42, -0.52]),
    BlockModelPart::new(ModelMesh::Small, ModelMaterial::Power, [0.0, 0.62, 0.08]),
];

pub struct DrillBlock;

pub static DRILL: DrillBlock = DrillBlock;

impl Block for DrillBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Drill
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.drill",
            "short.drill",
            rgb(0.32, 0.36, 0.40),
            rgb(0.24, 0.26, 0.30),
        )
    }

    fn is_directional(&self) -> bool {
        true
    }

    fn marker_behavior(&self, facing: Facing) -> Option<MarkerBehavior> {
        Some(MarkerBehavior::DrillHead {
            offset: facing.forward_ivec3(),
            facing,
        })
    }

    fn material_destroyer(&self, facing: Facing) -> Option<MaterialDestroyer> {
        Some(MaterialDestroyer::Drill {
            target: facing.forward_ivec3(),
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
        Some(BlockKind::Laser)
    }
}

impl FactoryBlock for DrillBlock {}
