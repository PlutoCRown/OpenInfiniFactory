//! Facade crate for headless simulation and E2E tooling.
//!
//! The implementation currently lives in the main `open_infinifactory` crate;
//! this package exposes a stable surface for tools and tests.

pub use open_infinifactory::game::simulation::core::{
    prepare_upcoming_generation, simulate_turn, TurnOutput,
};
pub use open_infinifactory::sim_core::{
    build_headless_sim_app, SimCorePlugin, SimCoreWorld, SimulationControl, SimulationDebugLog,
    TurnCache, TURN_PREFETCH_DEPTH,
};
