use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::game::blocks::{BlockData, BlockKind, PersistentLayer};
use crate::game::world::grid::{BlockSettings, WorldBlocks};
use crate::game::{
    state::BuilderMode,
    ui::{HotbarItems, InventoryItems},
};
use crate::shared::persistent_storage;
use crate::shared::save_format::{self, SaveBlocksData, SavedBlock, BLOCKS_FILE, META_FILE};

const SAVE_VERSION: u32 = 1;

/// 存档寻址：Puzzle 为顶层目录，Solution 在其 `solutions/` 子目录下
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SaveSlot {
    pub puzzle: String,
    pub solution: Option<String>,
}

impl SaveSlot {
    pub fn puzzle(name: impl Into<String>) -> Self {
        Self {
            puzzle: normalized_save_name(&name.into()),
            solution: None,
        }
    }

    pub fn solution(puzzle: impl Into<String>, solution: impl Into<String>) -> Self {
        Self {
            puzzle: normalized_save_name(&puzzle.into()),
            solution: Some(normalized_save_name(&solution.into())),
        }
    }

    pub fn kind(&self) -> SaveKind {
        if self.solution.is_some() {
            SaveKind::Solution
        } else {
            SaveKind::Puzzle
        }
    }

    pub fn storage_path(&self) -> String {
        match &self.solution {
            None => sanitize_save_name(&self.puzzle),
            Some(solution) => format!(
                "{}/solutions/{}",
                sanitize_save_name(&self.puzzle),
                sanitize_save_name(solution)
            ),
        }
    }

    pub fn display_name(&self) -> String {
        self.solution
            .as_ref()
            .cloned()
            .unwrap_or_else(|| self.puzzle.clone())
    }

    pub fn from_storage_path(path: &str) -> Option<Self> {
        let parts: Vec<&str> = path.split('/').collect();
        match parts.as_slice() {
            [puzzle] => Some(Self::puzzle(*puzzle)),
            [puzzle, "solutions", solution] if !puzzle.is_empty() && !solution.is_empty() => {
                Some(Self::solution(*puzzle, *solution))
            }
            _ => None,
        }
    }
}

#[derive(Resource, Default)]
pub struct SaveState {
    pub current: Option<SaveSlot>,
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
                    entry.kind == SaveKind::Solution && entry.slot.puzzle == puzzle
                })
                .cloned()
                .collect(),
            None => Vec::new(),
        };
    }
}

#[derive(Clone)]
pub struct SaveEntry {
    pub slot: SaveSlot,
    pub name: String,
    pub kind: SaveKind,
}

impl SaveEntry {
    pub fn puzzle_id(&self) -> Option<&str> {
        (self.kind == SaveKind::Solution).then_some(self.slot.puzzle.as_str())
    }
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
    slot: &SaveSlot,
    inventory: &InventoryItems,
    player: Option<PlayerSave>,
) -> bool {
    if slot.solution.is_some() {
        return false;
    }
    write_save(
        slot,
        &SaveFile::puzzle(capture_puzzle_layer(world, inventory), player),
    )
}

pub fn save_solution(
    world: &WorldBlocks,
    slot: &SaveSlot,
    inventory: &InventoryItems,
    player: Option<PlayerSave>,
) -> bool {
    let Some(solution) = slot.solution.as_deref() else {
        return false;
    };
    if solution.is_empty() {
        return false;
    }
    write_save(
        slot,
        &SaveFile::solution(
            &slot.puzzle,
            capture_factory_blocks(world),
            capture_wire_face_panels(world),
            inventory,
            player,
        ),
    )
}

pub fn load_world(world: &mut WorldBlocks, slot: &SaveSlot) -> Option<LoadedSave> {
    let loaded = decode_save_slot(slot)?;
    *world = loaded.world.clone();
    Some(loaded)
}

/// 仅解码存档（可在后台任务中跑，不碰 ECS）
pub fn decode_save_slot(slot: &SaveSlot) -> Option<LoadedSave> {
    let save = read_save(slot)?;
    save.into_loaded(slot)
}

pub fn save_kind(slot: &SaveSlot) -> Option<SaveKind> {
    let save = read_save(slot)?;
    Some(save.meta_kind())
}

pub fn has_solutions_for_puzzle(puzzle: &str) -> bool {
    !persistent_storage::list_solution_names(puzzle).is_empty()
}

pub fn invalidate_solutions_for_puzzle(puzzle: &str) -> usize {
    persistent_storage::list_solution_names(puzzle)
        .into_iter()
        .filter(|name| delete_save(&SaveSlot::solution(puzzle, name)))
        .count()
}

pub fn delete_save(slot: &SaveSlot) -> bool {
    persistent_storage::remove_save_folder(&slot.storage_path())
}

pub fn rename_save(old: &SaveSlot, new: &SaveSlot) -> bool {
    if old.kind() != new.kind() {
        return false;
    }
    if old.solution.is_some() && old.puzzle != new.puzzle {
        return false;
    }
    let old_path = old.storage_path();
    let new_path = new.storage_path();
    if old_path == new_path {
        return true;
    }
    if new.puzzle.is_empty() || new.solution.as_deref().is_some_and(str::is_empty) {
        return false;
    }
    if persistent_storage::save_exists(&new_path) {
        warn!("Cannot rename save {old_path} to {new_path}: target already exists");
        return false;
    }
    if !persistent_storage::save_exists(&old_path) {
        warn!("Cannot rename save {old_path}: source missing");
        return false;
    }
    persistent_storage::rename_save_folder(&old_path, &new_path)
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
            },
            blocks: SaveBlocksData {
                scene_blocks: layer.scene_blocks,
                system_blocks: layer.system_blocks,
                factory_blocks: Vec::new(),
                wire_face_panels: Vec::new(),
            },
        }
    }

    fn solution(
        puzzle_id: &str,
        factory_blocks: Vec<SavedBlock>,
        wire_face_panels: Vec<save_format::SavedWireFacePanel>,
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
            },
            blocks: SaveBlocksData {
                factory_blocks,
                wire_face_panels,
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

    fn into_loaded(self, slot: &SaveSlot) -> Option<LoadedSave> {
        match self.meta.kind {
            SaveMetaKind::Solution => {
                if slot.solution.is_none() {
                    return None;
                }
                let puzzle_id = slot.puzzle.clone();
                if self
                    .meta
                    .puzzle_id
                    .as_deref()
                    .filter(|id| !id.is_empty())
                    .is_some_and(|id| id != puzzle_id)
                {
                    warn!(
                        "Solution save path puzzle `{puzzle_id}` disagrees with meta puzzle_id"
                    );
                    return None;
                }
                let hotbar = self
                    .meta
                    .hotbar
                    .or_else(|| Some(InventoryItems::for_mode(BuilderMode::Play).hotbar));
                let puzzle_world = load_puzzle_world(&puzzle_id)?;
                let mut world = puzzle_world.clone();
                apply_factory_blocks(&mut world, self.blocks.factory_blocks);
                apply_wire_face_panels(&mut world, self.blocks.wire_face_panels);
                Some(LoadedSave {
                    world,
                    puzzle_snapshot: Some(puzzle_world),
                    puzzle_id: Some(puzzle_id),
                    hotbar,
                    player: self.meta.player,
                })
            }
            SaveMetaKind::Puzzle => {
                if slot.solution.is_some() {
                    return None;
                }
                let layer = PuzzleLayer::from_blocks(self.blocks, self.meta.hotbar);
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
    hotbar: Option<HotbarItems>,
}

impl PuzzleLayer {
    fn from_blocks(blocks: SaveBlocksData, hotbar: Option<HotbarItems>) -> Self {
        Self {
            scene_blocks: blocks.scene_blocks,
            system_blocks: blocks.system_blocks,
            hotbar,
        }
    }
}

fn load_puzzle_world(puzzle: &str) -> Option<WorldBlocks> {
    let slot = SaveSlot::puzzle(puzzle);
    let save = read_save(&slot)?;
    if !matches!(save.meta.kind, SaveMetaKind::Puzzle) {
        return None;
    }
    let mut world = WorldBlocks::default();
    apply_layer(
        &mut world,
        PuzzleLayer::from_blocks(save.blocks, save.meta.hotbar),
    );
    Some(world)
}

fn write_save(slot: &SaveSlot, save: &SaveFile) -> bool {
    let path = slot.storage_path();
    let meta = match serde_json::to_string_pretty(&save.meta) {
        Ok(serialized) => serialized,
        Err(error) => {
            warn!("Failed to serialize save meta: {error}");
            return false;
        }
    };
    if !persistent_storage::write_save_text(&path, META_FILE, &meta) {
        return false;
    }
    let blocks = save_format::encode_blocks(&save.blocks);
    persistent_storage::write_save_bytes(&path, BLOCKS_FILE, &blocks)
}

fn read_save(slot: &SaveSlot) -> Option<SaveFile> {
    let path = slot.storage_path();
    let meta_text = persistent_storage::read_save_text(&path, META_FILE)?;
    let meta = serde_json::from_str::<SaveMeta>(&meta_text).ok()?;
    let blocks_bytes = persistent_storage::read_save_bytes(&path, BLOCKS_FILE)?;
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
        hotbar: Some(inventory.hotbar),
    }
}

fn capture_factory_blocks(world: &WorldBlocks) -> Vec<SavedBlock> {
    world
        .blocks
        .iter()
        .filter_map(|(pos, data)| {
            (data.kind.persistent_layer() == Some(PersistentLayer::SolutionFactory)).then_some(
                saved_block(*pos, *data, world.block_settings.get(pos).cloned()),
            )
        })
        .collect()
}

fn capture_wire_face_panels(world: &WorldBlocks) -> Vec<save_format::SavedWireFacePanel> {
    let id_to_pos: std::collections::HashMap<_, _> = world
        .blocks
        .iter()
        .map(|(pos, block)| (block.id, *pos))
        .collect();
    world
        .wire_face_panels
        .iter()
        .filter_map(|face| {
            let pos = id_to_pos.get(&face.block)?;
            Some(save_format::SavedWireFacePanel {
                x: pos.x,
                y: pos.y,
                z: pos.z,
                nx: face.normal.x,
                ny: face.normal.y,
                nz: face.normal.z,
            })
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
    world.resync_acceptor_structures();
}

fn apply_factory_blocks(world: &mut WorldBlocks, factory_blocks: Vec<SavedBlock>) {
    for saved in factory_blocks {
        if saved.kind.persistent_layer() == Some(PersistentLayer::SolutionFactory) {
            world.insert(saved.pos(), saved.to_block_data());
            if let Some(settings) = &saved.settings {
                world.set_block_settings(saved.pos(), settings.clone());
            }
        }
    }
    world.rebuild_factory_attachments();
}

fn apply_wire_face_panels(world: &mut WorldBlocks, panels: Vec<save_format::SavedWireFacePanel>) {
    for panel in panels {
        let pos = panel.pos();
        let Some(block) = world.blocks.get(&pos).copied() else {
            continue;
        };
        if block.kind != BlockKind::Wire {
            continue;
        }
        world.set_wire_face_panel(
            crate::game::world::grid::MaterialFace::new(block.id, panel.normal()),
            true,
        );
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
    let mut entries = Vec::new();
    for puzzle in persistent_storage::list_puzzles() {
        entries.push(SaveEntry {
            slot: SaveSlot::puzzle(&puzzle),
            name: puzzle.clone(),
            kind: SaveKind::Puzzle,
        });
        for solution in persistent_storage::list_solution_names(&puzzle) {
            entries.push(SaveEntry {
                slot: SaveSlot::solution(&puzzle, &solution),
                name: solution,
                kind: SaveKind::Solution,
            });
        }
    }
    entries.sort_by(|a, b| a.name.cmp(&b.name).then(a.slot.puzzle.cmp(&b.slot.puzzle)));
    entries
}

pub fn puzzle_names(entries: &[SaveEntry]) -> Vec<String> {
    entries
        .iter()
        .filter(|entry| entry.kind == SaveKind::Puzzle)
        .map(|entry| entry.slot.puzzle.clone())
        .collect()
}

pub fn solution_names_for_puzzle(entries: &[SaveEntry], puzzle: &str) -> Vec<String> {
    entries
        .iter()
        .filter(|entry| entry.kind == SaveKind::Solution && entry.slot.puzzle == puzzle)
        .map(|entry| entry.name.clone())
        .collect()
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
    use crate::game::blocks::{BlockKind, Facing, MaterialKind, PaintColor, StampColor};
    use crate::game::world::grid::{
        ConverterMode, ConverterSettings, GeneratorMode, GeneratorSettings, RollerSettings,
        StamperSettings, TeleportSettings,
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
        world.set_stamper_settings(
            stamper,
            StamperSettings {
                color: StampColor::Blue,
            },
        );
        world.set_roller_settings(
            roller,
            RollerSettings {
                color: PaintColor::Yellow,
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
            .into_loaded(&SaveSlot::puzzle("round_trip"))
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
            round_trip.stamper_settings(stamper),
            StamperSettings {
                color: StampColor::Blue,
            }
        );
        assert_eq!(
            round_trip.roller_settings(roller),
            RollerSettings {
                color: PaintColor::Yellow,
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
        .into_loaded(&SaveSlot::puzzle("hotbar_puzzle"))
        .unwrap();
        assert_eq!(puzzle_loaded.hotbar, Some(puzzle_inventory.hotbar));

        let mut solution_inventory = InventoryItems::for_mode(BuilderMode::Play);
        solution_inventory.set_hotbar_block(0, BlockKind::Platform);
        solution_inventory.set_hotbar_block(1, BlockKind::Pusher);
        solution_inventory.hotbar[2] = None;

        let solution_file = SaveFile::solution("puzzle", Vec::new(), Vec::new(), &solution_inventory, None);
        assert_eq!(solution_file.meta.puzzle_id.as_deref(), Some("puzzle"));
        assert!(solution_file.blocks.scene_blocks.is_empty());
        assert!(solution_file.blocks.system_blocks.is_empty());
    }

    #[test]
    fn solution_loads_factory_blocks_from_puzzle_reference() {
        let puzzle_slot = SaveSlot::puzzle("test_puzzle_ref");
        let solution_slot = SaveSlot::solution("test_puzzle_ref", "test_solution_ref");

        let mut puzzle_world = WorldBlocks::default();
        puzzle_world.insert(IVec3::ZERO, BlockData::new(BlockKind::Stone, Facing::North));
        let puzzle_inventory = InventoryItems::for_mode(BuilderMode::Edit);
        let puzzle_save =
            SaveFile::puzzle(capture_puzzle_layer(&puzzle_world, &puzzle_inventory), None);
        write_save(&puzzle_slot, &puzzle_save);

        let mut solution_world = puzzle_world.clone();
        solution_world.insert(IVec3::X, BlockData::new(BlockKind::Platform, Facing::North));
        let solution_inventory = InventoryItems::for_mode(BuilderMode::Play);
        write_save(
            &solution_slot,
            &SaveFile::solution(
                "test_puzzle_ref",
                capture_factory_blocks(&solution_world),
                capture_wire_face_panels(&solution_world),
                &solution_inventory,
                None,
            ),
        );

        let mut loaded_world = WorldBlocks::default();
        let loaded = load_world(&mut loaded_world, &solution_slot).unwrap();
        assert_eq!(loaded.puzzle_id.as_deref(), Some("test_puzzle_ref"));
        assert!(loaded_world.blocks.contains_key(&IVec3::ZERO));
        assert!(loaded_world.blocks.contains_key(&IVec3::X));
        assert!(!loaded_world.blocks.contains_key(&IVec3::NEG_X));

        delete_save(&puzzle_slot);
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
        let puzzle_slot = SaveSlot::puzzle("test_player_pose");
        write_save(
            &puzzle_slot,
            &SaveFile::puzzle(
                capture_puzzle_layer(&WorldBlocks::default(), &inventory),
                Some(player.clone()),
            ),
        );
        let loaded = read_save(&puzzle_slot).unwrap();
        assert_eq!(loaded.meta.player, Some(player));
        delete_save(&puzzle_slot);
    }

    #[test]
    fn generator_link_anchor_round_trips_through_blocks_bin() {
        let goal = IVec3::new(-10, 10, 5);
        let generator = IVec3::new(1, 1, 0);

        let mut world = WorldBlocks::default();
        world.insert(goal, BlockData::new(BlockKind::Goal, Facing::North));
        world.insert(
            generator,
            BlockData::new(BlockKind::Generator, Facing::North),
        );
        world.resync_acceptor_structures();
        world.set_generator_settings(
            generator,
            GeneratorSettings {
                mode: GeneratorMode::Link { anchor: Some(goal) },
                material: MaterialKind::Copper,
            },
        );

        let inventory = InventoryItems::for_mode(BuilderMode::Edit);
        let puzzle_slot = SaveSlot::puzzle("test_generator_link_anchor");
        write_save(
            &puzzle_slot,
            &SaveFile::puzzle(capture_puzzle_layer(&world, &inventory), None),
        );

        let mut loaded_world = WorldBlocks::default();
        load_world(&mut loaded_world, &puzzle_slot).unwrap();
        assert_eq!(
            loaded_world.generator_settings(generator),
            GeneratorSettings {
                mode: GeneratorMode::Link { anchor: Some(goal) },
                material: MaterialKind::Copper,
            }
        );
        assert_eq!(
            loaded_world.acceptor_id_at(goal),
            loaded_world.acceptor_id_at(goal)
        );

        delete_save(&puzzle_slot);
    }

    #[test]
    fn binary_blocks_round_trip_on_disk() {
        let solution_slot = SaveSlot::solution("missing_puzzle", "test_binary_round_trip");
        let mut world = WorldBlocks::default();
        world.insert(
            IVec3::ZERO,
            BlockData::new(BlockKind::Conveyor, Facing::East),
        );
        let inventory = InventoryItems::for_mode(BuilderMode::Play);
        write_save(
            &solution_slot,
            &SaveFile::solution(
                "missing_puzzle",
                capture_factory_blocks(&world),
                capture_wire_face_panels(&world),
                &inventory,
                None,
            ),
        );

        let bytes =
            persistent_storage::read_save_bytes(&solution_slot.storage_path(), BLOCKS_FILE)
                .expect("blocks.bin should exist");
        let decoded = save_format::decode_blocks(&bytes).unwrap();
        assert_eq!(decoded.factory_blocks.len(), 1);
        assert_eq!(decoded.factory_blocks[0].kind, BlockKind::Conveyor);
        assert_eq!(decoded.factory_blocks[0].facing, Some(Facing::East));

        delete_save(&solution_slot);
    }
}
