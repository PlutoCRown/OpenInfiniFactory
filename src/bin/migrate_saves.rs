//! 将旧版单文件 RON 存档迁移为文件夹格式（meta.json + blocks.bin）。

use bevy::prelude::IVec3;
use open_infinifactory::game::blocks::{BlockData, BlockKind, PersistentLayer};
use open_infinifactory::game::state::BuilderMode;
use open_infinifactory::game::ui::InventoryItems;
use open_infinifactory::game::world::grid::{BlockSettings, StoredAcceptorStructure, WorldBlocks};
use open_infinifactory::shared::persistent_storage;
use open_infinifactory::shared::save::{delete_save, save_puzzle, save_solution};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
struct LegacySaveFile {
    kind: LegacySaveFileKind,
    #[serde(default)]
    puzzle_id: Option<String>,
    #[serde(default)]
    factory_blocks: Vec<LegacySavedBlock>,
    #[serde(default)]
    blocks: Vec<LegacySavedBlock>,
    #[serde(default)]
    system_blocks: Vec<LegacySavedBlock>,
    #[serde(default)]
    block_settings: Vec<LegacySavedBlockSettings>,
    #[serde(default)]
    next_acceptor_id: u64,
    #[serde(default)]
    acceptor_structures: Vec<StoredAcceptorStructure>,
    #[serde(default)]
    hotbar: Option<open_infinifactory::game::ui::HotbarItems>,
}

#[derive(Serialize, Deserialize)]
enum LegacySaveFileKind {
    Puzzle,
    Solution,
}

#[derive(Clone, Serialize, Deserialize)]
struct LegacySavedBlock {
    x: i32,
    y: i32,
    z: i32,
    data: BlockData,
}

#[derive(Clone, Serialize, Deserialize)]
struct LegacySavedBlockSettings {
    x: i32,
    y: i32,
    z: i32,
    settings: BlockSettings,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("migrate_saves: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let mut dir = PathBuf::from("saves");
    let mut delete_ron = false;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--dir" => {
                dir = PathBuf::from(args.next().ok_or("missing value after --dir")?);
            }
            "--delete-ron" => delete_ron = true,
            "--help" | "-h" => {
                print_help();
                return Ok(());
            }
            value => return Err(format!("unknown argument `{value}`")),
        }
    }

    if !dir.is_dir() {
        return Err(format!("directory not found: {}", dir.display()));
    }

    let mut migrated = 0usize;
    let mut skipped = 0usize;
    for entry in fs::read_dir(&dir).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("ron") {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        if stem == "config" {
            continue;
        }
        if persistent_storage::save_exists(stem) {
            println!("skip `{stem}`: folder save already exists");
            skipped += 1;
            continue;
        }

        let contents =
            fs::read_to_string(&path).map_err(|error| format!("read {}: {error}", path.display()))?;
        let legacy: LegacySaveFile =
            ron::from_str(&contents).map_err(|error| format!("parse {}: {error}", path.display()))?;

        migrate_one(stem, &legacy)?;
        migrated += 1;
        println!("migrated `{stem}`");

        if delete_ron {
            fs::remove_file(&path)
                .map_err(|error| format!("delete {}: {error}", path.display()))?;
        }
    }

    println!("done: migrated={migrated} skipped={skipped}");
    Ok(())
}

fn migrate_one(name: &str, legacy: &LegacySaveFile) -> Result<(), String> {
    let settings_by_pos: std::collections::HashMap<IVec3, BlockSettings> = legacy
        .block_settings
        .iter()
        .map(|entry| (IVec3::new(entry.x, entry.y, entry.z), entry.settings.clone()))
        .collect();

    match legacy.kind {
        LegacySaveFileKind::Puzzle => {
            let mut world = WorldBlocks::default();
            for saved in &legacy.blocks {
                world.insert(saved.pos(), saved.data);
            }
            for saved in &legacy.system_blocks {
                world.insert(saved.pos(), saved.data);
            }
            for (pos, settings) in &settings_by_pos {
                world.set_block_settings(*pos, settings.clone());
            }
            world.restore_acceptor_structures(
                legacy.next_acceptor_id,
                legacy.acceptor_structures.clone(),
            );

            let inventory = legacy
                .hotbar
                .map(|hotbar| {
                    let mut items = InventoryItems::for_mode(BuilderMode::Edit);
                    items.set_hotbar(hotbar);
                    items
                })
                .unwrap_or_else(|| InventoryItems::for_mode(BuilderMode::Edit));

            if !save_puzzle(&world, name, &inventory, None) {
                return Err(format!("failed to write migrated puzzle `{name}`"));
            }
        }
        LegacySaveFileKind::Solution => {
            let puzzle_id = legacy
                .puzzle_id
                .clone()
                .filter(|id| !id.is_empty())
                .ok_or_else(|| format!("solution `{name}` missing puzzle_id"))?;
            let mut world = WorldBlocks::default();
            for saved in &legacy.factory_blocks {
                if saved.data.kind.persistent_layer() == Some(PersistentLayer::SolutionFactory) {
                    world.insert(saved.pos(), saved.data);
                }
            }
            let inventory = legacy
                .hotbar
                .map(|hotbar| {
                    let mut items = InventoryItems::for_mode(BuilderMode::Play);
                    items.set_hotbar(hotbar);
                    items
                })
                .unwrap_or_else(|| InventoryItems::for_mode(BuilderMode::Play));
            if !save_solution(&world, name, &puzzle_id, &inventory, None) {
                delete_save(name);
                return Err(format!("failed to write migrated solution `{name}`"));
            }
        }
    }
    Ok(())
}

impl LegacySavedBlock {
    fn pos(&self) -> IVec3 {
        IVec3::new(self.x, self.y, self.z)
    }
}

fn print_help() {
    eprintln!(
        r#"Usage: migrate_saves [--dir ./saves] [--delete-ron]

Converts legacy saves/*.ron files into folder saves (meta.json + blocks.bin).
"#
    );
}
