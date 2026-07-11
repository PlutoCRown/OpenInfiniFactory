use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::game::blocks::{BlockData, PersistentLayer};
use crate::game::world::grid::{BlockSettings, StoredAcceptorStructure, WorldBlocks};
use crate::game::{
    state::BuilderMode,
    ui::{HotbarItems, InventoryItems},
};
use crate::shared::persistent_storage;
use crate::shared::save_format::{
    self, SavedBlock, SaveBlocksData, BLOCKS_FILE, META_FILE,
};

const SAVE_VERSION: u32 = 1;

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

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum SaveMetaKind {
    Puzzle,
    Solution,
}

#[derive(Serialize, Deserialize)]
struct SaveMeta {
    version: u32,
    kind: SaveMetaKind,
    #[serde(default)]
    puzzle_id: Option<String>,
    #[serde(default)]
    hotbar: Option<HotbarItems>,
    #[serde(default)]
    player: Option<PlayerSave>,
    #[serde(default)]
    next_acceptor_id: u64,
    #[serde(default)]
    acceptor_structures: Vec<StoredAcceptorStructure>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlayerSave {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub flying: bool,
}

struct SaveFile {
    meta: SaveMeta,
    blocks: SaveBlocksData,
}

pub fn save_puzzle(
    world: &WorldBlocks,
    name: &str,
    inventory: &InventoryItems,
    player: Option<PlayerSave>,
) -> bool {
    write_save(
        name,
        &SaveFile::puzzle(capture_puzzle_layer(world, inventory), player),
    )
}

pub fn save_solution(
    world: &WorldBlocks,
    name: &str,
    puzzle_id: &str,
    inventory: &InventoryItems,
    player: Option<PlayerSave>,
) -> bool {
    write_save(
        name,
        &SaveFile::solution(
            puzzle_id,
            capture_factory_blocks(world),
            inventory,
            player,
        ),
    )
}

pub fn load_world(world: &mut WorldBlocks, name: &str) -> Option<LoadedSave> {
    let save = read_save(name)?;
    let loaded = save.into_loaded()?;
    *world = loaded.world.clone();
    Some(loaded)
}

pub fn save_kind(name: &str) -> Option<SaveKind> {
    let save = read_save(name)?;
    Some(save.meta_kind())
}

pub fn puzzle_id_for_solution(name: &str) -> Option<String> {
    let save = read_save(name)?;
    if !matches!(save.meta.kind, SaveMetaKind::Solution) {
        return None;
    }
    save.meta.puzzle_id.clone().filter(|id| !id.is_empty())
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
    persistent_storage::remove_save_folder(&sanitize_save_name(name))
}

pub fn rename_save(old_name: &str, new_name: &str) -> bool {
    let old_name = sanitize_save_name(old_name);
    let new_name = normalized_save_name(new_name);
    if new_name.is_empty() || old_name == new_name {
        return false;
    }
    if persistent_storage::save_exists(&new_name) {
        warn!("Cannot rename save {old_name} to {new_name}: target already exists");
        return false;
    }
    if !persistent_storage::save_exists(&old_name) {
        warn!("Cannot rename save {old_name}: source missing");
        return false;
    }
    if !persistent_storage::rename_save_folder(&old_name, &new_name) {
        return false;
    }
    if save_kind(&new_name) == Some(SaveKind::Puzzle) {
        update_solution_puzzle_ids(&old_name, &new_name);
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
        if !matches!(save.meta.kind, SaveMetaKind::Solution) {
            continue;
        }
        save.meta.puzzle_id = Some(new_puzzle_id.to_string());
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
    pub player: Option<PlayerSave>,
}

impl SaveFile {
    fn puzzle(layer: PuzzleLayer, player: Option<PlayerSave>) -> Self {
        Self {
            meta: SaveMeta {
                version: SAVE_VERSION,
                kind: SaveMetaKind::Puzzle,
                puzzle_id: None,
                hotbar: layer.hotbar,
                player,
                next_acceptor_id: layer.next_acceptor_id,
                acceptor_structures: layer.acceptor_structures,
            },
            blocks: SaveBlocksData {
                scene_blocks: layer.scene_blocks,
                system_blocks: layer.system_blocks,
                factory_blocks: Vec::new(),
            },
        }
    }

    fn solution(
        puzzle_id: &str,
        factory_blocks: Vec<SavedBlock>,
        inventory: &InventoryItems,
        player: Option<PlayerSave>,
    ) -> Self {
        Self {
            meta: SaveMeta {
                version: SAVE_VERSION,
                kind: SaveMetaKind::Solution,
                puzzle_id: Some(puzzle_id.to_string()),
                hotbar: Some(inventory.hotbar),
                player,
                next_acceptor_id: 0,
                acceptor_structures: Vec::new(),
            },
            blocks: SaveBlocksData {
                factory_blocks,
                ..Default::default()
            },
        }
    }

    fn meta_kind(&self) -> SaveKind {
        match self.meta.kind {
            SaveMetaKind::Solution => SaveKind::Solution,
            SaveMetaKind::Puzzle => SaveKind::Puzzle,
        }
    }

    fn into_loaded(self) -> Option<LoadedSave> {
        match self.meta.kind {
            SaveMetaKind::Solution => {
                let puzzle_id = self.meta.puzzle_id.filter(|id| !id.is_empty())?;
                let hotbar = self
                    .meta
                    .hotbar
                    .or_else(|| Some(InventoryItems::for_mode(BuilderMode::Play).hotbar));
                let puzzle_world = load_puzzle_world(&puzzle_id)?;
                let mut world = puzzle_world.clone();
                apply_factory_blocks(&mut world, self.blocks.factory_blocks);
                Some(LoadedSave {
                    world,
                    puzzle_snapshot: Some(puzzle_world),
                    puzzle_id: Some(puzzle_id),
                    hotbar,
                    player: self.meta.player,
                })
            }
            SaveMetaKind::Puzzle => {
                let layer = PuzzleLayer::from_blocks(
                    self.blocks,
                    self.meta.hotbar,
                    self.meta.next_acceptor_id,
                    self.meta.acceptor_structures,
                );
                let hotbar = layer
                    .hotbar
                    .or_else(|| Some(InventoryItems::for_mode(BuilderMode::Edit).hotbar));
                let mut world = WorldBlocks::default();
                apply_layer(&mut world, layer);
                Some(LoadedSave {
                    world,
                    puzzle_snapshot: None,
                    puzzle_id: None,
                    hotbar,
                    player: self.meta.player,
                })
            }
        }
    }
}

struct PuzzleLayer {
    scene_blocks: Vec<SavedBlock>,
    system_blocks: Vec<SavedBlock>,
    next_acceptor_id: u64,
    acceptor_structures: Vec<StoredAcceptorStructure>,
    hotbar: Option<HotbarItems>,
}

impl PuzzleLayer {
    fn from_blocks(
        blocks: SaveBlocksData,
        hotbar: Option<HotbarItems>,
        next_acceptor_id: u64,
        acceptor_structures: Vec<StoredAcceptorStructure>,
    ) -> Self {
        Self {
            scene_blocks: blocks.scene_blocks,
            system_blocks: blocks.system_blocks,
            next_acceptor_id,
            acceptor_structures,
            hotbar,
        }
    }
}

fn load_puzzle_world(puzzle_id: &str) -> Option<WorldBlocks> {
    let save = read_save(puzzle_id)?;
    if !matches!(save.meta.kind, SaveMetaKind::Puzzle) {
        return None;
    }
    let mut world = WorldBlocks::default();
    apply_layer(
        &mut world,
        PuzzleLayer::from_blocks(
            save.blocks,
            save.meta.hotbar,
            save.meta.next_acceptor_id,
            save.meta.acceptor_structures,
        ),
    );
    Some(world)
}

fn write_save(name: &str, save: &SaveFile) -> bool {
    let name = sanitize_save_name(name);
    let meta = match serde_json::to_string_pretty(&save.meta) {
        Ok(serialized) => serialized,
        Err(error) => {
            warn!("Failed to serialize save meta: {error}");
            return false;
        }
    };
    if !persistent_storage::write_save_text(&name, META_FILE, &meta) {
        return false;
    }
    let blocks = save_format::encode_blocks(&save.blocks);
    persistent_storage::write_save_bytes(&name, BLOCKS_FILE, &blocks)
}

fn read_save(name: &str) -> Option<SaveFile> {
    let name = sanitize_save_name(name);
    let meta_text = persistent_storage::read_save_text(&name, META_FILE)?;
    let meta = serde_json::from_str::<SaveMeta>(&meta_text).ok()?;
    let blocks_bytes = persistent_storage::read_save_bytes(&name, BLOCKS_FILE)?;
    let blocks = save_format::decode_blocks(&blocks_bytes).ok()?;
    Some(SaveFile { meta, blocks })
}

fn capture_puzzle_layer(world: &WorldBlocks, inventory: &InventoryItems) -> PuzzleLayer {
    let scene_blocks: Vec<SavedBlock> = world
        .blocks
        .iter()
        .filter_map(|(pos, data)| {
            (data.kind.persistent_layer() == Some(PersistentLayer::Puzzle))
                .then_some(saved_block(*pos, *data, None))
        })
        .collect();
    let system_blocks: Vec<SavedBlock> = world
        .system_blocks
        .iter()
        .filter_map(|(pos, data)| {
            (data.kind.persistent_layer() == Some(PersistentLayer::Puzzle)).then_some({
                let settings = world.block_settings.get(pos).cloned();
                saved_block(*pos, *data, settings)
            })
        })
        .collect();

    PuzzleLayer {
        scene_blocks,
        system_blocks,
        next_acceptor_id: world.next_acceptor_id,
        acceptor_structures: world.acceptor_structures.clone(),
        hotbar: Some(inventory.hotbar),
    }
}

fn capture_factory_blocks(world: &WorldBlocks) -> Vec<SavedBlock> {
    world
        .blocks
        .iter()
        .filter_map(|(pos, data)| {
            (data.kind.persistent_layer() == Some(PersistentLayer::SolutionFactory))
                .then_some(saved_block(*pos, *data, None))
        })
        .collect()
}

fn apply_layer(world: &mut WorldBlocks, layer: PuzzleLayer) {
    for saved in layer.scene_blocks {
        world.insert(saved.pos(), saved.to_block_data());
    }
    for saved in layer.system_blocks {
        world.insert(saved.pos(), saved.to_block_data());
        if let Some(settings) = &saved.settings {
            world.set_block_settings(saved.pos(), settings.clone());
        }
    }
    world.restore_acceptor_structures(layer.next_acceptor_id, layer.acceptor_structures);
}

fn apply_factory_blocks(world: &mut WorldBlocks, factory_blocks: Vec<SavedBlock>) {
    for saved in factory_blocks {
        if saved.kind.persistent_layer() == Some(PersistentLayer::SolutionFactory) {
            world.insert(saved.pos(), saved.to_block_data());
        }
    }
}

fn saved_block(pos: IVec3, data: BlockData, settings: Option<BlockSettings>) -> SavedBlock {
    let mut saved = SavedBlock::from_block_data(pos, data);
    saved.settings = settings;
    saved
}

pub fn list_saves() -> Vec<String> {
    persistent_storage::list_saves()
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
    use crate::game::blocks::{BlockKind, Facing, MaterialKind, StampColor};
    use crate::game::world::grid::{
        ConverterMode, ConverterSettings, GeneratorMode, GeneratorSettings, LabelerSettings,
        TeleportSettings,
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
            world.insert(pos, BlockData::new(kind, Facing::North));
        }

        world.set_generator_settings(
            generator,
            GeneratorSettings {
                mode: GeneratorMode::Period {
                    period: 9,
                    offset: 0,
                },
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
        let loaded = SaveFile::puzzle(capture_puzzle_layer(&world, &inventory), None)
            .into_loaded()
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
                mode: GeneratorMode::Period {
                    period: 9,
                    offset: 0,
                },
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

        let puzzle_loaded = SaveFile::puzzle(
            capture_puzzle_layer(&WorldBlocks::default(), &puzzle_inventory),
            None,
        )
        .into_loaded()
        .unwrap();
        assert_eq!(puzzle_loaded.hotbar, Some(puzzle_inventory.hotbar));

        let mut solution_inventory = InventoryItems::for_mode(BuilderMode::Play);
        solution_inventory.set_hotbar_block(0, BlockKind::Platform);
        solution_inventory.set_hotbar_block(1, BlockKind::Pusher);
        solution_inventory.hotbar[2] = None;

        let solution_file = SaveFile::solution("puzzle", Vec::new(), &solution_inventory, None);
        assert_eq!(solution_file.meta.puzzle_id.as_deref(), Some("puzzle"));
        assert!(solution_file.blocks.scene_blocks.is_empty());
        assert!(solution_file.blocks.system_blocks.is_empty());
    }

    #[test]
    fn solution_loads_factory_blocks_from_puzzle_reference() {
        let mut puzzle_world = WorldBlocks::default();
        puzzle_world.insert(IVec3::ZERO, BlockData::new(BlockKind::Stone, Facing::North));
        let puzzle_inventory = InventoryItems::for_mode(BuilderMode::Edit);
        let puzzle_save = SaveFile::puzzle(
            capture_puzzle_layer(&puzzle_world, &puzzle_inventory),
            None,
        );
        write_save("test_puzzle_ref", &puzzle_save);

        let mut solution_world = puzzle_world.clone();
        solution_world.insert(IVec3::X, BlockData::new(BlockKind::Platform, Facing::North));
        let solution_inventory = InventoryItems::for_mode(BuilderMode::Play);
        write_save(
            "test_solution_ref",
            &SaveFile::solution(
                "test_puzzle_ref",
                capture_factory_blocks(&solution_world),
                &solution_inventory,
                None,
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

    #[test]
    fn player_pose_round_trips_through_meta() {
        let player = PlayerSave {
            x: 1.5,
            y: 2.8,
            z: 3.5,
            yaw: 1.2,
            pitch: -0.3,
            flying: true,
        };
        let inventory = InventoryItems::for_mode(BuilderMode::Edit);
        write_save(
            "test_player_pose",
            &SaveFile::puzzle(capture_puzzle_layer(&WorldBlocks::default(), &inventory), Some(player.clone())),
        );
        let loaded = read_save("test_player_pose").unwrap();
        assert_eq!(loaded.meta.player, Some(player));
        delete_save("test_player_pose");
    }

    #[test]
    fn binary_blocks_round_trip_on_disk() {
        let mut world = WorldBlocks::default();
        world.insert(IVec3::ZERO, BlockData::new(BlockKind::Conveyor, Facing::East));
        let inventory = InventoryItems::for_mode(BuilderMode::Play);
        write_save(
            "test_binary_round_trip",
            &SaveFile::solution(
                "missing_puzzle",
                capture_factory_blocks(&world),
                &inventory,
                None,
            ),
        );

        let bytes = persistent_storage::read_save_bytes("test_binary_round_trip", BLOCKS_FILE)
            .expect("blocks.bin should exist");
        let decoded = save_format::decode_blocks(&bytes).unwrap();
        assert_eq!(decoded.factory_blocks.len(), 1);
        assert_eq!(decoded.factory_blocks[0].kind, BlockKind::Conveyor);
        assert_eq!(decoded.factory_blocks[0].facing, Some(Facing::East));

        delete_save("test_binary_round_trip");
    }
}
