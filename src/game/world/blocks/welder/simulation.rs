use super::*;
use crate::game::world::blocks::*;
use crate::game::world::direction::Facing;

pub(super) fn is_directional(_block: &WelderBlock) -> bool {
    true
}

pub(super) fn marker_behavior(_block: &WelderBlock, facing: Facing) -> Option<MarkerBehavior> {
    Some(MarkerBehavior::WeldPoint {
        offset: facing.forward_ivec3(),
        facing,
    })
}

pub(super) fn factory_connection_blocker(_block: &WelderBlock, facing: Facing) -> Option<IVec3> {
    Some(facing.forward_ivec3())
}

pub(super) fn alternate(_block: &WelderBlock) -> Option<BlockKind> {
    Some(BlockKind::DownWelder)
}
