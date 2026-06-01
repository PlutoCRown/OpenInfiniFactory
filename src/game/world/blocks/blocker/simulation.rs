use super::*;
use crate::game::world::blocks::*;
use crate::game::world::direction::Facing;

pub(super) fn is_directional(_block: &BlockerBlock) -> bool {
    true
}

pub(super) fn marker_behavior(_block: &BlockerBlock, facing: Facing) -> Option<MarkerBehavior> {
    Some(MarkerBehavior::BlockerHead {
        offset: facing.forward_ivec3(),
        facing,
    })
}

pub(super) fn movement_rule(_block: &BlockerBlock, facing: Facing) -> Option<MovementRule> {
    Some(MovementRule::PoweredTranslate {
        source: facing.forward_ivec3(),
        offset: facing.forward_ivec3(),
    })
}

pub(super) fn signal_behavior(_block: &BlockerBlock, _facing: Facing) -> Option<SignalBehavior> {
    Some(SignalBehavior::PoweredDevice)
}

pub(super) fn alternate(_block: &BlockerBlock) -> Option<BlockKind> {
    Some(BlockKind::Pusher)
}
