use bevy::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};

use crate::game::blocks::{BlockData, BlockKind};
use crate::game::simulation::structure_state::StructureState;
use crate::game::systems::debug::DebugState;
use crate::game::world::animation::{
    AnimatedBlock, AnimationTiming, BlockAnimation, PusherAnimation,
};
use crate::game::world::grid::WorldBlocks;
use crate::game::world::rendering::{
    signal_neighbor_offsets, spawn_acceptance_sparks, spawn_laser_beams, spawn_weld_sparks,
    spawn_world_block_entity, BlockEntity, BlockEntityLayer, WorldRenderAssets,
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

fn despawn_entity(commands: &mut Commands, index: &mut BlockEntityIndex, entity: Entity) {
    index.remove_entity(entity);
    commands.entity(entity).despawn();
}

fn despawn_animatable_at(commands: &mut Commands, index: &mut BlockEntityIndex, pos: IVec3) {
    if let Some(entity) = index.remove_animatable(pos) {
        commands.entity(entity).despawn();
    }
}

fn despawn_system_at(commands: &mut Commands, index: &mut BlockEntityIndex, pos: IVec3) {
    if let Some(entity) = index.remove_system(pos) {
        commands.entity(entity).despawn();
    }
}

fn despawn_scene_at(commands: &mut Commands, index: &mut BlockEntityIndex, pos: IVec3) {
    if let Some(entity) = index.remove_scene(pos) {
        commands.entity(entity).despawn();
    }
}

fn spawn_and_index(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    index: &mut BlockEntityIndex,
    world: &WorldBlocks,
    assets: &WorldRenderAssets,
    pos: IVec3,
    data: BlockData,
    animation: Option<BlockAnimation>,
    pusher_animation: Option<PusherAnimation>,
    timing: AnimationTiming,
    powered_wire: bool,
    factory_debug: Option<&StructureState>,
) {
    let entity = spawn_world_block_entity(
        commands,
        meshes,
        assets,
        world,
        pos,
        data,
        animation,
        pusher_animation,
        timing,
        powered_wire,
        factory_debug,
    );
    index.insert(pos, data.id, BlockEntityLayer::from_kind(data.kind), entity);
}

// 无连接件的工厂/材料实例可原地保留，避免对向移动时未动方块被刷新闪烁
fn can_preserve_animatable(data: BlockData) -> bool {
    let behavior = data.kind.render_behavior(data.facing);
    behavior.wire_connector.is_none() && behavior.weld_connector.is_none()
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
        if world
            .blocks
            .get(&neighbor)
            .is_some_and(|block| block.kind.is_material())
        {
            continue;
        }
        refresh.insert(neighbor);
    }
}

fn expand_wire_connectivity(world: &WorldBlocks, seeds: &HashSet<IVec3>, out: &mut HashSet<IVec3>) {
    let mut queue: VecDeque<IVec3> = seeds
        .iter()
        .filter(|&&pos| {
            world
                .blocks
                .get(&pos)
                .is_some_and(|b| b.kind == BlockKind::Wire)
        })
        .copied()
        .collect();
    while let Some(pos) = queue.pop_front() {
        if !out.insert(pos) {
            continue;
        }
        for offset in signal_neighbor_offsets() {
            let neighbor = pos + offset;
            if world
                .blocks
                .get(&neighbor)
                .is_some_and(|block| block.kind.is_material())
            {
                continue;
            }
            out.insert(neighbor);
            if world
                .blocks
                .get(&neighbor)
                .is_some_and(|b| b.kind == BlockKind::Wire)
            {
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
        let blocks_changed = before.blocks.get(pos) != after.blocks.get(pos);
        let system_changed = before.system_blocks.get(pos) != after.system_blocks.get(pos);
        if blocks_changed || system_changed {
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
    for &pos in &changed {
        if after.blocks.get(&pos).is_some_and(|b| b.kind.is_material()) {
            if !animated_destinations.contains(&pos) {
                refresh.insert(pos);
            }
            continue;
        }
        refresh.insert(pos);
        if let Some(data) = block_data_at(after, pos) {
            if needs_connectivity_refresh(data) {
                insert_connectivity_neighbors(after, &mut refresh, pos);
            }
        }
        if let Some(before_data) = block_data_at(before, pos) {
            if needs_connectivity_refresh(before_data) {
                insert_connectivity_neighbors(before, &mut refresh, pos);
            }
        }
    }
    expand_wire_connectivity(after, &changed, &mut refresh);
    refresh
}

pub fn collect_wire_power_refresh_positions(
    world: &WorldBlocks,
    powered_wires: &HashSet<IVec3>,
    previous_powered_wires: &HashSet<IVec3>,
) -> HashSet<IVec3> {
    let mut refresh = HashSet::new();
    for &pos in powered_wires.symmetric_difference(previous_powered_wires) {
        refresh.insert(pos);
        if world
            .blocks
            .get(&pos)
            .is_some_and(|b| b.kind == BlockKind::Wire)
        {
            for offset in signal_neighbor_offsets() {
                refresh.insert(pos + offset);
            }
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

        // blocks 层（工厂/材料/场景）
        match world.blocks.get(&pos).copied() {
            Some(data) if data.kind.is_factory() || data.kind.is_material() => {
                let same_instance = index
                    .get_by_id(data.id)
                    .is_some_and(|entity| index.get_animatable(pos) == Some(entity));
                if !(same_instance && can_preserve_animatable(data)) {
                    despawn_animatable_at(commands, index, pos);
                    despawn_scene_at(commands, index, pos);
                    spawn_and_index(
                        commands,
                        meshes,
                        index,
                        world,
                        assets,
                        pos,
                        data,
                        None,
                        pusher_animations.get(&pos).copied(),
                        timing,
                        powered_wires.contains(&pos),
                        factory_debug,
                    );
                }
            }
            Some(data) => {
                despawn_animatable_at(commands, index, pos);
                despawn_scene_at(commands, index, pos);
                spawn_and_index(
                    commands,
                    meshes,
                    index,
                    world,
                    assets,
                    pos,
                    data,
                    None,
                    None,
                    timing,
                    false,
                    factory_debug,
                );
            }
            None => {
                despawn_animatable_at(commands, index, pos);
                despawn_scene_at(commands, index, pos);
            }
        }

        // 系统/虚拟层：与上者重叠，绝不参与工厂材料动画
        match world.system_blocks.get(&pos).copied() {
            Some(data) => {
                if index.get_system(pos).is_none() {
                    spawn_and_index(
                        commands,
                        meshes,
                        index,
                        world,
                        assets,
                        pos,
                        data,
                        None,
                        None,
                        timing,
                        false,
                        factory_debug,
                    );
                }
            }
            None => {
                despawn_system_at(commands, index, pos);
            }
        }
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
    let mut handled = HashSet::new();

    // 先收集本回合所有可动画移动，避免 HashMap 迭代顺序导致「后到的目标格把先走的实体误删」
    let mut planned: Vec<(IVec3, BlockAnimation, BlockData, Option<Entity>)> = Vec::new();
    for (&pos, animation) in animations {
        let Some(data) = world.blocks.get(&pos).copied() else {
            continue;
        };
        if !(data.kind.is_factory() || data.kind.is_material()) {
            continue;
        }
        handled.insert(pos);
        handled.insert(animation.from_pos);
        let entity = index
            .get_by_id(animation.block_id)
            .or_else(|| index.get_animatable(animation.from_pos));
        planned.push((pos, *animation, data, entity));
    }

    let moving_entities: HashSet<Entity> = planned.iter().filter_map(|(_, _, _, e)| *e).collect();

    // 阶段 1：全部从旧格解绑（不销毁、不清 by_id）
    for (_, animation, _, entity) in &planned {
        let Some(entity) = entity else {
            continue;
        };
        if index.get_animatable(animation.from_pos) == Some(*entity) {
            index.unbind_animatable_pos(animation.from_pos);
        }
    }

    // 阶段 2：绑定到目标格并挂上动画
    for (pos, animation, data, entity) in planned {
        if let Some(entity) = entity {
            if let Some(occupant) = index.get_animatable(pos) {
                if occupant != entity {
                    if moving_entities.contains(&occupant) {
                        // 对方也在本回合搬走，只解绑占位
                        index.unbind_animatable_pos(pos);
                    } else {
                        despawn_entity(commands, index, occupant);
                    }
                }
            }
            index.insert(
                pos,
                animation.block_id,
                BlockEntityLayer::Animatable,
                entity,
            );
            let animated = AnimatedBlock::new(animation, timing);
            let start = animated.start_transform();
            commands.entity(entity).insert((
                BlockEntity {
                    pos,
                    id: animation.block_id,
                    layer: BlockEntityLayer::Animatable,
                },
                animated,
                start,
            ));
            continue;
        }

        // 找不到原实体时才新建；不要误删其它正在移动的实体
        if let Some(occupant) = index.get_animatable(pos) {
            if !moving_entities.contains(&occupant) {
                despawn_entity(commands, index, occupant);
            } else {
                index.unbind_animatable_pos(pos);
            }
        }
        spawn_and_index(
            commands,
            meshes,
            index,
            world,
            assets,
            pos,
            data,
            Some(animation),
            pusher_animations.get(&pos).copied(),
            timing,
            powered_wires.contains(&pos),
            factory_debug,
        );
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
        if !data.kind.is_material() || index.get_animatable(pos).is_some() {
            continue;
        }
        spawn_and_index(
            commands,
            meshes,
            index,
            after,
            assets,
            pos,
            *data,
            None,
            output.pusher_animations.get(&pos).copied(),
            timing,
            output.render_powered_wires.contains(&pos),
            debug.factory_activity.then_some(structure_state),
        );
    }
    for (&pos, data) in &after.system_blocks {
        if index.get_system(pos).is_some() {
            continue;
        }
        spawn_and_index(
            commands,
            meshes,
            index,
            after,
            assets,
            pos,
            *data,
            None,
            None,
            timing,
            false,
            debug.factory_activity.then_some(structure_state),
        );
    }
    spawn_weld_sparks(commands, assets, &output.weld_sparks);
    spawn_weld_sparks(commands, assets, &output.behavior_sparks);
    spawn_laser_beams(commands, assets, &output.laser_beams, animation_duration);
    spawn_acceptance_sparks(commands, assets, &output.acceptance_sparks);
    stats.render_rebuild_ms = render_start.elapsed().as_secs_f64() * 1000.0;
    stats.total_ms += stats.render_rebuild_ms;
}
