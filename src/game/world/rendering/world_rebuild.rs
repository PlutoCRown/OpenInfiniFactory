use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

use super::components::BlockEntity;
use super::scene_chunks::{SceneChunkMeshes, clear_scene_chunks, rebuild_all_scene_chunks};
use super::spawn::{
    SpawnBlockOpts, SpawnMode, block_render_material, spawn_block, spawn_block_model,
};
use crate::game::world::animation::{AnimationTiming, BlockAnimation, PusherAnimation};
use crate::game::world::grid::WorldBlocks;
use crate::game::world::render_assets::WorldRenderAssets;
use crate::scene::BlockEntityIndex;

/// 无动画全量重建世界方块实体
pub fn rebuild_world(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    world: &WorldBlocks,
    assets: &WorldRenderAssets,
    index: &mut BlockEntityIndex,
    scene_chunks: &mut SceneChunkMeshes,
) {
    index.clear();
    for (pos, data) in world.blocks() {
        if data.kind.is_scene() {
            continue;
        }
        spawn_block(commands, meshes, assets, world, *pos, *data, index);
    }
    for (pos, data) in world.system_blocks() {
        spawn_block(commands, meshes, assets, world, *pos, *data, index);
    }
    rebuild_all_scene_chunks(commands, meshes, world, assets, scene_chunks);
}

/// 销毁全部方块实体并清空索引
pub fn despawn_world(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    block_entities: &Query<Entity, With<BlockEntity>>,
    index: &mut BlockEntityIndex,
    scene_chunks: &mut SceneChunkMeshes,
) {
    index.clear();
    clear_scene_chunks(commands, meshes, scene_chunks);
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
    index: &mut BlockEntityIndex,
    scene_chunks: &mut SceneChunkMeshes,
) {
    rebuild_world_with_timed_animations(
        commands,
        meshes,
        world,
        assets,
        animations,
        AnimationTiming::edit(),
        index,
        scene_chunks,
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
    index: &mut BlockEntityIndex,
    scene_chunks: &mut SceneChunkMeshes,
) {
    index.clear();
    for (pos, data) in world.blocks() {
        if data.kind.is_scene() {
            continue;
        }
        spawn_block_model(
            commands,
            meshes,
            assets,
            world,
            *pos,
            *data,
            SpawnBlockOpts {
                material: assets.block_material(data.kind),
                animation: animations.get(pos).copied(),
                pusher_animation: None,
                timing,
                mode: SpawnMode::World {
                    index,
                    show_generator_preview: true,
                },
            },
        );
    }
    for (pos, data) in world.system_blocks() {
        spawn_block_model(
            commands,
            meshes,
            assets,
            world,
            *pos,
            *data,
            SpawnBlockOpts {
                material: assets.block_material(data.kind),
                animation: animations.get(pos).copied(),
                pusher_animation: None,
                timing,
                mode: SpawnMode::World {
                    index,
                    show_generator_preview: true,
                },
            },
        );
    }
    rebuild_all_scene_chunks(commands, meshes, world, assets, scene_chunks);
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
    index: &mut BlockEntityIndex,
    scene_chunks: &mut SceneChunkMeshes,
) {
    index.clear();
    for (pos, data) in world.blocks() {
        if data.kind.is_scene() {
            continue;
        }
        let material = block_render_material(assets, *data, powered_wires.contains(pos));
        spawn_block_model(
            commands,
            meshes,
            assets,
            world,
            *pos,
            *data,
            SpawnBlockOpts {
                material,
                animation: animations.get(pos).copied(),
                pusher_animation: pusher_animations.get(pos).copied(),
                timing,
                mode: SpawnMode::World {
                    index,
                    show_generator_preview: false,
                },
            },
        );
    }
    for (pos, data) in world.system_blocks() {
        spawn_block_model(
            commands,
            meshes,
            assets,
            world,
            *pos,
            *data,
            SpawnBlockOpts {
                material: assets.block_material(data.kind),
                animation: animations.get(pos).copied(),
                pusher_animation: None,
                timing,
                mode: SpawnMode::World {
                    index,
                    show_generator_preview: false,
                },
            },
        );
    }
    rebuild_all_scene_chunks(commands, meshes, world, assets, scene_chunks);
}
