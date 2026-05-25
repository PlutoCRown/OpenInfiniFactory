use bevy::prelude::*;
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::blocks::BlockData;
use crate::world::WorldBlocks;

pub const SAVE_PATH: &str = "saves/world.ron";

#[derive(Serialize, Deserialize)]
struct SaveFile {
    blocks: Vec<SavedBlock>,
}

#[derive(Serialize, Deserialize)]
struct SavedBlock {
    x: i32,
    y: i32,
    z: i32,
    data: BlockData,
}

pub fn save_world(world: &WorldBlocks) {
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
    };

    if let Some(parent) = Path::new(SAVE_PATH).parent() {
        let _ = fs::create_dir_all(parent);
    }

    match ron::ser::to_string_pretty(&save, PrettyConfig::default()) {
        Ok(serialized) => {
            if let Err(error) = fs::write(SAVE_PATH, serialized) {
                warn!("Failed to save world: {error}");
            }
        }
        Err(error) => warn!("Failed to serialize world: {error}"),
    }
}

pub fn load_world(world: &mut WorldBlocks) -> bool {
    let Ok(contents) = fs::read_to_string(SAVE_PATH) else {
        return false;
    };
    let Ok(save) = ron::from_str::<SaveFile>(&contents) else {
        return false;
    };

    world.blocks.clear();
    for saved in save.blocks {
        world
            .blocks
            .insert(IVec3::new(saved.x, saved.y, saved.z), saved.data);
    }
    true
}
