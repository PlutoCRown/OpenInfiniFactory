use bevy::prelude::IVec3;

use super::{
    rgb, Block, BlockDefinition, BlockKind, BlockModel, Facing, MarkerBehavior, RenderBehavior,
    WeldConnectorBehavior,
};

const MODEL: &[super::BlockModelPart] = &[];

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
            rgb(0.14, 0.38, 0.74),
            rgb(0.08, 0.26, 0.58),
        )
    }

    fn is_directional(&self) -> bool {
        true
    }

    fn marker_behavior(&self, facing: Facing) -> Option<MarkerBehavior> {
        Some(MarkerBehavior::WeldPoint {
            offset: facing.forward_ivec3(),
            facing,
        })
    }

    fn factory_connection_blocker(&self, facing: Facing) -> Option<IVec3> {
        Some(facing.forward_ivec3())
    }

    fn render_behavior(&self, facing: Facing) -> RenderBehavior {
        RenderBehavior {
            weld_connector: Some(WeldConnectorBehavior::Offset(facing.forward_ivec3())),
            ..Default::default()
        }
    }

    fn model(&self) -> BlockModel {
        BlockModel::Parts(MODEL)
    }

    fn alternate(&self) -> Option<BlockKind> {
        Some(BlockKind::DownWelder)
    }
}
