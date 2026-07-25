use super::LifterBlock;

use glam::IVec3;

use crate::blocks::traits::BlockBehavior;
use crate::blocks::{MovementRule, SignalBehavior, WireFacePolicy};
use crate::world::direction::Facing;

impl BlockBehavior for LifterBlock {
    fn is_directional(&self) -> bool {
        true
    }

    fn movement_rule(&self, _facing: Facing) -> Option<MovementRule> {
        Some(MovementRule::Lift { range: 5 })
    }

    fn signal_behavior(&self, _facing: Facing) -> Option<SignalBehavior> {
        // 仅底面接线；通电时关闭抬升
        Some(SignalBehavior::PoweredDevice {
            wire_face: WireFacePolicy::AllowOnly(IVec3::NEG_Y),
        })
    }

    fn non_connection_face(&self, _facing: Facing) -> Option<IVec3> {
        Some(IVec3::Y)
    }
}
