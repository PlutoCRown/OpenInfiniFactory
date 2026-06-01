use super::*;
use crate::game::world::blocks::*;
use crate::game::world::direction::Facing;

pub(super) fn is_directional(_block: &CounterRotatorBlock) -> bool {
    true
}

pub(super) fn movement_rule(_block: &CounterRotatorBlock, _facing: Facing) -> Option<MovementRule> {
    Some(MovementRule::Rotate { clockwise: false })
}

pub(super) fn alternate(_block: &CounterRotatorBlock) -> Option<BlockKind> {
    Some(BlockKind::Rotator)
}
