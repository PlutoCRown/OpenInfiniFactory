use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::game::simulation::markers::refresh_static_generated_markers;
use crate::game::simulation::movement::PusherState;
use crate::game::simulation::runtime::{PendingGeneratedMaterials, SignalNetworkCache};
use crate::game::simulation::structure_state::StructureState;
use crate::game::simulation::structures::MovementInfluenceCache;
use crate::game::state::{BuilderMode, GameMode, PlayingUiState, SimulationState};
use crate::game::systems::debug::DebugState;
use crate::game::ui::UiRuntime;
use crate::game::world::grid::WorldBlocks;
use crate::game::world::rendering::{
    despawn_world, rebuild_world_for_debug_state, BlockEntity, WorldRenderAssets,
};
use crate::shared::config::{ActionKeyName, GameConfig};
use crate::sim_core::{SimSnapshot, SimulationWorker, TurnCache};

#[derive(SystemParam)]
pub(crate) struct SimulationControlDeps<'w> {
    builder_mode: Res<'w, BuilderMode>,
    mode: Res<'w, State<GameMode>>,
    playing_ui: Res<'w, PlayingUiState>,
    ui_runtime: Res<'w, UiRuntime>,
    simulation: ResMut<'w, SimulationState>,
    pending_generated: ResMut<'w, PendingGeneratedMaterials>,
    signal_cache: Res<'w, SignalNetworkCache>,
    structure_state: ResMut<'w, StructureState>,
    movement_influence: ResMut<'w, MovementInfluenceCache>,
    pusher_state: ResMut<'w, PusherState>,
    world: ResMut<'w, WorldBlocks>,
    turn_cache: ResMut<'w, TurnCache>,
    worker: Option<Res<'w, SimulationWorker>>,
    presentation: ResMut<'w, crate::game::simulation::runtime::SimulationPresentationState>,
    render_assets: Option<Res<'w, WorldRenderAssets>>,
    debug: Res<'w, DebugState>,
}

pub fn simulation_controls(
    keys: Res<ButtonInput<KeyCode>>,
    config: Res<GameConfig>,
    mut commands: Commands,
    block_entities: Query<Entity, With<BlockEntity>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut deps: SimulationControlDeps,
) {
    if *deps.builder_mode != BuilderMode::Play
        || *deps.mode.get() != GameMode::Playing
        || !deps.playing_ui.active_play()
        || deps.ui_runtime.blocks_gameplay()
    {
        return;
    }
    let Some(render_assets) = deps.render_assets.as_ref() else {
        return;
    };

    let simulate_key = config.key(ActionKeyName::Simulate).key_code();
    let fast_key = config.key(ActionKeyName::SimulationFast).key_code();
    let rollback_key = config.key(ActionKeyName::SimulationRollback).key_code();
    let step_key = config.key(ActionKeyName::SimulationStep).key_code();

    if keys.just_pressed(simulate_key) {
        start_simulation_if_needed(
            &mut deps.simulation,
            &deps.world,
            &mut deps.structure_state,
            &mut deps.pusher_state,
        );
        deps.presentation.committed_world = deps.world.clone();
        deps.presentation.last_render_powered_wires.clear();
        if let Some(worker) = deps.worker.as_ref() {
            worker.reset(
                SimSnapshot::from_world(
                    &deps.world,
                    &deps.pending_generated,
                    &deps.signal_cache,
                    &deps.structure_state,
                    &deps.movement_influence,
                    &deps.pusher_state,
                ),
                deps.simulation.turn,
            );
        }
        request_continuous_run(&mut deps.simulation);
    }

    if keys.just_pressed(step_key) {
        if !deps.simulation.is_active() {
            return;
        }
        if deps.simulation.running {
            deps.simulation.running = false;
            deps.simulation.speed = 1.0;
        } else {
            deps.simulation.step_requested = true;
        }
    }

    deps.simulation.speed = if deps.simulation.running && keys.pressed(fast_key) {
        4.0
    } else {
        1.0
    };

    if keys.just_pressed(rollback_key) && deps.simulation.is_active() {
        let factory_snapshot = rollback_simulation(&mut deps.simulation, &mut deps.world);
        refresh_static_generated_markers(&mut deps.world);
        deps.pending_generated.clear();
        deps.structure_state.clear();
        deps.movement_influence.clear();
        deps.pusher_state.clear();
        deps.turn_cache.reset_to_turn(0);
        deps.presentation.committed_world = deps.world.clone();
        deps.presentation.last_render_powered_wires.clear();
        if let Some(worker) = deps.worker.as_ref() {
            worker.reset(
                SimSnapshot::from_world(
                    &deps.world,
                    &deps.pending_generated,
                    &deps.signal_cache,
                    &deps.structure_state,
                    &deps.movement_influence,
                    &deps.pusher_state,
                ),
                0,
            );
        }
        if let Some(snapshot) = factory_snapshot {
            *deps.structure_state = snapshot;
        } else {
            deps.structure_state.clear();
        }
        despawn_world(&mut commands, &block_entities);
        rebuild_world_for_debug_state(
            &mut commands,
            &mut meshes,
            &deps.world,
            render_assets,
            &deps.debug,
            &deps.structure_state,
        );
    }
}

fn start_simulation_state(
    simulation: &mut SimulationState,
    world: &WorldBlocks,
    structure_state: &mut StructureState,
    pusher_state: &mut PusherState,
) {
    simulation.start_snapshot = Some(world.clone());
    *pusher_state = PusherState::rebuild_from_world(world);
    structure_state.rebuild_for_simulation(world);
    simulation.start_structures = Some(structure_state.clone());
}

pub fn start_simulation_if_needed(
    simulation: &mut SimulationState,
    world: &WorldBlocks,
    structure_state: &mut StructureState,
    pusher_state: &mut PusherState,
) {
    if !simulation.is_active() {
        start_simulation_state(simulation, world, structure_state, pusher_state);
    }
}

pub fn request_continuous_run(simulation: &mut SimulationState) {
    simulation.running = true;
}

pub fn request_one_turn(simulation: &mut SimulationState) -> Result<(), &'static str> {
    if !simulation.is_active() {
        return Err("simulation is not active");
    }
    simulation.running = false;
    simulation.speed = 1.0;
    simulation.step_requested = true;
    Ok(())
}
fn rollback_simulation(
    simulation: &mut SimulationState,
    world: &mut WorldBlocks,
) -> Option<StructureState> {
    simulation.running = false;
    simulation.step_requested = false;
    simulation.turn = 0;
    simulation.accumulator = 0.0;
    let factory_snapshot = simulation.start_structures.take();
    if let Some(snapshot) = simulation.start_snapshot.take() {
        *world = snapshot;
    } else {
        world.retain(|_, block| !block.kind.is_material());
        world.clear_generated_markers();
    }
    factory_snapshot
}
