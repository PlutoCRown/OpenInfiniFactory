use super::ReverseConveyorBlock;

use crate::game::blocks::traits::BlockBehavior;
use crate::game::blocks::MovementRule;
use crate::game::world::direction::Facing;
use bevy::prelude::IVec3;

impl BlockBehavior for ReverseConveyorBlock {
    fn is_directional(&self) -> bool {
        true
    }

    fn movement_rule(&self, facing: Facing) -> Option<MovementRule> {
        Some(MovementRule::Translate {
            source: IVec3::NEG_Y,
            offset: -facing.forward_ivec3(),
        })
    }

    fn non_connection_face(&self, _facing: Facing) -> Option<IVec3> {
        Some(IVec3::NEG_Y)
    }
}
