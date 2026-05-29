use bevy::prelude::*;

use super::{
    rgb, Block, BlockDefinition, BlockKind, Facing, FactoryBlock, MarkerBehavior, RenderBehavior,
    WeldConnectorBehavior,
};

pub struct DownWelderBlock;

pub static DOWN_WELDER: DownWelderBlock = DownWelderBlock;

impl Block for DownWelderBlock {
    fn id(&self) -> BlockKind {
        BlockKind::DownWelder
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::factory(
            self.id(),
            "block.down_welder",
            "short.down_welder",
            rgb(0.92, 0.32, 0.20),
            rgb(0.78, 0.22, 0.14),
        )
        .alternate(BlockKind::Welder)
    }

    fn marker_behavior(&self, _facing: Facing) -> Option<MarkerBehavior> {
        Some(MarkerBehavior::WeldPoint {
            offset: IVec3::NEG_Y,
            facing: Facing::North,
        })
    }

    fn render_behavior(&self, _facing: Facing) -> RenderBehavior {
        RenderBehavior {
            weld_connector: Some(WeldConnectorBehavior::Offset(IVec3::NEG_Y)),
            ..Default::default()
        }
    }
}

impl FactoryBlock for DownWelderBlock {}
