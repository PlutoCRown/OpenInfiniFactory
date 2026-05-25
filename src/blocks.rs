use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub const BLOCK_SIZE: f32 = 1.0;
pub const ALL_BLOCKS: [BlockKind; 5] = [
    BlockKind::Solid,
    BlockKind::Conveyor,
    BlockKind::Piston,
    BlockKind::Glass,
    BlockKind::Goal,
];

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BlockData {
    pub kind: BlockKind,
    pub facing: Facing,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum BlockKind {
    Solid,
    Conveyor,
    Piston,
    Glass,
    Goal,
}

impl BlockKind {
    pub fn name(self) -> &'static str {
        match self {
            BlockKind::Solid => "Solid",
            BlockKind::Conveyor => "Conveyor",
            BlockKind::Piston => "Piston",
            BlockKind::Glass => "Glass",
            BlockKind::Goal => "Goal",
        }
    }

    pub fn material(self) -> Color {
        match self {
            BlockKind::Solid => Color::srgb(0.46, 0.48, 0.50),
            BlockKind::Conveyor => Color::srgb(0.10, 0.22, 0.28),
            BlockKind::Piston => Color::srgb(0.78, 0.55, 0.28),
            BlockKind::Glass => Color::srgba(0.55, 0.82, 0.95, 0.45),
            BlockKind::Goal => Color::srgb(0.35, 0.72, 0.42),
        }
    }

    pub fn is_directional(self) -> bool {
        matches!(self, BlockKind::Conveyor | BlockKind::Piston)
    }
}

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

    pub fn name(self) -> &'static str {
        match self {
            Facing::North => "North",
            Facing::East => "East",
            Facing::South => "South",
            Facing::West => "West",
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
}
