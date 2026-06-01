use bevy::prelude::*;
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::game::world::blocks::{
    BlockData, ConverterSettings, GeneratorSettings, GoalSettings, PersistentLayer,
    SerializedBlockState, StamperSettings, TeleportSettings,
};
use crate::game::world::grid::WorldBlocks;
use crate::game::{
    state::BuilderMode,
    ui::{HotbarItems, InventoryItems},
};

pub const SAVE_DIR: &str = "saves";
#[derive(Resource, Default)]
pub struct SaveState {
    pub current: Option<String>,
    pub current_kind: Option<SaveKind>,
    pub entries: Vec<SaveEntry>,
    pub selected_puzzle: Option<String>,
    pub selected_puzzle_kind: Option<SavePuzzleSource>,
    selected_puzzle_solutions: Vec<SaveEntry>,
}

impl SaveState {
    pub fn refresh(&mut self) {
        self.entries = list_save_entries();
        self.refresh_selected_puzzle_solutions();
    }

    pub fn puzzles(&self) -> Vec<&SaveEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.kind == SaveKind::Puzzle)
            .collect()
    }

    pub fn puzzle_choices(&self) -> Vec<SavePuzzleChoice> {
        let mut choices: Vec<SavePuzzleChoice> = self
            .entries
            .iter()
            .filter_map(|entry| match entry.kind {
                SaveKind::Puzzle => Some(SavePuzzleChoice {
                    name: entry.name.clone(),
                    source: SavePuzzleSource::PuzzleFile,
                }),
                SaveKind::Solution => Some(SavePuzzleChoice {
                    name: entry.name.clone(),
                    source: SavePuzzleSource::SolutionSnapshot,
                }),
            })
            .collect();
        choices.sort_by(|a, b| a.name.cmp(&b.name).then(a.source.cmp(&b.source)));
        choices
    }

    pub fn select_puzzle(&mut self, puzzle: Option<String>, source: Option<SavePuzzleSource>) {
        if self.selected_puzzle == puzzle && self.selected_puzzle_kind == source {
            return;
        }
        self.selected_puzzle = puzzle;
        self.selected_puzzle_kind = source;
        self.refresh_selected_puzzle_solutions();
    }

    pub fn selected_puzzle_solutions(&self) -> &[SaveEntry] {
        &self.selected_puzzle_solutions
    }

    fn refresh_selected_puzzle_solutions(&mut self) {
        self.selected_puzzle_solutions =
            match (self.selected_puzzle.as_deref(), self.selected_puzzle_kind) {
                (Some(puzzle), Some(SavePuzzleSource::PuzzleFile)) => self
                    .entries
                    .iter()
                    .filter(|entry| {
                        entry.kind == SaveKind::Solution
                            && solution_matches_puzzle(&entry.name, puzzle)
                    })
                    .cloned()
                    .collect(),
                (Some(solution), Some(SavePuzzleSource::SolutionSnapshot)) => self
                    .entries
                    .iter()
                    .find(|entry| entry.kind == SaveKind::Solution && entry.name == solution)
                    .cloned()
                    .into_iter()
                    .collect(),
                _ => Vec::new(),
            };
    }
}

#[derive(Clone)]
pub struct SaveEntry {
    pub name: String,
    pub kind: SaveKind,
}

#[derive(Clone)]
pub struct SavePuzzleChoice {
    pub name: String,
    pub source: SavePuzzleSource,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum SavePuzzleSource {
    PuzzleFile,
    SolutionSnapshot,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum SaveKind {
    Puzzle,
    Solution,
}

#[derive(Serialize, Deserialize)]
struct SaveFile {
    #[serde(default)]
    kind: SaveFileKind,
    #[serde(default)]
    puzzle: Option<WorldLayer>,
    #[serde(default)]
    puzzle_ref: Option<String>,
    #[serde(default)]
    factory_blocks: Vec<SavedBlock>,
    blocks: Vec<SavedBlock>,
    #[serde(default)]
    system_blocks: Vec<SavedBlock>,
    #[serde(default)]
    block_states: Vec<SavedBlockState>,
    #[serde(default)]
    block_settings: Vec<LegacySavedBlockSettings>,
    #[serde(default)]
    hotbar: Option<HotbarItems>,
}

#[derive(Default, Serialize, Deserialize)]
enum SaveFileKind {
    #[default]
    Legacy,
    Puzzle,
    Solution,
}

#[derive(Clone, Serialize, Deserialize)]
struct WorldLayer {
    blocks: Vec<SavedBlock>,
    #[serde(default)]
    system_blocks: Vec<SavedBlock>,
    #[serde(default)]
    block_states: Vec<SavedBlockState>,
    #[serde(default)]
    block_settings: Vec<LegacySavedBlockSettings>,
    #[serde(default)]
    hotbar: Option<HotbarItems>,
}

impl WorldLayer {
    fn all_block_states(&self) -> Vec<SavedBlockState> {
        self.block_states
            .iter()
            .cloned()
            .chain(
                self.block_settings
                    .iter()
                    .filter_map(LegacySavedBlockSettings::to_block_state),
            )
            .collect()
    }
}

#[derive(Clone, Serialize, Deserialize)]
struct SavedBlock {
    x: i32,
    y: i32,
    z: i32,
    data: BlockData,
}

#[derive(Clone, Serialize, Deserialize)]
struct SavedBlockState {
    x: i32,
    y: i32,
    z: i32,
    state: SerializedBlockState,
}

#[derive(Clone, Serialize, Deserialize)]
struct LegacySavedBlockSettings {
    x: i32,
    y: i32,
    z: i32,
    settings: LegacyBlockSettings,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
enum LegacyBlockSettings {
    Generator(GeneratorSettings),
    Goal(GoalSettings),
    Labeler(StamperSettings),
    Converter(ConverterSettings),
    Teleport(TeleportSettings),
}

pub fn save_world(
    world: &WorldBlocks,
    name: &str,
    kind: SaveKind,
    inventory: &InventoryItems,
) -> bool {
    let save = match kind {
        SaveKind::Puzzle => SaveFile::puzzle(capture_puzzle_layer(world, inventory)),
        SaveKind::Solution => {
            SaveFile::legacy_solution(capture_puzzle_layer(world, inventory), world, inventory)
        }
    };

    write_save(name, &save)
}

pub fn save_solution_with_puzzle(
    world: &WorldBlocks,
    name: &str,
    puzzle_name: &str,
    _puzzle_snapshot: &WorldBlocks,
    inventory: &InventoryItems,
) -> bool {
    write_save(name, &SaveFile::solution(puzzle_name, world, inventory))
}

pub fn load_world(world: &mut WorldBlocks, name: &str) -> Option<LoadedSave> {
    let Ok(contents) = fs::read_to_string(save_path(name)) else {
        return None;
    };
    let Ok(save) = ron::from_str::<SaveFile>(&contents) else {
        return None;
    };

    let loaded = save.into_loaded();
    *world = loaded.world.clone();
    Some(loaded)
}

pub fn save_kind(name: &str) -> Option<SaveKind> {
    let contents = fs::read_to_string(save_path(name)).ok()?;
    let save = ron::from_str::<SaveFile>(&contents).ok()?;
    Some(match save.kind {
        SaveFileKind::Solution => SaveKind::Solution,
        SaveFileKind::Puzzle | SaveFileKind::Legacy => SaveKind::Puzzle,
    })
}

pub fn delete_save(name: &str) -> bool {
    match fs::remove_file(save_path(name)) {
        Ok(()) => true,
        Err(error) => {
            warn!("Failed to delete save {name}: {error}");
            false
        }
    }
}

pub fn rename_save(old_name: &str, new_name: &str) -> bool {
    let new_name = normalized_save_name(new_name);
    if new_name.is_empty() || old_name == new_name {
        return false;
    }
    let old_path = save_path(old_name);
    let new_path = save_path(&new_name);
    if new_path.exists() {
        warn!("Cannot rename save {old_name} to {new_name}: target already exists");
        return false;
    }
    match fs::rename(old_path, new_path) {
        Ok(()) => true,
        Err(error) => {
            warn!("Failed to rename save {old_name} to {new_name}: {error}");
            false
        }
    }
}

fn solution_matches_puzzle(solution: &str, puzzle: &str) -> bool {
    let Some(solution) = read_save(solution) else {
        return false;
    };
    if !matches!(solution.kind, SaveFileKind::Solution) {
        return false;
    }
    if solution.puzzle_ref.as_deref() == Some(puzzle) {
        return true;
    }
    let Some(puzzle_save) = read_save(puzzle) else {
        return false;
    };
    solution.puzzle_signature() == puzzle_save.puzzle_signature()
}

pub fn reset_solution_world(world: &mut WorldBlocks, puzzle_snapshot: &WorldBlocks) {
    *world = puzzle_snapshot.clone();
}

#[derive(Clone)]
pub struct LoadedSave {
    pub world: WorldBlocks,
    pub puzzle_snapshot: Option<WorldBlocks>,
    pub puzzle_name: Option<String>,
    pub hotbar: Option<HotbarItems>,
}

impl SaveFile {
    fn puzzle(puzzle: WorldLayer) -> Self {
        Self {
            kind: SaveFileKind::Puzzle,
            puzzle: Some(puzzle.clone()),
            puzzle_ref: None,
            factory_blocks: Vec::new(),
            blocks: puzzle.blocks,
            system_blocks: puzzle.system_blocks,
            block_states: puzzle.block_states,
            block_settings: Vec::new(),
            hotbar: puzzle.hotbar,
        }
    }

    fn solution(puzzle_name: &str, world: &WorldBlocks, inventory: &InventoryItems) -> Self {
        Self {
            kind: SaveFileKind::Solution,
            puzzle: None,
            puzzle_ref: Some(normalized_save_name(puzzle_name)),
            factory_blocks: capture_factory_blocks(world),
            blocks: Vec::new(),
            system_blocks: Vec::new(),
            block_states: Vec::new(),
            block_settings: Vec::new(),
            hotbar: Some(inventory.hotbar),
        }
    }

    fn legacy_solution(
        puzzle: WorldLayer,
        world: &WorldBlocks,
        inventory: &InventoryItems,
    ) -> Self {
        Self {
            kind: SaveFileKind::Solution,
            puzzle: Some(puzzle.clone()),
            puzzle_ref: None,
            factory_blocks: capture_factory_blocks(world),
            blocks: puzzle.blocks,
            system_blocks: puzzle.system_blocks,
            block_states: puzzle.block_states,
            block_settings: Vec::new(),
            hotbar: Some(inventory.hotbar),
        }
    }

    fn into_loaded(self) -> LoadedSave {
        match self.kind {
            SaveFileKind::Solution => {
                let puzzle = self.referenced_puzzle_layer().unwrap_or_else(|| {
                    self.puzzle
                        .clone()
                        .unwrap_or_else(|| self.legacy_puzzle_layer())
                });
                let hotbar = self.hotbar.or(puzzle.hotbar);
                let mut puzzle_world = WorldBlocks::default();
                apply_layer(&mut puzzle_world, puzzle.clone());

                let mut world = puzzle_world.clone();
                apply_factory_blocks(&mut world, self.factory_blocks);

                LoadedSave {
                    world,
                    puzzle_snapshot: Some(puzzle_world),
                    puzzle_name: self.puzzle_ref,
                    hotbar: hotbar
                        .or_else(|| Some(InventoryItems::for_mode(BuilderMode::Play).hotbar)),
                }
            }
            SaveFileKind::Puzzle | SaveFileKind::Legacy => {
                let puzzle = self
                    .puzzle
                    .clone()
                    .unwrap_or_else(|| self.legacy_puzzle_layer());
                let hotbar = self.hotbar.or(puzzle.hotbar);
                let mut world = WorldBlocks::default();
                apply_layer(&mut world, puzzle);
                LoadedSave {
                    world,
                    puzzle_snapshot: None,
                    puzzle_name: None,
                    hotbar: hotbar
                        .or_else(|| Some(InventoryItems::for_mode(BuilderMode::Edit).hotbar)),
                }
            }
        }
    }

    fn puzzle_signature(&self) -> Vec<String> {
        let puzzle = self
            .referenced_puzzle_layer()
            .or_else(|| self.puzzle.clone())
            .unwrap_or_else(|| self.legacy_puzzle_layer());
        let mut parts: Vec<String> = Vec::new();
        for saved in &puzzle.blocks {
            parts.push(format!(
                "b:{},{},{}:{:?}:{:?}",
                saved.x, saved.y, saved.z, saved.data.kind, saved.data.facing
            ));
        }
        for saved in &puzzle.system_blocks {
            parts.push(format!(
                "s:{},{},{}:{:?}:{:?}",
                saved.x, saved.y, saved.z, saved.data.kind, saved.data.facing
            ));
        }
        for saved in puzzle.all_block_states() {
            parts.push(format!(
                "bs:{},{},{}:{}",
                saved.x,
                saved.y,
                saved.z,
                saved.state.debug_signature()
            ));
        }
        parts.sort();
        parts
    }

    fn legacy_puzzle_layer(&self) -> WorldLayer {
        WorldLayer {
            blocks: self
                .blocks
                .iter()
                .filter(|saved| saved.data.kind.persistent_layer() == Some(PersistentLayer::Puzzle))
                .cloned()
                .collect(),
            system_blocks: self
                .system_blocks
                .iter()
                .filter(|saved| saved.data.kind.persistent_layer() == Some(PersistentLayer::Puzzle))
                .cloned()
                .collect(),
            block_states: self
                .block_states
                .iter()
                .filter(|saved| self.legacy_system_block_is_persistent(saved.pos()))
                .cloned()
                .collect(),
            block_settings: self
                .block_settings
                .iter()
                .filter(|saved| self.legacy_system_block_is_persistent(saved.pos()))
                .cloned()
                .collect(),
            hotbar: self.hotbar,
        }
    }

    fn referenced_puzzle_layer(&self) -> Option<WorldLayer> {
        let puzzle_name = self.puzzle_ref.as_deref()?;
        read_save(puzzle_name).map(|save| {
            save.puzzle
                .clone()
                .unwrap_or_else(|| save.legacy_puzzle_layer())
        })
    }

    fn legacy_system_block_is_persistent(&self, pos: IVec3) -> bool {
        self.system_blocks.iter().any(|block| {
            block.pos() == pos
                && block.data.kind.persistent_layer() == Some(PersistentLayer::Puzzle)
        })
    }
}

fn write_save(name: &str, save: &SaveFile) -> bool {
    if let Err(error) = fs::create_dir_all(SAVE_DIR) {
        warn!("Failed to create save directory: {error}");
        return false;
    }

    let path = save_path(name);
    match ron::ser::to_string_pretty(save, PrettyConfig::default()) {
        Ok(serialized) => {
            if let Err(error) = fs::write(path, serialized) {
                warn!("Failed to save world: {error}");
                return false;
            }
            true
        }
        Err(error) => {
            warn!("Failed to serialize world: {error}");
            false
        }
    }
}

fn read_save(name: &str) -> Option<SaveFile> {
    let contents = fs::read_to_string(save_path(name)).ok()?;
    ron::from_str::<SaveFile>(&contents).ok()
}

fn capture_puzzle_layer(world: &WorldBlocks, inventory: &InventoryItems) -> WorldLayer {
    let blocks: Vec<SavedBlock> = world
        .blocks
        .iter()
        .filter_map(|(pos, data)| {
            (data.kind.persistent_layer() == Some(PersistentLayer::Puzzle))
                .then_some(saved_block(*pos, *data))
        })
        .collect();
    let system_blocks: Vec<SavedBlock> = world
        .system_blocks
        .iter()
        .filter_map(|(pos, data)| {
            (data.kind.persistent_layer() == Some(PersistentLayer::Puzzle))
                .then_some(saved_block(*pos, *data))
        })
        .collect();

    WorldLayer {
        blocks,
        system_blocks,
        block_states: capture_block_states(world, PersistentLayer::Puzzle),
        block_settings: Vec::new(),
        hotbar: Some(inventory.hotbar),
    }
}

fn capture_block_states(world: &WorldBlocks, layer: PersistentLayer) -> Vec<SavedBlockState> {
    world
        .block_states
        .iter()
        .filter_map(|(pos, state)| {
            world
                .system_blocks
                .get(pos)
                .or_else(|| world.blocks.get(pos))
                .is_some_and(|block| block.kind.persistent_layer() == Some(layer))
                .then_some(SavedBlockState {
                    x: pos.x,
                    y: pos.y,
                    z: pos.z,
                    state: state.clone(),
                })
        })
        .collect()
}

fn capture_factory_blocks(world: &WorldBlocks) -> Vec<SavedBlock> {
    world
        .blocks
        .iter()
        .filter_map(|(pos, data)| {
            (data.kind.persistent_layer() == Some(PersistentLayer::SolutionFactory))
                .then_some(saved_block(*pos, *data))
        })
        .collect()
}

fn apply_layer(world: &mut WorldBlocks, layer: WorldLayer) {
    let block_states = layer.all_block_states();
    for saved in layer.blocks {
        world.insert(saved.pos(), saved.data);
    }
    for saved in layer.system_blocks {
        world.insert(saved.pos(), saved.data);
    }
    for saved in block_states {
        world.set_block_serialized_state(saved.pos(), saved.state);
    }
}

fn apply_factory_blocks(world: &mut WorldBlocks, factory_blocks: Vec<SavedBlock>) {
    for saved in factory_blocks {
        if saved.data.kind.persistent_layer() == Some(PersistentLayer::SolutionFactory) {
            world.insert(saved.pos(), saved.data);
        }
    }
}

fn saved_block(pos: IVec3, data: BlockData) -> SavedBlock {
    SavedBlock {
        x: pos.x,
        y: pos.y,
        z: pos.z,
        data,
    }
}

impl SavedBlock {
    fn pos(&self) -> IVec3 {
        IVec3::new(self.x, self.y, self.z)
    }
}

impl SavedBlockState {
    fn pos(&self) -> IVec3 {
        IVec3::new(self.x, self.y, self.z)
    }
}

impl LegacySavedBlockSettings {
    fn pos(&self) -> IVec3 {
        IVec3::new(self.x, self.y, self.z)
    }

    fn to_block_state(&self) -> Option<SavedBlockState> {
        let state = match &self.settings {
            LegacyBlockSettings::Generator(settings) => SerializedBlockState::from_state(settings),
            LegacyBlockSettings::Goal(settings) => SerializedBlockState::from_state(settings),
            LegacyBlockSettings::Labeler(settings) => {
                SerializedBlockState::from_state::<StamperSettings>(settings)
            }
            LegacyBlockSettings::Converter(settings) => SerializedBlockState::from_state(settings),
            LegacyBlockSettings::Teleport(settings) => SerializedBlockState::from_state(settings),
        }?;
        Some(SavedBlockState {
            x: self.x,
            y: self.y,
            z: self.z,
            state,
        })
    }
}

pub fn list_saves() -> Vec<String> {
    let Ok(entries) = fs::read_dir(SAVE_DIR) else {
        return Vec::new();
    };

    let mut saves: Vec<String> = entries
        .filter_map(Result::ok)
        .filter_map(|entry| save_name_from_path(&entry.path()))
        .collect();
    saves.sort();
    saves
}

pub fn list_save_entries() -> Vec<SaveEntry> {
    let mut entries: Vec<SaveEntry> = list_saves()
        .into_iter()
        .filter_map(|name| save_kind(&name).map(|kind| SaveEntry { name, kind }))
        .collect();
    entries.sort_by(|a, b| a.name.cmp(&b.name));
    entries
}

pub fn next_world_name(existing: &[String]) -> String {
    for index in 1.. {
        let candidate = format!("world_{index}");
        if !existing.iter().any(|name| name == &candidate) {
            return candidate;
        }
    }
    unreachable!()
}

pub fn next_named_save(existing: &[String], base: &str) -> String {
    let base = normalized_save_name(base);
    if base.is_empty() {
        return next_world_name(existing);
    }
    if !existing.iter().any(|name| name == &base) {
        return base;
    }
    for index in 2.. {
        let candidate = format!("{base}_{index}");
        if !existing.iter().any(|name| name == &candidate) {
            return candidate;
        }
    }
    unreachable!()
}

fn save_path(name: &str) -> PathBuf {
    Path::new(SAVE_DIR).join(format!("{}.ron", sanitize_save_name(name)))
}

fn save_name_from_path(path: &Path) -> Option<String> {
    let is_ron = path.extension().and_then(|ext| ext.to_str()) == Some("ron");
    is_ron.then(|| path.file_stem()?.to_str().map(ToOwned::to_owned))?
}

fn sanitize_save_name(name: &str) -> String {
    name.chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

pub fn normalized_save_name(name: &str) -> String {
    sanitize_save_name(name.trim())
        .trim_matches('_')
        .to_string()
}
