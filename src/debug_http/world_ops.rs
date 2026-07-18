use bevy::prelude::IVec3;
use std::path::{Path, PathBuf};

use crate::game::blocks::{all_blocks, BlockData, BlockKind};
use crate::game::simulation::markers::refresh_static_generated_markers;
use crate::game::world::direction::Facing;
use crate::game::world::grid::WorldBlocks;
use crate::shared::save::{load_world, SaveSlot};
use oif_sim::SimSession;

/// 解析方块种类名（场景 / 材料 / 印花字符串 id，或工厂枚举 Debug 名）
pub fn parse_block_kind(name: &str) -> Option<BlockKind> {
    let name = name.trim();
    let lower = name.to_ascii_lowercase();
    oif_sim::blocks::ensure_fallback_scene_catalog();
    oif_sim::blocks::ensure_fallback_material_catalog();
    oif_sim::blocks::ensure_fallback_stamp_catalog();
    if let Some(id) = oif_sim::blocks::scene_catalog().id_by_string(&lower) {
        return Some(BlockKind::Scene(id));
    }
    // 材料：catalog id，或旧 fixture 名 IronMaterial → iron
    let material_id = match lower.as_str() {
        "material" => Some("basic"),
        "ironmaterial" => Some("iron"),
        "coppermaterial" => Some("copper"),
        "glassmaterial" => Some("glass_material"),
        other => Some(other),
    };
    if let Some(id_str) = material_id {
        if let Some(id) = oif_sim::blocks::material_catalog().id_by_string(id_str) {
            return Some(BlockKind::Material(id));
        }
    }
    let stamp_id = match lower.as_str() {
        "stampmaterial" | "stamp" => Some("red"),
        other => Some(other),
    };
    if let Some(id_str) = stamp_id {
        if let Some(id) = oif_sim::blocks::stamp_catalog().id_by_string(id_str) {
            // 避免与材料同名 id 抢解析：仅当不是材料 id 时才当印花
            if oif_sim::blocks::material_catalog()
                .id_by_string(id_str)
                .is_none()
            {
                return Some(BlockKind::Stamp(id));
            }
        }
    }
    all_blocks()
        .into_iter()
        .find(|kind| format!("{:?}", kind).eq_ignore_ascii_case(name))
}

/// 解析朝向名
pub fn parse_facing(name: &str) -> Option<Facing> {
    match name.trim().to_ascii_lowercase().as_str() {
        "north" | "n" => Some(Facing::North),
        "east" | "e" => Some(Facing::East),
        "south" | "s" => Some(Facing::South),
        "west" | "w" => Some(Facing::West),
        _ => None,
    }
}

/// 重置无头会话
pub fn reset_session(core: &mut SimSession) {
    core.reset();
}

/// 在世界中放置方块（oif-sim 网格）
pub fn place_block(
    world: &mut oif_sim::WorldBlocks,
    pos: IVec3,
    kind: BlockKind,
    facing: Facing,
) -> Result<(), String> {
    if !world.can_place_block_kind_at(pos, kind) {
        return Err(format!(
            "cannot place {kind:?} at ({}, {}, {})",
            pos.x, pos.y, pos.z
        ));
    }
    world.insert(pos, BlockData::new(kind, facing));
    refresh_static_generated_markers(world);
    Ok(())
}

/// 把存档载入无头会话
pub fn load_save_into_session(core: &mut SimSession, name: &str) -> Result<(), String> {
    reset_session(core);
    let mut world = WorldBlocks(std::mem::take(&mut core.world));
    let slot = SaveSlot::from_storage_path(name)
        .ok_or_else(|| format!("invalid save path `{name}`"))?;
    load_world(&mut world, &slot).ok_or_else(|| format!("save `{name}` not found"))?;
    refresh_static_generated_markers(&mut world);
    core.world = world.0;
    Ok(())
}

/// e2e fixture 根目录
pub fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("e2e/fixtures")
}

/// 解析 fixture 路径（相对则相对 fixture 根）
pub fn resolve_fixture_path(path: &str) -> PathBuf {
    let path = Path::new(path);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        fixture_root().join(path)
    }
}

use super::snapshot::block_layer;

/// 方块种类列表 JSON
pub fn block_kinds_json() -> String {
    let kinds: Vec<_> = all_blocks()
        .into_iter()
        .map(|kind| {
            serde_json::json!({
                "kind": format!("{:?}", kind),
                "layer": block_layer(kind),
                "directional": kind.is_directional(),
            })
        })
        .collect();
    serde_json::json!({ "ok": true, "kinds": kinds }).to_string()
}
