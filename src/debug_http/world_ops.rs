use bevy::prelude::*;
use std::path::{Path, PathBuf};

use crate::game::blocks::{all_blocks, BlockData, BlockKind};
use crate::game::simulation::markers::refresh_static_generated_markers;
use crate::game::world::direction::Facing;
use crate::game::world::grid::WorldBlocks;
use crate::shared::save::{load_world, SaveSlot};
use crate::sim_core::SimCoreWorld;

pub fn parse_block_kind(name: &str) -> Option<BlockKind> {
    all_blocks()
        .into_iter()
        .find(|kind| format!("{:?}", kind).eq_ignore_ascii_case(name.trim()))
}

pub fn parse_facing(name: &str) -> Option<Facing> {
    match name.trim().to_ascii_lowercase().as_str() {
        "north" | "n" => Some(Facing::North),
        "east" | "e" => Some(Facing::East),
        "south" | "s" => Some(Facing::South),
        "west" | "w" => Some(Facing::West),
        _ => None,
    }
}

pub fn reset_session(core: &mut SimCoreWorld<'_>) {
    core.reset();
}

pub fn place_block(
    world: &mut WorldBlocks,
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

pub fn load_save_into_session(core: &mut SimCoreWorld<'_>, name: &str) -> Result<(), String> {
    reset_session(core);
    let mut world = core.world_blocks_mut();
    let slot = SaveSlot::from_storage_path(name)
        .ok_or_else(|| format!("invalid save path `{name}`"))?;
    load_world(&mut world, &slot).ok_or_else(|| format!("save `{name}` not found"))?;
    refresh_static_generated_markers(&mut world);
    Ok(())
}

pub fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("e2e/fixtures")
}

pub fn resolve_fixture_path(path: &str) -> PathBuf {
    let path = Path::new(path);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        fixture_root().join(path)
    }
}

use super::snapshot::block_layer;

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
