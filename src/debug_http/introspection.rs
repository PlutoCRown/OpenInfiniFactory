use bevy::prelude::*;
use serde_json::{json, Value};
use std::collections::HashMap;

use crate::debug_http::snapshot::{block_json, pos_json};
use crate::game::simulation::movement_plan::movement_plan_debug_json;
use crate::game::simulation::runtime::{detector_is_active_public, SignalNetworkCache};
use crate::game::simulation::structure_state::{
    FactoryActivity, StructureFreedom, StructureKind, StructureState,
};
use crate::game::simulation::structures::MovementInfluenceCache;
use crate::game::simulation::movement::PusherState;
use crate::game::world::grid::WorldBlocks;
use crate::sim_core::SimulationControl;

fn activity_label(activity: FactoryActivity) -> &'static str {
    match activity {
        FactoryActivity::Active => "Active",
        FactoryActivity::Inactive => "Inactive",
    }
}

fn freedom_label(freedom: StructureFreedom) -> &'static str {
    match freedom {
        StructureFreedom::None => "None",
        StructureFreedom::All => "All",
    }
}

fn kind_label(kind: StructureKind) -> &'static str {
    match kind {
        StructureKind::Material => "Material",
        StructureKind::Factory => "Factory",
    }
}

fn parse_region(params: &HashMap<String, String>) -> Result<(IVec3, IVec3), String> {
    let min_x: i32 = params
        .get("min_x")
        .or_else(|| params.get("minx"))
        .and_then(|v| v.parse().ok())
        .ok_or("missing min_x")?;
    let min_y: i32 = params
        .get("min_y")
        .or_else(|| params.get("miny"))
        .and_then(|v| v.parse().ok())
        .ok_or("missing min_y")?;
    let min_z: i32 = params
        .get("min_z")
        .or_else(|| params.get("minz"))
        .and_then(|v| v.parse().ok())
        .ok_or("missing min_z")?;
    let max_x: i32 = params
        .get("max_x")
        .or_else(|| params.get("maxx"))
        .and_then(|v| v.parse().ok())
        .ok_or("missing max_x")?;
    let max_y: i32 = params
        .get("max_y")
        .or_else(|| params.get("maxy"))
        .and_then(|v| v.parse().ok())
        .ok_or("missing max_y")?;
    let max_z: i32 = params
        .get("max_z")
        .or_else(|| params.get("maxz"))
        .and_then(|v| v.parse().ok())
        .ok_or("missing max_z")?;
    Ok((
        IVec3::new(min_x.min(max_x), min_y.min(max_y), min_z.min(max_z)),
        IVec3::new(min_x.max(max_x), min_y.max(max_y), min_z.max(max_z)),
    ))
}

fn offset_json(offset: IVec3) -> Value {
    json!({ "x": offset.x, "y": offset.y, "z": offset.z })
}

fn positions_json(positions: &std::collections::HashSet<IVec3>) -> Value {
    let mut list: Vec<_> = positions.iter().map(|pos| pos_json(*pos)).collect();
    list.sort_by_key(|pos| {
        (
            pos["x"].as_i64().unwrap_or(0),
            pos["y"].as_i64().unwrap_or(0),
            pos["z"].as_i64().unwrap_or(0),
        )
    });
    Value::Array(list)
}

pub fn get_region_json(
    world: &WorldBlocks,
    params: &HashMap<String, String>,
) -> Result<Value, String> {
    let (min, max) = parse_region(params)?;
    let mut blocks = Vec::new();
    for x in min.x..=max.x {
        for y in min.y..=max.y {
            for z in min.z..=max.z {
                let pos = IVec3::new(x, y, z);
                if world.blocks.contains_key(&pos) || world.system_blocks.contains_key(&pos) {
                    blocks.push(json!({
                        "pos": pos_json(pos),
                        "block": block_json(world, pos),
                    }));
                }
            }
        }
    }
    Ok(json!({
        "min": pos_json(min),
        "max": pos_json(max),
        "count": blocks.len(),
        "blocks": blocks,
    }))
}

pub fn get_power_networks_json(
    world: &WorldBlocks,
    signal_cache: &mut SignalNetworkCache,
) -> Value {
    signal_cache.ensure_fresh(world);
    let networks: Vec<_> = (0..signal_cache.network_count())
        .map(|id| {
            let wires = signal_cache.network_wires(id);
            let detector_count = signal_cache.network_detectors(id).map(|d| d.len()).unwrap_or(0);
            json!({
                "id": id,
                "powered": signal_cache.network_is_powered(world, id),
                "wire_count": wires.len(),
                "detector_count": detector_count,
                "device_count": signal_cache.devices_on_network(id).len(),
            })
        })
        .collect();
    json!({ "count": networks.len(), "networks": networks })
}

pub fn get_power_network_json(
    world: &WorldBlocks,
    signal_cache: &mut SignalNetworkCache,
    network_id: usize,
) -> Result<Value, String> {
    signal_cache.ensure_fresh(world);
    if network_id >= signal_cache.network_count() {
        return Err(format!("network id {network_id} out of range"));
    }
    let wires: Vec<_> = signal_cache
        .network_wires(network_id)
        .into_iter()
        .map(|pos| json!({ "pos": pos_json(pos), "block": block_json(world, pos) }))
        .collect();
    let detectors: Vec<_> = signal_cache
        .network_detectors(network_id)
        .map(|detectors| {
            detectors
                .iter()
                .map(|pos| {
                    json!({
                        "pos": pos_json(*pos),
                        "block": block_json(world, *pos),
                        "active": detector_is_active_public(world, *pos),
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let powered = signal_cache.powered_device_positions(world);
    let devices: Vec<_> = signal_cache
        .devices_on_network(network_id)
        .into_iter()
        .map(|pos| {
            json!({
                "pos": pos_json(pos),
                "block": block_json(world, pos),
                "powered": powered.contains(&pos),
                "network_ids": signal_cache.device_network_ids(pos),
            })
        })
        .collect();
    Ok(json!({
        "id": network_id,
        "powered": signal_cache.network_is_powered(world, network_id),
        "wires": wires,
        "detectors": detectors,
        "devices": devices,
    }))
}

pub fn get_powered_devices_json(
    world: &WorldBlocks,
    signal_cache: &mut SignalNetworkCache,
) -> Value {
    signal_cache.ensure_fresh(world);
    let powered: Vec<_> = signal_cache
        .powered_device_positions(world)
        .into_iter()
        .map(|pos| {
            json!({
                "pos": pos_json(pos),
                "block": block_json(world, pos),
                "network_ids": signal_cache.device_network_ids(pos),
            })
        })
        .collect();
    json!({ "count": powered.len(), "devices": powered })
}

pub fn get_factory_block_state_json(
    pos: IVec3,
    world: &WorldBlocks,
    turn_structures: &StructureState,
    solution_structures: &StructureState,
    control: &SimulationControl,
    signal_cache: &mut SignalNetworkCache,
) -> Result<Value, String> {
    signal_cache.ensure_fresh(world);
    let index = turn_structures
        .structure_index_at(pos)
        .ok_or_else(|| format!("no structure at ({},{},{})", pos.x, pos.y, pos.z))?;
    let empty_solution = WorldBlocks::default();
    let solution = control.start_snapshot.as_ref().unwrap_or(&empty_solution);
    let mut pusher_debug = Value::Null;
    for (pusher_pos, block) in &world.blocks {
        if !matches!(
            block.kind,
            crate::game::blocks::BlockKind::Pusher | crate::game::blocks::BlockKind::Blocker
        ) {
            continue;
        }
        let source = *pusher_pos + block.facing.forward_ivec3();
        if source != pos {
            continue;
        }
        let offset = block.facing.forward_ivec3();
        let turn_subset = solution_structures.pusher_target_structure(
            turn_structures,
            solution,
            world,
            *pusher_pos,
            source,
            offset,
        );
        let solution_subset = solution_structures.pusher_target_structure(
            turn_structures,
            solution,
            solution,
            *pusher_pos,
            source,
            offset,
        );
        pusher_debug = json!({
            "pusher": pos_json(*pusher_pos),
            "target": pos_json(source),
            "offset": offset_json(offset),
            "turn_subset": turn_subset.as_ref().map(positions_json),
            "solution_subset": solution_subset.as_ref().map(positions_json),
            "turn_can_push": turn_subset.is_some(),
            "solution_can_push": solution_subset.is_some(),
        });
        break;
    }
    Ok(json!({
        "pos": pos_json(pos),
        "block": block_json(world, pos),
        "structure_index": index,
        "kind": turn_structures.kind_at(pos).map(kind_label),
        "activity": turn_structures.activity_at(pos).map(activity_label),
        "freedom": turn_structures.freedom_at(pos).map(freedom_label),
        "pushable": turn_structures.pushable_at(pos),
        "member_count": turn_structures.member_count_at(pos),
        "wire_network_id": signal_cache.wire_network_id(pos),
        "device_network_ids": signal_cache.device_network_ids(pos),
        "pusher_push": pusher_debug,
    }))
}

pub fn get_structure_at_json(pos: IVec3, turn_structures: &StructureState) -> Result<Value, String> {
    let index = turn_structures
        .structure_index_at(pos)
        .ok_or_else(|| format!("no structure at ({},{},{})", pos.x, pos.y, pos.z))?;
    let positions = turn_structures
        .structure_positions(index)
        .ok_or_else(|| format!("missing structure index {index}"))?;
    Ok(json!({
        "index": index,
        "query": pos_json(pos),
        "kind": turn_structures.kind_at(pos).map(kind_label),
        "activity": turn_structures.activity_at(pos).map(activity_label),
        "freedom": turn_structures.freedom_at(pos).map(freedom_label),
        "pushable": turn_structures.pushable_at(pos),
        "member_count": positions.len(),
        "members": positions_json(positions),
    }))
}

pub fn preview_movement_plan_json(
    world: &WorldBlocks,
    turn_structures: &StructureState,
    control: &SimulationControl,
    signal_cache: &mut SignalNetworkCache,
    pusher_state: &PusherState,
    movement_influence: &mut MovementInfluenceCache,
) -> Value {
    let solution = control
        .start_snapshot
        .clone()
        .unwrap_or_else(|| world.clone());
    let solution_structures = control
        .start_structures
        .clone()
        .unwrap_or_else(|| turn_structures.clone());
    movement_plan_debug_json(
        world,
        turn_structures,
        &solution,
        &solution_structures,
        signal_cache,
        pusher_state,
        movement_influence,
        control.turn,
    )
}
