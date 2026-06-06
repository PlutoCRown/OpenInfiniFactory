use bevy::prelude::*;

use crate::game::simulation::structure_state::StructureState;
use crate::game::simulation::markers::refresh_static_generated_markers;
use crate::game::simulation::movement::PusherState;
use crate::game::simulation::structures::MovementInfluenceCache;
use crate::game::state::{
    BuilderMode, GameMode, PlacementState, PlayingUiState, SimulationState, SolutionState,
    StartMenuScreen, WorldEntryMode,
};
use crate::game::systems::debug::DebugState;
use crate::game::ui::{CarriedItem, InventoryItems};
use crate::game::world::grid::WorldBlocks;
use crate::game::world::rendering::{
    despawn_world, rebuild_world_for_debug_state, BlockEntity, WorldRenderAssets,
};
use crate::shared::save::{
    load_world, next_named_save, reset_solution_world, save_solution_with_puzzle, save_world, SaveKind,
    SaveState,
};

pub fn save_current_world(
    world: &WorldBlocks,
    inventory: &InventoryItems,
    save_state: &mut SaveState,
    solution_state: &mut SolutionState,
    simulation: &SimulationState,
) {
    let world = simulation.authoring_world(world);
    let kind = save_state.current_kind.unwrap_or(SaveKind::Puzzle);
    let name = save_state.current.clone().unwrap_or_else(|| {
        next_named_save(
            &save_state
                .entries
                .iter()
                .map(|entry| entry.name.clone())
                .collect::<Vec<_>>(),
            "world",
        )
    });
    let saved = match kind {
        SaveKind::Puzzle => save_world(world, &name, SaveKind::Puzzle, inventory),
        SaveKind::Solution => {
            if let Some(puzzle_snapshot) = &solution_state.puzzle_snapshot {
                save_solution_with_puzzle(world, &name, puzzle_snapshot, inventory)
            } else {
                save_world(world, &name, SaveKind::Solution, inventory)
            }
        }
    };
    if saved {
        save_state.current = Some(name);
        save_state.current_kind = Some(kind);
        solution_state.dirty = false;
        save_state.refresh();
    }
}

pub fn switch_to_edit_mode_and_rebuild(
    world: &mut WorldBlocks,
    builder_mode: &mut BuilderMode,
    inventory: &mut InventoryItems,
    carried: &mut CarriedItem,
    placement: &mut PlacementState,
    playing_ui: &mut PlayingUiState,
    save_state: &mut SaveState,
    solution_state: &mut SolutionState,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    block_entities: &Query<Entity, With<BlockEntity>>,
    render_assets: Option<&WorldRenderAssets>,
    debug: &DebugState,
    structure_state: &mut StructureState,
    movement_influence: &mut MovementInfluenceCache,
    pusher_state: &mut PusherState,
) {
    switch_to_edit_mode(
        world,
        builder_mode,
        inventory,
        carried,
        placement,
        playing_ui,
        save_state,
        solution_state,
    );
    if let Some(render_assets) = render_assets {
        despawn_world(commands, block_entities);
        structure_state.clear();
        movement_influence.clear();
        pusher_state.clear();
        rebuild_world_for_debug_state(
            commands,
            meshes,
            world,
            render_assets,
            debug,
            structure_state,
        );
    }
}

pub fn reset_current_solution(
    world: &mut WorldBlocks,
    simulation: &mut SimulationState,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    block_entities: &Query<Entity, With<BlockEntity>>,
    render_assets: Option<&WorldRenderAssets>,
    debug: &DebugState,
    structure_state: &mut StructureState,
    movement_influence: &mut MovementInfluenceCache,
    pusher_state: &mut PusherState,
    solution_state: &SolutionState,
) {
    if let Some(puzzle_snapshot) = &solution_state.puzzle_snapshot {
        reset_solution_world(world, puzzle_snapshot);
        refresh_static_generated_markers(world);
        simulation.running = false;
        simulation.step_requested = false;
        simulation.turn = 0;
        simulation.accumulator = 0.0;
        simulation.start_snapshot = None;
        simulation.start_structures = None;
        structure_state.clear();
        movement_influence.clear();
        pusher_state.clear();
        if let Some(render_assets) = render_assets {
            despawn_world(commands, block_entities);
            rebuild_world_for_debug_state(
                commands,
                meshes,
                world,
                render_assets,
                debug,
                structure_state,
            );
        }
    }
}

pub fn exit_to_main_menu(
    world: &mut WorldBlocks,
    placement: &mut PlacementState,
    save_state: &mut SaveState,
    solution_state: &mut SolutionState,
    simulation: &mut SimulationState,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    render_assets: Option<&WorldRenderAssets>,
    block_entities: &Query<Entity, With<BlockEntity>>,
    debug: &DebugState,
    structure_state: &mut StructureState,
    movement_influence: &mut MovementInfluenceCache,
    pusher_state: &mut PusherState,
    next_state: &mut NextState<GameMode>,
    start_menu_screen: &mut StartMenuScreen,
) {
    clear_loaded_world(
        world,
        placement,
        save_state,
        solution_state,
        simulation,
        commands,
        meshes,
        render_assets,
        block_entities,
        debug,
        structure_state,
        movement_influence,
        pusher_state,
    );
    *start_menu_screen = StartMenuScreen::Main;
    next_state.set(GameMode::StartMenu);
}

pub fn load_world_into_session(
    name: &str,
    entry: WorldEntryMode,
    world: &mut WorldBlocks,
    builder_mode: &mut BuilderMode,
    inventory: &mut InventoryItems,
    carried: &mut CarriedItem,
    placement: &mut PlacementState,
    save_state: &mut SaveState,
    solution_state: &mut SolutionState,
    simulation: &mut SimulationState,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    block_entities: &Query<Entity, With<BlockEntity>>,
    render_assets: Option<&WorldRenderAssets>,
    debug: &DebugState,
    structure_state: &mut StructureState,
    movement_influence: &mut MovementInfluenceCache,
    pusher_state: &mut PusherState,
    current_mode: GameMode,
    next_state: &mut NextState<GameMode>,
) {
    let Some(loaded) = load_world(world, name) else {
        return;
    };

    simulation.running = false;
    simulation.step_requested = false;
    simulation.turn = 0;
    simulation.accumulator = 0.0;
    simulation.start_snapshot = None;
    simulation.start_structures = None;
    placement.selection.clear();
    placement.edit_gesture = None;
    carried.clear();

    *builder_mode = match entry {
        WorldEntryMode::EditPuzzle => BuilderMode::Edit,
        WorldEntryMode::PlaySolution => BuilderMode::Play,
    };
    *inventory = InventoryItems::for_mode(*builder_mode);
    if let Some(hotbar) = loaded.hotbar {
        inventory.set_hotbar(hotbar);
    }
    placement.selected = 0;

    save_state.current = Some(name.to_string());
    save_state.current_kind = Some(match entry {
        WorldEntryMode::EditPuzzle => SaveKind::Puzzle,
        WorldEntryMode::PlaySolution => SaveKind::Solution,
    });
    save_state.select_puzzle(None, None);

    solution_state.entry = entry;
    solution_state.dirty = false;
    solution_state.puzzle_snapshot = match entry {
        WorldEntryMode::EditPuzzle => None,
        WorldEntryMode::PlaySolution => loaded.puzzle_snapshot.or_else(|| Some(loaded.world)),
    };

    refresh_static_generated_markers(world);
    structure_state.clear();
    movement_influence.clear();
    pusher_state.clear();

    match current_mode {
        GameMode::StartMenu => next_state.set(GameMode::Playing),
        GameMode::Playing => {
            if let Some(render_assets) = render_assets {
                despawn_world(commands, block_entities);
                rebuild_world_for_debug_state(
                    commands,
                    meshes,
                    world,
                    render_assets,
                    debug,
                    structure_state,
                );
            }
        }
    }
}

pub fn clear_loaded_world(
    world: &mut WorldBlocks,
    placement: &mut PlacementState,
    save_state: &mut SaveState,
    solution_state: &mut SolutionState,
    simulation: &mut SimulationState,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    render_assets: Option<&WorldRenderAssets>,
    block_entities: &Query<Entity, With<BlockEntity>>,
    debug: &DebugState,
    structure_state: &mut StructureState,
    movement_influence: &mut MovementInfluenceCache,
    pusher_state: &mut PusherState,
) {
    simulation.running = false;
    simulation.step_requested = false;
    simulation.accumulator = 0.0;
    simulation.start_snapshot = None;
    simulation.start_structures = None;
    placement.selection.clear();
    placement.edit_gesture = None;
    world.clear();
    save_state.current = None;
    save_state.current_kind = None;
    save_state.select_puzzle(None, None);
    solution_state.puzzle_snapshot = None;
    solution_state.dirty = false;
    solution_state.entry = WorldEntryMode::EditPuzzle;
    structure_state.clear();
    movement_influence.clear();
    pusher_state.clear();
    if let Some(render_assets) = render_assets {
        despawn_world(commands, block_entities);
        rebuild_world_for_debug_state(
            commands,
            meshes,
            world,
            render_assets,
            debug,
            structure_state,
        );
    }
}

fn switch_to_edit_mode(
    world: &mut WorldBlocks,
    builder_mode: &mut BuilderMode,
    inventory: &mut InventoryItems,
    carried: &mut CarriedItem,
    placement: &mut PlacementState,
    playing_ui: &mut PlayingUiState,
    save_state: &mut SaveState,
    solution_state: &mut SolutionState,
) {
    if let Some(puzzle_snapshot) = &solution_state.puzzle_snapshot {
        *world = puzzle_snapshot.clone();
        refresh_static_generated_markers(world);
    }
    *builder_mode = BuilderMode::Edit;
    *inventory = InventoryItems::for_mode(*builder_mode);
    carried.clear();
    placement.selected = 0;
    save_state.current_kind = Some(SaveKind::Puzzle);
    solution_state.puzzle_snapshot = None;
    playing_ui.paused = true;
}
