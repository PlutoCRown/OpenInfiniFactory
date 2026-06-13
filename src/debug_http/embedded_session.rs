use bevy::prelude::*;

use crate::game::simulation::movement::PusherState;
use crate::game::simulation::runtime::{
    PendingGeneratedMaterials, SignalNetworkCache,
};
use crate::game::simulation::structure_state::StructureState;
use crate::game::simulation::structures::MovementInfluenceCache;
use crate::game::state::SimulationState;
use crate::game::world::factory_registry::FactoryBlockRegistry;
use crate::game::world::grid::WorldBlocks;
use crate::sim_core::{SimSnapshot, SimulationControl};

pub fn simulation_control_adapter(simulation: &SimulationState) -> SimulationControl {
    SimulationControl {
        turn: simulation.turn,
        running: simulation.running,
        step_requested: simulation.step_requested,
        speed: simulation.speed,
        accumulator: simulation.accumulator,
        start_snapshot: simulation.start_snapshot.clone(),
        start_structures: simulation.start_structures.clone(),
    }
}

pub fn build_runtime_snapshot(
    simulation: &SimulationState,
    world: &WorldBlocks,
    structure_state: &StructureState,
    factory_registry: &FactoryBlockRegistry,
    pending_generated: &PendingGeneratedMaterials,
    signal_cache: &SignalNetworkCache,
    movement_influence: &MovementInfluenceCache,
    pusher_state: &PusherState,
) -> Result<SimSnapshot, &'static str> {
    let solution = simulation
        .start_snapshot
        .as_ref()
        .ok_or("simulation is not active")?;
    let solution_structures = simulation
        .start_structures
        .as_ref()
        .ok_or("simulation is not active")?;
    Ok(SimSnapshot {
        solution: solution.clone(),
        solution_structures: solution_structures.clone(),
        world: world.clone(),
        structure_state: structure_state.clone(),
        factory_registry: factory_registry.clone(),
        pending_generated: pending_generated.clone(),
        signal_cache: signal_cache.clone(),
        movement_influence: movement_influence.clone(),
        pusher_state: pusher_state.clone(),
    })
}
