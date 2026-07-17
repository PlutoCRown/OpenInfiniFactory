use super::RollerBlock;

use crate::blocks::traits::BlockBehavior;
use crate::blocks::{MarkerBehavior, MaterialLabeler};
use crate::world::direction::Facing;

impl BlockBehavior for RollerBlock {
    fn is_directional(&self) -> bool {
        true
    }

    fn marker_behavior(&self, facing: Facing) -> Option<MarkerBehavior> {
        Some(MarkerBehavior::RollerBody { facing })
    }

    fn material_labeler(&self, facing: Facing) -> Option<MaterialLabeler> {
        Some(MaterialLabeler::Roller {
            target: facing.forward_ivec3(),
        })
    }
}
