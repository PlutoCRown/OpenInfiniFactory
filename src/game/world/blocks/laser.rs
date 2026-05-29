use super::{
    rgb, Block, BlockDefinition, BlockKind, FactoryBlock, MaterialDestroyer, RenderBehavior,
    SignalBehavior, WireConnectorBehavior,
};

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

    fn alternate(&self) -> Option<BlockKind> {
        Some(BlockKind::Drill)
    }
}

impl FactoryBlock for LaserBlock {}
