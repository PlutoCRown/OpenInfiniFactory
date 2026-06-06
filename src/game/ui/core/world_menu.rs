use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::game::simulation::factory_activity::FactoryStructureState;
use crate::game::simulation::markers::refresh_static_generated_markers;
use crate::game::simulation::movement::PusherState;
use crate::game::simulation::structures::MovementInfluenceCache;
use crate::game::state::{
    BuilderMode, GameMode, PlacementState, PlayingUiState, SimulationState, SolutionState,
    StartMenuScreen, WorldEntryMode,
};
use crate::game::systems::debug::DebugState;
use crate::game::ui::features::save::types::TextPromptKind;
use crate::game::ui::features::save::types::TextPromptState;
use crate::game::world::grid::{seed_demo_world, WorldBlocks};
use crate::game::world::rendering::{
    despawn_world, rebuild_world_for_debug_state, BlockEntity, WorldRenderAssets,
};
use crate::shared::save::{
    load_world, next_named_save, rename_save, reset_solution_world, save_solution_with_puzzle,
    save_world, SaveKind, SaveState,
};

#[derive(SystemParam)]
pub struct WorldMenuParams<'w, 's> {
    pub commands: Commands<'w, 's>,
    pub meshes: ResMut<'w, Assets<Mesh>>,
    pub world: ResMut<'w, WorldBlocks>,
    pub render_assets: Option<Res<'w, WorldRenderAssets>>,
    pub debug: Res<'w, DebugState>,
    pub factory_structures: ResMut<'w, FactoryStructureState>,
    pub movement_influence: ResMut<'w, MovementInfluenceCache>,
    pub pusher_state: ResMut<'w, PusherState>,
    pub block_entities: Query<'w, 's, Entity, With<BlockEntity>>,
}

pub(crate) fn open_text_prompt(prompt: &mut TextPromptState, kind: TextPromptKind, value: &str) {
    prompt.kind = Some(kind);
    prompt.value = value.chars().take(24).collect();
}

pub(crate) fn confirm_text_prompt(
    prompt: &mut TextPromptState,
    current_mode: GameMode,
    next_state: &mut NextState<GameMode>,
    builder_mode: &mut BuilderMode,
    inventory: &mut crate::game::ui::types::InventoryItems,
    carried: &mut crate::game::ui::types::CarriedItem,
    placement: &mut PlacementState,
    save_state: &mut SaveState,
    solution_state: &mut SolutionState,
    simulation: &mut SimulationState,
    world_menu: &mut WorldMenuParams,
) {
    let Some(kind) = prompt.kind.clone() else {
        return;
    };
    let requested = prompt.value.clone();
    let existing = save_state
        .entries
        .iter()
        .map(|entry| entry.name.clone())
        .collect::<Vec<_>>();
    let name = match &kind {
        TextPromptKind::RenamePuzzle { name: old }
        | TextPromptKind::RenameSolution { name: old }
            if requested.trim() == old =>
        {
            old.clone()
        }
        _ => next_named_save(&existing, &requested),
    };
    if name.is_empty() {
        return;
    }

    match kind {
        TextPromptKind::NewPuzzle => {
            world_menu.world.clear();
            seed_demo_world(&mut world_menu.world);
            *inventory = crate::game::ui::types::InventoryItems::for_mode(BuilderMode::Edit);
            if save_world(&world_menu.world, &name, SaveKind::Puzzle, inventory) {
                save_state.refresh();
                open_loaded_world_from_menu(
                    &name,
                    WorldEntryMode::EditPuzzle,
                    current_mode,
                    next_state,
                    builder_mode,
                    inventory,
                    carried,
                    placement,
                    save_state,
                    solution_state,
                    simulation,
                    world_menu,
                );
            }
        }
        TextPromptKind::NewSolution { puzzle } => {
            let Some(loaded) = load_world(&mut world_menu.world, &puzzle) else {
                return;
            };
            let puzzle_snapshot = loaded
                .puzzle_snapshot
                .unwrap_or_else(|| world_menu.world.clone());
            *world_menu.world = puzzle_snapshot.clone();
            *inventory = crate::game::ui::types::InventoryItems::for_mode(BuilderMode::Play);
            if save_solution_with_puzzle(&world_menu.world, &name, &puzzle_snapshot, inventory) {
                save_state.refresh();
                open_loaded_world_from_menu(
                    &name,
                    WorldEntryMode::PlaySolution,
                    current_mode,
                    next_state,
                    builder_mode,
                    inventory,
                    carried,
                    placement,
                    save_state,
                    solution_state,
                    simulation,
                    world_menu,
                );
            }
        }
        TextPromptKind::RenamePuzzle { name: old }
        | TextPromptKind::RenameSolution { name: old } => {
            if old == name || rename_save(&old, &name) {
                if save_state.current.as_deref() == Some(old.as_str()) {
                    save_state.current = Some(name.clone());
                }
                if save_state.selected_puzzle.as_deref() == Some(old.as_str()) {
                    save_state.select_puzzle(Some(name.clone()), save_state.selected_puzzle_kind);
                }
                save_state.refresh();
            }
        }
        TextPromptKind::SaveAsNewPuzzle => {
            let world = simulation.authoring_world(&world_menu.world);
            if save_world(world, &name, SaveKind::Puzzle, inventory) {
                save_state.current = Some(name);
                save_state.current_kind = Some(SaveKind::Puzzle);
                solution_state.dirty = false;
                save_state.refresh();
            }
        }
    }
    prompt.kind = None;
    prompt.value.clear();
}

fn open_loaded_world_from_menu(
    name: &str,
    entry: WorldEntryMode,
    current_mode: GameMode,
    next_state: &mut NextState<GameMode>,
    builder_mode: &mut BuilderMode,
    inventory: &mut crate::game::ui::types::InventoryItems,
    carried: &mut crate::game::ui::types::CarriedItem,
    placement: &mut PlacementState,
    save_state: &mut SaveState,
    solution_state: &mut SolutionState,
    simulation: &mut SimulationState,
    world_menu: &mut WorldMenuParams,
) {
    open_loaded_world(
        name,
        entry,
        &mut world_menu.world,
        builder_mode,
        inventory,
        carried,
        placement,
        save_state,
        solution_state,
        simulation,
        &mut world_menu.commands,
        &mut world_menu.meshes,
        &world_menu.block_entities,
        world_menu.render_assets.as_deref(),
        &world_menu.debug,
        &mut world_menu.factory_structures,
        &mut world_menu.movement_influence,
        &mut world_menu.pusher_state,
        current_mode,
        next_state,
    );
}

pub(crate) fn save_current_world(
    world: &WorldBlocks,
    inventory: &crate::game::ui::types::InventoryItems,
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

pub(crate) fn switch_to_edit_mode_and_rebuild(
    world: &mut WorldBlocks,
    builder_mode: &mut BuilderMode,
    inventory: &mut crate::game::ui::types::InventoryItems,
    carried: &mut crate::game::ui::types::CarriedItem,
    placement: &mut PlacementState,
    playing_ui: &mut PlayingUiState,
    save_state: &mut SaveState,
    solution_state: &mut SolutionState,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    block_entities: &Query<Entity, With<BlockEntity>>,
    render_assets: Option<&WorldRenderAssets>,
    debug: &DebugState,
    factory_structures: &mut FactoryStructureState,
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
        factory_structures.clear();
        movement_influence.clear();
        pusher_state.clear();
        factory_structures.ensure_current_world(world);
        rebuild_world_for_debug_state(
            commands,
            meshes,
            world,
            render_assets,
            debug,
            factory_structures,
        );
    }
}

pub(crate) fn reset_current_solution(
    world: &mut WorldBlocks,
    simulation: &mut SimulationState,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    block_entities: &Query<Entity, With<BlockEntity>>,
    render_assets: Option<&WorldRenderAssets>,
    debug: &DebugState,
    factory_structures: &mut FactoryStructureState,
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
        simulation.start_factory_structures = None;
        factory_structures.clear();
        movement_influence.clear();
        pusher_state.clear();
        factory_structures.ensure_current_world(world);
        if let Some(render_assets) = render_assets {
            despawn_world(commands, block_entities);
            rebuild_world_for_debug_state(
                commands,
                meshes,
                world,
                render_assets,
                debug,
                factory_structures,
            );
        }
    }
}

pub(crate) fn return_to_main_menu(
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
    factory_structures: &mut FactoryStructureState,
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
        factory_structures,
        movement_influence,
        pusher_state,
    );
    *start_menu_screen = StartMenuScreen::Main;
    next_state.set(GameMode::StartMenu);
}

pub(crate) fn open_loaded_world(
    name: &str,
    entry: WorldEntryMode,
    world: &mut WorldBlocks,
    builder_mode: &mut BuilderMode,
    inventory: &mut crate::game::ui::types::InventoryItems,
    carried: &mut crate::game::ui::types::CarriedItem,
    placement: &mut PlacementState,
    save_state: &mut SaveState,
    solution_state: &mut SolutionState,
    simulation: &mut SimulationState,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    block_entities: &Query<Entity, With<BlockEntity>>,
    render_assets: Option<&WorldRenderAssets>,
    debug: &DebugState,
    factory_structures: &mut FactoryStructureState,
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
    simulation.start_factory_structures = None;
    placement.selection.clear();
    placement.edit_gesture = None;
    carried.clear();

    *builder_mode = match entry {
        WorldEntryMode::EditPuzzle => BuilderMode::Edit,
        WorldEntryMode::PlaySolution => BuilderMode::Play,
    };
    *inventory = crate::game::ui::types::InventoryItems::for_mode(*builder_mode);
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
    factory_structures.clear();
    movement_influence.clear();
    pusher_state.clear();
    factory_structures.ensure_current_world(world);

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
                    factory_structures,
                );
            }
        }
    }
}

pub(crate) fn clear_loaded_world(
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
    factory_structures: &mut FactoryStructureState,
    movement_influence: &mut MovementInfluenceCache,
    pusher_state: &mut PusherState,
) {
    simulation.running = false;
    simulation.step_requested = false;
    simulation.accumulator = 0.0;
    simulation.start_snapshot = None;
    simulation.start_factory_structures = None;
    placement.selection.clear();
    placement.edit_gesture = None;
    world.clear();
    save_state.current = None;
    save_state.current_kind = None;
    save_state.select_puzzle(None, None);
    solution_state.puzzle_snapshot = None;
    solution_state.dirty = false;
    solution_state.entry = WorldEntryMode::EditPuzzle;
    factory_structures.clear();
    movement_influence.clear();
    pusher_state.clear();
    factory_structures.ensure_current_world(world);
    if let Some(render_assets) = render_assets {
        despawn_world(commands, block_entities);
        rebuild_world_for_debug_state(
            commands,
            meshes,
            world,
            render_assets,
            debug,
            factory_structures,
        );
    }
}

fn switch_to_edit_mode(
    world: &mut WorldBlocks,
    builder_mode: &mut BuilderMode,
    inventory: &mut crate::game::ui::types::InventoryItems,
    carried: &mut crate::game::ui::types::CarriedItem,
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
    *inventory = crate::game::ui::types::InventoryItems::for_mode(*builder_mode);
    carried.clear();
    placement.selected = 0;
    save_state.current_kind = Some(SaveKind::Puzzle);
    solution_state.puzzle_snapshot = None;
    playing_ui.paused = true;
}
