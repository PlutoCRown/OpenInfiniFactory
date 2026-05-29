use bevy::prelude::*;
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::game::world::blocks::BlockData;
use crate::game::world::grid::{
    ConverterSettings, GeneratorSettings, LabelerSettings, MaterialFace, MaterialFaceMark,
    MaterialWeld, WorldBlocks,
};

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
    system_blocks: Vec<SavedBlock>,
    #[serde(default)]
    material_welds: Vec<MaterialWeld>,
    #[serde(default)]
    material_face_marks: Vec<SavedMaterialFaceMark>,
    #[serde(default)]
    generator_settings: Vec<SavedGeneratorSettings>,
    #[serde(default)]
    labeler_settings: Vec<SavedLabelerSettings>,
    #[serde(default)]
    converter_settings: Vec<SavedConverterSettings>,
}

#[derive(Serialize, Deserialize)]
struct SavedBlock {
    x: i32,
    y: i32,
    z: i32,
    data: BlockData,
}

#[derive(Serialize, Deserialize)]
struct SavedGeneratorSettings {
    x: i32,
    y: i32,
    z: i32,
    settings: GeneratorSettings,
}

#[derive(Serialize, Deserialize)]
struct SavedLabelerSettings {
    x: i32,
    y: i32,
    z: i32,
    settings: LabelerSettings,
}

#[derive(Serialize, Deserialize)]
struct SavedConverterSettings {
    x: i32,
    y: i32,
    z: i32,
    settings: ConverterSettings,
}

#[derive(Serialize, Deserialize)]
struct SavedMaterialFaceMark {
    face: MaterialFace,
    mark: MaterialFaceMark,
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
        system_blocks: world
            .system_blocks
            .iter()
            .map(|(pos, data)| SavedBlock {
                x: pos.x,
                y: pos.y,
                z: pos.z,
                data: *data,
            })
            .collect(),
        material_welds: world.material_welds.iter().copied().collect(),
        material_face_marks: world
            .material_face_marks
            .iter()
            .map(|(face, mark)| SavedMaterialFaceMark {
                face: *face,
                mark: *mark,
            })
            .collect(),
        generator_settings: world
            .generator_settings
            .iter()
            .map(|(pos, settings)| SavedGeneratorSettings {
                x: pos.x,
                y: pos.y,
                z: pos.z,
                settings: *settings,
            })
            .collect(),
        labeler_settings: world
            .labeler_settings
            .iter()
            .map(|(pos, settings)| SavedLabelerSettings {
                x: pos.x,
                y: pos.y,
                z: pos.z,
                settings: *settings,
            })
            .collect(),
        converter_settings: world
            .converter_settings
            .iter()
            .map(|(pos, settings)| SavedConverterSettings {
                x: pos.x,
                y: pos.y,
                z: pos.z,
                settings: *settings,
            })
            .collect(),
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
    for saved in save.system_blocks {
        world.insert(IVec3::new(saved.x, saved.y, saved.z), saved.data);
    }
    world.replace_material_welds(
        save.material_welds
            .into_iter()
            .filter(|weld| world.is_material_at(weld.a) && world.is_material_at(weld.b))
            .collect(),
    );
    world.replace_material_face_marks(
        save.material_face_marks
            .into_iter()
            .filter_map(|saved| {
                world
                    .is_material_at(saved.face.pos)
                    .then_some((saved.face, saved.mark))
            })
            .collect(),
    );
    for saved in save.generator_settings {
        world.set_generator_settings(IVec3::new(saved.x, saved.y, saved.z), saved.settings);
    }
    for saved in save.labeler_settings {
        world.set_labeler_settings(IVec3::new(saved.x, saved.y, saved.z), saved.settings);
    }
    for saved in save.converter_settings {
        world.set_converter_settings(IVec3::new(saved.x, saved.y, saved.z), saved.settings);
    }
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
