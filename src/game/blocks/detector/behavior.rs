use super::DetectorBlock;

use crate::game::blocks::traits::BlockBehavior;
use crate::game::blocks::{SignalBehavior};
use bevy::prelude::IVec3;
use crate::game::world::direction::{Facing};

impl BlockBehavior for DetectorBlock {
    fn is_directional(&self) -> bool {
        true
    }

    fn signal_behavior(&self, facing: Facing) -> Option<SignalBehavior> {
        Some(SignalBehavior::Detector {
            detection_pos: facing.forward_ivec3(),
        })
    }

    fn non_connection_face(&self, facing: Facing) -> Option<IVec3> {
        Some(facing.forward_ivec3())
    }
}
