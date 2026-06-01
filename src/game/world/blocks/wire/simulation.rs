use super::*;
use crate::game::world::blocks::*;
use crate::game::world::direction::Facing;

pub(super) fn signal_behavior(_block: &WireBlock, _facing: Facing) -> Option<SignalBehavior> {
    Some(SignalBehavior::Wire)
}
