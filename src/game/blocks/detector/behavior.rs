use super::DetectorBlock;

use crate::game::blocks::traits::BlockBehavior;
use crate::game::blocks::{SignalBehavior};
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
}
