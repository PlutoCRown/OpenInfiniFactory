use bevy::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};

use crate::game::blocks::{BlockData, BlockKind};
use crate::game::simulation::structure_state::StructureState;
use crate::game::systems::debug::DebugState;
use crate::game::world::animation::{AnimationTiming, BlockAnimation, PusherAnimation};
use crate::game::world::grid::WorldBlocks;
use crate::game::world::rendering::{
    signal_neighbor_offsets, spawn_acceptance_sparks, spawn_laser_beams, spawn_weld_sparks,
    spawn_world_block_entity, WorldRenderAssets,
};
use crate::sim_core::TurnOutput;

use super::entity_index::BlockEntityIndex;

pub fn block_data_at(world: &WorldBlocks, pos: IVec3) -> Option<BlockData> {
    world
        .blocks
        .get(&pos)
        .copied()
        .or_else(|| world.system_blocks.get(&pos).copied())
}

pub fn despawn_block_at(commands: &mut Commands, index: &mut BlockEntityIndex, pos: IVec3) {
    if let Some(entity) = index.remove(pos) {
        commands.entity(entity).despawn();
    }
}

fn needs_connectivity_refresh(data: BlockData) -> bool {
    if data.kind.is_factory() || data.kind.is_system_layer() || data.kind == BlockKind::Wire {
        return true;
    }
    let behavior = data.kind.render_behavior(data.facing);
    behavior.wire_connector.is_some() || behavior.weld_connector.is_some()
}

fn insert_connectivity_neighbors(world: &WorldBlocks, refresh: &mut HashSet<IVec3>, pos: IVec3) {
    for offset in signal_neighbor_offsets() {
        let neighbor = pos + offset;
        if block_data_at(world, neighbor).is_some_and(|block| block.kind.is_material()) {
            continue;
        }
        refresh.insert(neighbor);
    }
}

fn expand_wire_connectivity(world: &WorldBlocks, seeds: &HashSet<IVec3>, out: &mut HashSet<IVec3>) {
    let mut queue: VecDeque<IVec3> = seeds
        .iter()
        .filter(|&&pos| block_data_at(world, pos).is_some_and(|b| b.kind == BlockKind::Wire))
        .copied()
        .collect();
    while let Some(pos) = queue.pop_front() {
        if !out.insert(pos) {
            continue;
        }
        for offset in signal_neighbor_offsets() {
            let neighbor = pos + offset;
            if block_data_at(world, neighbor).is_some_and(|block| block.kind.is_material()) {
                continue;
            }
            out.insert(neighbor);
            if block_data_at(world, neighbor).is_some_and(|b| b.kind == BlockKind::Wire) {
                queue.push_back(neighbor);
            }
        }
    }
}

fn expand_local_neighborhood(positions: &HashSet<IVec3>) -> HashSet<IVec3> {
    let mut expanded = HashSet::new();
    for &pos in positions {
        expanded.insert(pos);
        for offset in signal_neighbor_offsets() {
            expanded.insert(pos + offset);
        }
    }
    expanded
}

pub fn collect_edit_refresh_positions(
    world: &WorldBlocks,
    changed: &HashSet<IVec3>,
) -> HashSet<IVec3> {
    let mut refresh = expand_local_neighborhood(changed);
    expand_wire_connectivity(world, changed, &mut refresh);
    refresh
}

pub fn diff_block_positions(before: &WorldBlocks, after: &WorldBlocks) -> HashSet<IVec3> {
    let mut changed = HashSet::new();
    for pos in before
        .blocks
        .keys()
        .chain(before.system_blocks.keys())
        .chain(after.blocks.keys())
        .chain(after.system_blocks.keys())
    {
        let prev = block_data_at(before, *pos);
        let next = block_data_at(after, *pos);
        if prev != next {
            changed.insert(*pos);
        }
    }
    changed
}

pub fn collect_sim_refresh_positions(
    before: &WorldBlocks,
    after: &WorldBlocks,
    output: &TurnOutput,
) -> HashSet<IVec3> {
    let changed = diff_block_positions(before, after);
    let animated_destinations: HashSet<IVec3> = output.animations.keys().copied().collect();
    let mut refresh = HashSet::new();
    for pos in output.pusher_animations.keys() {
        refresh.insert(*pos);
    }
    for &pos in &changed {
        if block_data_at(after, pos).is_some_and(|block| block.kind.is_material()) {
            if !animated_destinations.contains(&pos) {
                refresh.insert(pos);
            }
            continue;
        }
        refresh.insert(pos);
        if block_data_at(after, pos).is_some_and(needs_connectivity_refresh) {
            insert_connectivity_neighbors(after, &mut refresh, pos);
        } else if block_data_at(before, pos).is_some_and(needs_connectivity_refresh) {
            insert_connectivity_neighbors(after, &mut refresh, pos);
        }
    }
    expand_wire_connectivity(after, &changed, &mut refresh);
    refresh
}

pub fn collect_wire_power_refresh_positions(
    after: &WorldBlocks,
    current: &HashSet<IVec3>,
    previous: &HashSet<IVec3>,
) -> HashSet<IVec3> {
    let mut refresh = HashSet::new();
    for &pos in current {
        refresh.insert(pos);
    }
    for &pos in previous {
        if block_data_at(after, pos).is_some_and(|block| block.kind == BlockKind::Wire) {
            refresh.insert(pos);
        }
    }
    refresh
}

pub fn refresh_positions(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    index: &mut BlockEntityIndex,
    world: &WorldBlocks,
    assets: &WorldRenderAssets,
    debug: &DebugState,
    structure_state: &StructureState,
    powered_wires: &HashSet<IVec3>,
    positions: &HashSet<IVec3>,
    skip: &HashSet<IVec3>,
    pusher_animations: &HashMap<IVec3, PusherAnimation>,
    timing: AnimationTiming,
) {
    let factory_debug = debug.factory_activity.then_some(structure_state);
    for &pos in positions {
        if skip.contains(&pos) {
            continue;
        }
        despawn_block_at(commands, index, pos);
        let Some(data) = block_data_at(world, pos) else {
            continue;
        };
        let entity = spawn_world_block_entity(
            commands,
            meshes,
            assets,
            world,
            pos,
            data,
            None,
            pusher_animations.get(&pos).copied(),
            timing,
            powered_wires.contains(&pos),
            factory_debug,
        );
        index.insert(pos, entity);
    }
}

pub fn apply_structure_animations(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    index: &mut BlockEntityIndex,
    world: &WorldBlocks,
    assets: &WorldRenderAssets,
    debug: &DebugState,
    structure_state: &StructureState,
    powered_wires: &HashSet<IVec3>,
    animations: &HashMap<IVec3, BlockAnimation>,
    pusher_animations: &HashMap<IVec3, PusherAnimation>,
    timing: AnimationTiming,
) -> HashSet<IVec3> {
    let factory_debug = debug.factory_activity.then_some(structure_state);
    let destinations: HashSet<IVec3> = animations.keys().copied().collect();
    let mut handled = HashSet::new();
    for (&pos, animation) in animations {
        let Some(data) = block_data_at(world, pos) else {
            continue;
        };
        handled.insert(pos);
        handled.insert(animation.from_pos);
        let from_is_also_destination =
            animation.from_pos != pos && destinations.contains(&animation.from_pos);
        if !from_is_also_destination {
            despawn_block_at(commands, index, animation.from_pos);
        }
        if animation.from_pos != pos {
            despawn_block_at(commands, index, pos);
        }
        let entity = spawn_world_block_entity(
            commands,
            meshes,
            assets,
            world,
            pos,
            data,
            Some(*animation),
            pusher_animations.get(&pos).copied(),
            timing,
            powered_wires.contains(&pos),
            factory_debug,
        );
        index.insert(pos, entity);
    }
    handled
}

pub fn refresh_edit_changes(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    index: &mut BlockEntityIndex,
    world: &WorldBlocks,
    assets: &WorldRenderAssets,
    debug: &DebugState,
    structure_state: &StructureState,
    changed: &HashSet<IVec3>,
) {
    let refresh = collect_edit_refresh_positions(world, changed);
    refresh_positions(
        commands,
        meshes,
        index,
        world,
        assets,
        debug,
        structure_state,
        &HashSet::new(),
        &refresh,
        &HashSet::new(),
        &HashMap::new(),
        AnimationTiming::edit(),
    );
}

pub fn apply_turn_output_incremental(
    before: &WorldBlocks,
    after: &WorldBlocks,
    output: &TurnOutput,
    previous_powered_wires: &HashSet<IVec3>,
    animation_duration: f32,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    index: &mut BlockEntityIndex,
    assets: &WorldRenderAssets,
    debug: &DebugState,
    structure_state: &StructureState,
    stats: &mut crate::game::simulation::runtime::SimulationStepStats,
) {
    let render_start = bevy::platform::time::Instant::now();
    let timing = AnimationTiming::simulation(animation_duration);
    let animated = apply_structure_animations(
        commands,
        meshes,
        index,
        after,
        assets,
        debug,
        structure_state,
        &output.render_powered_wires,
        &output.animations,
        &output.pusher_animations,
        timing,
    );
    let mut refresh = collect_sim_refresh_positions(before, after, output);
    refresh.extend(collect_wire_power_refresh_positions(
        after,
        &output.render_powered_wires,
        previous_powered_wires,
    ));
    refresh_positions(
        commands,
        meshes,
        index,
        after,
        assets,
        debug,
        structure_state,
        &output.render_powered_wires,
        &refresh,
        &animated,
        &output.pusher_animations,
        timing,
    );
    for (&pos, data) in &after.blocks {
        if !data.kind.is_material() || index.get(pos).is_some() {
            continue;
        }
        let entity = spawn_world_block_entity(
            commands,
            meshes,
            assets,
            after,
            pos,
            *data,
            None,
            output.pusher_animations.get(&pos).copied(),
            timing,
            output.render_powered_wires.contains(&pos),
            debug.factory_activity.then_some(structure_state),
        );
        index.insert(pos, entity);
    }
    spawn_weld_sparks(commands, assets, &output.weld_sparks);
    spawn_weld_sparks(commands, assets, &output.behavior_sparks);
    spawn_laser_beams(commands, assets, &output.laser_beams, animation_duration);
    spawn_acceptance_sparks(commands, assets, &output.acceptance_sparks);
    stats.render_rebuild_ms = render_start.elapsed().as_secs_f64() * 1000.0;
    stats.total_ms += stats.render_rebuild_ms;
}
