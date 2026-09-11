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

/// 模拟桥接插件：集中拥有回合资源、业务控制和表现提交的生命周期。
pub struct SimulationBridgePlugin;

impl bevy::prelude::Plugin for SimulationBridgePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        use crate::game::schedule::GameSet;
        use crate::game::simulation;
        use crate::game::state::SimulationState;
        use crate::game::systems::simulation_controls::{
            simulation_controls, sync_generator_config_material_preview,
        };
        use bevy::prelude::*;
        app.insert_resource(SimulationState::default())
            .init_resource::<oif_sim::simulation::core::TurnRunner>()
            .insert_resource(simulation::signals::SignalNetworkCache::default())
            .insert_resource(simulation::stats::SimulationStepStats::default())
            .insert_resource(simulation::pending::PendingTurnEffects::default())
            .insert_resource(simulation::structure_state::StructureState::default())
            .insert_resource(simulation::movement::PusherState::default())
            .insert_resource(simulation::structures::MovementHistory::default())
            .insert_resource(crate::sim_bridge::SimulationPresentationState::default())
            .add_message::<crate::sim_bridge::TurnCommitted>()
            .add_systems(
                Update,
                simulation_controls.in_set(GameSet::SimulationControls),
            )
            .add_systems(
                Update,
                sync_generator_config_material_preview
                    .in_set(GameSet::SimulationControls)
                    .after(simulation_controls),
            )
            .add_systems(
                Update,
                crate::sim_bridge::advance_simulation.in_set(GameSet::Simulation),
            )
            .add_systems(
                Update,
                (crate::sim_bridge::present_simulation_turns, ApplyDeferred)
                    .chain()
                    .in_set(GameSet::Presentation),
            )
            .add_systems(
                Update,
                crate::sim_bridge::refresh_pending_generated_previews
                    .after(crate::sim_bridge::present_simulation_turns)
                    .in_set(GameSet::Presentation),
            );
    }
}
