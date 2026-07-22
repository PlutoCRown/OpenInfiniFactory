//! 扫描材料 / 印花 / 滚刷资源目录并安装 catalog / 表现注册表

use std::path::{Path, PathBuf};

use super::meta::{MaterialBlockMetaFile, PaintMaterialMetaFile, StampMaterialMetaFile};
use super::registry::{
    MaterialBlockPresentation, MaterialBlockRegistry, PaintMaterialPresentation,
    PaintMaterialRegistry, StampMaterialPresentation, StampMaterialRegistry,
};
use crate::game::blocks::{
    ColorSpec, MaterialBlockCatalog, MaterialBlockDef, PaintMaterialCatalog, PaintMaterialDef,
    StampMaterialCatalog, StampMaterialDef, install_material_catalog, install_paint_catalog,
    install_stamp_catalog, leak_str, rgb, stamp_seed_color,
};
use crate::shared::{asset_io, platform};

const MATERIAL_BLOCKS_DIR: &str = "material_blocks";
const STAMP_MATERIALS_DIR: &str = "stamp_materials";
const PAINT_MATERIALS_DIR: &str = "paint_materials";
const META_FILE: &str = "meta.json";
const MODEL_FILE: &str = "model.glb";
const TEXTURE_FILE: &str = "texture.png";
const NORMAL_FILE: &str = "normal.png";
const ICON_FILE: &str = "icon.png";

/// 三个表现注册表的可变引用集合（加载入口共用）
pub struct MaterialPackRegistries<'a> {
    pub materials: &'a mut MaterialBlockRegistry,
    pub stamps: &'a mut StampMaterialRegistry,
    pub paints: &'a mut PaintMaterialRegistry,
}

/// 加载全局 `assets/material_blocks|stamp_materials|paint_materials/`
pub fn load_global_material_packs(registries: MaterialPackRegistries<'_>) -> Result<(), String> {
    let asset_root = PathBuf::from(platform::asset_path());
    load_from_roots(
        &[asset_root.join(MATERIAL_BLOCKS_DIR)],
        &[asset_root.join(STAMP_MATERIALS_DIR)],
        &[asset_root.join(PAINT_MATERIALS_DIR)],
        registries,
    )
}

/// 仅保留全局包（离开 puzzle 时调用）
pub fn reload_global_only(registries: MaterialPackRegistries<'_>) -> Result<(), String> {
    load_global_material_packs(registries)
}

/// 合并全局 + puzzle 本地同类目录（重复 id 跳过并警告）
pub fn merge_puzzle_material_packs(
    registries: MaterialPackRegistries<'_>,
    puzzle_dir: &Path,
) -> Result<(), String> {
    let asset_root = PathBuf::from(platform::asset_path());
    let puzzle_assets = puzzle_dir.join("assets");

    let global_material = asset_root.join(MATERIAL_BLOCKS_DIR);
    let puzzle_material = puzzle_assets.join(MATERIAL_BLOCKS_DIR);
    let material_roots: Vec<PathBuf> = if asset_io::is_dir(&puzzle_material) {
        vec![global_material, puzzle_material]
    } else {
        vec![global_material]
    };

    let global_stamp = asset_root.join(STAMP_MATERIALS_DIR);
    let puzzle_stamp = puzzle_assets.join(STAMP_MATERIALS_DIR);
    let stamp_roots: Vec<PathBuf> = if asset_io::is_dir(&puzzle_stamp) {
        vec![global_stamp, puzzle_stamp]
    } else {
        vec![global_stamp]
    };

    let global_paint = asset_root.join(PAINT_MATERIALS_DIR);
    let puzzle_paint = puzzle_assets.join(PAINT_MATERIALS_DIR);
    let paint_roots: Vec<PathBuf> = if asset_io::is_dir(&puzzle_paint) {
        vec![global_paint, puzzle_paint]
    } else {
        vec![global_paint]
    };

    load_from_roots(&material_roots, &stamp_roots, &paint_roots, registries)
}

fn load_from_roots(
    material_roots: &[PathBuf],
    stamp_roots: &[PathBuf],
    paint_roots: &[PathBuf],
    registries: MaterialPackRegistries<'_>,
) -> Result<(), String> {
    let (material_catalog, material_presentations) = scan_material_roots(material_roots)?;
    let (stamp_catalog, stamp_presentations) = scan_stamp_roots(stamp_roots)?;
    let (paint_catalog, paint_presentations) = scan_paint_roots(paint_roots)?;

    install_material_catalog(material_catalog);
    install_stamp_catalog(stamp_catalog);
    install_paint_catalog(paint_catalog);

    registries.materials.clear();
    for presentation in material_presentations {
        registries.materials.insert(presentation);
    }
    registries.stamps.clear();
    for presentation in stamp_presentations {
        registries.stamps.insert(presentation);
    }
    registries.paints.clear();
    for presentation in paint_presentations {
        registries.paints.insert(presentation);
    }
    Ok(())
}

fn scan_material_roots(
    roots: &[PathBuf],
) -> Result<(MaterialBlockCatalog, Vec<MaterialBlockPresentation>), String> {
    let mut catalog = MaterialBlockCatalog::new();
    let mut presentations = Vec::new();
    for root in roots {
        scan_material_into(root, &mut catalog, &mut presentations)?;
    }
    Ok((catalog, presentations))
}

fn scan_stamp_roots(
    roots: &[PathBuf],
) -> Result<(StampMaterialCatalog, Vec<StampMaterialPresentation>), String> {
    let mut catalog = StampMaterialCatalog::new();
    let mut presentations = Vec::new();
    for root in roots {
        scan_stamp_into(root, &mut catalog, &mut presentations)?;
    }
    Ok((catalog, presentations))
}

fn scan_paint_roots(
    roots: &[PathBuf],
) -> Result<(PaintMaterialCatalog, Vec<PaintMaterialPresentation>), String> {
    let mut catalog = PaintMaterialCatalog::new();
    let mut presentations = Vec::new();
    for root in roots {
        scan_paint_into(root, &mut catalog, &mut presentations)?;
    }
    Ok((catalog, presentations))
}

fn list_pack_dirs(root: &Path) -> Result<Vec<PathBuf>, String> {
    asset_io::list_subdirs(root)
}

fn optional_file(dir: &Path, name: &str) -> Option<PathBuf> {
    let path = dir.join(name);
    asset_io::is_file(&path).then_some(path)
}

fn scan_material_into(
    root: &Path,
    catalog: &mut MaterialBlockCatalog,
    presentations: &mut Vec<MaterialBlockPresentation>,
) -> Result<(), String> {
    for dir in list_pack_dirs(root)? {
        load_one_material(&dir, catalog, presentations)?;
    }
    Ok(())
}

fn load_one_material(
    dir: &Path,
    catalog: &mut MaterialBlockCatalog,
    presentations: &mut Vec<MaterialBlockPresentation>,
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
    let meta: MaterialBlockMetaFile =
        serde_json::from_str(&text).map_err(|e| format!("parse {}: {e}", meta_path.display()))?;
    if meta.id.is_empty() {
        return Err(format!("{}: id must not be empty", meta_path.display()));
    }

    let color = material_seed_color(&meta.id);
    let name_key = leak_str(&format!("block.{}", meta.id));
    let short_name_key = leak_str(&format!("short.{}", meta.id));
    let description_key = leak_str(&format!("desc.{}", meta.id));

    let id = match catalog.register(MaterialBlockDef {
        string_id: meta.id.clone(),
        name_key,
        short_name_key,
        description_key,
        connectable: meta.connectable,
        fragile: meta.fragile,
        directional: meta.directional,
        color,
    }) {
        Ok(id) => id,
        Err(err) => {
            bevy::log::warn!("skip material pack {}: {err}", dir.display());
            return Ok(());
        }
    };

    presentations.push(MaterialBlockPresentation {
        id,
        string_id: meta.id,
        model_path,
        texture_path,
        normal_path: optional_file(dir, NORMAL_FILE),
        icon_path: optional_file(dir, ICON_FILE),
        color,
    });
    Ok(())
}

fn scan_stamp_into(
    root: &Path,
    catalog: &mut StampMaterialCatalog,
    presentations: &mut Vec<StampMaterialPresentation>,
) -> Result<(), String> {
    for dir in list_pack_dirs(root)? {
        load_one_stamp(&dir, catalog, presentations)?;
    }
    Ok(())
}

fn load_one_stamp(
    dir: &Path,
    catalog: &mut StampMaterialCatalog,
    presentations: &mut Vec<StampMaterialPresentation>,
) -> Result<(), String> {
    let meta_path = dir.join(META_FILE);
    if !asset_io::is_file(&meta_path) {
        return Err(format!("missing {META_FILE} in {}", dir.display()));
    }
    let model_path = optional_file(dir, MODEL_FILE);
    if model_path.is_none() {
        return Err(format!("missing {MODEL_FILE} in {}", dir.display()));
    }
    let texture_path = optional_file(dir, TEXTURE_FILE);

    let text = asset_io::read_to_string(&meta_path)
        .map_err(|e| format!("read {}: {e}", meta_path.display()))?;
    let meta: StampMaterialMetaFile =
        serde_json::from_str(&text).map_err(|e| format!("parse {}: {e}", meta_path.display()))?;
    if meta.id.is_empty() {
        return Err(format!("{}: id must not be empty", meta_path.display()));
    }

    let color = stamp_seed_color(&meta.id);
    let name_key = leak_str(&format!("stamp.{}", meta.id));
    let short_name_key = leak_str(&format!("short.stamp.{}", meta.id));
    let description_key = leak_str(&format!("desc.stamp.{}", meta.id));

    let id = match catalog.register(StampMaterialDef {
        string_id: meta.id.clone(),
        name_key,
        short_name_key,
        description_key,
        fragile: meta.fragile,
        color,
    }) {
        Ok(id) => id,
        Err(err) => {
            bevy::log::warn!("skip stamp pack {}: {err}", dir.display());
            return Ok(());
        }
    };

    presentations.push(StampMaterialPresentation {
        id,
        string_id: meta.id,
        model_path,
        texture_path,
        icon_path: optional_file(dir, ICON_FILE),
        color,
    });
    Ok(())
}

fn scan_paint_into(
    root: &Path,
    catalog: &mut PaintMaterialCatalog,
    presentations: &mut Vec<PaintMaterialPresentation>,
) -> Result<(), String> {
    for dir in list_pack_dirs(root)? {
        load_one_paint(&dir, catalog, presentations)?;
    }
    Ok(())
}

fn load_one_paint(
    dir: &Path,
    catalog: &mut PaintMaterialCatalog,
    presentations: &mut Vec<PaintMaterialPresentation>,
) -> Result<(), String> {
    let meta_path = dir.join(META_FILE);
    if !asset_io::is_file(&meta_path) {
        return Err(format!("missing {META_FILE} in {}", dir.display()));
    }
    let texture_path = dir.join(TEXTURE_FILE);
    if !asset_io::is_file(&texture_path) {
        return Err(format!("missing {TEXTURE_FILE} in {}", dir.display()));
    }

    let text = asset_io::read_to_string(&meta_path)
        .map_err(|e| format!("read {}: {e}", meta_path.display()))?;
    let meta: PaintMaterialMetaFile =
        serde_json::from_str(&text).map_err(|e| format!("parse {}: {e}", meta_path.display()))?;
    if meta.id.is_empty() {
        return Err(format!("{}: id must not be empty", meta_path.display()));
    }

    let name_key = leak_str(&format!("paint.{}", meta.id));
    let short_name_key = leak_str(&format!("short.paint.{}", meta.id));
    let description_key = leak_str(&format!("desc.paint.{}", meta.id));

    let id = match catalog.register(PaintMaterialDef {
        string_id: meta.id.clone(),
        name_key,
        short_name_key,
        description_key,
    }) {
        Ok(id) => id,
        Err(err) => {
            bevy::log::warn!("skip paint pack {}: {err}", dir.display());
            return Ok(());
        }
    };

    presentations.push(PaintMaterialPresentation {
        id,
        string_id: meta.id,
        texture_path,
    });
    Ok(())
}

/// 已知材料包缺省色（包无 color 字段时用统一灰）
fn material_seed_color(_id: &str) -> ColorSpec {
    rgb(0.7, 0.7, 0.7)
}
