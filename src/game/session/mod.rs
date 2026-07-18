mod busy;
mod cover;
mod dispatch;
mod load;
mod messages;
mod navigation;
mod save;
mod solution;
mod world_access;
mod world_ops;

pub use busy::SessionBusy;
pub use dispatch::{
    create_new_puzzle_in_world, create_new_solution_in_world, exit_to_main_menu,
    exit_to_main_menu_in_world, load_world, puzzle_save_needs_confirm, reset_solution_in_world,
    save_current_world, save_current_world_in_world, save_current_world_invalidate_in_world,
    save_current_world_invalidate_resources, save_current_world_resources,
    save_world_as_new_puzzle_in_world, switch_to_edit_mode_in_world,
};
pub use messages::LoadWorld;
pub use world_access::PlayingWorldParams;
pub use world_ops::SaveCurrentWorldResult;

use cover::PendingMainMenuExit;
#[cfg(not(target_arch = "wasm32"))]
use cover::{CoverScreenshotComplete, on_screenshot_saved_for_exit};

use bevy::prelude::*;

use crate::game::cameras::GameplayViewImage;
use crate::game::simulation::structure_state::StructureState;
use crate::game::state::{PlayingUiState, StartMenuScreen};
use crate::game::systems::debug::DebugState;
use crate::game::systems::perf::PerfScope;
use crate::game::ui::core::host::{PlayingUiRootEntity, UiHost};
use crate::game::ui::{PlayingUiRoot, UiRuntime};
use crate::game::world::grid::WorldBlocks;
use crate::game::world::rendering::{
    BlockIconRenderRoot, GameplayScene, WorldRenderAssets, teardown_playing_scene,
};

use load::{
    PendingWorldLoad, handle_create_new_puzzle, handle_create_new_solution, handle_load_world,
    poll_pending_world_load, release_session_busy_after_playing,
};
use messages::{
    CreateNewPuzzle, CreateNewSolution, ExitToMainMenu, ResetSolution, SaveCurrentWorld,
    SaveCurrentWorldInvalidateSolutions, SaveWorldAsNewPuzzle, SwitchToEditMode,
};
use navigation::{
    handle_exit_to_main_menu, process_deferred_main_menu_exit, release_session_busy_after_menu,
};
use save::{
    PendingSave, handle_save_current_world, handle_save_current_world_invalidate_solutions,
    handle_save_world_as_new_puzzle, process_pending_save,
};
use solution::{handle_reset_solution, handle_switch_to_edit_mode};

pub struct SessionPlugin;

impl Plugin for SessionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SessionBusy>()
            .init_resource::<PendingMainMenuExit>()
            .init_resource::<PendingWorldLoad>()
            .init_resource::<PendingSave>()
            .add_message::<SaveCurrentWorld>()
            .add_message::<SaveCurrentWorldInvalidateSolutions>()
            .add_message::<SaveWorldAsNewPuzzle>()
            .add_message::<ExitToMainMenu>()
            .add_message::<ResetSolution>()
            .add_message::<SwitchToEditMode>()
            .add_message::<LoadWorld>()
            .add_message::<CreateNewPuzzle>()
            .add_message::<CreateNewSolution>();
        #[cfg(not(target_arch = "wasm32"))]
        app.add_message::<CoverScreenshotComplete>()
            .add_observer(on_screenshot_saved_for_exit);
        app.add_systems(
            Update,
            (
                handle_save_current_world,
                handle_save_current_world_invalidate_solutions,
                process_pending_save,
                handle_save_world_as_new_puzzle,
                handle_exit_to_main_menu,
                process_deferred_main_menu_exit,
                handle_reset_solution,
                handle_switch_to_edit_mode,
                handle_load_world,
                poll_pending_world_load,
                release_session_busy_after_playing,
                release_session_busy_after_menu,
                handle_create_new_puzzle,
                handle_create_new_solution,
            )
                .chain()
                .after(PerfScope::Menus)
                .before(PerfScope::Simulation),
        );
        #[cfg(not(target_arch = "wasm32"))]
        app.add_systems(
            Update,
            navigation::finish_pending_main_menu_exit
                .after(process_deferred_main_menu_exit)
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
    mut render_assets: ResMut<WorldRenderAssets>,
    builder_mode: Res<crate::game::state::BuilderMode>,
    debug: Res<DebugState>,
    mut structure_state: ResMut<StructureState>,
    mut index: ResMut<crate::scene::BlockEntityIndex>,
    mut scene_chunks: ResMut<crate::game::world::rendering::SceneChunkMeshes>,
) {
    render_assets.set_goal_play_visual(*builder_mode == crate::game::state::BuilderMode::Play);
    crate::game::world::rendering::rebuild_world_on_enter(
        &mut commands,
        &mut meshes,
        &world,
        &render_assets,
        &debug,
        &mut structure_state,
        &mut index,
        &mut scene_chunks,
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
    mut meshes: ResMut<Assets<Mesh>>,
    mut scene_chunks: ResMut<crate::game::world::rendering::SceneChunkMeshes>,
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
    ui_host.unmount_all_panels(&mut ui_runtime, None);
    commands.insert_resource(
        crate::game::ui::features::playing_overlays::PlayingOverlayMounts::default(),
    );
    commands.remove_resource::<PlayingUiRootEntity>();

    crate::game::world::rendering::forget_scene_chunks(&mut meshes, &mut scene_chunks);
    teardown_playing_scene(&mut commands);
    commands.remove_resource::<GameplayViewImage>();
}
