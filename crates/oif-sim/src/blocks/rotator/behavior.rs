use super::RotatorBlock;

use crate::blocks::traits::BlockBehavior;
use crate::blocks::{MovementRule};
use crate::world::direction::{Facing};

impl BlockBehavior for RotatorBlock {
    fn is_directional(&self) -> bool {
        true
    }

    fn movement_rule(&self, _facing: Facing) -> Option<MovementRule> {
        Some(MovementRule::Rotate { clockwise: true })
    }
}
