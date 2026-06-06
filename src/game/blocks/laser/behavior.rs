use super::LaserBlock;

use crate::game::blocks::traits::BlockBehavior;
use crate::game::blocks::{MaterialDestroyer, SignalBehavior};
use crate::game::world::direction::{Facing};

impl BlockBehavior for LaserBlock {
    fn is_directional(&self) -> bool {
        true
    }

    fn material_destroyer(&self, facing: Facing) -> Option<MaterialDestroyer> {
        Some(MaterialDestroyer::Laser {
            direction: facing.forward_ivec3(),
            range: 30,
        })
    }

    fn signal_behavior(&self, _facing: Facing) -> Option<SignalBehavior> {
        Some(SignalBehavior::PoweredDevice)
    }
}
