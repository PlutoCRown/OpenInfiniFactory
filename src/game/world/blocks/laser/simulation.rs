use super::*;
use crate::game::world::blocks::*;
use crate::game::world::direction::Facing;

pub(super) fn is_directional(_block: &LaserBlock) -> bool {
    true
}

pub(super) fn material_destroyer(_block: &LaserBlock, facing: Facing) -> Option<MaterialDestroyer> {
    Some(MaterialDestroyer::Laser {
        direction: facing.forward_ivec3(),
        range: 30,
    })
}

pub(super) fn signal_behavior(_block: &LaserBlock, _facing: Facing) -> Option<SignalBehavior> {
    Some(SignalBehavior::PoweredDevice)
}

pub(super) fn alternate(_block: &LaserBlock) -> Option<BlockKind> {
    Some(BlockKind::Drill)
}
