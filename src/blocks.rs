use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub const BLOCK_SIZE: f32 = 1.0;

pub const EDIT_BLOCKS: [BlockKind; 3] = [BlockKind::Solid, BlockKind::Glass, BlockKind::Goal];

pub const PLAY_BLOCKS: [BlockKind; 4] = [
    BlockKind::Generator,
    BlockKind::Welder,
    BlockKind::Conveyor,
    BlockKind::Piston,
];

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BlockData {
    pub kind: BlockKind,
    pub facing: Facing,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum BlockKind {
    Solid,
    Glass,
    Generator,
    Welder,
    Conveyor,
    Piston,
    Goal,
    Material,
    WeldPoint,
}

impl BlockKind {
    pub fn name(self) -> &'static str {
        match self {
            BlockKind::Solid => "Solid",
            BlockKind::Glass => "Glass",
            BlockKind::Generator => "Generator",
            BlockKind::Welder => "Welder",
            BlockKind::Conveyor => "Conveyor",
            BlockKind::Piston => "Piston",
            BlockKind::Goal => "Goal",
            BlockKind::Material => "Material",
            BlockKind::WeldPoint => "Weld Point",
        }
    }

    pub fn material(self) -> Color {
        match self {
            BlockKind::Solid => Color::srgb(0.46, 0.48, 0.50),
            BlockKind::Glass => Color::srgba(0.55, 0.82, 0.95, 0.45),
            BlockKind::Generator => Color::srgb(0.52, 0.30, 0.68),
            BlockKind::Welder => Color::srgb(0.76, 0.18, 0.16),
            BlockKind::Conveyor => Color::srgb(0.10, 0.22, 0.28),
            BlockKind::Piston => Color::srgb(0.78, 0.55, 0.28),
            BlockKind::Goal => Color::srgb(0.35, 0.72, 0.42),
            BlockKind::Material => Color::srgb(0.82, 0.82, 0.86),
            BlockKind::WeldPoint => Color::srgba(1.0, 0.28, 0.18, 0.45),
        }
    }

    pub fn is_directional(self) -> bool {
        matches!(
            self,
            BlockKind::Generator | BlockKind::Welder | BlockKind::Conveyor | BlockKind::Piston
        )
    }

    pub fn has_collision(self) -> bool {
        !matches!(self, BlockKind::WeldPoint)
    }

    pub fn is_factory(self) -> bool {
        matches!(
            self,
            BlockKind::Generator | BlockKind::Welder | BlockKind::Conveyor | BlockKind::Piston
        )
    }

    pub fn is_scene(self) -> bool {
        matches!(self, BlockKind::Solid | BlockKind::Glass | BlockKind::Goal)
    }

    pub fn is_material(self) -> bool {
        matches!(self, BlockKind::Material)
    }

    pub fn is_generated_marker(self) -> bool {
        matches!(self, BlockKind::WeldPoint)
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

    pub fn forward_ivec3(self) -> IVec3 {
        match self {
            Facing::North => IVec3::new(0, 0, -1),
            Facing::East => IVec3::new(1, 0, 0),
            Facing::South => IVec3::new(0, 0, 1),
            Facing::West => IVec3::new(-1, 0, 0),
        }
    }
}
