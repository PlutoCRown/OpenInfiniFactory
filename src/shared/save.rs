use bevy::prelude::*;
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::game::world::blocks::{BlockData, BlockKind};
use crate::game::world::grid::{
    ConverterSettings, GeneratorSettings, LabelerSettings, TeleportSettings, WorldBlocks,
};

pub const SAVE_DIR: &str = "saves";
pub const SAVE_SLOTS: usize = 8;

#[derive(Resource, Default)]
pub struct SaveState {
    pub current: Option<String>,
    pub current_kind: Option<SaveKind>,
    pub slots: Vec<String>,
}

impl SaveState {
    pub fn refresh(&mut self) {
        self.slots = list_saves();
    }
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
    generator_settings: Vec<SavedGeneratorSettings>,
    #[serde(default)]
    labeler_settings: Vec<SavedLabelerSettings>,
    #[serde(default)]
    converter_settings: Vec<SavedConverterSettings>,
    #[serde(default)]
    teleport_settings: Vec<SavedTeleportSettings>,
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
    generator_settings: Vec<SavedGeneratorSettings>,
    #[serde(default)]
    labeler_settings: Vec<SavedLabelerSettings>,
    #[serde(default)]
    converter_settings: Vec<SavedConverterSettings>,
    #[serde(default)]
    teleport_settings: Vec<SavedTeleportSettings>,
}

#[derive(Clone, Serialize, Deserialize)]
struct SavedBlock {
    x: i32,
    y: i32,
    z: i32,
    data: BlockData,
}

#[derive(Clone, Serialize, Deserialize)]
struct SavedGeneratorSettings {
    x: i32,
    y: i32,
    z: i32,
    settings: GeneratorSettings,
}

#[derive(Clone, Serialize, Deserialize)]
struct SavedLabelerSettings {
    x: i32,
    y: i32,
    z: i32,
    settings: LabelerSettings,
}

#[derive(Clone, Serialize, Deserialize)]
struct SavedConverterSettings {
    x: i32,
    y: i32,
    z: i32,
    settings: ConverterSettings,
}

#[derive(Clone, Serialize, Deserialize)]
struct SavedTeleportSettings {
    x: i32,
    y: i32,
    z: i32,
    settings: TeleportSettings,
}

pub fn save_world(world: &WorldBlocks, name: &str, kind: SaveKind) -> bool {
    let save = match kind {
        SaveKind::Puzzle => SaveFile::puzzle(capture_puzzle_layer(world)),
        SaveKind::Solution => SaveFile::solution(capture_puzzle_layer(world), world),
    };

    write_save(name, &save)
}

pub fn save_solution_with_puzzle(
    world: &WorldBlocks,
    name: &str,
    puzzle_snapshot: &WorldBlocks,
) -> bool {
    write_save(
        name,
        &SaveFile::solution(capture_puzzle_layer(puzzle_snapshot), world),
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

pub fn reset_solution_world(world: &mut WorldBlocks, puzzle_snapshot: &WorldBlocks) {
    *world = puzzle_snapshot.clone();
}

#[derive(Clone)]
pub struct LoadedSave {
    pub kind: SaveKind,
    pub world: WorldBlocks,
    pub puzzle_snapshot: Option<WorldBlocks>,
}

impl SaveFile {
    fn puzzle(puzzle: WorldLayer) -> Self {
        Self {
            kind: SaveFileKind::Puzzle,
            puzzle: Some(puzzle.clone()),
            factory_blocks: Vec::new(),
            blocks: puzzle.blocks,
            system_blocks: puzzle.system_blocks,
            generator_settings: puzzle.generator_settings,
            labeler_settings: puzzle.labeler_settings,
            converter_settings: puzzle.converter_settings,
            teleport_settings: puzzle.teleport_settings,
        }
    }

    fn solution(puzzle: WorldLayer, world: &WorldBlocks) -> Self {
        Self {
            kind: SaveFileKind::Solution,
            puzzle: Some(puzzle.clone()),
            factory_blocks: capture_factory_blocks(world),
            blocks: puzzle.blocks,
            system_blocks: puzzle.system_blocks,
            generator_settings: puzzle.generator_settings,
            labeler_settings: puzzle.labeler_settings,
            converter_settings: puzzle.converter_settings,
            teleport_settings: puzzle.teleport_settings,
        }
    }

    fn into_loaded(self) -> LoadedSave {
        match self.kind {
            SaveFileKind::Solution => {
                let puzzle = self
                    .puzzle
                    .clone()
                    .unwrap_or_else(|| self.legacy_puzzle_layer());
                let mut puzzle_world = WorldBlocks::default();
                apply_layer(&mut puzzle_world, puzzle.clone());

                let mut world = puzzle_world.clone();
                apply_factory_blocks(&mut world, self.factory_blocks);

                LoadedSave {
                    kind: SaveKind::Solution,
                    world,
                    puzzle_snapshot: Some(puzzle_world),
                }
            }
            SaveFileKind::Puzzle | SaveFileKind::Legacy => {
                let puzzle = self
                    .puzzle
                    .clone()
                    .unwrap_or_else(|| self.legacy_puzzle_layer());
                let mut world = WorldBlocks::default();
                apply_layer(&mut world, puzzle);
                LoadedSave {
                    kind: SaveKind::Puzzle,
                    world,
                    puzzle_snapshot: None,
                }
            }
        }
    }

    fn legacy_puzzle_layer(&self) -> WorldLayer {
        WorldLayer {
            blocks: self
                .blocks
                .iter()
                .filter(|saved| is_puzzle_block(saved.data.kind))
                .cloned()
                .collect(),
            system_blocks: self
                .system_blocks
                .iter()
                .filter(|saved| is_puzzle_block(saved.data.kind))
                .cloned()
                .collect(),
            generator_settings: self
                .generator_settings
                .iter()
                .filter(|saved| {
                    self.system_blocks
                        .iter()
                        .any(|block| block.pos() == saved.pos() && is_puzzle_block(block.data.kind))
                })
                .cloned()
                .collect(),
            labeler_settings: self
                .labeler_settings
                .iter()
                .filter(|saved| {
                    self.system_blocks
                        .iter()
                        .any(|block| block.pos() == saved.pos() && is_puzzle_block(block.data.kind))
                })
                .cloned()
                .collect(),
            converter_settings: self
                .converter_settings
                .iter()
                .filter(|saved| {
                    self.system_blocks
                        .iter()
                        .any(|block| block.pos() == saved.pos() && is_puzzle_block(block.data.kind))
                })
                .cloned()
                .collect(),
            teleport_settings: self
                .teleport_settings
                .iter()
                .filter(|saved| {
                    self.system_blocks
                        .iter()
                        .any(|block| block.pos() == saved.pos() && is_puzzle_block(block.data.kind))
                })
                .cloned()
                .collect(),
        }
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

fn capture_puzzle_layer(world: &WorldBlocks) -> WorldLayer {
    let blocks: Vec<SavedBlock> = world
        .blocks
        .iter()
        .filter_map(|(pos, data)| is_puzzle_block(data.kind).then_some(saved_block(*pos, *data)))
        .collect();
    let system_blocks: Vec<SavedBlock> = world
        .system_blocks
        .iter()
        .filter_map(|(pos, data)| is_puzzle_block(data.kind).then_some(saved_block(*pos, *data)))
        .collect();

    WorldLayer {
        blocks,
        system_blocks,
        generator_settings: world
            .generator_settings
            .iter()
            .filter_map(|(pos, settings)| {
                world
                    .system_blocks
                    .get(pos)
                    .is_some_and(|block| is_puzzle_block(block.kind))
                    .then_some(SavedGeneratorSettings {
                        x: pos.x,
                        y: pos.y,
                        z: pos.z,
                        settings: *settings,
                    })
            })
            .collect(),
        labeler_settings: world
            .labeler_settings
            .iter()
            .filter_map(|(pos, settings)| {
                world
                    .system_blocks
                    .get(pos)
                    .is_some_and(|block| is_puzzle_block(block.kind))
                    .then_some(SavedLabelerSettings {
                        x: pos.x,
                        y: pos.y,
                        z: pos.z,
                        settings: *settings,
                    })
            })
            .collect(),
        converter_settings: world
            .converter_settings
            .iter()
            .filter_map(|(pos, settings)| {
                world
                    .system_blocks
                    .get(pos)
                    .is_some_and(|block| is_puzzle_block(block.kind))
                    .then_some(SavedConverterSettings {
                        x: pos.x,
                        y: pos.y,
                        z: pos.z,
                        settings: *settings,
                    })
            })
            .collect(),
        teleport_settings: world
            .teleport_settings
            .iter()
            .filter_map(|(pos, settings)| {
                world
                    .system_blocks
                    .get(pos)
                    .is_some_and(|block| is_puzzle_block(block.kind))
                    .then_some(SavedTeleportSettings {
                        x: pos.x,
                        y: pos.y,
                        z: pos.z,
                        settings: settings.clone(),
                    })
            })
            .collect(),
    }
}

fn capture_factory_blocks(world: &WorldBlocks) -> Vec<SavedBlock> {
    world
        .blocks
        .iter()
        .filter_map(|(pos, data)| data.kind.is_factory().then_some(saved_block(*pos, *data)))
        .collect()
}

fn apply_layer(world: &mut WorldBlocks, layer: WorldLayer) {
    for saved in layer.blocks {
        world.insert(saved.pos(), saved.data);
    }
    for saved in layer.system_blocks {
        world.insert(saved.pos(), saved.data);
    }
    for saved in layer.generator_settings {
        world.set_generator_settings(IVec3::new(saved.x, saved.y, saved.z), saved.settings);
    }
    for saved in layer.labeler_settings {
        world.set_labeler_settings(IVec3::new(saved.x, saved.y, saved.z), saved.settings);
    }
    for saved in layer.converter_settings {
        world.set_converter_settings(IVec3::new(saved.x, saved.y, saved.z), saved.settings);
    }
    for saved in layer.teleport_settings {
        world.set_teleport_settings(IVec3::new(saved.x, saved.y, saved.z), saved.settings);
    }
}

fn apply_factory_blocks(world: &mut WorldBlocks, factory_blocks: Vec<SavedBlock>) {
    for saved in factory_blocks {
        if saved.data.kind.is_factory() {
            world.insert(saved.pos(), saved.data);
        }
    }
}

fn is_puzzle_block(kind: BlockKind) -> bool {
    kind.is_scene() || kind.is_system_layer()
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

impl SavedGeneratorSettings {
    fn pos(&self) -> IVec3 {
        IVec3::new(self.x, self.y, self.z)
    }
}

impl SavedLabelerSettings {
    fn pos(&self) -> IVec3 {
        IVec3::new(self.x, self.y, self.z)
    }
}

impl SavedConverterSettings {
    fn pos(&self) -> IVec3 {
        IVec3::new(self.x, self.y, self.z)
    }
}

impl SavedTeleportSettings {
    fn pos(&self) -> IVec3 {
        IVec3::new(self.x, self.y, self.z)
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

pub fn next_world_name(existing: &[String]) -> String {
    for index in 1.. {
        let candidate = format!("world_{index}");
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
