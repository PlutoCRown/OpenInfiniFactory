use super::StamperBlock;

use crate::blocks::traits::BlockBehavior;
use crate::blocks::{MarkerBehavior, MaterialLabeler};
use crate::world::direction::Facing;

impl BlockBehavior for StamperBlock {
    fn is_directional(&self) -> bool {
        true
    }

    fn marker_behavior(&self, facing: Facing) -> Option<MarkerBehavior> {
        Some(MarkerBehavior::StamperBody { facing })
    }

    fn material_labeler(&self, facing: Facing) -> Option<MaterialLabeler> {
        Some(MaterialLabeler::Stamper {
            target: facing.forward_ivec3(),
        })
    }
}
