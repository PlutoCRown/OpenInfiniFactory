use super::WireBlock;

use crate::blocks::traits::BlockBehavior;
use crate::blocks::{SignalBehavior};
use crate::world::direction::{Facing};

impl BlockBehavior for WireBlock {
    fn signal_behavior(&self, _facing: Facing) -> Option<SignalBehavior> {
        Some(SignalBehavior::Wire)
    }
}
