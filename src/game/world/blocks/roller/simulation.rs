use super::*;
use crate::game::world::blocks::*;
use crate::game::world::direction::Facing;

pub(super) fn is_directional(_block: &RollerBlock) -> bool {
    true
}

pub(super) fn material_labeler(_block: &RollerBlock, facing: Facing) -> Option<MaterialLabeler> {
    Some(MaterialLabeler::Roller {
        target: facing.forward_ivec3(),
    })
}
