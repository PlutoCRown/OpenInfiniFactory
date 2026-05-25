mod blocks;
mod debug;
mod gameplay;
mod inventory;
mod player;
mod rendering;
mod save;
mod simulation;
mod state;
mod world;

use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;

use gameplay::{
    apply_fov, gameplay_input, main_menu_actions, pause_menu_actions, placement_input,
    save_list_actions, simulation_controls, update_hover,
};
use inventory::{CarriedItem, InventoryItems};
use player::{camera_look, camera_move, spawn_player, sync_cursor_grab};
use rendering::setup_scene;
use save::SaveState;
use state::{BuilderMode, GameMode, GameSettings, PlacementState, SimulationState};
use world::WorldBlocks;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.58, 0.68, 0.76)))
        .insert_resource(WorldBlocks::default())
        .insert_resource(PlacementState::default())
        .insert_resource(InventoryItems::default())
        .insert_resource(GameMode::MainMenu)
        .insert_resource(BuilderMode::default())
        .insert_resource(SimulationState::default())
        .insert_resource(GameSettings::default())
        .insert_resource(SaveState::default())
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
                refresh_saves_on_startup,
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
                main_menu_actions,
                save_list_actions,
                pause_menu_actions,
                simulation_controls,
                simulation::tick_simulation,
                apply_fov,
                update_hover,
            ),
        )
        .add_systems(
            Update,
            (
                inventory::inventory_slot_clicks,
                inventory::update_status_ui,
                inventory::update_panel_visibility,
                inventory::update_inventory_slots,
                inventory::update_save_list_ui,
                sync_cursor_grab,
            ),
        )
        .add_systems(
            Update,
            (
                debug::toggle_debug,
                debug::update_debug_ui,
                debug::draw_player_collider,
            ),
        )
        .run();
}

fn refresh_saves_on_startup(mut save_state: ResMut<SaveState>) {
    save_state.refresh();
}
