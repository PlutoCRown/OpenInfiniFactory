use super::ReverseConveyorBlock;

use crate::game::blocks::traits::BlockBehavior;
use crate::game::blocks::{MovementRule};
use bevy::prelude::{IVec3};
use crate::game::world::direction::{Facing};

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
}
