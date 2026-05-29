use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Facing {
    North,
    East,
    South,
    West,
}

impl Facing {
    pub fn rotate(self) -> Self {
        match self {
            Facing::North => Facing::East,
            Facing::East => Facing::South,
            Facing::South => Facing::West,
            Facing::West => Facing::North,
        }
    }

    pub fn rotate_counter(self) -> Self {
        match self {
            Facing::North => Facing::West,
            Facing::West => Facing::South,
            Facing::South => Facing::East,
            Facing::East => Facing::North,
        }
    }

    pub fn name_key(self) -> &'static str {
        match self {
            Facing::North => "facing.north",
            Facing::East => "facing.east",
            Facing::South => "facing.south",
            Facing::West => "facing.west",
        }
    }

    pub fn yaw(self) -> f32 {
        match self {
            Facing::North => 0.0,
            Facing::East => -std::f32::consts::FRAC_PI_2,
            Facing::South => std::f32::consts::PI,
            Facing::West => std::f32::consts::FRAC_PI_2,
        }
    }

    pub fn forward(self) -> Vec3 {
        match self {
            Facing::North => Vec3::new(0.0, 0.0, -1.0),
            Facing::East => Vec3::new(1.0, 0.0, 0.0),
            Facing::South => Vec3::new(0.0, 0.0, 1.0),
            Facing::West => Vec3::new(-1.0, 0.0, 0.0),
        }
    }

    pub fn forward_ivec3(self) -> IVec3 {
        match self {
            Facing::North => IVec3::new(0, 0, -1),
            Facing::East => IVec3::new(1, 0, 0),
            Facing::South => IVec3::new(0, 0, 1),
            Facing::West => IVec3::new(-1, 0, 0),
        }
    }

    pub fn inverse_rotate_offset(self, offset: IVec3) -> IVec3 {
        match self {
            Facing::North => offset,
            Facing::East => IVec3::new(-offset.z, offset.y, offset.x),
            Facing::South => IVec3::new(-offset.x, offset.y, -offset.z),
            Facing::West => IVec3::new(offset.z, offset.y, -offset.x),
        }
    }
}
