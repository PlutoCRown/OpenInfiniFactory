use super::*;
use crate::game::world::blocks::*;
use crate::game::world::direction::Facing;

pub(super) fn signal_behavior(
    _block: &DownDetectorBlock,
    _facing: Facing,
) -> Option<SignalBehavior> {
    Some(SignalBehavior::Detector {
        detection_pos: IVec3::NEG_Y,
    })
}

pub(super) fn factory_connection_blocker(
    _block: &DownDetectorBlock,
    _facing: Facing,
) -> Option<IVec3> {
    Some(IVec3::NEG_Y)
}

pub(super) fn alternate(_block: &DownDetectorBlock) -> Option<BlockKind> {
    Some(BlockKind::Detector)
}
