use crate::game::simulation::structure_state::StructureState;
use crate::game::world::grid::WorldBlocks;

#[derive(Clone, Default)]
pub struct SimulationControl {
    pub turn: u64,
    pub running: bool,
    pub step_requested: bool,
    pub speed: f32,
    pub accumulator: f32,
    pub(crate) start_snapshot: Option<WorldBlocks>,
    pub(crate) start_structures: Option<StructureState>,
}

impl SimulationControl {
    pub fn is_active(&self) -> bool {
        self.start_snapshot.is_some() || self.running || self.turn > 0
    }
}
