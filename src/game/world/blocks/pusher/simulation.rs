use super::*;
use crate::game::world::blocks::*;
use crate::game::world::direction::Facing;

pub(super) fn is_directional(_block: &PusherBlock) -> bool {
    true
}

pub(super) fn movement_rule(_block: &PusherBlock, facing: Facing) -> Option<MovementRule> {
    Some(MovementRule::PoweredTranslate {
        source: facing.forward_ivec3(),
        offset: facing.forward_ivec3(),
    })
}

pub(super) fn signal_behavior(_block: &PusherBlock, _facing: Facing) -> Option<SignalBehavior> {
    Some(SignalBehavior::PoweredDevice)
}

pub(super) fn alternate(_block: &PusherBlock) -> Option<BlockKind> {
    Some(BlockKind::Blocker)
}
