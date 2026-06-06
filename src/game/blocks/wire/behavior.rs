use super::WireBlock;

use crate::game::blocks::traits::BlockBehavior;
use crate::game::blocks::{SignalBehavior};
use crate::game::world::direction::{Facing};

impl BlockBehavior for WireBlock {
    fn signal_behavior(&self, _facing: Facing) -> Option<SignalBehavior> {
        Some(SignalBehavior::Wire)
    }
}
