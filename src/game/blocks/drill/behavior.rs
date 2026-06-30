use super::DrillBlock;

use crate::game::blocks::traits::BlockBehavior;
use crate::game::blocks::{MarkerBehavior, MaterialDestroyer};
use bevy::prelude::IVec3;
use crate::game::world::direction::Facing;

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
