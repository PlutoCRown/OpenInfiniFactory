use super::*;
use crate::game::world::blocks::*;
use crate::game::world::direction::Facing;

pub(super) fn material_destroyer(
    _block: &DrillHeadBlock,
    _facing: Facing,
) -> Option<MaterialDestroyer> {
    Some(MaterialDestroyer::AdjacentDrillHead)
}
