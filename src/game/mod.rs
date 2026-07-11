pub mod block_editing;
pub mod blocks;
pub mod cameras;
pub mod debug;
pub mod edit_history;
pub mod player;
pub mod session;
pub mod simulation;
pub mod state;
pub mod systems;
pub mod ui;
pub mod world;

use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::light::{DirectionalLightShadowMap, GlobalAmbientLight};
use bevy::prelude::*;
use bevy::ui_widgets::slider_self_update;

use crate::scene::BlockEntityIndex;
use crate::shared::config::{load_config, GameConfig};
use crate::shared::i18n::{resolve_language, I18n};
use crate::shared::launch::LaunchOptions;
use crate::shared::persistent_storage::{self, StoragePlugin, StorageReady};
use crate::shared::save::SaveState;
use crate::sim_bridge::{SimulationWorker, TurnCache};

use cameras::{spawn_ui_camera, sync_gameplay_view_image_size};
#[cfg(not(target_arch = "wasm32"))]
use debug::DebugToolsPlugin;
use edit_history::{edit_history_input, EditHistory};
use player::controller::{
    apply_pending_player_spawn, camera_look, camera_move, spawn_player, sync_cursor_grab,
};
use session::{on_exit_playing, prepare_playing_session, rebuild_playing_world, SessionPlugin};
use state::{
    BuilderMode, GameMode, GameSettings, PendingPlayerSpawn, PlacementState, PlayingUiState,
    SimulationState, SolutionState, StartMenuScreen,
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
pub const MOUSE_SENSITIVITY_MIN: f32 = 0.5;
pub const MOUSE_SENSITIVITY_MAX: f32 = 2.0;
pub const MOUSE_SENSITIVITY_DEFAULT: f32 = 1.0;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        blocks::assert_registry_consistent();

        let launch = app.world().get_resource::<LaunchOptions>().cloned();
        if let Some(path) = launch
            .as_ref()
            .and_then(|options| options.config_path.clone())
        {
            persistent_storage::set_config_path_override(path);
        }

        let mut config = load_config();
        if let Some(language) = launch.as_ref().and_then(|options| options.language) {
            config.language = Some(language);
        }
        config.ui_scale = config.ui_scale.clamp(UI_SCALE_MIN, UI_SCALE_MAX);
        config.gravity_scale = config
            .gravity_scale
            .clamp(GRAVITY_SCALE_MIN, GRAVITY_SCALE_MAX);
        config.mouse_sensitivity_x = config
            .mouse_sensitivity_x
            .clamp(MOUSE_SENSITIVITY_MIN, MOUSE_SENSITIVITY_MAX);
        config.mouse_sensitivity_y = config
            .mouse_sensitivity_y
            .clamp(MOUSE_SENSITIVITY_MIN, MOUSE_SENSITIVITY_MAX);
        let i18n = I18n::new(resolve_language(config.language));
        let settings = GameSettings {
            fov_degrees: config.fov_degrees,
            ui_scale: config.ui_scale,
            gravity_scale: config.gravity_scale,
            mouse_sensitivity_x: config.mouse_sensitivity_x,
            mouse_sensitivity_y: config.mouse_sensitivity_y,
        };

        app.add_plugins(StoragePlugin)
            .insert_resource(ClearColor(Color::srgb(0.58, 0.68, 0.76)))
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
            .insert_resource(simulation::signals::SignalNetworkCache::default())
            .insert_resource(simulation::stats::SimulationStepStats::default())
            .insert_resource(simulation::pending::PendingGeneratedMaterials::default())
            .insert_resource(simulation::structure_state::StructureState::default())
            .insert_resource(simulation::movement::PusherState::default())
            .insert_resource(simulation::structures::MovementInfluenceCache::default())
            .insert_resource(crate::sim_bridge::SimulationPresentationState::default())
            .insert_resource(BlockEntityIndex::default())
            .insert_resource(SimulationWorker::spawn())
            .insert_resource(TurnCache::default())
            .insert_resource(settings)
            .insert_resource(UiScale(config.ui_scale))
            .insert_resource(config)
            .insert_resource(i18n)
            .insert_resource(SaveState::default())
            .init_resource::<EditHistory>()
            .init_resource::<PendingPlayerSpawn>()
            .insert_resource(systems::debug::DebugState::default())
            .add_plugins(FrameTimeDiagnosticsPlugin::default())
            .add_plugins(SessionPlugin)
            .add_plugins(GameUiPlugin)
            .add_plugins(PerfPlugin);
        #[cfg(not(target_arch = "wasm32"))]
        app.add_plugins(DebugToolsPlugin);
        app.add_observer(slider_self_update)
            .add_systems(
                Startup,
                (
                    spawn_ui_camera,
                    ui::load_ui_font,
                    systems::debug::load_debug_font,
                    ui::setup_menu_ui,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    apply_storage_ready,
                    apply_launch_load_save_when_ready,
                )
                    .chain()
                    .before(PerfScope::Menus),
            )
            .add_systems(
                OnEnter(GameMode::Playing),
                (
                    prepare_playing_session,
                    cameras::configure_ui_camera_for_playing,
                    world::rendering::setup_scene,
                    world::rendering::setup_block_icons,
                    spawn_player,
                    apply_pending_player_spawn,
                    ui::setup_playing_ui_system,
                    systems::debug::setup_debug_ui,
                    rebuild_playing_world,
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
            .add_systems(Update, sync_gameplay_view_image_size)
            .add_systems(Update, world::rendering::sync_shadow_settings)
            .add_systems(Update, world::rendering::sync_vsync_settings)
            .add_systems(Update, gameplay_input.before(PerfScope::Input))
            .add_systems(
                Update,
                edit_history_input
                    .after(gameplay_input)
                    .before(placement_input),
            )
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
                    .before(crate::sim_bridge::tick_simulation),
            )
            .add_systems(
                Update,
                crate::sim_bridge::poll_simulation_worker
                    .after(simulation_controls)
                    .before(crate::sim_bridge::tick_simulation),
            )
            .add_systems(
                Update,
                crate::sim_bridge::tick_simulation
                    .after(crate::sim_bridge::poll_simulation_worker)
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
    }
}

fn apply_storage_ready(
    ready: Res<StorageReady>,
    mut applied: Local<bool>,
    mut save_state: ResMut<SaveState>,
    mut config: ResMut<GameConfig>,
    mut i18n: ResMut<I18n>,
    mut settings: ResMut<GameSettings>,
    mut ui_scale: ResMut<UiScale>,
    launch: Res<LaunchOptions>,
) {
    if *applied || !ready.0 {
        return;
    }
    *applied = true;

    let mut loaded = load_config();
    if let Some(language) = launch.language {
        loaded.language = Some(language);
    }
    loaded.ui_scale = loaded.ui_scale.clamp(UI_SCALE_MIN, UI_SCALE_MAX);
    loaded.gravity_scale = loaded
        .gravity_scale
        .clamp(GRAVITY_SCALE_MIN, GRAVITY_SCALE_MAX);
    loaded.mouse_sensitivity_x = loaded
        .mouse_sensitivity_x
        .clamp(MOUSE_SENSITIVITY_MIN, MOUSE_SENSITIVITY_MAX);
    loaded.mouse_sensitivity_y = loaded
        .mouse_sensitivity_y
        .clamp(MOUSE_SENSITIVITY_MIN, MOUSE_SENSITIVITY_MAX);

    *i18n = I18n::new(resolve_language(loaded.language));
    settings.fov_degrees = loaded.fov_degrees;
    settings.ui_scale = loaded.ui_scale;
    settings.gravity_scale = loaded.gravity_scale;
    settings.mouse_sensitivity_x = loaded.mouse_sensitivity_x;
    settings.mouse_sensitivity_y = loaded.mouse_sensitivity_y;
    ui_scale.0 = loaded.ui_scale;
    *config = loaded;
    save_state.refresh();
}

/// 启动参数 `--load-save`：等存储就绪后再排队加载
fn apply_launch_load_save_when_ready(
    ready: Res<StorageReady>,
    launch: Res<LaunchOptions>,
    mut applied: Local<bool>,
    mut commands: Commands,
) {
    if *applied || !ready.0 {
        return;
    }
    *applied = true;
    let Some(raw) = launch.load_save.as_deref() else {
        return;
    };
    let Some(slot) = crate::shared::launch::resolve_launch_save_slot(raw) else {
        bevy::log::warn!("--load-save ignored: save `{raw}` not found or path invalid");
        return;
    };
    let entry = match slot.kind() {
        crate::shared::save::SaveKind::Puzzle => state::WorldEntryMode::EditPuzzle,
        crate::shared::save::SaveKind::Solution => state::WorldEntryMode::PlaySolution,
    };
    session::load_world(&mut commands, slot, entry);
}
