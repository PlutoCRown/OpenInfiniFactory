use super::*;
use crate::game::world::blocks::*;
use crate::game::world::direction::Facing;

pub(super) fn is_directional(_block: &RotatorBlock) -> bool {
    true
}

pub(super) fn movement_rule(_block: &RotatorBlock, _facing: Facing) -> Option<MovementRule> {
    Some(MovementRule::Rotate { clockwise: true })
}

pub(super) fn alternate(_block: &RotatorBlock) -> Option<BlockKind> {
    Some(BlockKind::CounterRotator)
}
