use bevy::prelude::*;

use crate::blocks::Facing;
use crate::world::TargetHit;

#[derive(Resource)]
pub struct PlacementState {
    pub selected: usize,
    pub facing: Facing,
    pub target: Option<TargetHit>,
}

impl Default for PlacementState {
    fn default() -> Self {
        Self {
            selected: 0,
            facing: Facing::North,
            target: None,
        }
    }
}

#[derive(Resource, Clone, Copy, Eq, PartialEq)]
pub enum GameMode {
    Playing,
    Inventory,
    Paused,
}

#[derive(Resource)]
pub struct GameSettings {
    pub fov_degrees: f32,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self { fov_degrees: 70.0 }
    }
}
