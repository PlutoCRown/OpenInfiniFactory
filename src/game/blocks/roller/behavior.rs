use super::RollerBlock;

use crate::game::blocks::traits::BlockBehavior;
use crate::game::blocks::{MaterialLabeler};
use crate::game::world::direction::{Facing};

impl BlockBehavior for RollerBlock {
    fn is_directional(&self) -> bool {
        true
    }

    fn material_labeler(&self, facing: Facing) -> Option<MaterialLabeler> {
        Some(MaterialLabeler::Roller {
            target: facing.forward_ivec3(),
        })
    }
}
