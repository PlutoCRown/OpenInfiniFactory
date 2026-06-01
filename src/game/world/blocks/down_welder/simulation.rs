use super::*;
use crate::game::world::blocks::*;
use crate::game::world::direction::Facing;

pub(super) fn marker_behavior(_block: &DownWelderBlock, _facing: Facing) -> Option<MarkerBehavior> {
    Some(MarkerBehavior::WeldPoint {
        offset: IVec3::NEG_Y,
        facing: Facing::North,
    })
}

pub(super) fn factory_connection_blocker(
    _block: &DownWelderBlock,
    _facing: Facing,
) -> Option<IVec3> {
    Some(IVec3::NEG_Y)
}

pub(super) fn alternate(_block: &DownWelderBlock) -> Option<BlockKind> {
    Some(BlockKind::Welder)
}
