pub mod player;
pub mod simulation;
pub mod state;
pub mod systems;
pub mod ui;
pub mod world;

use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::camera::visibility::VisibilitySystems;
use bevy::input_focus::InputDispatchPlugin;
use bevy::light::{DirectionalLightShadowMap, GlobalAmbientLight};
use bevy::pbr::MaterialPlugin;
use bevy::prelude::*;
use bevy::transform::TransformSystems;
use bevy::ui::UiSystems;
use bevy::ui_widgets::{slider_self_update, UiWidgetsPlugins};

use crate::shared::config::load_config;
use crate::shared::i18n::{resolve_language, I18n};
use crate::shared::save::SaveState;

use player::controller::{camera_look, camera_move, spawn_player};
use state::{
    BuilderMode, GameMode, GameSettings, PlacementState, SimulationState, SolutionState,
    TeleportRenameState,
};
use systems::gameplay::{
    apply_fov, draw_hover_structure_bounds, gameplay_input, placement_input, update_hover,
};
use systems::menus::{app_exit_requests, main_menu_actions, pause_menu_actions, save_list_actions};
use systems::simulation_controls::simulation_controls;
use ui::{GameUiPlugin, InventoryItems};
use world::animation::animate_blocks;
use world::grid::WorldBlocks;
use world::rendering::{
    retire_block_icon_renderers, setup_block_icons, setup_scene, HoverStructureBounds,
};

pub struct GamePlugin;

pub const UI_SCALE_MIN: f32 = 1.0;
pub const UI_SCALE_MAX: f32 = 3.0;
pub const GRAVITY_SCALE_MIN: f32 = 1.0;
pub const GRAVITY_SCALE_MAX: f32 = 2.0;
pub const GRAVITY_SCALE_DEFAULT: f32 = 1.2;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        world::blocks::assert_registry_consistent();

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
                color: Color::srgb(0.78, 0.86, 1.0),
                brightness: 260.0,
                affects_lightmapped_meshes: true,
            })
            .insert_resource(DirectionalLightShadowMap { size: 2048 })
            .insert_resource(WorldBlocks::default())
            .insert_resource(HoverStructureBounds::default())
            .insert_resource(PlacementState::default())
            .insert_resource(TeleportRenameState::default())
            .insert_resource(InventoryItems::default())
            .insert_resource(GameMode::MainMenu)
            .insert_resource(BuilderMode::default())
            .insert_resource(SimulationState::default())
            .insert_resource(SolutionState::default())
            .insert_resource(simulation::runtime::SignalNetworkCache::default())
            .insert_resource(simulation::runtime::SimulationStepStats::default())
            .insert_resource(simulation::runtime::PendingGeneratedMaterials::default())
            .insert_resource(simulation::factory_activity::FactoryStructureState::default())
            .insert_resource(settings)
            .insert_resource(UiScale(config.ui_scale))
            .insert_resource(config)
            .insert_resource(i18n)
            .insert_resource(SaveState::default())
            .insert_resource(ui::PendingAppExit::default())
            .insert_resource(systems::debug::DebugState::default())
            .insert_resource(systems::debug::PerfStats::default())
            .add_plugins(FrameTimeDiagnosticsPlugin::default())
            .add_plugins(InputDispatchPlugin)
            .add_plugins(UiWidgetsPlugins)
            .add_plugins(GameUiPlugin)
            .add_observer(slider_self_update)
            .add_plugins(MaterialPlugin::<world::scene_material::SceneBlockMaterial>::default())
            .add_systems(
                Startup,
                (
                    setup_scene,
                    setup_block_icons,
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
                PreUpdate,
                systems::debug::mark_perf_pre_update.after(UiSystems::Focus),
            )
            .add_systems(
                Update,
                (camera_move, camera_look, gameplay_input, placement_input)
                    .chain()
                    .before(systems::debug::mark_perf_input),
            )
            .add_systems(Update, systems::debug::mark_perf_input)
            .add_systems(Last, app_exit_requests.before(systems::debug::mark_perf_last))
            .add_systems(
                Update,
                (main_menu_actions, save_list_actions, pause_menu_actions)
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
                (apply_fov, update_hover, draw_hover_structure_bounds)
                    .chain()
                    .after(systems::debug::mark_perf_simulation)
                    .before(systems::debug::mark_perf_view),
            )
            .add_systems(Update, systems::debug::mark_perf_view)
            .add_systems(Update, animate_blocks.after(systems::debug::mark_perf_view))
            .add_systems(Update, systems::debug::mark_perf_animation)
            .add_systems(Update, systems::debug::mark_perf_ui)
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
                    .after(systems::debug::mark_perf_ui)
                    .before(systems::debug::mark_perf_debug),
            )
            .add_systems(Update, systems::debug::mark_perf_debug)
            .add_systems(
                PostUpdate,
                systems::debug::mark_perf_post_update_start.before(UiSystems::Prepare),
            )
            .add_systems(
                PostUpdate,
                systems::debug::mark_perf_post_update_ui
                    .after(UiSystems::Layout)
                    .before(TransformSystems::Propagate),
            )
            .add_systems(
                PostUpdate,
                systems::debug::mark_perf_post_update_transform
                    .after(TransformSystems::Propagate)
                    .before(VisibilitySystems::UpdateFrusta),
            )
            .add_systems(
                PostUpdate,
                systems::debug::mark_perf_post_update_visibility
                    .after(VisibilitySystems::MarkNewlyHiddenEntitiesInvisible),
            )
            .add_systems(
                Last,
                systems::debug::mark_perf_last.before(systems::debug::finish_perf_frame),
            )
            .add_systems(Last, systems::debug::finish_perf_frame);
    }
}

fn refresh_saves_on_startup(mut save_state: ResMut<SaveState>) {
    save_state.refresh();
}
