use crate::game::simulation::movement::PusherState;
use crate::game::simulation::runtime::{PendingGeneratedMaterials, SignalNetworkCache};
use crate::game::simulation::structure_state::StructureState;
use crate::game::simulation::structures::MovementInfluenceCache;
use crate::game::world::factory_registry::FactoryBlockRegistry;
use crate::game::world::grid::WorldBlocks;

use super::TurnOutput;

#[derive(Clone)]
pub struct SimSnapshot {
    pub solution: WorldBlocks,
    pub solution_structures: StructureState,
    pub world: WorldBlocks,
    pub structure_state: StructureState,
    pub factory_registry: FactoryBlockRegistry,
    pub pending_generated: PendingGeneratedMaterials,
    pub signal_cache: SignalNetworkCache,
    pub movement_influence: MovementInfluenceCache,
    pub pusher_state: PusherState,
}

#[derive(Clone)]
pub struct CachedTurn {
    pub output: TurnOutput,
    pub after: SimSnapshot,
}

impl SimSnapshot {
    pub fn at_simulation_start(
        world: &WorldBlocks,
        pending_generated: &PendingGeneratedMaterials,
        signal_cache: &SignalNetworkCache,
        movement_influence: &MovementInfluenceCache,
        pusher_state: &PusherState,
    ) -> Self {
        let mut structure_state = StructureState::default();
        structure_state.rebuild_for_simulation(world);
        let mut factory_registry = FactoryBlockRegistry::rebuild_from_world(world);
        factory_registry.freeze_solution();
        Self {
            solution: world.clone(),
            solution_structures: structure_state.clone(),
            world: world.clone(),
            structure_state,
            factory_registry,
            pending_generated: pending_generated.clone(),
            signal_cache: signal_cache.clone(),
            movement_influence: movement_influence.clone(),
            pusher_state: pusher_state.clone(),
        }
    }
}
