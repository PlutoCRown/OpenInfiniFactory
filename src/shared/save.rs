use bevy::prelude::*;
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::game::world::blocks::BlockData;
use crate::game::world::grid::{MaterialWeld, WorldBlocks};

pub const SAVE_DIR: &str = "saves";
pub const SAVE_SLOTS: usize = 8;

#[derive(Resource, Default)]
pub struct SaveState {
    pub current: Option<String>,
    pub slots: Vec<String>,
}

impl SaveState {
    pub fn refresh(&mut self) {
        self.slots = list_saves();
    }
}

#[derive(Serialize, Deserialize)]
struct SaveFile {
    blocks: Vec<SavedBlock>,
    #[serde(default)]
    material_welds: Vec<MaterialWeld>,
}

#[derive(Serialize, Deserialize)]
struct SavedBlock {
    x: i32,
    y: i32,
    z: i32,
    data: BlockData,
}

pub fn save_world(world: &WorldBlocks, name: &str) -> bool {
    let save = SaveFile {
        blocks: world
            .blocks
            .iter()
            .map(|(pos, data)| SavedBlock {
                x: pos.x,
                y: pos.y,
                z: pos.z,
                data: *data,
            })
            .collect(),
        material_welds: world.material_welds.iter().copied().collect(),
    };

    if let Err(error) = fs::create_dir_all(SAVE_DIR) {
        warn!("Failed to create save directory: {error}");
        return false;
    }

    let path = save_path(name);
    match ron::ser::to_string_pretty(&save, PrettyConfig::default()) {
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

pub fn load_world(world: &mut WorldBlocks, name: &str) -> bool {
    let Ok(contents) = fs::read_to_string(save_path(name)) else {
        return false;
    };
    let Ok(save) = ron::from_str::<SaveFile>(&contents) else {
        return false;
    };

    world.clear();
    for saved in save.blocks {
        world.insert(IVec3::new(saved.x, saved.y, saved.z), saved.data);
    }
    world.replace_material_welds(
        save.material_welds
            .into_iter()
            .filter(|weld| world.is_material_at(weld.a) && world.is_material_at(weld.b))
            .collect(),
    );
    true
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
