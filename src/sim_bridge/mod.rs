//! 模拟表现桥接与预取：把 SimSnapshot / TurnOutput 应用到 Bevy，并托管预取 worker

pub mod cache;
pub mod present;
pub mod snapshot;
pub mod worker;

pub use cache::{TURN_PREFETCH_DEPTH, TurnCache};
pub use oif_sim::session::{SimSession, SimulationControl, SimulationDebugLog};
pub use oif_sim::simulation::core::TurnOutput;
pub use present::{
    SimulationPresentationState, SimulationTickDeps, apply_sim_snapshot, poll_simulation_worker,
    prefetch_simulation_turn, tick_simulation,
};
pub use snapshot::{CachedTurn, SimSnapshot};
pub use worker::SimulationWorker;

/// 进档：新建 worker 并按当前世界重置预取（替换旧线程，避免卡住时拖死后续存档）
pub fn rebind_simulation_worker_for_world(
    commands: &mut bevy::prelude::Commands,
    turn_cache: &mut TurnCache,
    presentation: &mut SimulationPresentationState,
    world: &crate::game::world::grid::WorldBlocks,
    pending_generated: &crate::game::simulation::pending::PendingGeneratedMaterials,
    signal_cache: &crate::game::simulation::signals::SignalNetworkCache,
    structure_state: &crate::game::simulation::structure_state::StructureState,
    movement_influence: &crate::game::simulation::structures::MovementInfluenceCache,
    pusher_state: &crate::game::simulation::movement::PusherState,
    display_turn: u64,
) {
    let worker = SimulationWorker::spawn();
    invalidate_simulation_prefetch(
        turn_cache,
        presentation,
        Some(&worker),
        world,
        pending_generated,
        signal_cache,
        structure_state,
        movement_influence,
        pusher_state,
        display_turn,
    );
    // insert 会 Drop 旧 Backend（发 Shutdown）；卡住的旧线程不再被本会话引用
    commands.insert_resource(worker);
}

/// 换档或停模拟时作废预取：bump epoch、清空 TurnCache、重置 worker 并丢弃频道残留
pub fn invalidate_simulation_prefetch(
    turn_cache: &mut TurnCache,
    presentation: &mut SimulationPresentationState,
    worker: Option<&SimulationWorker>,
    world: &crate::game::world::grid::WorldBlocks,
    pending_generated: &crate::game::simulation::pending::PendingGeneratedMaterials,
    signal_cache: &crate::game::simulation::signals::SignalNetworkCache,
    structure_state: &crate::game::simulation::structure_state::StructureState,
    movement_influence: &crate::game::simulation::structures::MovementInfluenceCache,
    pusher_state: &crate::game::simulation::movement::PusherState,
    display_turn: u64,
) {
    let epoch = turn_cache.invalidate_prefetch(display_turn);
    presentation.committed_world = world.clone();
    presentation.last_powered_wires.clear();
    let Some(worker) = worker else {
        return;
    };
    worker.reset(
        SimSnapshot::from_world(
            world,
            pending_generated,
            signal_cache,
            structure_state,
            movement_influence,
            pusher_state,
        ),
        display_turn,
        epoch,
    );
    // 丢掉 reset 之前已进频道的旧回合
    let _ = worker.drain_results();
}
