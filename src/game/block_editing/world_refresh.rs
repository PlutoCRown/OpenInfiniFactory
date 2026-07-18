use std::collections::HashSet;

use bevy::prelude::IVec3;

use crate::game::session::PlayingWorldParams;
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
