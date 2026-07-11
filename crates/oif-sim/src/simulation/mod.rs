mod behaviors;
pub mod core;
mod gravity;
pub mod markers;
mod mirror;
pub mod motion;
pub mod movement;
pub mod pending;
pub mod signals;
pub mod stats;
pub mod structure_state;
pub mod structures;
mod suction;

pub use behaviors::{LaserBeam, LaserBeamStop};
pub use suction::SuctionLinks;

use glam::IVec3;

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
