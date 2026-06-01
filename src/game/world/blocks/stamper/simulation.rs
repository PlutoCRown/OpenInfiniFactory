use super::*;
use crate::game::world::blocks::*;
use crate::game::world::direction::Facing;

pub(super) fn is_directional(_block: &StamperBlock) -> bool {
    true
}

pub(super) fn material_labeler(_block: &StamperBlock, facing: Facing) -> Option<MaterialLabeler> {
    Some(MaterialLabeler::Stamper {
        target: facing.forward_ivec3(),
    })
}
