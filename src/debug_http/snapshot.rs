use bevy::prelude::*;
use serde_json::{json, Value};

use crate::game::blocks::BlockKind;
use crate::game::state::{BuilderMode, PlacementState, SimulationState};
use crate::game::world::direction::Facing;
use crate::game::world::grid::{TargetHit, WorldBlocks};
use crate::sim_core::SimulationControl;

pub fn block_json(world: &WorldBlocks, pos: IVec3) -> Value {
    if let Some(block) = world.blocks.get(&pos) {
        json!({
            "layer": block_layer(block.kind),
            "kind": format!("{:?}", block.kind),
            "facing": format!("{:?}", block.facing),
        })
    } else if let Some(block) = world.system_blocks.get(&pos) {
        json!({
            "layer": "system",
            "kind": format!("{:?}", block.kind),
            "facing": format!("{:?}", block.facing),
        })
    } else {
        Value::Null
    }
}

pub fn block_layer(kind: BlockKind) -> &'static str {
    if kind.is_scene() {
        "scene"
    } else if kind.is_factory() {
        "factory"
    } else if kind.is_material() {
        "material"
    } else if kind.is_system_layer() && kind.is_system_block() {
        "system"
    } else {
        "virtual"
    }
}

pub fn pos_json(pos: IVec3) -> Value {
    json!({ "x": pos.x, "y": pos.y, "z": pos.z })
}

pub fn target_hit_json(world: &WorldBlocks, hit: &TargetHit) -> Value {
    json!({
        "pos": pos_json(hit.pos),
        "normal": pos_json(hit.normal),
        "block": block_json(world, hit.pos),
        "place_at": pos_json(hit.pos + hit.normal),
    })
}

pub fn cursor_target_json(placement: &PlacementState, world: &WorldBlocks) -> Value {
    match placement.target.as_ref() {
        Some(hit) => target_hit_json(world, hit),
        None => Value::Null,
    }
}

pub fn simulation_status_json(
    simulation: &SimulationState,
    builder_mode: BuilderMode,
) -> Value {
    json!({
        "builder_mode": match builder_mode {
            BuilderMode::Edit => "Edit",
            BuilderMode::Play => "Play",
        },
        "active": simulation.is_active(),
        "running": simulation.running,
        "step_requested": simulation.step_requested,
        "turn": simulation.turn,
        "speed": simulation.speed,
    })
}

pub fn session_status_json(control: &SimulationControl) -> Value {
    json!({
        "mode": "headless",
        "active": control.is_active(),
        "running": control.running,
        "step_requested": control.step_requested,
        "turn": control.turn,
        "speed": control.speed,
    })
}

pub fn facing_label(facing: Facing) -> &'static str {
    match facing {
        Facing::North => "N",
        Facing::East => "E",
        Facing::South => "S",
        Facing::West => "W",
    }
}

pub fn target_status_line(placement: &PlacementState, world: &WorldBlocks) -> String {
    let Some(hit) = placement.target.as_ref() else {
        return "Target: —".into();
    };
    let place_at = hit.pos + hit.normal;
    let block_label = world
        .blocks
        .get(&hit.pos)
        .map(|block| format!("{:?}", block.kind))
        .or_else(|| {
            world
                .system_blocks
                .get(&hit.pos)
                .map(|block| format!("{:?}", block.kind))
        })
        .unwrap_or_else(|| "Scene".into());
    let facing = world
        .blocks
        .get(&hit.pos)
        .or_else(|| world.system_blocks.get(&hit.pos))
        .map(|block| facing_label(block.facing))
        .unwrap_or("-");
    format!(
        "Target: ({}, {}, {}) {} [{facing}]   Place: ({}, {}, {})",
        hit.pos.x, hit.pos.y, hit.pos.z, block_label, place_at.x, place_at.y, place_at.z
    )
}
