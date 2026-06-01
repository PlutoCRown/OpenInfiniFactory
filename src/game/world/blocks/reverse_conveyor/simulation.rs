use super::*;
use crate::game::world::blocks::*;
use crate::game::world::direction::Facing;

pub(super) fn is_directional(_block: &ReverseConveyorBlock) -> bool {
    true
}

pub(super) fn movement_rule(_block: &ReverseConveyorBlock, facing: Facing) -> Option<MovementRule> {
    Some(MovementRule::Translate {
        source: IVec3::NEG_Y,
        offset: -facing.forward_ivec3(),
    })
}

pub(super) fn factory_connection_blocker(
    _block: &ReverseConveyorBlock,
    _facing: Facing,
) -> Option<IVec3> {
    Some(IVec3::NEG_Y)
}

pub(super) fn alternate(_block: &ReverseConveyorBlock) -> Option<BlockKind> {
    Some(BlockKind::Conveyor)
}
