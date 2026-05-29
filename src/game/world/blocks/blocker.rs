use super::{
    rgb, Block, BlockDefinition, BlockKind, Facing, FactoryBlock, MarkerBehavior, RenderBehavior,
    SignalBehavior, WireConnectorBehavior,
};

pub struct BlockerBlock;

pub static BLOCKER: BlockerBlock = BlockerBlock;

impl Block for BlockerBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Blocker
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.blocker",
            "short.blocker",
            rgb(0.58, 0.40, 0.24),
            rgb(0.50, 0.32, 0.20),
        )
        .directional()
        .alternate(BlockKind::Piston)
    }

    fn marker_behavior(&self, facing: Facing) -> Option<MarkerBehavior> {
        Some(MarkerBehavior::BlockerHead {
            offset: facing.forward_ivec3(),
            facing,
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

impl FactoryBlock for BlockerBlock {}
