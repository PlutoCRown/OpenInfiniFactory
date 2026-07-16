use super::WelderBlock;

use crate::blocks::traits::BlockBehavior;
use crate::blocks::{MarkerBehavior};
use glam::IVec3;
use crate::world::direction::{Facing};

impl BlockBehavior for WelderBlock {
    fn is_directional(&self) -> bool {
        true
    }

    fn marker_behavior(&self, facing: Facing) -> Option<MarkerBehavior> {
        Some(MarkerBehavior::WeldPoint {
            offset: facing.forward_ivec3(),
            facing,
        })
    }

    fn non_connection_face(&self, facing: Facing) -> Option<IVec3> {
        Some(facing.forward_ivec3())
    }
}
