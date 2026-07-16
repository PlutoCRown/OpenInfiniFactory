use super::DownWelderBlock;

use crate::blocks::traits::BlockBehavior;
use crate::blocks::{MarkerBehavior};
use glam::IVec3;
use crate::world::direction::{Facing};

impl BlockBehavior for DownWelderBlock {
    fn marker_behavior(&self, _facing: Facing) -> Option<MarkerBehavior> {
        Some(MarkerBehavior::WeldPoint {
            offset: IVec3::NEG_Y,
            facing: Facing::North,
        })
    }

    fn non_connection_face(&self, _facing: Facing) -> Option<IVec3> {
        Some(IVec3::NEG_Y)
    }
}
