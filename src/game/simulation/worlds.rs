use crate::game::simulation::structure_state::StructureState;
use crate::game::world::block_instance::MaterialBlockRegistry;
use crate::game::world::factory_registry::FactoryBlockRegistry;
use crate::game::world::grid::WorldBlocks;

/// Runtime simulation worlds: solution (frozen at sim start), turn (committed state), realtime (scratch during movement).
#[derive(Clone)]
pub struct SimulationWorlds {
    pub solution: WorldBlocks,
    pub solution_structures: StructureState,
    pub turn: WorldBlocks,
    pub turn_structures: StructureState,
    pub factory_registry: FactoryBlockRegistry,
    pub material_registry: MaterialBlockRegistry,
}

impl SimulationWorlds {
    pub fn at_simulation_start(turn: WorldBlocks, turn_structures: StructureState) -> Self {
        let mut factory_registry = FactoryBlockRegistry::rebuild_from_world(&turn);
        factory_registry.freeze_solution();
        let material_registry = MaterialBlockRegistry::rebuild_from_world(&turn);
        Self {
            solution: turn.clone(),
            solution_structures: turn_structures.clone(),
            turn,
            turn_structures,
            factory_registry,
            material_registry,
        }
    }

    pub fn from_snapshot_parts(
        solution: WorldBlocks,
        solution_structures: StructureState,
        turn: WorldBlocks,
        turn_structures: StructureState,
        factory_registry: FactoryBlockRegistry,
        material_registry: MaterialBlockRegistry,
    ) -> Self {
        Self {
            solution,
            solution_structures,
            turn,
            turn_structures,
            factory_registry,
            material_registry,
        }
    }
}
