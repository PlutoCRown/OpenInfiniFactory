//! 模拟表现桥接：权威状态原地推进，并把回合结果应用到 Bevy 场景

pub mod present;

pub use oif_sim::session::{SimSession, SimulationControl, SimulationDebugLog};
pub use oif_sim::simulation::core::TurnOutput;
pub use present::{
    SimulationPresentationDeps, SimulationPresentationState, TurnCommitted, advance_simulation,
    present_simulation_turns, refresh_pending_generated_previews,
};

/// 换档、编辑或回滚时清空可丢弃的表现状态
pub fn reset_simulation_presentation(presentation: &mut SimulationPresentationState) {
    presentation.last_powered_wires.clear();
    presentation.last_powered_devices.clear();
}
