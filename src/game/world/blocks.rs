use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub const BLOCK_SIZE: f32 = 1.0;
pub const DEFAULT_GENERATOR_PERIOD: u64 = 3;

pub const EDIT_BLOCKS: [BlockKind; 4] = [
    BlockKind::SelectionTool,
    BlockKind::Solid,
    BlockKind::Glass,
    BlockKind::Goal,
];

pub const PLAY_BLOCKS: [BlockKind; 12] = [
    BlockKind::SelectionTool,
    BlockKind::Generator,
    BlockKind::Welder,
    BlockKind::Conveyor,
    BlockKind::Detector,
    BlockKind::Wire,
    BlockKind::Piston,
    BlockKind::Lifter,
    BlockKind::Rotator,
    BlockKind::Blocker,
    BlockKind::Drill,
    BlockKind::Laser,
];

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BlockData {
    pub kind: BlockKind,
    pub facing: Facing,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum BlockKind {
    SelectionTool,
    Solid,
    Glass,
    Generator,
    Welder,
    Conveyor,
    Detector,
    Wire,
    Piston,
    Lifter,
    Rotator,
    Blocker,
    Drill,
    Laser,
    Goal,
    Material,
    WeldPoint,
    BlockerHead,
    DrillHead,
}

impl BlockKind {
    pub fn name_key(self) -> &'static str {
        match self {
            BlockKind::SelectionTool => "block.selection_tool",
            BlockKind::Solid => "block.solid",
            BlockKind::Glass => "block.glass",
            BlockKind::Generator => "block.generator",
            BlockKind::Welder => "block.welder",
            BlockKind::Conveyor => "block.conveyor",
            BlockKind::Detector => "block.detector",
            BlockKind::Wire => "block.wire",
            BlockKind::Piston => "block.piston",
            BlockKind::Lifter => "block.lifter",
            BlockKind::Rotator => "block.rotator",
            BlockKind::Blocker => "block.blocker",
            BlockKind::Drill => "block.drill",
            BlockKind::Laser => "block.laser",
            BlockKind::Goal => "block.goal",
            BlockKind::Material => "block.material",
            BlockKind::WeldPoint => "block.weld_point",
            BlockKind::BlockerHead => "block.blocker_head",
            BlockKind::DrillHead => "block.drill_head",
        }
    }

    pub fn material(self) -> Color {
        match self {
            BlockKind::SelectionTool => Color::srgb(0.30, 0.86, 0.82),
            BlockKind::Solid => Color::srgb(0.46, 0.48, 0.50),
            BlockKind::Glass => Color::srgba(0.55, 0.82, 0.95, 0.45),
            BlockKind::Generator => Color::srgb(0.52, 0.30, 0.68),
            BlockKind::Welder => Color::srgb(0.76, 0.18, 0.16),
            BlockKind::Conveyor => Color::srgb(0.10, 0.22, 0.28),
            BlockKind::Detector => Color::srgb(0.15, 0.45, 0.72),
            BlockKind::Wire => Color::srgb(0.95, 0.72, 0.18),
            BlockKind::Piston => Color::srgb(0.78, 0.55, 0.28),
            BlockKind::Lifter => Color::srgb(0.25, 0.58, 0.72),
            BlockKind::Rotator => Color::srgb(0.48, 0.32, 0.72),
            BlockKind::Blocker => Color::srgb(0.58, 0.40, 0.24),
            BlockKind::Drill => Color::srgb(0.32, 0.36, 0.40),
            BlockKind::Laser => Color::srgb(0.85, 0.20, 0.34),
            BlockKind::Goal => Color::srgb(0.35, 0.72, 0.42),
            BlockKind::Material => Color::srgb(0.82, 0.82, 0.86),
            BlockKind::WeldPoint => Color::srgba(1.0, 0.28, 0.18, 0.45),
            BlockKind::BlockerHead => Color::srgb(0.70, 0.48, 0.28),
            BlockKind::DrillHead => Color::srgb(0.12, 0.14, 0.16),
        }
    }

    pub fn is_directional(self) -> bool {
        matches!(
            self,
            BlockKind::Generator
                | BlockKind::Welder
                | BlockKind::Conveyor
                | BlockKind::Detector
                | BlockKind::Piston
                | BlockKind::Blocker
                | BlockKind::Drill
                | BlockKind::Laser
        )
    }

    pub fn has_collision(self) -> bool {
        !matches!(
            self,
            BlockKind::SelectionTool | BlockKind::WeldPoint | BlockKind::DrillHead
        )
    }

    pub fn is_factory(self) -> bool {
        matches!(
            self,
            BlockKind::Generator
                | BlockKind::Welder
                | BlockKind::Conveyor
                | BlockKind::Detector
                | BlockKind::Wire
                | BlockKind::Piston
                | BlockKind::Lifter
                | BlockKind::Rotator
                | BlockKind::Blocker
                | BlockKind::Drill
                | BlockKind::Laser
        )
    }

    pub fn is_scene(self) -> bool {
        matches!(self, BlockKind::Solid | BlockKind::Glass | BlockKind::Goal)
    }

    pub fn is_material(self) -> bool {
        matches!(self, BlockKind::Material)
    }

    pub fn is_generated_marker(self) -> bool {
        matches!(
            self,
            BlockKind::WeldPoint | BlockKind::BlockerHead | BlockKind::DrillHead
        )
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
}
