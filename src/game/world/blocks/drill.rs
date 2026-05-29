use super::{
    rgb, Block, BlockDefinition, BlockKind, Facing, FactoryBlock, MarkerBehavior,
    MaterialDestroyer, RenderBehavior, SignalBehavior, WireConnectorBehavior,
};

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
        .directional()
        .alternate(BlockKind::Laser)
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
}

impl FactoryBlock for DrillBlock {}
