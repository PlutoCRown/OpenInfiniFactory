use super::WelderBlock;

use crate::game::blocks::traits::BlockBehavior;
use crate::game::blocks::{MarkerBehavior};
use crate::game::world::direction::{Facing};

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
}
