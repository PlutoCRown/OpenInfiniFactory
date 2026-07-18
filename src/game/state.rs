use bevy::prelude::*;

use crate::game::blocks::BlockData;
use crate::game::world::direction::Facing;
use crate::game::world::grid::TargetHit;

#[derive(Resource)]
pub struct PlacementState {
    pub selected: usize,
    pub target: Option<TargetHit>,
    pub preview_facing: Facing,
    pub edit_gesture: Option<EditGesture>,
    pub selection: SelectionState,
}

impl Default for PlacementState {
    fn default() -> Self {
        Self {
            selected: 0,
            target: None,
            preview_facing: Facing::North,
            edit_gesture: None,
            selection: SelectionState::default(),
        }
    }
}

#[derive(Clone)]
pub struct EditGesture {
    pub kind: EditGestureKind,
    pub start: IVec3,
    pub canceled: bool,
    pub plane_normal: IVec3,
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

    /// 是否仍有选区相关状态（可被右键清空）
    pub fn is_active(&self) -> bool {
        self.first_corner.is_some() || self.bounds.is_some() || self.drag.is_some()
    }
}

/// 选区状态快照，用于撤销/重做
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SelectionSnapshot {
    pub first_corner: Option<IVec3>,
    pub bounds: Option<SelectionBounds>,
}

impl SelectionSnapshot {
    pub fn from_state(state: &SelectionState) -> Self {
        Self {
            first_corner: state.first_corner,
            bounds: state.bounds,
        }
    }

    pub fn apply_to(&self, state: &mut SelectionState) {
        state.first_corner = self.first_corner;
        state.bounds = self.bounds;
        state.drag = None;
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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

    pub fn from_positions(positions: &[IVec3]) -> Option<Self> {
        let first = *positions.first()?;
        let mut min = first;
        let mut max = first;
        for pos in positions.iter().copied().skip(1) {
            min = IVec3::new(min.x.min(pos.x), min.y.min(pos.y), min.z.min(pos.z));
            max = IVec3::new(max.x.max(pos.x), max.y.max(pos.y), max.z.max(pos.z));
        }
        Some(Self { min, max })
    }
}

#[derive(Clone, Copy)]
pub struct SelectionDrag {
    pub start: IVec3,
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

#[derive(States, Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
pub enum GameMode {
    #[default]
    StartMenu,
    Playing,
}

#[derive(Resource, Debug, Clone, Copy, Default, Eq, PartialEq)]
pub enum StartMenuScreen {
    #[default]
    Main,
    SaveList,
}

#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct PlayingUiState {
    pub paused: bool,
    pub inventory_open: bool,
}

impl PlayingUiState {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn active_play(&self) -> bool {
        !self.paused && !self.inventory_open
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum UiPanelId {
    Settings,
    Generator,
    Goal,
    Stamper,
    Roller,
    Converter,
    Teleport,
    Sign,
}

impl UiPanelId {
    pub fn is_settings(self) -> bool {
        self == Self::Settings
    }

    pub fn is_closable(self) -> bool {
        matches!(
            self,
            Self::Settings
                | Self::Generator
                | Self::Goal
                | Self::Stamper
                | Self::Roller
                | Self::Converter
                | Self::Teleport
                | Self::Sign
        )
    }

    pub fn is_blocking_gameplay(self) -> bool {
        true
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
    pub step_requested: bool,
    pub speed: f32,
    pub turn: u64,
    pub accumulator: f32,
    pub start_snapshot: Option<crate::game::world::grid::WorldBlocks>,
    pub start_structures: Option<crate::game::simulation::structure_state::StructureState>,
}

impl Default for SimulationState {
    fn default() -> Self {
        Self {
            running: false,
            step_requested: false,
            speed: 1.0,
            turn: 0,
            accumulator: 0.0,
            start_snapshot: None,
            start_structures: None,
        }
    }
}

impl SimulationState {
    pub fn is_active(&self) -> bool {
        self.running || self.turn > 0
    }

    pub fn authoring_world<'a>(
        &'a self,
        current: &'a crate::game::world::grid::WorldBlocks,
    ) -> &'a crate::game::world::grid::WorldBlocks {
        self.start_snapshot.as_ref().unwrap_or(current)
    }
}

#[derive(Resource, Default)]
pub struct SolutionState {
    pub puzzle_snapshot: Option<crate::game::world::grid::WorldBlocks>,
    pub puzzle_id: Option<String>,
    pub entry: WorldEntryMode,
    pub dirty: bool,
    pub save_list_entry: WorldEntryMode,
}

#[derive(Clone, Copy, Default, Eq, PartialEq)]
pub enum WorldEntryMode {
    #[default]
    EditPuzzle,
    PlaySolution,
}

use crate::shared::save::PlayerSave;

#[derive(Resource, Default)]
pub struct PendingPlayerSpawn(pub Option<PlayerSave>);

#[derive(Resource)]
pub struct GameSettings {
    pub fov_degrees: f32,
    pub ui_scale: f32,
    pub gravity_scale: f32,
    pub mouse_sensitivity_x: f32,
    pub mouse_sensitivity_y: f32,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            fov_degrees: 70.0,
            ui_scale: 1.0,
            gravity_scale: crate::game::GRAVITY_SCALE_DEFAULT,
            mouse_sensitivity_x: crate::game::MOUSE_SENSITIVITY_DEFAULT,
            mouse_sensitivity_y: crate::game::MOUSE_SENSITIVITY_DEFAULT,
        }
    }
}
