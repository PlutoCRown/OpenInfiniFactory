use super::{
    rgb, rgba, Block, BlockDefinition, BlockKind, RenderBehavior, SystemBlock,
    WeldBehavior, WeldConnectorBehavior,
};

pub struct WeldPointBlock;

pub static WELD_POINT: WeldPointBlock = WeldPointBlock;

impl Block for WeldPointBlock {
    fn id(&self) -> BlockKind {
        BlockKind::WeldPoint
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::marker(
            self.id(),
            "block.weld_point",
            "short.weld_point",
            rgba(1.0, 0.28, 0.18, 0.45),
            rgb(0.86, 0.16, 0.12),
        )
        .node()
        .transparent()
        .no_collision()
    }

    fn render_behavior(&self, _facing: super::Facing) -> RenderBehavior {
        RenderBehavior {
            weld_connector: Some(WeldConnectorBehavior::AllSides),
            ..Default::default()
        }
    }

    fn weld_behavior(&self) -> Option<WeldBehavior> {
        Some(WeldBehavior::Node)
    }
}

impl SystemBlock for WeldPointBlock {}
