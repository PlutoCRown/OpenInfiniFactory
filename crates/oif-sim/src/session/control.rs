use crate::simulation::structure_state::StructureState;
use crate::world::grid::WorldBlocks;

/// 模拟控制面：回合计数、运行意图与开局快照
#[derive(Clone, Default)]
pub struct SimulationControl {
    pub turn: u64,
    pub running: bool,
    pub step_requested: bool,
    pub speed: f32,
    pub accumulator: f32,
    pub start_snapshot: Option<WorldBlocks>,
    pub start_structures: Option<StructureState>,
}

impl SimulationControl {
    /// 是否处于模拟中
    pub fn is_active(&self) -> bool {
        self.running || self.turn > 0
    }
}
