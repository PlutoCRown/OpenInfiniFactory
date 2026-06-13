use std::collections::HashSet;

use bevy::prelude::IVec3;

use crate::game::session::PlayingWorldParams;
use crate::game::world::block_instance::MaterialBlockRegistry;
use crate::game::world::factory_registry::FactoryBlockRegistry;
use crate::scene::refresh_edit_changes;

pub fn refresh_world_after_edit(world: &mut PlayingWorldParams, pos: IVec3) {
    refresh_world_after_edit_many(world, HashSet::from([pos]));
}

pub fn refresh_world_after_edit_many(world: &mut PlayingWorldParams, changed: HashSet<IVec3>) {
    world.structure_state.clear();
    world.movement_influence.clear();
    world.pusher_state.clear();
    *world.factory_registry = FactoryBlockRegistry::rebuild_from_world(&world.world);
    *world.material_registry = MaterialBlockRegistry::rebuild_from_world(&world.world);
    if let Some(render_assets) = world.render_assets.as_deref() {
        refresh_edit_changes(
            &mut world.commands,
            &mut world.meshes,
            &world.block_entities,
            &world.world,
            render_assets,
            &world.debug,
            &mut world.structure_state,
            &world.factory_registry,
            &world.material_registry,
            &changed,
        );
    }
}
