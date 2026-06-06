use super::LifterBlock;

use crate::game::blocks::traits::BlockBehavior;
use crate::game::blocks::{MovementRule};
use crate::game::world::direction::{Facing};

impl BlockBehavior for LifterBlock {
    fn is_directional(&self) -> bool {
        true
    }

    fn movement_rule(&self, _facing: Facing) -> Option<MovementRule> {
        Some(MovementRule::Lift { range: 5 })
    }
}
