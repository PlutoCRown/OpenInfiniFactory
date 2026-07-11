use super::DrillBlock;

use crate::blocks::traits::BlockBehavior;
use crate::blocks::{MarkerBehavior, MaterialDestroyer};
use glam::IVec3;
use crate::world::direction::Facing;

impl BlockBehavior for DrillBlock {
    fn is_directional(&self) -> bool {
        true
    }

    fn marker_behavior(&self, facing: Facing) -> Option<MarkerBehavior> {
        Some(MarkerBehavior::DrillHead {
            offset: facing.forward_ivec3(),
            facing,
        })
    }

    fn material_destroyer(&self, facing: Facing) -> Option<MaterialDestroyer> {
        Some(MaterialDestroyer::Drill {
            target: facing.forward_ivec3(),
        })
    }

    fn non_connection_face(&self, facing: Facing) -> Option<IVec3> {
        Some(facing.forward_ivec3())
    }
}
