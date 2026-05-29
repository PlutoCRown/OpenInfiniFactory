use super::{
    rgb, Block, BlockDefinition, BlockKind, FactoryBlock, MaterialMover, RenderBehavior,
    SignalBehavior, WireConnectorBehavior,
};

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

    fn material_mover(&self, facing: super::Facing) -> Option<MaterialMover> {
        Some(MaterialMover::Piston {
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

    fn alternate(&self) -> Option<BlockKind> {
        Some(BlockKind::Blocker)
    }
}

impl FactoryBlock for PistonBlock {}
