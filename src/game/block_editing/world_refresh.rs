use crate::game::session::PlayingWorldParams;
use crate::game::world::rendering::{despawn_world, rebuild_world_for_debug_state};

pub fn refresh_world_after_edit(world: &mut PlayingWorldParams) {
    despawn_world(&mut world.commands, &world.block_entities);
    world.structure_state.clear();
    world.movement_influence.clear();
    world.pusher_state.clear();
    if let Some(render_assets) = world.render_assets.as_deref() {
        rebuild_world_for_debug_state(
            &mut world.commands,
            &mut world.meshes,
            &world.world,
            render_assets,
            &world.debug,
            &mut world.structure_state,
        );
    }
}
