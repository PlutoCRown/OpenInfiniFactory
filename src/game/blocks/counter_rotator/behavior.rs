use super::CounterRotatorBlock;

use crate::game::blocks::traits::BlockBehavior;
use crate::game::blocks::{MovementRule};
use crate::game::world::direction::{Facing};

impl BlockBehavior for CounterRotatorBlock {
    fn is_directional(&self) -> bool {
        true
    }

    fn movement_rule(&self, _facing: Facing) -> Option<MovementRule> {
        Some(MovementRule::Rotate { clockwise: false })
    }
}
