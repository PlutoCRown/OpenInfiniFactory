use super::CounterRotatorBlock;

use glam::IVec3;

use crate::blocks::traits::BlockBehavior;
use crate::blocks::{MovementRule, SignalBehavior, WireFacePolicy};
use crate::world::direction::Facing;

impl BlockBehavior for CounterRotatorBlock {
    fn is_directional(&self) -> bool {
        true
    }

    fn movement_rule(&self, _facing: Facing) -> Option<MovementRule> {
        Some(MovementRule::Rotate { clockwise: false })
    }

    fn signal_behavior(&self, _facing: Facing) -> Option<SignalBehavior> {
        Some(SignalBehavior::PoweredDevice {
            wire_face: WireFacePolicy::AllowOnly(IVec3::NEG_Y),
        })
    }
}
