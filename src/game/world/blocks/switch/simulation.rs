use super::*;
use crate::game::world::blocks::*;
use crate::game::world::direction::Facing;
use bevy::prelude::*;

pub(super) fn signal_behavior(_block: &SwitchBlock, _facing: Facing) -> Option<SignalBehavior> {
    Some(SignalBehavior::Detector {
        detection_pos: IVec3::ZERO,
    })
}
