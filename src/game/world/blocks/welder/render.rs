use super::*;
use crate::game::world::blocks::*;
use crate::game::world::direction::Facing;

const MODEL: &[BlockModelPart] = &[];

pub(super) fn render_behavior(_block: &WelderBlock, facing: Facing) -> RenderBehavior {
    RenderBehavior {
        weld_connector: Some(WeldConnectorBehavior::Offset(facing.forward_ivec3())),
        ..Default::default()
    }
}

pub(super) fn model(_block: &WelderBlock) -> BlockModel {
    BlockModel::Parts(MODEL)
}
