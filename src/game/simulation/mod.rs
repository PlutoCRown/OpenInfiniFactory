mod behaviors;
pub mod core;
mod gravity;
pub mod markers;
pub mod movement;
pub mod runtime;
mod signals;
pub mod structure_state;
pub mod structures;

use bevy::prelude::*;

fn signal_offsets() -> [IVec3; 6] {
    [
        IVec3::X,
        IVec3::NEG_X,
        IVec3::Y,
        IVec3::NEG_Y,
        IVec3::Z,
        IVec3::NEG_Z,
    ]
}
