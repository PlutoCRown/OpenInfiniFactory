use bevy::prelude::*;

use crate::game::simulation::factory_activity::FactoryStructureState;
use crate::game::simulation::markers::refresh_static_generated_markers;
use crate::game::simulation::movement::PusherState;
use crate::game::simulation::runtime::PendingGeneratedMaterials;
use crate::game::simulation::structures::MovementInfluenceCache;
use crate::game::state::{BuilderMode, GameMode, SimulationState};
use crate::game::systems::debug::DebugState;
use crate::game::ui::UiRuntime;
use crate::game::world::grid::WorldBlocks;
use crate::game::world::rendering::{
    despawn_world, rebuild_world_for_debug_state, BlockEntity, WorldRenderManager,
};
use crate::shared::config::{ConfigAction, GameConfig};

pub fn simulation_controls(
    keys: Res<ButtonInput<KeyCode>>,
    config: Res<GameConfig>,
    mut commands: Commands,
    builder_mode: Res<BuilderMode>,
    mode: Res<GameMode>,
    ui_runtime: Res<UiRuntime>,
    mut simulation: ResMut<SimulationState>,
    mut pending_generated: ResMut<PendingGeneratedMaterials>,
    mut factory_structures: ResMut<FactoryStructureState>,
    mut movement_influence: ResMut<MovementInfluenceCache>,
    mut pusher_state: ResMut<PusherState>,
    mut world: ResMut<WorldBlocks>,
    block_entities: Query<Entity, With<BlockEntity>>,
    mut meshes: ResMut<Assets<Mesh>>,
    render_manager: Res<WorldRenderManager>,
    debug: Res<DebugState>,
) {
    if *builder_mode != BuilderMode::Play
        || *mode != GameMode::Playing
        || ui_runtime.blocks_gameplay()
    {
        return;
    }

    let simulate_key = config.key(ConfigAction::Simulate).key_code();
    let fast_key = config.key(ConfigAction::SimulationFast).key_code();
    let rollback_key = config.key(ConfigAction::SimulationRollback).key_code();
    let step_key = config.key(ConfigAction::SimulationStep).key_code();

    if keys.just_pressed(simulate_key) {
        if !simulation.is_active() {
            start_simulation_state(
                &mut simulation,
                &world,
                &mut factory_structures,
                &mut pusher_state,
            );
        }
        simulation.running = true;
    }

    if keys.just_pressed(step_key) {
        if !simulation.is_active() {
            return;
        }
        if simulation.running {
            simulation.running = false;
            simulation.speed = 1.0;
        } else {
            simulation.step_requested = true;
        }
    }

    simulation.speed = if simulation.running && keys.pressed(fast_key) {
        4.0
    } else {
        1.0
    };

    if keys.just_pressed(rollback_key) && simulation.is_active() {
        let factory_snapshot = rollback_simulation(&mut simulation, &mut world);
        refresh_static_generated_markers(&mut world);
        pending_generated.clear();
        factory_structures.clear();
        movement_influence.clear();
        pusher_state.clear();
        if let Some(snapshot) = factory_snapshot {
            *factory_structures = snapshot;
        } else {
            factory_structures.rebuild_from_world(&world);
        }
        despawn_world(&mut commands, &block_entities);
        rebuild_world_for_debug_state(
            &mut commands,
            &mut meshes,
            &world,
            &render_manager,
            &debug,
            &factory_structures,
        );
    }
}

fn start_simulation_state(
    simulation: &mut SimulationState,
    world: &WorldBlocks,
    factory_structures: &mut FactoryStructureState,
    pusher_state: &mut PusherState,
) {
    simulation.start_snapshot = Some(world.clone());
    *pusher_state = PusherState::rebuild_from_world(world);
    factory_structures.rebuild_from_world(world);
    simulation.start_factory_structures = Some(factory_structures.clone());
}
fn rollback_simulation(
    simulation: &mut SimulationState,
    world: &mut WorldBlocks,
) -> Option<FactoryStructureState> {
    simulation.running = false;
    simulation.step_requested = false;
    simulation.turn = 0;
    simulation.accumulator = 0.0;
    let factory_snapshot = simulation.start_factory_structures.take();
    if let Some(snapshot) = simulation.start_snapshot.take() {
        *world = snapshot;
    } else {
        world.retain(|_, block| !block.kind.is_material());
        world.clear_generated_markers();
    }
    factory_snapshot
}
