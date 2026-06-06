use super::DownWelderBlock;

use crate::game::blocks::traits::BlockBehavior;
use crate::game::blocks::{MarkerBehavior};
use bevy::prelude::{IVec3};
use crate::game::world::direction::{Facing};

impl BlockBehavior for DownWelderBlock {
    fn marker_behavior(&self, _facing: Facing) -> Option<MarkerBehavior> {
        Some(MarkerBehavior::WeldPoint {
            offset: IVec3::NEG_Y,
            facing: Facing::North,
        })
    }
}
