use super::*;
use crate::game::world::blocks::*;
use crate::game::world::direction::Facing;

pub(super) fn is_directional(_block: &ConveyorBlock) -> bool {
    true
}

pub(super) fn movement_rule(_block: &ConveyorBlock, facing: Facing) -> Option<MovementRule> {
    Some(MovementRule::Translate {
        source: IVec3::Y,
        offset: facing.forward_ivec3(),
    })
}

pub(super) fn factory_connection_blocker(_block: &ConveyorBlock, _facing: Facing) -> Option<IVec3> {
    Some(IVec3::Y)
}

pub(super) fn alternate(_block: &ConveyorBlock) -> Option<BlockKind> {
    Some(BlockKind::ReverseConveyor)
}
