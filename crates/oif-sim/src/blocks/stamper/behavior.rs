use super::StamperBlock;

use crate::blocks::traits::BlockBehavior;
use crate::blocks::{MaterialLabeler};
use crate::world::direction::{Facing};

impl BlockBehavior for StamperBlock {
    fn is_directional(&self) -> bool {
        true
    }

    fn material_labeler(&self, facing: Facing) -> Option<MaterialLabeler> {
        Some(MaterialLabeler::Stamper {
            target: facing.forward_ivec3(),
        })
    }
}
