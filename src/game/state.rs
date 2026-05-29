use bevy::prelude::*;

use crate::game::world::blocks::BlockData;
use crate::game::world::direction::Facing;
use crate::game::world::grid::TargetHit;

#[derive(Resource)]
pub struct PlacementState {
    pub selected: usize,
    pub facing: Facing,
    pub target: Option<TargetHit>,
    pub edit_gesture: Option<EditGesture>,
    pub selection: SelectionState,
    pub generator_panel: Option<IVec3>,
    pub labeler_panel: Option<IVec3>,
    pub converter_panel: Option<IVec3>,
    pub teleport_panel: Option<IVec3>,
}

#[derive(Resource, Default)]
pub struct TeleportRenameState {
    pub editing: Option<IVec3>,
    pub buffer: String,
}

impl Default for PlacementState {
    fn default() -> Self {
        Self {
            selected: 0,
            facing: Facing::North,
            target: None,
            edit_gesture: None,
            selection: SelectionState::default(),
            generator_panel: None,
            labeler_panel: None,
            converter_panel: None,
            teleport_panel: None,
        }
    }
}

#[derive(Clone)]
pub struct EditGesture {
    pub kind: EditGestureKind,
    pub start: IVec3,
    pub canceled: bool,
}

#[derive(Clone)]
pub enum EditGestureKind {
    Place { block: BlockData },
    Delete,
}

#[derive(Clone, Default)]
pub struct SelectionState {
    pub first_corner: Option<IVec3>,
    pub bounds: Option<SelectionBounds>,
    pub drag: Option<SelectionDrag>,
}

impl SelectionState {
    pub fn clear(&mut self) {
        *self = Self::default();
    }
}

#[derive(Clone, Copy)]
pub struct SelectionBounds {
    pub min: IVec3,
    pub max: IVec3,
}

impl SelectionBounds {
    pub fn from_corners(a: IVec3, b: IVec3) -> Self {
        Self {
            min: IVec3::new(a.x.min(b.x), a.y.min(b.y), a.z.min(b.z)),
            max: IVec3::new(a.x.max(b.x), a.y.max(b.y), a.z.max(b.z)),
        }
    }

    pub fn contains(self, pos: IVec3) -> bool {
        (self.min.x..=self.max.x).contains(&pos.x)
            && (self.min.y..=self.max.y).contains(&pos.y)
            && (self.min.z..=self.max.z).contains(&pos.z)
    }

    pub fn moved(self, offset: IVec3) -> Self {
        Self {
            min: self.min + offset,
            max: self.max + offset,
        }
    }

    pub fn positions(self) -> Vec<IVec3> {
        let mut positions = Vec::new();
        for x in self.min.x..=self.max.x {
            for y in self.min.y..=self.max.y {
                for z in self.min.z..=self.max.z {
                    positions.push(IVec3::new(x, y, z));
                }
            }
        }
        positions
    }
}

#[derive(Clone, Copy)]
pub struct SelectionDrag {
    pub start: IVec3,
    pub axis: Option<SelectionAxis>,
    pub offset: IVec3,
}

#[derive(Clone, Copy)]
pub enum SelectionAxis {
    X,
    Y,
    Z,
}

impl SelectionAxis {
    pub fn offset(self, distance: i32) -> IVec3 {
        match self {
            Self::X => IVec3::X * distance,
            Self::Y => IVec3::Y * distance,
            Self::Z => IVec3::Z * distance,
        }
    }
}

#[derive(Resource, Clone, Copy, Eq, PartialEq)]
pub enum GameMode {
    MainMenu,
    SaveListMain,
    SaveListPause,
    GeneratorSettings,
    LabelerSettings,
    ConverterSettings,
    TeleportSettings,
    Settings,
    Playing,
    Inventory,
    Paused,
}

#[derive(Resource, Clone, Copy, Eq, PartialEq)]
pub struct SettingsReturnMode(pub GameMode);

impl Default for SettingsReturnMode {
    fn default() -> Self {
        Self(GameMode::Paused)
    }
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
