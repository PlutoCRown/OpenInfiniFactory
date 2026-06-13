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
        let forward = facing.forward_ivec3();
        Some(MovementRule::Translate {
            source: IVec3::NEG_Y,
            offset: -forward,
        })
    }
}
