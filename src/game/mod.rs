pub mod bevy_bridge;
pub mod block_editing;
pub mod blocks;
pub mod cameras;
pub mod debug;
pub mod player;
pub mod session;
pub mod simulation;
pub mod state;
pub mod systems;
pub mod ui;
pub mod world;

use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::input_focus::InputDispatchPlugin;
use bevy::light::{DirectionalLightShadowMap, GlobalAmbientLight};
use bevy::prelude::*;
use bevy::ui_widgets::{slider_self_update, UiWidgetsPlugins};

use crate::shared::config::load_config;
use crate::shared::i18n::{resolve_language, I18n};
use crate::shared::launch::LaunchOptions;
use crate::shared::save::SaveState;
use crate::scene::{sync_block_entity_index, BlockEntityIndex};
use crate::sim_core::{SimulationWorker, TurnCache};

use cameras::spawn_ui_camera;
use debug::DebugToolsPlugin;
use player::controller::{camera_look, camera_move, spawn_player, sync_cursor_grab};
use session::{on_exit_playing, prepare_playing_session, rebuild_playing_world, SessionPlugin};
use state::{
    BuilderMode, GameMode, GameSettings, PlacementState, PlayingUiState, SimulationState,
    SolutionState, StartMenuScreen,
};
use systems::gameplay::{
    apply_fov, draw_hover_structure_bounds, gameplay_input, placement_input, update_hover,
};
use systems::perf::{PerfPlugin, PerfScope};
use systems::simulation_controls::simulation_controls;
use ui::{GameUiPlugin, InventoryItems};
use world::animation::animate_blocks;
use world::grid::WorldBlocks;
use world::rendering::{retire_block_icon_renderers, HoverStructureBounds};

pub struct GamePlugin;

pub const UI_SCALE_MIN: f32 = 1.0;
pub const UI_SCALE_MAX: f32 = 3.0;
pub const GRAVITY_SCALE_MIN: f32 = 1.0;
pub const GRAVITY_SCALE_MAX: f32 = 2.0;
pub const GRAVITY_SCALE_DEFAULT: f32 = 1.2;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        blocks::assert_registry_consistent();

        let mut config = load_config();
        config.ui_scale = config.ui_scale.clamp(UI_SCALE_MIN, UI_SCALE_MAX);
        config.gravity_scale = config
            .gravity_scale
            .clamp(GRAVITY_SCALE_MIN, GRAVITY_SCALE_MAX);
        let i18n = I18n::new(resolve_language(config.language));
        let settings = GameSettings {
            fov_degrees: config.fov_degrees,
            ui_scale: config.ui_scale,
            gravity_scale: config.gravity_scale,
        };

        app.insert_resource(ClearColor(Color::srgb(0.58, 0.68, 0.76)))
            .insert_resource(GlobalAmbientLight {
                color: Color::srgb(0.90, 0.94, 1.0),
                brightness: 680.0,
                affects_lightmapped_meshes: true,
            })
            .insert_resource(DirectionalLightShadowMap { size: 2048 })
            .insert_resource(WorldBlocks::default())
            .insert_resource(HoverStructureBounds::default())
            .insert_resource(PlacementState::default())
            .insert_resource(InventoryItems::default())
            .init_state::<GameMode>()
            .insert_resource(StartMenuScreen::default())
            .insert_resource(PlayingUiState::default())
            .insert_resource(BuilderMode::default())
            .insert_resource(SimulationState::default())
            .insert_resource(SolutionState::default())
            .insert_resource(simulation::runtime::SignalNetworkCache::default())
            .insert_resource(simulation::runtime::SimulationStepStats::default())
            .insert_resource(simulation::runtime::PendingGeneratedMaterials::default())
            .insert_resource(simulation::structure_state::StructureState::default())
            .insert_resource(simulation::movement::PusherState::default())
            .insert_resource(simulation::structures::MovementInfluenceCache::default())
            .insert_resource(simulation::runtime::SimulationPresentationState::default())
            .insert_resource(BlockEntityIndex::default())
            .insert_resource(SimulationWorker::spawn())
            .insert_resource(TurnCache::default())
            .insert_resource(settings)
            .insert_resource(UiScale(config.ui_scale))
            .insert_resource(config)
            .insert_resource(i18n)
            .insert_resource(SaveState::default())
            .insert_resource(systems::debug::DebugState::default())
            .add_plugins(FrameTimeDiagnosticsPlugin::default())
            .add_plugins(InputDispatchPlugin)
            .add_plugins(UiWidgetsPlugins)
            .add_plugins(SessionPlugin)
            .add_plugins(GameUiPlugin)
            .add_plugins(PerfPlugin)
            .add_plugins(DebugToolsPlugin)
            .add_observer(slider_self_update)
            .add_systems(
                Startup,
                (
                    spawn_ui_camera,
                    refresh_saves_on_startup,
                    ui::load_ui_font,
                    systems::debug::load_debug_font,
                    ui::setup_menu_ui,
                )
                    .chain(),
            )
            .add_systems(
                OnEnter(GameMode::Playing),
                (
                    prepare_playing_session,
                    cameras::configure_ui_camera_for_playing,
                    world::rendering::setup_scene,
                    world::rendering::setup_block_icons,
                    spawn_player,
                    ui::setup_playing_ui_system,
                    systems::debug::setup_debug_ui,
                    rebuild_playing_world,
                    sync_block_entity_index,
                )
                    .chain(),
            )
            .add_systems(
                OnExit(GameMode::Playing),
                (on_exit_playing, cameras::configure_ui_camera_for_start_menu).chain(),
            )
            .add_systems(
                Update,
                (camera_move, camera_look, sync_cursor_grab)
                    .chain()
                    .before(PerfScope::Input),
            )
            .add_systems(Update, gameplay_input.before(PerfScope::Input))
            .add_systems(
                Update,
                update_hover.after(gameplay_input).before(placement_input),
            )
            .add_systems(
                Update,
                placement_input
                    .after(gameplay_input)
                    .before(PerfScope::Input),
            )
            .add_systems(
                Update,
                simulation_controls
                    .after(PerfScope::Menus)
                    .before(simulation::runtime::tick_simulation),
            )
            .add_systems(
                Update,
                simulation::runtime::poll_simulation_worker
                    .after(simulation_controls)
                    .before(simulation::runtime::tick_simulation),
            )
            .add_systems(
                Update,
                simulation::runtime::tick_simulation
                    .after(simulation::runtime::poll_simulation_worker)
                    .before(PerfScope::Simulation),
            )
            .add_systems(
                Update,
                (apply_fov, draw_hover_structure_bounds)
                    .chain()
                    .after(PerfScope::Simulation)
                    .before(PerfScope::View),
            )
            .add_systems(Update, animate_blocks.after(PerfScope::View))
            .add_systems(PostUpdate, sync_block_entity_index)
            .add_systems(Update, retire_block_icon_renderers)
            .add_systems(
                Update,
                (
                    systems::debug::toggle_debug,
                    systems::debug::toggle_factory_activity_debug,
                    systems::debug::update_debug_ui,
                    systems::debug::draw_player_collider,
                )
                    .chain()
                    .after(PerfScope::Ui)
                    .before(PerfScope::Debug),
            );

        #[cfg(not(target_arch = "wasm32"))]
        if app
            .world()
            .get_resource::<LaunchOptions>()
            .is_some_and(LaunchOptions::debug_http_enabled)
        {
            app.add_systems(Update, debug::poll_debug_http.before(simulation_controls));
        }
    }
}

fn refresh_saves_on_startup(mut save_state: ResMut<SaveState>) {
    save_state.refresh();
}
