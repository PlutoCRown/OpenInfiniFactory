use super::LifterBlock;

use crate::blocks::traits::BlockBehavior;
use crate::blocks::{MovementRule};
use glam::IVec3;
use crate::world::direction::{Facing};

impl BlockBehavior for LifterBlock {
    fn is_directional(&self) -> bool {
        true
    }

    fn movement_rule(&self, _facing: Facing) -> Option<MovementRule> {
        Some(MovementRule::Lift { range: 5 })
    }

    fn non_connection_face(&self, _facing: Facing) -> Option<IVec3> {
        Some(IVec3::Y)
    }
}
