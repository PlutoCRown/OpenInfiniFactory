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
use crate::shared::i18n::{resolve_language, I18n};
use crate::shared::save::SaveState;

use player::controller::{camera_look, camera_move, spawn_player, sync_cursor_grab};
use state::{BuilderMode, GameMode, GameSettings, PlacementState, SimulationState};
use systems::gameplay::{apply_fov, gameplay_input, placement_input, update_hover};
use systems::menus::{
    generator_menu_actions, main_menu_actions, pause_menu_actions, save_list_actions,
    settings_menu_actions,
};
use systems::simulation_controls::simulation_controls;
use ui::{CarriedItem, InventoryItems, PendingKeyBind, SettingsTab};
use world::grid::WorldBlocks;
use world::rendering::setup_scene;

pub struct GamePlugin;

pub const UI_SCALE_MIN: f32 = 1.0;
pub const UI_SCALE_MAX: f32 = 3.0;
pub const UI_SCALE_STEP: f32 = 0.1;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        let mut config = load_config();
        config.ui_scale = config.ui_scale.clamp(UI_SCALE_MIN, UI_SCALE_MAX);
        let i18n = I18n::new(resolve_language(config.language));
        let settings = GameSettings {
            fov_degrees: config.fov_degrees,
            ui_scale: config.ui_scale,
        };

        app.insert_resource(ClearColor(Color::srgb(0.58, 0.68, 0.76)))
            .insert_resource(AmbientLight {
                color: Color::srgb(0.78, 0.86, 1.0),
                brightness: 260.0,
            })
            .insert_resource(DirectionalLightShadowMap { size: 2048 })
            .insert_resource(WorldBlocks::default())
            .insert_resource(PlacementState::default())
            .insert_resource(InventoryItems::default())
            .insert_resource(GameMode::MainMenu)
            .insert_resource(BuilderMode::default())
            .insert_resource(SimulationState::default())
            .insert_resource(simulation::runtime::SignalNetworkCache::default())
            .insert_resource(settings)
            .insert_resource(UiScale(config.ui_scale))
            .insert_resource(config)
            .insert_resource(i18n)
            .insert_resource(SaveState::default())
            .insert_resource(SettingsTab::default())
            .insert_resource(PendingKeyBind::default())
            .insert_resource(systems::debug::DebugState::default())
            .insert_resource(systems::debug::PerfStats::default())
            .insert_resource(CarriedItem::default())
            .add_plugins((FrameTimeDiagnosticsPlugin, TemporalAntiAliasPlugin))
            .add_systems(
                Startup,
                (
                    setup_scene,
                    spawn_player,
                    refresh_saves_on_startup,
                    ui::load_ui_font,
                    ui::setup_ui,
                    systems::debug::setup_debug_ui,
                )
                    .chain(),
            )
            .add_systems(First, systems::debug::begin_perf_frame)
            .add_systems(
                Update,
                (camera_move, camera_look, gameplay_input, placement_input)
                    .chain()
                    .before(systems::debug::mark_perf_input),
            )
            .add_systems(Update, systems::debug::mark_perf_input)
            .add_systems(
                Update,
                (
                    main_menu_actions,
                    save_list_actions,
                    pause_menu_actions,
                    generator_menu_actions,
                    settings_menu_actions,
                )
                    .chain()
                    .after(systems::debug::mark_perf_input)
                    .before(systems::debug::mark_perf_menus),
            )
            .add_systems(Update, systems::debug::mark_perf_menus)
            .add_systems(
                Update,
                (simulation_controls, simulation::runtime::tick_simulation)
                    .chain()
                    .after(systems::debug::mark_perf_menus)
                    .before(systems::debug::mark_perf_simulation),
            )
            .add_systems(Update, systems::debug::mark_perf_simulation)
            .add_systems(
                Update,
                (apply_fov, update_hover)
                    .chain()
                    .after(systems::debug::mark_perf_simulation)
                    .before(systems::debug::mark_perf_view),
            )
            .add_systems(Update, systems::debug::mark_perf_view)
            .add_systems(
                Update,
                (
                    ui::inventory_slot_clicks,
                    ui::update_status_ui,
                    ui::update_localized_ui,
                    ui::update_settings_status_ui,
                    ui::update_panel_visibility,
                    ui::update_hud_visibility,
                    ui::update_generator_ui,
                    ui::update_inventory_slots,
                    ui::update_save_list_ui,
                    ui::apply_ui_font,
                    sync_cursor_grab,
                )
                    .after(systems::debug::mark_perf_view)
                    .before(systems::debug::mark_perf_ui),
            )
            .add_systems(Update, systems::debug::mark_perf_ui)
            .add_systems(
                Update,
                (
                    systems::debug::toggle_debug,
                    systems::debug::update_debug_ui,
                    systems::debug::draw_player_collider,
                )
                    .chain()
                    .after(systems::debug::mark_perf_ui)
                    .before(systems::debug::mark_perf_debug),
            )
            .add_systems(Update, systems::debug::mark_perf_debug)
            .add_systems(Last, systems::debug::finish_perf_frame);
    }
}

fn refresh_saves_on_startup(mut save_state: ResMut<SaveState>) {
    save_state.refresh();
}
