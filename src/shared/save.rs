use bevy::prelude::*;
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use crate::game::blocks::{BlockData, PersistentLayer};
use crate::game::world::grid::{BlockSettings, WorldBlocks};
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
    factory_blocks: Vec<SavedBlock>,
    blocks: Vec<SavedBlock>,
    #[serde(default)]
    system_blocks: Vec<SavedBlock>,
    #[serde(default)]
    block_settings: Vec<SavedBlockSettings>,
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
    block_settings: Vec<SavedBlockSettings>,
    #[serde(default)]
    hotbar: Option<HotbarItems>,
}

#[derive(Clone, Serialize, Deserialize)]
struct SavedBlock {
    x: i32,
    y: i32,
    z: i32,
    data: BlockData,
}

#[derive(Clone, Serialize, Deserialize)]
struct SavedBlockSettings {
    x: i32,
    y: i32,
    z: i32,
    settings: BlockSettings,
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
            SaveFile::solution(capture_puzzle_layer(world, inventory), world, inventory)
        }
    };

    write_save(name, &save)
}

pub fn save_solution_with_puzzle(
    world: &WorldBlocks,
    name: &str,
    puzzle_snapshot: &WorldBlocks,
    inventory: &InventoryItems,
) -> bool {
    write_save(
        name,
        &SaveFile::solution(
            capture_puzzle_layer(puzzle_snapshot, inventory),
            world,
            inventory,
        ),
    )
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
    pub hotbar: Option<HotbarItems>,
}

impl SaveFile {
    fn puzzle(puzzle: WorldLayer) -> Self {
        Self {
            kind: SaveFileKind::Puzzle,
            puzzle: Some(puzzle.clone()),
            factory_blocks: Vec::new(),
            blocks: puzzle.blocks,
            system_blocks: puzzle.system_blocks,
            block_settings: puzzle.block_settings,
            hotbar: puzzle.hotbar,
        }
    }

    fn solution(puzzle: WorldLayer, world: &WorldBlocks, inventory: &InventoryItems) -> Self {
        Self {
            kind: SaveFileKind::Solution,
            puzzle: Some(puzzle.clone()),
            factory_blocks: capture_factory_blocks(world),
            blocks: puzzle.blocks,
            system_blocks: puzzle.system_blocks,
            block_settings: puzzle.block_settings,
            hotbar: Some(inventory.hotbar),
        }
    }

    fn into_loaded(self) -> LoadedSave {
        match self.kind {
            SaveFileKind::Solution => {
                let puzzle = self
                    .puzzle
                    .clone()
                    .unwrap_or_else(|| self.legacy_puzzle_layer());
                let hotbar = self.hotbar.or(puzzle.hotbar);
                let mut puzzle_world = WorldBlocks::default();
                apply_layer(&mut puzzle_world, puzzle.clone());

                let mut world = puzzle_world.clone();
                apply_factory_blocks(&mut world, self.factory_blocks);

                LoadedSave {
                    world,
                    puzzle_snapshot: Some(puzzle_world),
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
                    hotbar: hotbar
                        .or_else(|| Some(InventoryItems::for_mode(BuilderMode::Edit).hotbar)),
                }
            }
        }
    }

    fn puzzle_signature(&self) -> Vec<String> {
        let puzzle = self
            .puzzle
            .clone()
            .unwrap_or_else(|| self.legacy_puzzle_layer());
        let mut parts: Vec<String> = Vec::new();
        for saved in puzzle.blocks {
            parts.push(format!(
                "b:{},{},{}:{:?}:{:?}",
                saved.x, saved.y, saved.z, saved.data.kind, saved.data.facing
            ));
        }
        for saved in puzzle.system_blocks {
            parts.push(format!(
                "s:{},{},{}:{:?}:{:?}",
                saved.x, saved.y, saved.z, saved.data.kind, saved.data.facing
            ));
        }
        for saved in puzzle.block_settings {
            parts.push(format!(
                "bs:{},{},{}:{:?}",
                saved.x, saved.y, saved.z, saved.settings
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
            block_settings: self
                .block_settings
                .iter()
                .filter(|saved| self.legacy_system_block_is_persistent(saved.pos()))
                .cloned()
                .collect(),
            hotbar: self.hotbar,
        }
    }

    fn legacy_system_block_is_persistent(&self, pos: IVec3) -> bool {
        self.system_blocks.iter().any(|block| {
            block.pos() == pos
                && block.data.kind.persistent_layer() == Some(PersistentLayer::Puzzle)
        })
    }
}

fn write_save(name: &str, save: &SaveFile) -> bool {
    let dir = saves_directory().to_path_buf();
    if let Err(error) = fs::create_dir_all(&dir) {
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
        block_settings: world
            .block_settings
            .iter()
            .filter_map(|(pos, settings)| {
                world
                    .system_blocks
                    .get(pos)
                    .is_some_and(|block| {
                        block.kind.persistent_layer() == Some(PersistentLayer::Puzzle)
                    })
                    .then_some(SavedBlockSettings {
                        x: pos.x,
                        y: pos.y,
                        z: pos.z,
                        settings: settings.clone(),
                    })
            })
            .collect(),
        hotbar: Some(inventory.hotbar),
    }
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
    for saved in layer.blocks {
        world.insert(saved.pos(), saved.data);
    }
    for saved in layer.system_blocks {
        world.insert(saved.pos(), saved.data);
    }
    for saved in layer.block_settings {
        world.set_block_settings(saved.pos(), saved.settings);
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

impl SavedBlockSettings {
    fn pos(&self) -> IVec3 {
        IVec3::new(self.x, self.y, self.z)
    }
}

pub fn list_saves() -> Vec<String> {
    let dir = saves_directory();
    let Ok(entries) = fs::read_dir(dir) else {
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

/// Resolved save directory. Prefers `./saves` under cwd, then walks up from the
/// executable (covers `cargo run` launching from `target/debug/`).
pub fn saves_directory() -> &'static Path {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        let cwd_dir = PathBuf::from(SAVE_DIR);
        if cwd_dir.is_dir() {
            return cwd_dir
                .canonicalize()
                .unwrap_or(cwd_dir);
        }

        if let Ok(exe) = std::env::current_exe() {
            if let Some(exe_dir) = exe.parent() {
                for ancestor in exe_dir.ancestors().take(6) {
                    let candidate = ancestor.join(SAVE_DIR);
                    if candidate.is_dir() {
                        return candidate
                            .canonicalize()
                            .unwrap_or(candidate);
                    }
                }
            }
        }

        cwd_dir
    })
}

fn save_path(name: &str) -> PathBuf {
    saves_directory().join(format!("{}.ron", sanitize_save_name(name)))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::blocks::{BlockData, BlockKind, Facing, MaterialKind, StampColor};
    use crate::game::world::grid::{
        ConverterMode, ConverterSettings, GeneratorSettings, LabelerSettings, TeleportSettings,
    };

    #[test]
    fn puzzle_layer_round_trips_edit_system_blocks_and_settings() {
        let generator = IVec3::new(1, 1, 0);
        let goal = IVec3::new(2, 1, 0);
        let stamper = IVec3::new(3, 1, 0);
        let roller = IVec3::new(4, 1, 0);
        let converter = IVec3::new(5, 1, 0);
        let entrance = IVec3::new(6, 1, 0);
        let exit = IVec3::new(7, 1, 0);
        let generated_marker = IVec3::new(8, 1, 0);

        let mut world = WorldBlocks::default();
        for (pos, kind) in [
            (generator, BlockKind::Generator),
            (goal, BlockKind::Goal),
            (stamper, BlockKind::Stamper),
            (roller, BlockKind::Roller),
            (converter, BlockKind::Converter),
            (entrance, BlockKind::TeleportEntrance),
            (exit, BlockKind::TeleportExit),
            (generated_marker, BlockKind::WeldPoint),
        ] {
            world.insert(
                pos,
                BlockData {
                    kind,
                    facing: Facing::North,
                },
            );
        }

        world.set_generator_settings(
            generator,
            GeneratorSettings {
                period: 9,
                material: MaterialKind::Copper,
            },
        );
        world.set_labeler_settings(
            stamper,
            LabelerSettings {
                color: StampColor::Blue,
            },
        );
        world.set_labeler_settings(
            roller,
            LabelerSettings {
                color: StampColor::Yellow,
            },
        );
        world.set_converter_settings(
            converter,
            ConverterSettings {
                mode: ConverterMode::SpecificInput,
                input: MaterialKind::Iron,
                output: MaterialKind::Copper,
            },
        );
        world.set_teleport_settings(
            entrance,
            TeleportSettings {
                name: "Entrance".to_string(),
                pair: Some(exit),
            },
        );
        world.set_teleport_settings(
            exit,
            TeleportSettings {
                name: "Exit".to_string(),
                pair: Some(entrance),
            },
        );

        let inventory = InventoryItems::for_mode(BuilderMode::Edit);
        let loaded = SaveFile::puzzle(capture_puzzle_layer(&world, &inventory)).into_loaded();
        let round_trip = loaded.world;

        for pos in [generator, goal, stamper, roller, converter, entrance, exit] {
            assert!(
                round_trip.system_blocks.contains_key(&pos),
                "expected {pos:?} to be saved as a puzzle system block"
            );
        }
        assert!(!round_trip.system_blocks.contains_key(&generated_marker));

        assert_eq!(
            round_trip.generator_settings(generator),
            GeneratorSettings {
                period: 9,
                material: MaterialKind::Copper,
            }
        );
        assert_eq!(
            round_trip.labeler_settings(stamper),
            LabelerSettings {
                color: StampColor::Blue,
            }
        );
        assert_eq!(
            round_trip.labeler_settings(roller),
            LabelerSettings {
                color: StampColor::Yellow,
            }
        );
        assert_eq!(
            round_trip.converter_settings(converter),
            ConverterSettings {
                mode: ConverterMode::SpecificInput,
                input: MaterialKind::Iron,
                output: MaterialKind::Copper,
            }
        );
        assert_eq!(round_trip.teleport_settings(entrance).name, "Entrance");
        assert_eq!(round_trip.teleport_settings(entrance).pair, Some(exit));
        assert_eq!(round_trip.teleport_settings(exit).name, "Exit");
        assert_eq!(round_trip.teleport_settings(exit).pair, Some(entrance));
    }

    #[test]
    fn hotbar_round_trips_for_puzzle_and_solution() {
        let mut puzzle_inventory = InventoryItems::for_mode(BuilderMode::Edit);
        puzzle_inventory.set_hotbar_block(0, BlockKind::Stone);
        puzzle_inventory.set_hotbar_block(1, BlockKind::TeleportEntrance);

        let puzzle_loaded = SaveFile::puzzle(capture_puzzle_layer(
            &WorldBlocks::default(),
            &puzzle_inventory,
        ))
        .into_loaded();
        assert_eq!(puzzle_loaded.hotbar, Some(puzzle_inventory.hotbar));

        let mut solution_inventory = InventoryItems::for_mode(BuilderMode::Play);
        solution_inventory.set_hotbar_block(0, BlockKind::Platform);
        solution_inventory.set_hotbar_block(1, BlockKind::Pusher);
        solution_inventory.hotbar[2] = None;

        let solution_loaded = SaveFile::solution(
            capture_puzzle_layer(&WorldBlocks::default(), &puzzle_inventory),
            &WorldBlocks::default(),
            &solution_inventory,
        )
        .into_loaded();
        assert_eq!(solution_loaded.hotbar, Some(solution_inventory.hotbar));
    }
}
