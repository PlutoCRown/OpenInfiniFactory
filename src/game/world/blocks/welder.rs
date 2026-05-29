use super::{
    rgb, Block, BlockDefinition, BlockKind, Facing, FactoryBlock, MarkerBehavior, RenderBehavior,
    WeldConnectorBehavior,
};

pub struct WelderBlock;

pub static WELDER: WelderBlock = WelderBlock;

impl Block for WelderBlock {
    fn id(&self) -> BlockKind {
        BlockKind::Welder
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.welder",
            "short.welder",
            rgb(0.76, 0.18, 0.16),
            rgb(0.62, 0.12, 0.12),
        )
        .directional()
        .alternate(BlockKind::DownWelder)
    }

    fn marker_behavior(&self, facing: Facing) -> Option<MarkerBehavior> {
        Some(MarkerBehavior::WeldPoint {
            offset: facing.forward_ivec3(),
            facing,
        })
    }

    fn render_behavior(&self, facing: Facing) -> RenderBehavior {
        RenderBehavior {
            weld_connector: Some(WeldConnectorBehavior::Offset(facing.forward_ivec3())),
            ..Default::default()
        }
    }
}

impl FactoryBlock for WelderBlock {}
