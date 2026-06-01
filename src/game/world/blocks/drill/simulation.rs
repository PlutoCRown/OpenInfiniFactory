use super::*;
use crate::game::world::blocks::*;
use crate::game::world::direction::Facing;

pub(super) fn is_directional(_block: &DrillBlock) -> bool {
    true
}

pub(super) fn marker_behavior(_block: &DrillBlock, facing: Facing) -> Option<MarkerBehavior> {
    Some(MarkerBehavior::DrillHead {
        offset: facing.forward_ivec3(),
        facing,
    })
}

pub(super) fn material_destroyer(_block: &DrillBlock, facing: Facing) -> Option<MaterialDestroyer> {
    Some(MaterialDestroyer::Drill {
        target: facing.forward_ivec3(),
    })
}

pub(super) fn factory_connection_blocker(_block: &DrillBlock, facing: Facing) -> Option<IVec3> {
    Some(facing.forward_ivec3())
}

pub(super) fn signal_behavior(_block: &DrillBlock, _facing: Facing) -> Option<SignalBehavior> {
    Some(SignalBehavior::PoweredDevice)
}

pub(super) fn alternate(_block: &DrillBlock) -> Option<BlockKind> {
    Some(BlockKind::Laser)
}
