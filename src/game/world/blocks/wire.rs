use super::{
    rgb, Block, BlockDefinition, BlockKind, RenderBehavior, SignalBehavior, WireConnectorBehavior,
};

pub struct WireBlock;

pub static WIRE: WireBlock = WireBlock;

impl Block for WireBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Wire
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.wire",
            "short.wire",
            rgb(0.95, 0.72, 0.18),
            rgb(0.88, 0.62, 0.12),
        )
        .node()
    }

    fn signal_behavior(&self, _facing: super::Facing) -> Option<SignalBehavior> {
        Some(SignalBehavior::Wire)
    }

    fn render_behavior(&self, _facing: super::Facing) -> RenderBehavior {
        RenderBehavior {
            wire_connector: Some(WireConnectorBehavior::Wire),
            ..Default::default()
        }
    }
}
