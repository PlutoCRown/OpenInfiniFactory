use super::*;
use crate::game::world::blocks::*;
use crate::game::world::direction::Facing;

pub(super) fn is_directional(_block: &DetectorBlock) -> bool {
    true
}

pub(super) fn signal_behavior(_block: &DetectorBlock, facing: Facing) -> Option<SignalBehavior> {
    Some(SignalBehavior::Detector {
        detection_pos: facing.forward_ivec3(),
    })
}

pub(super) fn factory_connection_blocker(_block: &DetectorBlock, facing: Facing) -> Option<IVec3> {
    Some(facing.forward_ivec3())
}

pub(super) fn alternate(_block: &DetectorBlock) -> Option<BlockKind> {
    Some(BlockKind::DownDetector)
}
