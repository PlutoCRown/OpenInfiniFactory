use super::CounterRotatorBlock;

use crate::blocks::traits::BlockBehavior;
use crate::blocks::{MovementRule};
use crate::world::direction::{Facing};

impl BlockBehavior for CounterRotatorBlock {
    fn is_directional(&self) -> bool {
        true
    }

    fn movement_rule(&self, _facing: Facing) -> Option<MovementRule> {
        Some(MovementRule::Rotate { clockwise: false })
    }
}
