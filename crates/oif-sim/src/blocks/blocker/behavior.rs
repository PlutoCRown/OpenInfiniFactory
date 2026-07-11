use super::BlockerBlock;

use crate::blocks::traits::BlockBehavior;
use crate::blocks::{MovementRule, SignalBehavior};
use crate::world::direction::Facing;

impl BlockBehavior for BlockerBlock {
    fn is_directional(&self) -> bool {
        true
    }

    fn movement_rule(&self, facing: Facing) -> Option<MovementRule> {
        Some(MovementRule::PoweredTranslate {
            source: facing.forward_ivec3(),
            offset: facing.forward_ivec3(),
            extend_when_powered: false,
        })
    }

    fn signal_behavior(&self, _facing: Facing) -> Option<SignalBehavior> {
        Some(SignalBehavior::PoweredDevice)
    }
}
