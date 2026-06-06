use super::BlockerBlock;

use crate::game::blocks::traits::BlockBehavior;
use crate::game::blocks::{MarkerBehavior, MovementRule, SignalBehavior};
use crate::game::world::direction::{Facing};

impl BlockBehavior for BlockerBlock {
    fn is_directional(&self) -> bool {
        true
    }

    fn marker_behavior(&self, facing: Facing) -> Option<MarkerBehavior> {
        Some(MarkerBehavior::BlockerHead {
            offset: facing.forward_ivec3(),
            facing,
        })
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
