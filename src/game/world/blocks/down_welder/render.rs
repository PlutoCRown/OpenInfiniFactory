use super::*;
use crate::game::world::blocks::*;
use crate::game::world::direction::Facing;

const MODEL: &[BlockModelPart] = &[];

pub(super) fn render_behavior(_block: &DownWelderBlock, _facing: Facing) -> RenderBehavior {
    RenderBehavior {
        weld_connector: Some(WeldConnectorBehavior::Offset(IVec3::NEG_Y)),
        ..Default::default()
    }
}

pub(super) fn model(_block: &DownWelderBlock) -> BlockModel {
    BlockModel::Parts(MODEL)
}
