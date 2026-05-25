pub mod player;
pub mod simulation;
pub mod state;
pub mod systems;
pub mod ui;
pub mod world;

use bevy::core_pipeline::experimental::taa::TemporalAntiAliasPlugin;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::pbr::DirectionalLightShadowMap;
use bevy::prelude::*;

use crate::shared::config::load_config;
use crate::shared::save::SaveState;

use player::controller::{camera_look, camera_move, spawn_player, sync_cursor_grab};
use state::{BuilderMode, GameMode, GameSettings, PlacementState, SimulationState};
use systems::gameplay::{apply_fov, gameplay_input, placement_input, update_hover};
use systems::menus::{
    main_menu_actions, pause_menu_actions, save_list_actions, settings_menu_actions,
};
use systems::simulation_controls::simulation_controls;
use ui::{CarriedItem, InventoryItems, PendingKeyBind, SettingsTab};
use world::grid::WorldBlocks;
use world::rendering::setup_scene;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        let config = load_config();
        let settings = GameSettings {
            fov_degrees: config.fov_degrees,
        };

        app.insert_resource(ClearColor(Color::srgb(0.58, 0.68, 0.76)))
            .insert_resource(AmbientLight {
                color: Color::srgb(0.78, 0.86, 1.0),
                brightness: 260.0,
            })
            .insert_resource(DirectionalLightShadowMap { size: 4096 })
            .insert_resource(WorldBlocks::default())
            .insert_resource(PlacementState::default())
            .insert_resource(InventoryItems::default())
            .insert_resource(GameMode::MainMenu)
            .insert_resource(BuilderMode::default())
            .insert_resource(SimulationState::default())
            .insert_resource(settings)
            .insert_resource(config)
            .insert_resource(SaveState::default())
            .insert_resource(SettingsTab::default())
            .insert_resource(PendingKeyBind::default())
            .insert_resource(systems::debug::DebugState::default())
            .insert_resource(CarriedItem::default())
            .add_plugins((FrameTimeDiagnosticsPlugin, TemporalAntiAliasPlugin))
            .add_systems(
                Startup,
                (
                    setup_scene,
                    spawn_player,
                    refresh_saves_on_startup,
                    ui::setup_ui,
                    systems::debug::setup_debug_ui,
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
                    settings_menu_actions,
                    simulation_controls,
                    simulation::runtime::tick_simulation,
                    apply_fov,
                    update_hover,
                ),
            )
            .add_systems(
                Update,
                (
                    ui::inventory_slot_clicks,
                    ui::update_status_ui,
                    ui::update_settings_status_ui,
                    ui::update_panel_visibility,
                    ui::update_inventory_slots,
                    ui::update_save_list_ui,
                    sync_cursor_grab,
                ),
            )
            .add_systems(
                Update,
                (
                    systems::debug::toggle_debug,
                    systems::debug::update_debug_ui,
                    systems::debug::draw_player_collider,
                ),
            );
    }
}

fn refresh_saves_on_startup(mut save_state: ResMut<SaveState>) {
    save_state.refresh();
}
