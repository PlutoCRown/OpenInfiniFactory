//! 扫描场景方块资源目录并安装 catalog / 表现注册表

use std::path::{Path, PathBuf};

use super::glb::load_collision_triangles;
use super::meta::SceneBlockMetaFile;
use super::registry::{SceneBlockPresentation, SceneBlockRegistry};
use crate::game::blocks::{
    ColorSpec, SceneBlockCatalog, SceneBlockDef, install_scene_catalog, leak_str, rgb,
};
use crate::shared::{asset_io, platform};

const SCENE_BLOCKS_DIR: &str = "scene_blocks";
const META_FILE: &str = "meta.json";
const MODEL_FILE: &str = "model.glb";
const TEXTURE_FILE: &str = "texture.png";
const COLLISION_FILE: &str = "collision.glb";
const ICON_FILE: &str = "icon.png";

/// 加载全局 `assets/scene_blocks/`，安装模拟 catalog 与表现注册表
pub fn load_global_scene_blocks(registry: &mut SceneBlockRegistry) -> Result<(), String> {
    let root = PathBuf::from(platform::asset_path()).join(SCENE_BLOCKS_DIR);
    let (catalog, presentations) = scan_roots(&[root.as_path()])?;
    install_scene_catalog(catalog);
    registry.clear();
    for presentation in presentations {
        registry.insert(presentation);
    }
    Ok(())
}

/// 仅保留全局包（离开 puzzle 时调用）
pub fn reload_global_only(registry: &mut SceneBlockRegistry) -> Result<(), String> {
    load_global_scene_blocks(registry)
}

/// 合并全局 + puzzle 本地 `assets/scene_blocks/`（重复 id 跳过并警告）
pub fn merge_puzzle_scene_blocks(
    registry: &mut SceneBlockRegistry,
    puzzle_dir: &Path,
) -> Result<(), String> {
    let global_root = PathBuf::from(platform::asset_path()).join(SCENE_BLOCKS_DIR);
    let puzzle_assets = puzzle_dir.join("assets").join(SCENE_BLOCKS_DIR);
    let roots: Vec<&Path> = if asset_io::is_dir(&puzzle_assets) {
        vec![global_root.as_path(), puzzle_assets.as_path()]
    } else {
        vec![global_root.as_path()]
    };
    let (catalog, presentations) = scan_roots(&roots)?;
    install_scene_catalog(catalog);
    registry.clear();
    for presentation in presentations {
        registry.insert(presentation);
    }
    Ok(())
}

fn scan_roots(roots: &[&Path]) -> Result<(SceneBlockCatalog, Vec<SceneBlockPresentation>), String> {
    let mut catalog = SceneBlockCatalog::new();
    let mut presentations = Vec::new();
    for root in roots {
        scan_into(root, &mut catalog, &mut presentations)?;
    }
    Ok((catalog, presentations))
}

fn scan_into(
    root: &Path,
    catalog: &mut SceneBlockCatalog,
    presentations: &mut Vec<SceneBlockPresentation>,
) -> Result<(), String> {
    for dir in asset_io::list_subdirs(root)? {
        load_one_pack(&dir, catalog, presentations)?;
    }
    Ok(())
}

fn optional_file(dir: &Path, name: &str) -> Option<PathBuf> {
    let path = dir.join(name);
    asset_io::is_file(&path).then_some(path)
}

fn load_one_pack(
    dir: &Path,
    catalog: &mut SceneBlockCatalog,
    presentations: &mut Vec<SceneBlockPresentation>,
) -> Result<(), String> {
    let meta_path = dir.join(META_FILE);
    if !asset_io::is_file(&meta_path) {
        return Err(format!("missing {META_FILE} in {}", dir.display()));
    }

    let model_path = optional_file(dir, MODEL_FILE);
    let texture_path = optional_file(dir, TEXTURE_FILE);
    if model_path.is_none() && texture_path.is_none() {
        return Err(format!(
            "missing {MODEL_FILE} and {TEXTURE_FILE} in {}",
            dir.display()
        ));
    }

    let text = asset_io::read_to_string(&meta_path)
        .map_err(|e| format!("read {}: {e}", meta_path.display()))?;
    let meta: SceneBlockMetaFile =
        serde_json::from_str(&text).map_err(|e| format!("parse {}: {e}", meta_path.display()))?;

    if meta.id.is_empty() {
        return Err(format!("{}: id must not be empty", meta_path.display()));
    }

    let name_key = leak_str(&format!("block.{}", meta.id));
    let short_name_key = leak_str(&format!("short.{}", meta.id));
    let description_key = leak_str(&format!("desc.{}", meta.id));
    // 展示色兜底；真正外观以 model.glb / texture.png 为准
    let color = default_scene_color();

    let id = match catalog.register(SceneBlockDef {
        string_id: meta.id.clone(),
        name_key,
        short_name_key,
        description_key,
        collision: meta.collision,
        connectable: meta.connectable,
        directional: meta.directional,
        color,
    }) {
        Ok(id) => id,
        Err(err) => {
            // 重复 id：跳过该包，不拖垮整次加载
            bevy::log::warn!("skip scene pack {}: {err}", dir.display());
            return Ok(());
        }
    };

    let collision_model_path = optional_file(dir, COLLISION_FILE);
    let collision_tris = match &collision_model_path {
        Some(path) => match load_collision_triangles(path) {
            Ok(tris) => Some(tris),
            Err(err) => {
                bevy::log::error!("scene collision load failed: {err}");
                None
            }
        },
        None => None,
    };
    let icon_path = optional_file(dir, ICON_FILE);

    presentations.push(SceneBlockPresentation {
        id,
        string_id: meta.id,
        model_path,
        texture_path,
        collision_model_path,
        collision_tris,
        icon_path,
        color,
    });
    Ok(())
}

fn default_scene_color() -> ColorSpec {
    rgb(0.55, 0.55, 0.55)
}
