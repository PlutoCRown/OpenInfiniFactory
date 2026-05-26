use bevy::prelude::*;

use crate::game::world::blocks::Facing;
use crate::game::world::grid::TargetHit;

#[derive(Resource)]
pub struct PlacementState {
    pub selected: usize,
    pub facing: Facing,
    pub target: Option<TargetHit>,
    pub pending_delete: Option<IVec3>,
}

impl Default for PlacementState {
    fn default() -> Self {
        Self {
            selected: 0,
            facing: Facing::North,
            target: None,
            pending_delete: None,
        }
    }
}

#[derive(Resource, Clone, Copy, Eq, PartialEq)]
pub enum GameMode {
    MainMenu,
    SaveListMain,
    SaveListPause,
    Settings,
    Playing,
    Inventory,
    Paused,
}

#[derive(Resource, Clone, Copy, Eq, PartialEq)]
pub enum BuilderMode {
    Edit,
    Play,
}

impl Default for BuilderMode {
    fn default() -> Self {
        Self::Edit
    }
}

#[derive(Resource)]
pub struct SimulationState {
    pub running: bool,
    pub speed: f32,
    pub turn: u64,
    pub accumulator: f32,
}

impl Default for SimulationState {
    fn default() -> Self {
        Self {
            running: false,
            speed: 1.0,
            turn: 0,
            accumulator: 0.0,
        }
    }
}

impl SimulationState {
    pub fn is_active(&self) -> bool {
        self.running || self.turn > 0
    }
}

#[derive(Resource)]
pub struct GameSettings {
    pub fov_degrees: f32,
    pub ui_scale: f32,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            fov_degrees: 70.0,
            ui_scale: 1.0,
        }
    }
}
