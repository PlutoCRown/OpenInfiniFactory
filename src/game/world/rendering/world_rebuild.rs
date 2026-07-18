use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

use super::components::BlockEntity;
use super::spawn::{block_render_material, spawn_block, spawn_block_model};
use crate::game::simulation::structure_state::StructureState;
use crate::game::systems::debug::DebugState;
use crate::game::world::animation::{AnimationTiming, BlockAnimation, PusherAnimation};
use crate::game::world::grid::WorldBlocks;
use crate::game::world::render_assets::WorldRenderAssets;
use crate::scene::BlockEntityIndex;

/// 进入游玩时按调试状态重建结构与世界渲染
pub fn rebuild_world_on_enter(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    world: &WorldBlocks,
    render_assets: &WorldRenderAssets,
    debug: &DebugState,
    structure_state: &mut StructureState,
    index: &mut BlockEntityIndex,
) {
    structure_state.clear();
    if debug.factory_activity {
        structure_state.rebuild_factory_for_debug(world);
    }
    rebuild_world_for_debug_state(
        commands,
        meshes,
        world,
        render_assets,
        debug,
        structure_state,
        index,
    );
}

/// 无动画全量重建世界方块实体
pub fn rebuild_world(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    world: &WorldBlocks,
    assets: &WorldRenderAssets,
    factory_debug: Option<&StructureState>,
    index: &mut BlockEntityIndex,
) {
    index.clear();
    for (pos, data) in &world.blocks {
        spawn_block(
            commands,
            meshes,
            assets,
            world,
            *pos,
            *data,
            factory_debug,
            index,
        );
    }
    for (pos, data) in &world.system_blocks {
        spawn_block(
            commands,
            meshes,
            assets,
            world,
            *pos,
            *data,
            factory_debug,
            index,
        );
    }
}

/// 按调试开关决定是否带工厂叠层重建
pub fn rebuild_world_for_debug_state(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    world: &WorldBlocks,
    assets: &WorldRenderAssets,
    debug: &DebugState,
    structure_state: &StructureState,
    index: &mut BlockEntityIndex,
) {
    rebuild_world(
        commands,
        meshes,
        world,
        assets,
        debug.factory_activity.then_some(structure_state),
        index,
    );
}

/// 带编辑动画并按调试开关重建
pub fn rebuild_world_with_animations_for_debug_state(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    world: &WorldBlocks,
    assets: &WorldRenderAssets,
    animations: &HashMap<IVec3, BlockAnimation>,
    debug: &DebugState,
    structure_state: &StructureState,
    index: &mut BlockEntityIndex,
) {
    rebuild_world_with_animations(
        commands,
        meshes,
        world,
        assets,
        animations,
        debug.factory_activity.then_some(structure_state),
        index,
    );
}

/// 销毁全部方块实体并清空索引
pub fn despawn_world(
    commands: &mut Commands,
    block_entities: &Query<Entity, With<BlockEntity>>,
    index: &mut BlockEntityIndex,
) {
    index.clear();
    for entity in block_entities {
        commands.entity(entity).despawn();
    }
}

/// 以编辑时序带动画重建世界
pub fn rebuild_world_with_animations(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    world: &WorldBlocks,
    assets: &WorldRenderAssets,
    animations: &HashMap<IVec3, BlockAnimation>,
    factory_debug: Option<&StructureState>,
    index: &mut BlockEntityIndex,
) {
    rebuild_world_with_timed_animations(
        commands,
        meshes,
        world,
        assets,
        animations,
        AnimationTiming::edit(),
        factory_debug,
        index,
    );
}

/// 以指定时序带动画重建世界
pub fn rebuild_world_with_timed_animations(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    world: &WorldBlocks,
    assets: &WorldRenderAssets,
    animations: &HashMap<IVec3, BlockAnimation>,
    timing: AnimationTiming,
    factory_debug: Option<&StructureState>,
    index: &mut BlockEntityIndex,
) {
    index.clear();
    for (pos, data) in &world.blocks {
        spawn_block_model(
            commands,
            meshes,
            assets,
            world,
            *pos,
            *data,
            assets.block_material(data.kind),
            None,
            animations.get(pos).copied(),
            None,
            timing,
            true,
            false,
            true,
            None,
            factory_debug,
            Some(index),
        );
    }
    for (pos, data) in &world.system_blocks {
        spawn_block_model(
            commands,
            meshes,
            assets,
            world,
            *pos,
            *data,
            assets.block_material(data.kind),
            None,
            animations.get(pos).copied(),
            None,
            timing,
            true,
            false,
            true,
            None,
            None,
            Some(index),
        );
    }
}

/// 运行时重建：含推动动画与通电电线材质
pub fn rebuild_world_with_runtime_animations(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    world: &WorldBlocks,
    assets: &WorldRenderAssets,
    animations: &HashMap<IVec3, BlockAnimation>,
    pusher_animations: &HashMap<IVec3, PusherAnimation>,
    timing: AnimationTiming,
    powered_wires: &HashSet<IVec3>,
    factory_debug: Option<&StructureState>,
    index: &mut BlockEntityIndex,
) {
    index.clear();
    for (pos, data) in &world.blocks {
        let material = block_render_material(assets, *data, powered_wires.contains(pos));
        spawn_block_model(
            commands,
            meshes,
            assets,
            world,
            *pos,
            *data,
            material,
            None,
            animations.get(pos).copied(),
            pusher_animations.get(pos).copied(),
            timing,
            true,
            false,
            false,
            None,
            factory_debug,
            Some(index),
        );
    }
    for (pos, data) in &world.system_blocks {
        spawn_block_model(
            commands,
            meshes,
            assets,
            world,
            *pos,
            *data,
            assets.block_material(data.kind),
            None,
            animations.get(pos).copied(),
            None,
            timing,
            true,
            false,
            false,
            None,
            None,
            Some(index),
        );
    }
}

/// 运行时重建并按调试开关挂工厂叠层
pub fn rebuild_world_with_runtime_animations_for_debug_state(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    world: &WorldBlocks,
    assets: &WorldRenderAssets,
    animations: &HashMap<IVec3, BlockAnimation>,
    pusher_animations: &HashMap<IVec3, PusherAnimation>,
    timing: AnimationTiming,
    debug: &DebugState,
    structure_state: &StructureState,
    powered_wires: &HashSet<IVec3>,
    index: &mut BlockEntityIndex,
) {
    rebuild_world_with_runtime_animations(
        commands,
        meshes,
        world,
        assets,
        animations,
        pusher_animations,
        timing,
        powered_wires,
        debug.factory_activity.then_some(structure_state),
        index,
    );
}
