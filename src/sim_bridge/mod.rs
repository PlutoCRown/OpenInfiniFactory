//! 模拟表现桥接与预取：把 SimSnapshot / TurnOutput 应用到 Bevy，并托管预取 worker

pub mod cache;
pub mod present;
pub mod snapshot;
pub mod worker;

pub use cache::{TurnCache, TURN_PREFETCH_DEPTH};
pub use oif_sim::session::{SimSession, SimulationControl, SimulationDebugLog};
pub use oif_sim::simulation::core::TurnOutput;
pub use present::{
    apply_sim_snapshot, poll_simulation_worker, prefetch_simulation_turn, tick_simulation,
    SimulationPresentationState, SimulationTickDeps,
};
pub use snapshot::{CachedTurn, SimSnapshot};
pub use worker::SimulationWorker;
