use bevy::prelude::*;

use crate::game::state::{PlayingUiState, StartMenuScreen};
use crate::game::systems::debug::DebugPanel;
use crate::game::ui::PlayingUiRoot;
use crate::game::world::rendering::{
    teardown_playing_scene, BlockIconRenderRoot, GameplayScene, WorldRenderAssets,
};
use crate::game::world::grid::WorldBlocks;
use crate::game::systems::debug::DebugState;
use crate::game::simulation::factory_activity::FactoryStructureState;

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
    mut factory_structures: ResMut<FactoryStructureState>,
) {
    crate::game::world::rendering::rebuild_world_on_enter(
        &mut commands,
        &mut meshes,
        &world,
        &render_assets,
        &debug,
        &mut factory_structures,
    );
}

pub fn on_exit_playing(
    mut commands: Commands,
    mut playing_ui: ResMut<PlayingUiState>,
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

    teardown_playing_scene(&mut commands);
}
