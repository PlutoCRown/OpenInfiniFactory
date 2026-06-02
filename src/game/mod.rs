pub mod player;
pub mod simulation;
pub mod state;
pub mod systems;
pub mod ui;
pub mod world;

use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::light::{DirectionalLightShadowMap, GlobalAmbientLight};
use bevy::prelude::*;
use bevy::ui_widgets::slider_self_update;

use crate::shared::config::load_config;
use crate::shared::i18n::{resolve_language, I18n};
use crate::shared::save::SaveState;

use player::controller::{camera_look, camera_move};
use state::{
    BuilderMode, GameMode, GameSettings, PlacementState, SimulationState, SolutionState,
    TeleportRenameState,
};
use systems::gameplay::{
    apply_fov, draw_hover_structure_bounds, gameplay_input, placement_input, update_hover,
};
use systems::simulation_controls::simulation_controls;
use systems::virtual_controls::{
    update_virtual_controls, update_virtual_controls_ui, VirtualControls, VirtualTouchState,
};
use ui::GameUiPlugin;
use world::animation::animate_blocks;
use world::grid::WorldBlocks;
use world::rendering::{
    retire_block_icon_renderers, setup_render_manager, GameWorldRuntime, HoverStructureBounds,
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
                color: Color::srgb(0.90, 0.94, 1.0),
                brightness: 680.0,
                affects_lightmapped_meshes: true,
            })
            .insert_resource(DirectionalLightShadowMap { size: 2048 })
            .insert_resource(WorldBlocks::default())
            .insert_resource(HoverStructureBounds::default())
            .insert_resource(GameWorldRuntime::default())
            .insert_resource(PlacementState::default())
            .insert_resource(TeleportRenameState::default())
            .insert_resource(GameMode::MainMenu)
            .insert_resource(BuilderMode::default())
            .insert_resource(VirtualControls::default())
            .insert_resource(VirtualTouchState::default())
            .insert_resource(settings)
            .insert_resource(UiScale(config.ui_scale))
            .insert_resource(config)
            .insert_resource(i18n)
            .insert_resource(SaveState::default())
            .insert_resource(systems::debug::DebugState::default())
            .add_plugins(FrameTimeDiagnosticsPlugin::default())
            .add_plugins(GameUiPlugin)
            .add_observer(slider_self_update)
            .add_systems(
                Startup,
                (
                    setup_render_manager,
                    refresh_saves_on_startup,
                    ui::load_ui_font,
                    ui::setup_ui,
                    setup_gameplay_state_resources,
                    systems::debug::setup_debug_ui,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    update_virtual_controls,
                    camera_move,
                    camera_look,
                    gameplay_input,
                    placement_input,
                )
                    .chain()
                    .run_if(world_loaded)
            )
            .add_systems(
                Update,
                (simulation_controls, simulation::runtime::tick_simulation)
                    .chain()
                    .run_if(world_loaded)
                    .after(placement_input),
            )
            .add_systems(
                Update,
                (apply_fov, update_hover, draw_hover_structure_bounds)
                    .chain()
                    .run_if(world_loaded)
                    .after(simulation::runtime::tick_simulation),
            )
            .add_systems(
                Update,
                simulation::runtime::update_movement_preview
                    .run_if(world_loaded)
                    .after(update_hover),
            )
            .add_systems(
                Update,
                animate_blocks
                    .run_if(world_loaded)
                    .after(simulation::runtime::update_movement_preview),
            )
            .add_systems(
                Update,
                update_virtual_controls_ui
                    .run_if(world_loaded)
                    .after(animate_blocks),
            )
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
                    .run_if(world_loaded)
                    .after(update_virtual_controls_ui),
            );
    }
}

fn refresh_saves_on_startup(mut save_state: ResMut<SaveState>) {
    save_state.refresh();
}

fn setup_gameplay_state_resources(mut commands: Commands) {
    commands.insert_resource(SimulationState::default());
    commands.insert_resource(SolutionState::default());
    commands.insert_resource(simulation::runtime::SignalNetworkCache::default());
    commands.insert_resource(simulation::runtime::SimulationStepStats::default());
    commands.insert_resource(simulation::runtime::MovementPreview::default());
    commands.insert_resource(simulation::runtime::PendingGeneratedMaterials::default());
    commands.insert_resource(simulation::factory_activity::FactoryStructureState::default());
    commands.insert_resource(simulation::movement::PusherState::default());
    commands.insert_resource(simulation::structures::MovementInfluenceCache::default());
}

pub(crate) fn world_loaded(save_state: Res<SaveState>) -> bool {
    save_state.current.is_some()
}
