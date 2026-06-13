use crate::game::world::grid::WorldBlocks;

use super::structure_state::StructureState;

/// Runtime simulation worlds: solution (frozen at sim start), turn (committed state), realtime (scratch during movement).
#[derive(Clone)]
pub struct SimulationWorlds {
    pub solution: WorldBlocks,
    pub solution_structures: StructureState,
    pub turn: WorldBlocks,
    pub turn_structures: StructureState,
}

impl SimulationWorlds {
    pub fn at_simulation_start(turn: WorldBlocks, turn_structures: StructureState) -> Self {
        Self {
            solution: turn.clone(),
            solution_structures: turn_structures.clone(),
            turn,
            turn_structures,
        }
    }

    pub fn from_snapshot_parts(
        solution: WorldBlocks,
        solution_structures: StructureState,
        turn: WorldBlocks,
        turn_structures: StructureState,
    ) -> Self {
        Self {
            solution,
            solution_structures,
            turn,
            turn_structures,
        }
    }
}
