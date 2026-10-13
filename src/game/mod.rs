pub mod audio;
pub mod block_editing;
pub mod blocks;
pub mod cameras;
pub mod debug;
pub mod edit_history;
pub mod input;
pub mod local_player;
pub mod material_blocks;
pub mod player;
pub mod scene_blocks;
pub mod schedule;
pub mod session;
pub mod showcase;
pub mod simulation;
pub mod state;
pub mod systems;
pub mod ui;
pub mod world;

use bevy::prelude::*;

use crate::shared::config::{GameConfig, load_config};
use crate::shared::i18n::{I18n, resolve_language};
use crate::shared::launch::LaunchOptions;
use crate::shared::persistent_storage::{self, StoragePlugin, StorageReady};
use crate::shared::save::SaveState;
use crate::shared::touch_profile::TouchProfile;

use cameras::spawn_player;
#[cfg(not(target_arch = "wasm32"))]
use debug::DebugToolsPlugin;
use player::controller::apply_pending_player_spawn;
use schedule::{GameSchedulePlugin, GameSet, GameStartupSet};
use session::{SessionPlugin, on_exit_playing, prepare_playing_session, rebuild_playing_world};
use state::GameMode;
use systems::perf::PerfPlugin;
use ui::GameUiPlugin;
use world::rendering::WorldRenderingPlugin;

/// 组装游戏模块与跨模块的场景生命周期。
pub struct GamePlugin;

/// 记录存储配置是否已应用，避免启动与异步就绪重复加载。
#[derive(Resource, Default)]
struct StorageConfigApplied(bool);

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

        let touch_profile = TouchProfile::detect_with_force(
            launch.as_ref().is_some_and(|options| options.force_touch),
        );
        app.add_plugins(GameSchedulePlugin)
            .add_plugins(StoragePlugin)
            .init_state::<GameMode>()
            .init_resource::<GameConfig>()
            .init_resource::<StorageConfigApplied>()
            .insert_resource(UiScale(
                touch_profile.effective_ui_scale(GameConfig::default().ui_scale),
            ))
            .insert_resource(I18n::new(resolve_language(None)))
            .insert_resource(touch_profile)
            .add_plugins(audio::GameAudioPlugin)
            .add_plugins(input::GameplayInputPlugin)
            .add_plugins(systems::gameplay::GameplayPlugin)
            .add_plugins(SessionPlugin)
            .add_plugins(crate::sim_bridge::SimulationBridgePlugin)
            .add_plugins(GameUiPlugin)
            .add_plugins(showcase::StartMenuShowcasePlugin)
            .add_plugins(PerfPlugin)
            .add_plugins(WorldRenderingPlugin);
        #[cfg(not(target_arch = "wasm32"))]
        app.add_plugins(DebugToolsPlugin);
        app.add_systems(
            Startup,
            apply_storage_ready.in_set(GameStartupSet::Configuration),
        )
        .add_systems(
            Update,
            (apply_storage_ready, apply_launch_load_save_when_ready)
                .chain()
                .before(GameSet::Menus),
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
        );
    }
}

/// 存储就绪后只应用一次配置及启动覆盖，并刷新存档列表。
fn apply_storage_ready(
    ready: Res<StorageReady>,
    mut applied: ResMut<StorageConfigApplied>,
    mut save_state: ResMut<SaveState>,
    mut config: ResMut<GameConfig>,
    mut i18n: ResMut<I18n>,
    mut ui_scale: ResMut<UiScale>,
    touch: Res<TouchProfile>,
    launch: Option<Res<LaunchOptions>>,
) {
    if applied.0 || !ready.0 {
        return;
    }
    applied.0 = true;

    let mut loaded = load_config();
    if let Some(language) = launch.as_ref().and_then(|launch| launch.language) {
        loaded.language = Some(language);
    }
    *i18n = I18n::new(resolve_language(loaded.language));
    ui_scale.0 = touch.effective_ui_scale(loaded.ui_scale);
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
    if let Some(name) = launch.create_test_free.as_deref() {
        let Some(slot) = crate::shared::save::create_test_free_from_single_block(name) else {
            bevy::log::warn!("failed to create test Free save `{name}`");
            return;
        };
        bevy::log::info!(
            "created test Free save `{}` from --create-test-free",
            slot.storage_path()
        );
        session::load_world(&mut commands, slot, state::WorldEntryMode::Free);
        return;
    }
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
        crate::shared::save::SaveKind::Free => state::WorldEntryMode::Free,
    };
    session::load_world(&mut commands, slot, entry);
}
