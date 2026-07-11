use super::SuctionCupBlock;

use glam::IVec3;

use crate::blocks::traits::BlockBehavior;
use crate::blocks::SignalBehavior;
use crate::world::direction::Facing;

impl BlockBehavior for SuctionCupBlock {
    fn is_directional(&self) -> bool {
        true
    }

    fn signal_behavior(&self, _facing: Facing) -> Option<SignalBehavior> {
        Some(SignalBehavior::PoweredDevice)
    }

    fn non_connection_face(&self, facing: Facing) -> Option<IVec3> {
        Some(facing.forward_ivec3())
    }
}
