use bevy::prelude::IVec3;

use crate::game::blocks::{BlockData, BlockKind, all_blocks};
use crate::game::simulation::markers::refresh_static_generated_markers;
use crate::game::world::direction::Facing;
use crate::game::world::grid::WorldBlocks;
use crate::shared::save::{load_world, SaveSlot};
use oif_sim::SimSession;

/// 解析方块种类名（场景 / 材料 / 印花字符串 id，或工厂枚举 Debug 名）
pub fn parse_block_kind(name: &str) -> Option<BlockKind> {
    parse_block_kind_exact(name).or_else(|| {
        // 未识别：材料兜底
        Some(BlockKind::Material(oif_sim::blocks::fallback_material_id()))
    })
}

/// 精确解析查询用的方块种类，不把未知名称静默转换为兜底材料
pub fn parse_block_kind_exact(name: &str) -> Option<BlockKind> {
    let name = name.trim();
    if name.is_empty() {
        return None;
    }
    let lower = name.to_ascii_lowercase();
    oif_sim::blocks::ensure_fallback_scene_catalog();
    oif_sim::blocks::ensure_fallback_material_catalog();
    oif_sim::blocks::ensure_fallback_stamp_catalog();
    if oif_sim::blocks::scene_catalog()
        .id_by_string(&lower)
        .is_some()
    {
        return Some(BlockKind::Scene(oif_sim::blocks::resolve_scene_id(&lower)));
    }
    if oif_sim::blocks::material_catalog()
        .id_by_string(&lower)
        .is_some()
    {
        return Some(BlockKind::Material(oif_sim::blocks::resolve_material_id(
            &lower,
        )));
    }
    if let Some(id) = oif_sim::blocks::stamp_catalog().id_by_string(&lower) {
        return Some(BlockKind::Stamp(id));
    }
    if let Some(kind) = all_blocks()
        .into_iter()
        .find(|kind| format!("{:?}", kind).eq_ignore_ascii_case(name))
    {
        return Some(kind);
    }
    None
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

/// 在包容 AABB 内批量放置；返回成功坐标与跳过坐标
pub fn place_blocks_box(
    world: &mut oif_sim::WorldBlocks,
    a: IVec3,
    b: IVec3,
    kind: BlockKind,
    facing: Facing,
) -> (Vec<IVec3>, Vec<IVec3>) {
    let min = IVec3::new(a.x.min(b.x), a.y.min(b.y), a.z.min(b.z));
    let max = IVec3::new(a.x.max(b.x), a.y.max(b.y), a.z.max(b.z));
    let mut placed = Vec::new();
    let mut skipped = Vec::new();
    for x in min.x..=max.x {
        for y in min.y..=max.y {
            for z in min.z..=max.z {
                let pos = IVec3::new(x, y, z);
                if !world.can_place_block_kind_at(pos, kind) {
                    skipped.push(pos);
                    continue;
                }
                world.insert(pos, BlockData::new(kind, facing));
                placed.push(pos);
            }
        }
    }
    if !placed.is_empty() {
        refresh_static_generated_markers(world);
    }
    (placed, skipped)
}

/// 把存档载入无头会话，返回加载耗时毫秒
pub fn load_save_into_session(core: &mut SimSession, name: &str) -> Result<f64, String> {
    let started = std::time::Instant::now();
    reset_session(core);
    let mut world = WorldBlocks(std::mem::take(&mut core.world));
    let slot = SaveSlot::from_storage_path(name)
        .ok_or_else(|| format!("invalid save path `{name}`"))?;
    load_world(&mut world, &slot).ok_or_else(|| format!("save `{name}` not found"))?;
    refresh_static_generated_markers(&mut world);
    core.world = world.0;
    Ok(started.elapsed().as_secs_f64() * 1000.0)
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
