mod dispatch;
mod load;
mod messages;
mod navigation;
mod save;
mod solution;
mod world_access;
mod world_ops;

#[allow(unused_imports)]
pub use dispatch::{
    create_new_puzzle, create_new_solution, reset_solution, save_world_as_new_puzzle,
    switch_to_edit_mode,
};
pub use dispatch::{
    create_new_puzzle_in_world, create_new_solution_in_world, exit_to_main_menu,
    exit_to_main_menu_in_world, load_world, reset_solution_in_world, save_current_world,
    save_world_as_new_puzzle_in_world, switch_to_edit_mode_in_world,
};
pub use world_access::PlayingWorldParams;

use bevy::prelude::*;

use crate::game::simulation::structure_state::StructureState;
use crate::game::state::{PlayingUiState, StartMenuScreen, UiPanelId};
use crate::game::systems::debug::DebugPanel;
use crate::game::systems::debug::DebugState;
use crate::game::systems::perf::PerfScope;
use crate::game::ui::core::host::{PlayingUiRootEntity, UiHost};
use crate::game::ui::{PlayingUiRoot, UiRuntime};
use crate::game::world::grid::WorldBlocks;
use crate::game::world::rendering::{
    teardown_playing_scene, BlockIconRenderRoot, GameplayScene, WorldRenderAssets,
};

use load::{handle_create_new_puzzle, handle_create_new_solution, handle_load_world};
use messages::{
    CreateNewPuzzle, CreateNewSolution, ExitToMainMenu, LoadWorld, ResetSolution, SaveCurrentWorld,
    SaveWorldAsNewPuzzle, SwitchToEditMode,
};
use navigation::handle_exit_to_main_menu;
use save::{handle_save_current_world, handle_save_world_as_new_puzzle};
use solution::{handle_reset_solution, handle_switch_to_edit_mode};

pub struct SessionPlugin;

impl Plugin for SessionPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SaveCurrentWorld>()
            .add_message::<SaveWorldAsNewPuzzle>()
            .add_message::<ExitToMainMenu>()
            .add_message::<ResetSolution>()
            .add_message::<SwitchToEditMode>()
            .add_message::<LoadWorld>()
            .add_message::<CreateNewPuzzle>()
            .add_message::<CreateNewSolution>()
            .add_systems(
                Update,
                (
                    handle_save_current_world,
                    handle_save_world_as_new_puzzle,
                    handle_exit_to_main_menu,
                    handle_reset_solution,
                    handle_switch_to_edit_mode,
                    handle_load_world,
                    handle_create_new_puzzle,
                    handle_create_new_solution,
                )
                    .chain()
                    .after(PerfScope::Menus)
                    .before(PerfScope::Simulation),
            );
    }
}

pub fn prepare_playing_session(
    mut playing_ui: ResMut<PlayingUiState>,
    mut start_menu_screen: ResMut<StartMenuScreen>,
) {
    playing_ui.reset();
    *start_menu_screen = StartMenuScreen::Main;
}

pub fn rebuild_playing_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    world: Res<WorldBlocks>,
    render_assets: Res<WorldRenderAssets>,
    debug: Res<DebugState>,
    mut structure_state: ResMut<StructureState>,
) {
    crate::game::world::rendering::rebuild_world_on_enter(
        &mut commands,
        &mut meshes,
        &world,
        &render_assets,
        &debug,
        &mut structure_state,
    );
}

pub fn on_exit_playing(
    mut commands: Commands,
    mut playing_ui: ResMut<PlayingUiState>,
    mut ui_runtime: ResMut<UiRuntime>,
    mut ui_host: ResMut<UiHost>,
    gameplay_scene: Query<Entity, With<GameplayScene>>,
    icon_roots: Query<Entity, With<BlockIconRenderRoot>>,
    playing_ui_roots: Query<Entity, With<PlayingUiRoot>>,
    debug_panels: Query<Entity, With<DebugPanel>>,
) {
    playing_ui.reset();

    for entity in &gameplay_scene {
        commands.entity(entity).despawn();
    }
    for entity in &icon_roots {
        commands.entity(entity).despawn();
    }
    for entity in &playing_ui_roots {
        commands.entity(entity).despawn();
    }
    for entity in &debug_panels {
        commands.entity(entity).despawn();
    }
    ui_host.unmount_panel(UiPanelId::Settings, &mut ui_runtime, None);
    commands.remove_resource::<PlayingUiRootEntity>();

    teardown_playing_scene(&mut commands);
}
