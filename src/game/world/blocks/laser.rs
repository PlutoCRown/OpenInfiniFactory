use super::{
    rgb, Block, BlockDefinition, BlockKind, BlockModel, BlockModelPart, FactoryBlock,
    MaterialDestroyer, ModelMaterial, ModelMesh, RenderBehavior, SignalBehavior,
    WireConnectorBehavior,
};

const MODEL: &[BlockModelPart] = &[
    BlockModelPart::new(
        ModelMesh::Medium,
        ModelMaterial::DarkFrame,
        [0.0, 0.42, 0.08],
    ),
    BlockModelPart::new(ModelMesh::RodZ, ModelMaterial::Laser, [0.0, 0.42, -0.30])
        .scaled([0.54, 0.54, 0.76]),
    BlockModelPart::new(ModelMesh::Small, ModelMaterial::Laser, [0.0, 0.42, -0.56]),
    BlockModelPart::new(ModelMesh::Plate, ModelMaterial::Power, [0.0, 0.62, 0.08])
        .scaled([0.58, 0.58, 0.58]),
];

pub struct LaserBlock;

pub static LASER: LaserBlock = LaserBlock;

impl Block for LaserBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Laser
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.laser",
            "short.laser",
            rgb(0.85, 0.20, 0.34),
            rgb(0.72, 0.12, 0.26),
        )
    }

    fn is_directional(&self) -> bool {
        true
    }

    fn material_destroyer(&self, facing: super::Facing) -> Option<MaterialDestroyer> {
        Some(MaterialDestroyer::Laser {
            direction: facing.forward_ivec3(),
            range: 30,
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
        Some(BlockKind::Drill)
    }
}

impl FactoryBlock for LaserBlock {}
