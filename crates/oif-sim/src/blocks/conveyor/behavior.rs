use super::ConveyorBlock;

use crate::blocks::traits::BlockBehavior;
use crate::blocks::MovementRule;
use crate::world::direction::Facing;
use glam::IVec3;

impl BlockBehavior for ConveyorBlock {
    fn is_directional(&self) -> bool {
        true
    }

    fn movement_rule(&self, facing: Facing) -> Option<MovementRule> {
        Some(MovementRule::Translate {
            source: IVec3::Y,
            offset: facing.forward_ivec3(),
        })
    }

    fn non_connection_face(&self, _facing: Facing) -> Option<IVec3> {
        Some(IVec3::Y)
    }
}
