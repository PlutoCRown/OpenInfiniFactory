mod blocks;
mod debug;
mod gameplay;
mod inventory;
mod player;
mod rendering;
mod save;
mod state;
mod world;

use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;

use gameplay::{
    apply_fov, gameplay_input, pause_menu_actions, placement_input, save_load_input,
    simulation_controls, simulation_tick, update_hover,
};
use inventory::{CarriedItem, InventoryItems};
use player::{camera_look, camera_move, spawn_player, sync_cursor_grab};
use rendering::{rebuild_world, setup_scene};
use save::load_world;
use state::{BuilderMode, GameMode, GameSettings, PlacementState, SimulationState};
use world::{seed_demo_world, WorldBlocks};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.58, 0.68, 0.76)))
        .insert_resource(WorldBlocks::default())
        .insert_resource(PlacementState::default())
        .insert_resource(InventoryItems::default())
        .insert_resource(GameMode::Playing)
        .insert_resource(BuilderMode::default())
        .insert_resource(SimulationState::default())
        .insert_resource(GameSettings::default())
        .insert_resource(debug::DebugState::default())
        .insert_resource(CarriedItem::default())
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "OpenInfiniFactory Prototype".to_string(),
                    resolution: (1280.0, 720.0).into(),
                    present_mode: bevy::window::PresentMode::AutoVsync,
                    ..default()
                }),
                ..default()
            }),
            FrameTimeDiagnosticsPlugin,
        ))
        .add_systems(
            Startup,
            (
                setup_scene,
                spawn_player,
                load_world_on_startup,
                inventory::setup_ui,
                debug::setup_debug_ui,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                camera_move,
                camera_look,
                gameplay_input,
                placement_input,
                save_load_input,
                pause_menu_actions,
                simulation_controls,
                simulation_tick,
                apply_fov,
                update_hover,
                debug::toggle_debug,
                debug::update_debug_ui,
                debug::draw_player_collider,
                inventory::inventory_slot_clicks,
                inventory::update_ui,
                sync_cursor_grab,
            ),
        )
        .run();
}

fn load_world_on_startup(
    mut commands: Commands,
    mut world: ResMut<WorldBlocks>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !load_world(&mut world) {
        seed_demo_world(&mut world);
    }
    rebuild_world(&mut commands, &world, &mut meshes, &mut materials);
}
