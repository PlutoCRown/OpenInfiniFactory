use bevy::prelude::*;
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};

use crate::game::blocks::{BlockData, PersistentLayer};
use crate::game::world::grid::{BlockSettings, WorldBlocks};
use crate::game::{
    state::BuilderMode,
    ui::{HotbarItems, InventoryItems},
};
use crate::shared::persistent_storage::{self, SAVE_PREFIX};

#[derive(Resource, Default)]
pub struct SaveState {
    pub current: Option<String>,
    pub current_kind: Option<SaveKind>,
    pub entries: Vec<SaveEntry>,
    pub selected_puzzle: Option<String>,
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

    pub fn select_puzzle(&mut self, puzzle: Option<String>) {
        if self.selected_puzzle == puzzle {
            return;
        }
        self.selected_puzzle = puzzle;
        self.refresh_selected_puzzle_solutions();
    }

    pub fn selected_puzzle_solutions(&self) -> &[SaveEntry] {
        &self.selected_puzzle_solutions
    }

    fn refresh_selected_puzzle_solutions(&mut self) {
        self.selected_puzzle_solutions = match self.selected_puzzle.as_deref() {
            Some(puzzle) => self
                .entries
                .iter()
                .filter(|entry| {
                    entry.kind == SaveKind::Solution && entry.puzzle_id.as_deref() == Some(puzzle)
                })
                .cloned()
                .collect(),
            None => Vec::new(),
        };
    }
}

#[derive(Clone)]
pub struct SaveEntry {
    pub name: String,
    pub kind: SaveKind,
    pub puzzle_id: Option<String>,
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
    puzzle_id: Option<String>,
    #[serde(default)]
    factory_blocks: Vec<SavedBlock>,
    blocks: Vec<SavedBlock>,
    #[serde(default)]
    system_blocks: Vec<SavedBlock>,
    #[serde(default)]
    block_settings: Vec<SavedBlockSettings>,
    #[serde(default)]
    hotbar: Option<HotbarItems>,
    #[serde(default)]
    puzzle: Option<WorldLayer>,
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

pub fn save_puzzle(world: &WorldBlocks, name: &str, inventory: &InventoryItems) -> bool {
    write_save(
        name,
        &SaveFile::puzzle(capture_puzzle_layer(world, inventory)),
    )
}

pub fn save_solution(
    world: &WorldBlocks,
    name: &str,
    puzzle_id: &str,
    inventory: &InventoryItems,
) -> bool {
    write_save(
        name,
        &SaveFile::solution(puzzle_id, capture_factory_blocks(world), inventory),
    )
}

pub fn load_world(world: &mut WorldBlocks, name: &str) -> Option<LoadedSave> {
    let save = read_save(name)?;
    let loaded = save.into_loaded(name)?;
    *world = loaded.world.clone();
    Some(loaded)
}

pub fn save_kind(name: &str) -> Option<SaveKind> {
    let save = read_save(name)?;
    Some(match save.kind {
        SaveFileKind::Solution => SaveKind::Solution,
        SaveFileKind::Puzzle | SaveFileKind::Legacy => SaveKind::Puzzle,
    })
}

pub fn puzzle_id_for_solution(name: &str) -> Option<String> {
    let save = read_save(name)?;
    if !matches!(save.kind, SaveFileKind::Solution) {
        return None;
    }
    save.stored_puzzle_id()
        .or_else(|| legacy_puzzle_id_from_embedded(&save))
}

fn legacy_puzzle_id_from_embedded(save: &SaveFile) -> Option<String> {
    let embedded = save.embedded_puzzle_layer()?;
    let signature = puzzle_layer_signature(&embedded);
    list_saves()
        .into_iter()
        .filter(|name| save_kind(name) == Some(SaveKind::Puzzle))
        .find(|name| {
            read_save(name)
                .map(|puzzle_save| puzzle_layer_signature(&puzzle_save.puzzle_layer()) == signature)
                .unwrap_or(false)
        })
}

pub fn has_solutions_for_puzzle(puzzle_id: &str) -> bool {
    list_save_entries()
        .into_iter()
        .any(|entry| entry.puzzle_id.as_deref() == Some(puzzle_id))
}

pub fn invalidate_solutions_for_puzzle(puzzle_id: &str) -> usize {
    let solutions: Vec<String> = list_save_entries()
        .into_iter()
        .filter(|entry| entry.puzzle_id.as_deref() == Some(puzzle_id))
        .map(|entry| entry.name)
        .collect();
    solutions.iter().filter(|name| delete_save(name)).count()
}

pub fn delete_save(name: &str) -> bool {
    persistent_storage::remove(&save_storage_key(name))
}

pub fn rename_save(old_name: &str, new_name: &str) -> bool {
    let new_name = normalized_save_name(new_name);
    if new_name.is_empty() || old_name == new_name {
        return false;
    }
    if persistent_storage::exists(&save_storage_key(&new_name)) {
        warn!("Cannot rename save {old_name} to {new_name}: target already exists");
        return false;
    }
    let Some(contents) = persistent_storage::read(&save_storage_key(old_name)) else {
        warn!("Cannot rename save {old_name}: source missing");
        return false;
    };
    if !persistent_storage::write(&save_storage_key(&new_name), &contents) {
        return false;
    }
    if !persistent_storage::remove(&save_storage_key(old_name)) {
        return false;
    }
    if save_kind(&new_name) == Some(SaveKind::Puzzle) {
        update_solution_puzzle_ids(old_name, &new_name);
    }
    true
}

fn update_solution_puzzle_ids(old_puzzle_id: &str, new_puzzle_id: &str) {
    let solutions: Vec<String> = list_save_entries()
        .into_iter()
        .filter(|entry| entry.puzzle_id.as_deref() == Some(old_puzzle_id))
        .map(|entry| entry.name)
        .collect();
    for name in solutions {
        let Some(mut save) = read_save(&name) else {
            continue;
        };
        if !matches!(save.kind, SaveFileKind::Solution) {
            continue;
        }
        save.puzzle_id = Some(new_puzzle_id.to_string());
        write_save(&name, &save);
    }
}

pub fn reset_solution_world(world: &mut WorldBlocks, puzzle_snapshot: &WorldBlocks) {
    *world = puzzle_snapshot.clone();
}

#[derive(Clone)]
pub struct LoadedSave {
    pub world: WorldBlocks,
    pub puzzle_snapshot: Option<WorldBlocks>,
    pub puzzle_id: Option<String>,
    pub hotbar: Option<HotbarItems>,
}

impl SaveFile {
    fn puzzle(puzzle: WorldLayer) -> Self {
        Self {
            kind: SaveFileKind::Puzzle,
            puzzle_id: None,
            factory_blocks: Vec::new(),
            blocks: puzzle.blocks,
            system_blocks: puzzle.system_blocks,
            block_settings: puzzle.block_settings,
            hotbar: puzzle.hotbar,
            puzzle: None,
        }
    }

    fn solution(
        puzzle_id: &str,
        factory_blocks: Vec<SavedBlock>,
        inventory: &InventoryItems,
    ) -> Self {
        Self {
            kind: SaveFileKind::Solution,
            puzzle_id: Some(puzzle_id.to_string()),
            factory_blocks,
            blocks: Vec::new(),
            system_blocks: Vec::new(),
            block_settings: Vec::new(),
            hotbar: Some(inventory.hotbar),
            puzzle: None,
        }
    }

    fn into_loaded(self, _name: &str) -> Option<LoadedSave> {
        match self.kind {
            SaveFileKind::Solution => {
                let puzzle_id = self.resolved_puzzle_id()?;
                let hotbar = self
                    .hotbar
                    .or_else(|| Some(InventoryItems::for_mode(BuilderMode::Play).hotbar));
                let embedded_puzzle = self.embedded_puzzle_layer();
                let puzzle_world = load_puzzle_world(&puzzle_id).or_else(|| {
                    embedded_puzzle.as_ref().map(|layer| {
                        let mut world = WorldBlocks::default();
                        apply_layer(&mut world, layer.clone());
                        world
                    })
                })?;
                let mut world = puzzle_world.clone();
                apply_factory_blocks(&mut world, self.factory_blocks);
                Some(LoadedSave {
                    world,
                    puzzle_snapshot: Some(puzzle_world),
                    puzzle_id: Some(puzzle_id),
                    hotbar,
                })
            }
            SaveFileKind::Puzzle | SaveFileKind::Legacy => {
                let puzzle = self.puzzle_layer();
                let hotbar = self.hotbar.or(puzzle.hotbar);
                let mut world = WorldBlocks::default();
                apply_layer(&mut world, puzzle);
                Some(LoadedSave {
                    world,
                    puzzle_snapshot: None,
                    puzzle_id: None,
                    hotbar: hotbar
                        .or_else(|| Some(InventoryItems::for_mode(BuilderMode::Edit).hotbar)),
                })
            }
        }
    }

    fn resolved_puzzle_id(&self) -> Option<String> {
        self.stored_puzzle_id()
            .or_else(|| legacy_puzzle_id_from_embedded(self))
    }

    fn stored_puzzle_id(&self) -> Option<String> {
        self.puzzle_id.clone().filter(|id| !id.is_empty())
    }

    fn puzzle_layer(&self) -> WorldLayer {
        if let Some(puzzle) = self.puzzle.clone() {
            return puzzle;
        }
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

    fn embedded_puzzle_layer(&self) -> Option<WorldLayer> {
        self.puzzle.clone().or_else(|| {
            let layer = self.puzzle_layer();
            if layer.blocks.is_empty()
                && layer.system_blocks.is_empty()
                && layer.block_settings.is_empty()
            {
                None
            } else {
                Some(layer)
            }
        })
    }

    fn legacy_system_block_is_persistent(&self, pos: IVec3) -> bool {
        self.system_blocks.iter().any(|block| {
            block.pos() == pos
                && block.data.kind.persistent_layer() == Some(PersistentLayer::Puzzle)
        })
    }
}

fn load_puzzle_world(puzzle_id: &str) -> Option<WorldBlocks> {
    let save = read_save(puzzle_id)?;
    if !matches!(save.kind, SaveFileKind::Puzzle | SaveFileKind::Legacy) {
        return None;
    }
    let mut world = WorldBlocks::default();
    apply_layer(&mut world, save.puzzle_layer());
    Some(world)
}

fn puzzle_layer_signature(layer: &WorldLayer) -> Vec<String> {
    let mut parts: Vec<String> = Vec::new();
    for saved in &layer.blocks {
        parts.push(format!(
            "b:{},{},{}:{:?}:{:?}",
            saved.x, saved.y, saved.z, saved.data.kind, saved.data.facing
        ));
    }
    for saved in &layer.system_blocks {
        parts.push(format!(
            "s:{},{},{}:{:?}:{:?}",
            saved.x, saved.y, saved.z, saved.data.kind, saved.data.facing
        ));
    }
    for saved in &layer.block_settings {
        parts.push(format!(
            "bs:{},{},{}:{:?}",
            saved.x, saved.y, saved.z, saved.settings
        ));
    }
    parts.sort();
    parts
}

fn write_save(name: &str, save: &SaveFile) -> bool {
    match ron::ser::to_string_pretty(save, PrettyConfig::default()) {
        Ok(serialized) => persistent_storage::write(&save_storage_key(name), &serialized),
        Err(error) => {
            warn!("Failed to serialize world: {error}");
            false
        }
    }
}

fn read_save(name: &str) -> Option<SaveFile> {
    let contents = persistent_storage::read(&save_storage_key(name))?;
    ron::from_str::<SaveFile>(&contents).ok()
}

fn save_storage_key(name: &str) -> String {
    persistent_storage::save_storage_key(&sanitize_save_name(name))
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
    persistent_storage::list_under_prefix(SAVE_PREFIX)
}

pub fn list_save_entries() -> Vec<SaveEntry> {
    let mut entries: Vec<SaveEntry> = list_saves()
        .into_iter()
        .filter_map(|name| {
            let kind = save_kind(&name)?;
            let puzzle_id = if kind == SaveKind::Solution {
                puzzle_id_for_solution(&name)
            } else {
                None
            };
            Some(SaveEntry {
                name,
                kind,
                puzzle_id,
            })
        })
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
        let loaded = SaveFile::puzzle(capture_puzzle_layer(&world, &inventory))
            .into_loaded("puzzle")
            .unwrap();
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
        .into_loaded("puzzle")
        .unwrap();
        assert_eq!(puzzle_loaded.hotbar, Some(puzzle_inventory.hotbar));

        let mut solution_inventory = InventoryItems::for_mode(BuilderMode::Play);
        solution_inventory.set_hotbar_block(0, BlockKind::Platform);
        solution_inventory.set_hotbar_block(1, BlockKind::Pusher);
        solution_inventory.hotbar[2] = None;

        let solution_file = SaveFile::solution("puzzle", Vec::new(), &solution_inventory);
        assert_eq!(solution_file.puzzle_id.as_deref(), Some("puzzle"));
        assert!(solution_file.blocks.is_empty());
        assert!(solution_file.system_blocks.is_empty());
        assert!(solution_file.puzzle.is_none());
    }

    #[test]
    fn solution_loads_factory_blocks_from_puzzle_reference() {
        let mut puzzle_world = WorldBlocks::default();
        puzzle_world.insert(
            IVec3::ZERO,
            BlockData {
                kind: BlockKind::Stone,
                facing: Facing::North,
            },
        );
        let puzzle_inventory = InventoryItems::for_mode(BuilderMode::Edit);
        let puzzle_save = SaveFile::puzzle(capture_puzzle_layer(&puzzle_world, &puzzle_inventory));
        write_save("test_puzzle_ref", &puzzle_save);

        let mut solution_world = puzzle_world.clone();
        solution_world.insert(
            IVec3::X,
            BlockData {
                kind: BlockKind::Platform,
                facing: Facing::North,
            },
        );
        let solution_inventory = InventoryItems::for_mode(BuilderMode::Play);
        write_save(
            "test_solution_ref",
            &SaveFile::solution(
                "test_puzzle_ref",
                capture_factory_blocks(&solution_world),
                &solution_inventory,
            ),
        );

        let mut loaded_world = WorldBlocks::default();
        let loaded = load_world(&mut loaded_world, "test_solution_ref").unwrap();
        assert_eq!(loaded.puzzle_id.as_deref(), Some("test_puzzle_ref"));
        assert!(loaded_world.blocks.contains_key(&IVec3::ZERO));
        assert!(loaded_world.blocks.contains_key(&IVec3::X));
        assert!(!loaded_world.blocks.contains_key(&IVec3::NEG_X));

        delete_save("test_puzzle_ref");
        delete_save("test_solution_ref");
    }
}
