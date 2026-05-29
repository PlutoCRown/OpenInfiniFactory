use bevy::prelude::*;

use crate::game::state::{BuilderMode, GameMode, SimulationState};
use crate::game::simulation::runtime::PendingGeneratedMaterials;
use crate::game::world::grid::WorldBlocks;
use crate::game::world::rendering::{despawn_world, rebuild_world, BlockEntity, WorldRenderAssets};
use crate::shared::config::{ConfigAction, GameConfig};

pub fn simulation_controls(
    keys: Res<ButtonInput<KeyCode>>,
    config: Res<GameConfig>,
    mut commands: Commands,
    builder_mode: Res<BuilderMode>,
    mode: Res<GameMode>,
    mut simulation: ResMut<SimulationState>,
    mut pending_generated: ResMut<PendingGeneratedMaterials>,
    mut world: ResMut<WorldBlocks>,
    block_entities: Query<Entity, With<BlockEntity>>,
    render_assets: Res<WorldRenderAssets>,
) {
    if *builder_mode != BuilderMode::Play || *mode != GameMode::Playing {
        return;
    }

    let simulate_key = config.key(ConfigAction::Simulate).key_code();
    let fast_key = config.key(ConfigAction::SimulationFast).key_code();
    let rollback_key = config.key(ConfigAction::SimulationRollback).key_code();
    let step_key = config.key(ConfigAction::SimulationStep).key_code();

    if keys.just_pressed(simulate_key) {
        if !simulation.is_active() {
            simulation.start_snapshot = Some(world.clone());
        }
        simulation.running = true;
    }

    if simulation.is_active() && keys.just_pressed(step_key) {
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
        rollback_simulation(&mut simulation, &mut world);
        pending_generated.clear();
        despawn_world(&mut commands, &block_entities);
        rebuild_world(&mut commands, &world, &render_assets);
    }
}
fn rollback_simulation(simulation: &mut SimulationState, world: &mut WorldBlocks) {
    simulation.running = false;
    simulation.step_requested = false;
    simulation.turn = 0;
    simulation.accumulator = 0.0;
    if let Some(snapshot) = simulation.start_snapshot.take() {
        *world = snapshot;
    } else {
        world.retain(|_, block| !block.kind.is_material());
        world.clear_generated_markers();
    }
}
