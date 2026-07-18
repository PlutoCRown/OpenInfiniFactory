use std::collections::HashSet;

use bevy::prelude::IVec3;

use crate::game::edit_history::{
    EditHistory, apply_block_settings_with_history, apply_teleport_pair_with_history,
};
use crate::game::session::PlayingWorldParams;
use crate::game::world::grid::WorldBlocks;
use crate::scene::refresh_edit_changes;

pub fn refresh_world_after_edit(world: &mut PlayingWorldParams, pos: IVec3) {
    refresh_world_after_edit_many(world, HashSet::from([pos]));
}

pub fn refresh_world_after_edit_many(world: &mut PlayingWorldParams, changed: HashSet<IVec3>) {
    world.structure_state.clear();
    world.movement_influence.clear();
    world.pusher_state.clear();
    if let Some(render_assets) = world.render_assets.as_deref() {
        refresh_edit_changes(
            &mut world.commands,
            &mut world.meshes,
            &mut world.block_index,
            &world.world,
            render_assets,
            &world.debug,
            &mut world.structure_state,
            &changed,
            &mut world.scene_chunks,
        );
    }
}

/// 写入方块配置、记入历史，并重建受影响格的渲染（生成器/验收器材料预览等）
pub fn apply_block_settings_edit(
    history: &mut EditHistory,
    world: &mut PlayingWorldParams,
    pos: IVec3,
    apply: impl FnOnce(&mut WorldBlocks),
) {
    apply_block_settings_with_history(history, &mut world.world, pos, apply);
    refresh_world_after_edit(world, pos);
}

/// 写入传送门配对、记入历史，并刷新相关格渲染
pub fn apply_teleport_pair_edit(
    history: &mut EditHistory,
    world: &mut PlayingWorldParams,
    pos: IVec3,
    partner: Option<IVec3>,
) {
    let mut changed = HashSet::from([pos]);
    if let Some(old) = world.world.teleport_settings(pos).pair {
        changed.insert(old);
    }
    if let Some(new) = partner {
        changed.insert(new);
        if let Some(previous) = world.world.teleport_settings(new).pair {
            changed.insert(previous);
        }
    }
    apply_teleport_pair_with_history(history, &mut world.world, pos, partner);
    refresh_world_after_edit_many(world, changed);
}
