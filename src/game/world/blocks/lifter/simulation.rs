use super::*;
use crate::game::world::blocks::*;
use crate::game::world::direction::Facing;

pub(super) fn is_directional(_block: &LifterBlock) -> bool {
    true
}

pub(super) fn movement_rule(_block: &LifterBlock, _facing: Facing) -> Option<MovementRule> {
    Some(MovementRule::Lift { range: 5 })
}

pub(super) fn factory_connection_blocker(_block: &LifterBlock, _facing: Facing) -> Option<IVec3> {
    Some(IVec3::Y)
}
