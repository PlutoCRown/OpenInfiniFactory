use bevy::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};

use crate::game::blocks::{BlockData, BlockKind};
use crate::game::simulation::structure_state::StructureState;
use crate::game::systems::debug::DebugState;
use crate::game::world::animation::{AnimationTiming, BlockAnimation, PusherAnimation};
use crate::game::world::block_instance::{BlockInstanceId, MaterialBlockRegistry};
use crate::game::world::factory_registry::{FactoryBlockId, FactoryBlockRegistry};
use crate::game::world::grid::WorldBlocks;
use crate::game::world::rendering::{
    rebuild_world, signal_neighbor_offsets, spawn_acceptance_sparks, spawn_laser_beams,
    spawn_weld_sparks, spawn_world_block_entity, BlockEntity, WorldRenderAssets,
};
use crate::sim_core::TurnOutput;

use super::block_entities::BlockEntityTracker;

pub fn block_data_at(world: &WorldBlocks, pos: IVec3) -> Option<BlockData> {
    world
        .blocks
        .get(&pos)
        .copied()
        .or_else(|| world.system_blocks.get(&pos).copied())
}

pub fn refresh_edit_changes(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    blocks: &Query<(Entity, &BlockEntity)>,
    world: &WorldBlocks,
    assets: &WorldRenderAssets,
    debug: &DebugState,
    structure_state: &StructureState,
    factory_registry: &FactoryBlockRegistry,
    material_registry: &MaterialBlockRegistry,
    changed: &HashSet<IVec3>,
) {
    let mut tracker = BlockEntityTracker::capture(blocks);
    let refresh = collect_edit_refresh_positions(world, changed);
    refresh_positions(
        commands,
        meshes,
        &mut tracker,
        world,
        assets,
        debug,
        structure_state,
        factory_registry,
        material_registry,
        &HashSet::new(),
        &refresh,
        &HashSet::new(),
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
    presentation_duration: f32,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    tracker: &mut BlockEntityTracker,
    assets: &WorldRenderAssets,
    debug: &DebugState,
    structure_state: &StructureState,
    factory_registry: &FactoryBlockRegistry,
    material_registry: &MaterialBlockRegistry,
    stats: &mut crate::game::simulation::runtime::SimulationStepStats,
) {
    let render_start = bevy::platform::time::Instant::now();
    let timing = AnimationTiming::simulation(presentation_duration);
    let mut refresh = collect_sim_refresh_positions(before, after, output, factory_registry);
    refresh.extend(collect_wire_power_refresh_positions(
        after,
        &output.render_powered_wires,
        previous_powered_wires,
    ));
    sync_turn_block_entities(
        commands,
        meshes,
        tracker,
        after,
        assets,
        debug,
        structure_state,
        factory_registry,
        material_registry,
        &output.render_powered_wires,
        &output.animations,
        &refresh,
        &output.pusher_animations,
        timing,
    );
    spawn_weld_sparks(commands, assets, &output.weld_sparks);
    spawn_weld_sparks(commands, assets, &output.behavior_sparks);
    spawn_laser_beams(
        commands,
        assets,
        &output.laser_beams,
        presentation_duration * 0.5,
    );
    spawn_acceptance_sparks(commands, assets, &output.acceptance_sparks);
    stats.render_rebuild_ms = render_start.elapsed().as_secs_f64() * 1000.0;
    stats.total_ms += stats.render_rebuild_ms;
}

pub fn resync_world_rendering(
    world: &WorldBlocks,
    output: &TurnOutput,
    presentation_duration: f32,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    blocks: &Query<(Entity, &BlockEntity)>,
    assets: &WorldRenderAssets,
    debug: &DebugState,
    structure_state: &StructureState,
    factory_registry: &FactoryBlockRegistry,
    material_registry: &MaterialBlockRegistry,
    stats: &mut crate::game::simulation::runtime::SimulationStepStats,
) {
    let render_start = bevy::platform::time::Instant::now();
    for (entity, _) in blocks.iter() {
        commands.entity(entity).despawn();
    }
    rebuild_world(
        commands,
        meshes,
        world,
        assets,
        debug.factory_activity.then_some(structure_state),
        factory_registry,
        material_registry,
    );
    spawn_weld_sparks(commands, assets, &output.weld_sparks);
    spawn_weld_sparks(commands, assets, &output.behavior_sparks);
    spawn_laser_beams(
        commands,
        assets,
        &output.laser_beams,
        presentation_duration * 0.5,
    );
    spawn_acceptance_sparks(commands, assets, &output.acceptance_sparks);
    stats.render_rebuild_ms = render_start.elapsed().as_secs_f64() * 1000.0;
    stats.total_ms += stats.render_rebuild_ms;
}

pub fn snap_block_entities_to_world(
    world: &WorldBlocks,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    blocks: &Query<(Entity, &BlockEntity)>,
    assets: &WorldRenderAssets,
    debug: &DebugState,
    structure_state: &StructureState,
    factory_registry: &FactoryBlockRegistry,
    material_registry: &MaterialBlockRegistry,
) {
    for (entity, _) in blocks.iter() {
        commands.entity(entity).despawn();
    }
    rebuild_world(
        commands,
        meshes,
        world,
        assets,
        debug.factory_activity.then_some(structure_state),
        factory_registry,
        material_registry,
    );
}

enum BlockVisualUpdate {
    Animated(BlockAnimation),
    Static(IVec3),
}

fn sync_turn_block_entities(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    tracker: &mut BlockEntityTracker,
    world: &WorldBlocks,
    assets: &WorldRenderAssets,
    debug: &DebugState,
    structure_state: &StructureState,
    factory_registry: &FactoryBlockRegistry,
    material_registry: &MaterialBlockRegistry,
    powered_wires: &HashSet<IVec3>,
    animations: &HashMap<BlockInstanceId, BlockAnimation>,
    refresh_positions: &HashSet<IVec3>,
    pusher_animations: &HashMap<FactoryBlockId, PusherAnimation>,
    timing: AnimationTiming,
) {
    tracker.dedupe_duplicate_ids(commands);
    let factory_debug = debug.factory_activity.then_some(structure_state);
    let mut updates: HashMap<BlockInstanceId, BlockVisualUpdate> = HashMap::new();

    for (&block_id, animation) in animations {
        if block_data_at(world, animation.to_pos).is_some() {
            updates.insert(block_id, BlockVisualUpdate::Animated(*animation));
        }
    }

    for &pos in refresh_positions {
        let Some(block_id) =
            BlockInstanceId::resolve(world, factory_registry, material_registry, pos)
        else {
            tracker.despawn_at_pos(commands, pos);
            continue;
        };
        if updates.contains_key(&block_id) {
            continue;
        }
        if block_data_at(world, pos).is_none() {
            tracker.despawn_id(commands, block_id);
            continue;
        }
        updates.insert(block_id, BlockVisualUpdate::Static(pos));
    }

    for (&pos, data) in &world.blocks {
        if !data.kind.is_material() {
            continue;
        }
        let Some(block_id) =
            BlockInstanceId::resolve(world, factory_registry, material_registry, pos)
        else {
            continue;
        };
        if updates.contains_key(&block_id) || tracker.has_id(block_id) {
            continue;
        }
        updates.insert(block_id, BlockVisualUpdate::Static(pos));
    }

    for (block_id, update) in updates {
        tracker.despawn_id(commands, block_id);
        match update {
            BlockVisualUpdate::Animated(animation) => {
                if animation.from_pos != animation.to_pos {
                    tracker.despawn_at_pos(commands, animation.from_pos);
                }
                let data = block_data_at(world, animation.to_pos).expect("animated block");
                let pusher_animation = scaled_pusher_animation_at(
                    animation.to_pos,
                    factory_registry,
                    pusher_animations,
                    timing,
                );
                spawn_world_block_entity(
                    commands,
                    meshes,
                    assets,
                    world,
                    animation.to_pos,
                    data,
                    block_id,
                    Some(animation),
                    pusher_animation,
                    timing,
                    powered_wires.contains(&animation.to_pos),
                    factory_debug,
                );
            }
            BlockVisualUpdate::Static(pos) => {
                let data = block_data_at(world, pos).expect("static block");
                let pusher_animation =
                    scaled_pusher_animation_at(pos, factory_registry, pusher_animations, timing);
                spawn_world_block_entity(
                    commands,
                    meshes,
                    assets,
                    world,
                    pos,
                    data,
                    block_id,
                    None,
                    pusher_animation,
                    timing,
                    powered_wires.contains(&pos),
                    factory_debug,
                );
            }
        }
        tracker.mark_spawned(block_id);
    }
}

fn scaled_pusher_animation_at(
    pos: IVec3,
    factory_registry: &FactoryBlockRegistry,
    pusher_animations: &HashMap<FactoryBlockId, PusherAnimation>,
    timing: AnimationTiming,
) -> Option<PusherAnimation> {
    let mut animation = pusher_animation_at(pos, factory_registry, pusher_animations)?;
    animation.duration = timing.duration;
    Some(animation)
}

fn pusher_animation_at(
    pos: IVec3,
    factory_registry: &FactoryBlockRegistry,
    pusher_animations: &HashMap<FactoryBlockId, PusherAnimation>,
) -> Option<PusherAnimation> {
    factory_registry
        .turn_id_at(pos)
        .and_then(|id| pusher_animations.get(&id).copied())
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
    factory_registry: &FactoryBlockRegistry,
) -> HashSet<IVec3> {
    let changed = diff_block_positions(before, after);
    let animated_destinations: HashSet<IVec3> = output
        .animations
        .values()
        .map(|animation| animation.to_pos)
        .collect();
    let mut refresh = HashSet::new();
    for animation in output.animations.values() {
        if animation.from_pos != animation.to_pos {
            refresh.insert(animation.from_pos);
        }
    }
    for &pusher_id in output.pusher_animations.keys() {
        if let Some(pos) = factory_registry.turn_pos(pusher_id) {
            refresh.insert(pos);
        }
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
    let mut seeds = HashSet::new();
    for &pos in current {
        refresh.insert(pos);
        seeds.insert(pos);
    }
    for &pos in previous {
        if block_data_at(after, pos).is_some_and(|block| block.kind == BlockKind::Wire) {
            refresh.insert(pos);
            seeds.insert(pos);
        }
    }
    for &pos in &seeds {
        insert_connectivity_neighbors(after, &mut refresh, pos);
    }
    refresh
}

pub fn refresh_positions(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    tracker: &mut BlockEntityTracker,
    world: &WorldBlocks,
    assets: &WorldRenderAssets,
    debug: &DebugState,
    structure_state: &StructureState,
    factory_registry: &FactoryBlockRegistry,
    material_registry: &MaterialBlockRegistry,
    powered_wires: &HashSet<IVec3>,
    positions: &HashSet<IVec3>,
    skip_ids: &HashSet<BlockInstanceId>,
    skip_spawn_positions: &HashSet<IVec3>,
    pusher_animations: &HashMap<FactoryBlockId, PusherAnimation>,
    timing: AnimationTiming,
) {
    let factory_debug = debug.factory_activity.then_some(structure_state);
    for &pos in positions {
        let Some(block_id) =
            BlockInstanceId::resolve(world, factory_registry, material_registry, pos)
        else {
            tracker.despawn_at_pos(commands, pos);
            continue;
        };
        if skip_ids.contains(&block_id) || skip_spawn_positions.contains(&pos) {
            continue;
        }
        tracker.despawn_id(commands, block_id);
        let Some(data) = block_data_at(world, pos) else {
            continue;
        };
        spawn_world_block_entity(
            commands,
            meshes,
            assets,
            world,
            pos,
            data,
            block_id,
            None,
            pusher_animation_at(pos, factory_registry, pusher_animations),
            timing,
            powered_wires.contains(&pos),
            factory_debug,
        );
        tracker.mark_spawned(block_id);
    }
}
