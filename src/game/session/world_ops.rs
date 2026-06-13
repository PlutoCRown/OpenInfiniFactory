use bevy::prelude::*;

use crate::game::simulation::markers::refresh_static_generated_markers;
use crate::game::simulation::movement::PusherState;
use crate::game::simulation::structure_state::StructureState;
use crate::game::simulation::structures::MovementInfluenceCache;
use crate::game::state::{
    BuilderMode, GameMode, PlacementState, PlayingUiState, SimulationState, SolutionState,
    StartMenuScreen, WorldEntryMode,
};
use crate::game::systems::debug::DebugState;
use crate::game::ui::{CarriedItem, InventoryItems};
use crate::game::world::block_instance::MaterialBlockRegistry;
use crate::game::world::factory_registry::FactoryBlockRegistry;
use crate::game::world::grid::WorldBlocks;
use crate::game::world::rendering::{
    despawn_world, rebuild_world_for_debug_state, BlockEntity, WorldRenderAssets,
};
use crate::shared::save::{
    has_solutions_for_puzzle, invalidate_solutions_for_puzzle, load_world, next_named_save,
    reset_solution_world, save_puzzle, save_solution, SaveKind, SaveState,
};

pub enum SaveCurrentWorldResult {
    Saved,
    NeedsPuzzleConfirm,
    Failed,
}

pub fn puzzle_save_needs_confirm(save_state: &SaveState) -> bool {
    save_state.current_kind == Some(SaveKind::Puzzle)
        && save_state
            .current
            .as_deref()
            .is_some_and(has_solutions_for_puzzle)
}

pub fn save_current_world(
    world: &WorldBlocks,
    inventory: &InventoryItems,
    save_state: &mut SaveState,
    solution_state: &mut SolutionState,
    simulation: &SimulationState,
) -> SaveCurrentWorldResult {
    if puzzle_save_needs_confirm(save_state) {
        return SaveCurrentWorldResult::NeedsPuzzleConfirm;
    }
    if commit_save_current_world(
        world,
        inventory,
        save_state,
        solution_state,
        simulation,
        false,
    ) {
        SaveCurrentWorldResult::Saved
    } else {
        SaveCurrentWorldResult::Failed
    }
}

pub fn save_current_world_invalidate_solutions(
    world: &WorldBlocks,
    inventory: &InventoryItems,
    save_state: &mut SaveState,
    solution_state: &mut SolutionState,
    simulation: &SimulationState,
) -> bool {
    commit_save_current_world(
        world,
        inventory,
        save_state,
        solution_state,
        simulation,
        true,
    )
}

fn commit_save_current_world(
    world: &WorldBlocks,
    inventory: &InventoryItems,
    save_state: &mut SaveState,
    solution_state: &mut SolutionState,
    simulation: &SimulationState,
    invalidate_solutions: bool,
) -> bool {
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
        SaveKind::Puzzle => {
            if invalidate_solutions {
                invalidate_solutions_for_puzzle(&name);
            }
            save_puzzle(world, &name, inventory)
        }
        SaveKind::Solution => {
            let Some(puzzle_id) = solution_state
                .puzzle_id
                .clone()
                .or_else(|| save_state.current.clone())
            else {
                return false;
            };
            save_solution(world, &name, &puzzle_id, inventory)
        }
    };
    if saved {
        save_state.current = Some(name);
        save_state.current_kind = Some(kind);
        solution_state.dirty = false;
        save_state.refresh();
    }
    saved
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
    block_entities: &Query<(Entity, &BlockEntity)>,
    render_assets: Option<&WorldRenderAssets>,
    debug: &DebugState,
    structure_state: &mut StructureState,
    factory_registry: &mut FactoryBlockRegistry,
    material_registry: &mut MaterialBlockRegistry,
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
        *factory_registry = FactoryBlockRegistry::rebuild_from_world(world);
        *material_registry = MaterialBlockRegistry::rebuild_from_world(world);
        rebuild_world_for_debug_state(
            commands,
            meshes,
            world,
            render_assets,
            debug,
            structure_state,
            factory_registry,
            material_registry,
        );
    }
}

pub fn reset_current_solution(
    world: &mut WorldBlocks,
    simulation: &mut SimulationState,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    block_entities: &Query<(Entity, &BlockEntity)>,
    render_assets: Option<&WorldRenderAssets>,
    debug: &DebugState,
    structure_state: &mut StructureState,
    factory_registry: &mut FactoryBlockRegistry,
    material_registry: &mut MaterialBlockRegistry,
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
        *factory_registry = FactoryBlockRegistry::rebuild_from_world(world);
        *material_registry = MaterialBlockRegistry::rebuild_from_world(world);
        if let Some(render_assets) = render_assets {
            despawn_world(commands, block_entities);
            rebuild_world_for_debug_state(
                commands,
                meshes,
                world,
                render_assets,
                debug,
                structure_state,
                factory_registry,
                material_registry,
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
    block_entities: &Query<(Entity, &BlockEntity)>,
    debug: &DebugState,
    structure_state: &mut StructureState,
    factory_registry: &mut FactoryBlockRegistry,
    material_registry: &mut MaterialBlockRegistry,
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
        factory_registry,
        material_registry,
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
    block_entities: &Query<(Entity, &BlockEntity)>,
    render_assets: Option<&WorldRenderAssets>,
    debug: &DebugState,
    structure_state: &mut StructureState,
    factory_registry: &mut FactoryBlockRegistry,
    material_registry: &mut MaterialBlockRegistry,
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
    save_state.select_puzzle(None);

    solution_state.entry = entry;
    solution_state.dirty = false;
    solution_state.puzzle_id = loaded.puzzle_id;
    solution_state.puzzle_snapshot = match entry {
        WorldEntryMode::EditPuzzle => None,
        WorldEntryMode::PlaySolution => loaded.puzzle_snapshot,
    };

    refresh_static_generated_markers(world);
    structure_state.clear();
    movement_influence.clear();
    pusher_state.clear();
    *factory_registry = FactoryBlockRegistry::rebuild_from_world(world);
    *material_registry = MaterialBlockRegistry::rebuild_from_world(world);

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
                    factory_registry,
                    material_registry,
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
    block_entities: &Query<(Entity, &BlockEntity)>,
    debug: &DebugState,
    structure_state: &mut StructureState,
    factory_registry: &mut FactoryBlockRegistry,
    material_registry: &mut MaterialBlockRegistry,
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
    save_state.select_puzzle(None);
    solution_state.puzzle_snapshot = None;
    solution_state.puzzle_id = None;
    solution_state.dirty = false;
    solution_state.entry = WorldEntryMode::EditPuzzle;
    structure_state.clear();
    movement_influence.clear();
    pusher_state.clear();
    factory_registry.clear();
    material_registry.clear();
    if let Some(render_assets) = render_assets {
        despawn_world(commands, block_entities);
        rebuild_world_for_debug_state(
            commands,
            meshes,
            world,
            render_assets,
            debug,
            structure_state,
            factory_registry,
            material_registry,
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
    solution_state.puzzle_id = None;
    playing_ui.paused = true;
}
