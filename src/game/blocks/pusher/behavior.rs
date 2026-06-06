use super::PusherBlock;

use crate::game::blocks::traits::BlockBehavior;
use crate::game::blocks::{MovementRule, SignalBehavior};
use crate::game::world::direction::{Facing};

impl BlockBehavior for PusherBlock {
    fn is_directional(&self) -> bool {
        true
    }

    fn movement_rule(&self, facing: Facing) -> Option<MovementRule> {
        Some(MovementRule::PoweredTranslate {
            source: facing.forward_ivec3(),
            offset: facing.forward_ivec3(),
        })
    }

    fn signal_behavior(&self, _facing: Facing) -> Option<SignalBehavior> {
        Some(SignalBehavior::PoweredDevice)
    }
}
