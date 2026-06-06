use super::DownDetectorBlock;

use crate::game::blocks::traits::BlockBehavior;
use crate::game::blocks::{SignalBehavior};
use bevy::prelude::{IVec3};
use crate::game::world::direction::{Facing};

impl BlockBehavior for DownDetectorBlock {
    fn signal_behavior(&self, _facing: Facing) -> Option<SignalBehavior> {
        Some(SignalBehavior::Detector {
            detection_pos: IVec3::NEG_Y,
        })
    }
}
