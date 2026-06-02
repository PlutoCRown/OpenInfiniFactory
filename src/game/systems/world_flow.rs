use bevy::ecs::system::SystemParam;
use bevy::input::keyboard::KeyboardInput;
use bevy::picking::pointer::PointerButton;
use bevy::picking::prelude::{Click, Pointer};
use bevy::prelude::*;

use crate::game::player::controller::spawn_player_entity;
use crate::game::simulation::factory_activity::FactoryStructureState;
use crate::game::simulation::markers::refresh_static_generated_markers;
use crate::game::simulation::movement::PusherState;
use crate::game::simulation::structures::MovementInfluenceCache;
use crate::game::state::{
    BuilderMode, GameMode, PlacementState, SimulationState, SolutionState, WorldEntryMode,
};
use crate::game::systems::debug::DebugState;
use crate::game::systems::virtual_controls::spawn_virtual_controls_ui;
use crate::game::ui::{
    spawn_carried_label, spawn_hotbar, spawn_inventory_tooltip, BlockPanelDropdown, CarriedItem,
    ConfirmDialogButtonSpec, ConfirmDialogEffect, ConfirmDialogMessage, ConfirmDialogSpec,
    InventoryItems, OpenBlockPanelDropdown, UiRoot,
};
use crate::game::world::grid::WorldBlocks;
use crate::game::world::rendering::{
    despawn_world, rebuild_world_for_debug_state, spawn_block_icons, spawn_scene_entities,
    BlockEntity, GameWorldRuntime, GameplayRuntimeEntity, WorldRenderManager,
};
use crate::shared::save::{
    load_world, next_named_save, reset_solution_world, save_solution_with_puzzle, save_world,
    SaveKind, SaveState,
};

#[derive(SystemParam)]
pub struct WorldMenuParams<'w, 's> {
    pub commands: Commands<'w, 's>,
    pub meshes: ResMut<'w, Assets<Mesh>>,
    pub materials: ResMut<'w, Assets<StandardMaterial>>,
    pub images: ResMut<'w, Assets<Image>>,
    pub world: ResMut<'w, WorldBlocks>,
    pub render_manager: Res<'w, WorldRenderManager>,
    pub runtime: ResMut<'w, GameWorldRuntime>,
    pub debug: Res<'w, DebugState>,
    pub factory_structures: ResMut<'w, FactoryStructureState>,
    pub movement_influence: ResMut<'w, MovementInfluenceCache>,
    pub pusher_state: ResMut<'w, PusherState>,
    pub block_entities: Query<'w, 's, Entity, With<BlockEntity>>,
    pub runtime_entities: Query<'w, 's, Entity, With<GameplayRuntimeEntity>>,
    pub ui_roots: Query<'w, 's, Entity, With<UiRoot>>,
}

pub(crate) fn ensure_gameplay_runtime(world_menu: &mut WorldMenuParams) {
    if !world_menu.runtime.scene_ready {
        spawn_scene_entities(
            &mut world_menu.commands,
            &mut world_menu.meshes,
            &mut world_menu.materials,
            &world_menu.render_manager,
        );
        world_menu.runtime.scene_ready = true;
    }
    if !world_menu.runtime.block_icons_ready {
        spawn_block_icons(
            &mut world_menu.commands,
            &mut world_menu.images,
            &mut world_menu.meshes,
            &world_menu.render_manager,
        );
        world_menu.runtime.block_icons_ready = true;
    }
    if !world_menu.runtime.player_ready {
        spawn_player_entity(&mut world_menu.commands);
        world_menu.runtime.player_ready = true;
    }
    if !world_menu.runtime.virtual_controls_ready {
        spawn_virtual_controls_ui(&mut world_menu.commands);
        world_menu.runtime.virtual_controls_ready = true;
    }
    if !world_menu.runtime.inventory_ui_ready {
        if let Ok(root) = world_menu.ui_roots.single() {
            world_menu.commands.entity(root).with_children(|root| {
                spawn_hotbar(root);
                spawn_carried_label(root);
                spawn_inventory_tooltip(root);
            });
            world_menu.runtime.inventory_ui_ready = true;
        }
    }
}

pub(crate) fn teardown_gameplay_runtime(world_menu: &mut WorldMenuParams) {
    for entity in &world_menu.runtime_entities {
        world_menu.commands.entity(entity).despawn();
    }
    world_menu
        .commands
        .remove_resource::<crate::game::world::rendering::BlockIconAssets>();
    world_menu
        .commands
        .remove_resource::<crate::game::world::rendering::BlockIconRenderState>();
    world_menu.commands.remove_resource::<InventoryItems>();
    *world_menu.runtime = GameWorldRuntime::default();
}

pub(crate) fn clear_loaded_world_from_menu(
    placement: &mut PlacementState,
    save_state: &mut SaveState,
    solution_state: &mut SolutionState,
    simulation: &mut SimulationState,
    world_menu: &mut WorldMenuParams,
) {
    clear_loaded_world(
        &mut world_menu.world,
        placement,
        save_state,
        solution_state,
        simulation,
        &mut world_menu.factory_structures,
        &mut world_menu.movement_influence,
        &mut world_menu.pusher_state,
    );
    teardown_gameplay_runtime(world_menu);
}

pub(crate) fn return_to_main_menu_from_menu(
    placement: &mut PlacementState,
    save_state: &mut SaveState,
    solution_state: &mut SolutionState,
    simulation: &mut SimulationState,
    mode: &mut GameMode,
    world_menu: &mut WorldMenuParams,
) {
    clear_loaded_world_from_menu(
        placement,
        save_state,
        solution_state,
        simulation,
        world_menu,
    );
    *mode = GameMode::MainMenu;
}

pub(crate) fn puzzle_has_solutions(save_state: &mut SaveState, puzzle: &str) -> bool {
    let previous_puzzle = save_state.selected_puzzle.clone();
    let previous_source = save_state.selected_puzzle_kind;
    save_state.select_puzzle(
        Some(puzzle.to_string()),
        Some(crate::shared::save::SavePuzzleSource::PuzzleFile),
    );
    let has_solutions = !save_state.selected_puzzle_solutions().is_empty();
    save_state.select_puzzle(previous_puzzle, previous_source);
    has_solutions
}

pub(crate) fn delete_save_dialog(name: String) -> ConfirmDialogSpec {
    ConfirmDialogSpec::new(
        ConfirmDialogMessage::Named {
            key: "save.confirm_delete",
            name: name.clone(),
        },
        ConfirmDialogButtonSpec::new("button.delete", ConfirmDialogEffect::DeleteSave { name }),
        None,
    )
}

pub(crate) fn open_loaded_world_from_menu(
    name: &str,
    entry: WorldEntryMode,
    mode: &mut GameMode,
    builder_mode: &mut BuilderMode,
    carried: &mut CarriedItem,
    placement: &mut PlacementState,
    save_state: &mut SaveState,
    solution_state: &mut SolutionState,
    simulation: &mut SimulationState,
    world_menu: &mut WorldMenuParams,
) -> bool {
    ensure_gameplay_runtime(world_menu);
    open_loaded_world(
        name,
        entry,
        &mut world_menu.world,
        builder_mode,
        carried,
        placement,
        save_state,
        solution_state,
        simulation,
        &mut world_menu.commands,
        &mut world_menu.meshes,
        &world_menu.block_entities,
        &world_menu.render_manager,
        &world_menu.debug,
        &mut world_menu.factory_structures,
        &mut world_menu.movement_influence,
        &mut world_menu.pusher_state,
        mode,
    )
}

pub(crate) fn save_current_world(
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
            if let (Some(puzzle_name), Some(puzzle_snapshot)) =
                (&solution_state.puzzle_name, &solution_state.puzzle_snapshot)
            {
                save_solution_with_puzzle(world, &name, puzzle_name, puzzle_snapshot, inventory)
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
    inventory: &mut InventoryItems,
    carried: &mut CarriedItem,
    placement: &mut PlacementState,
    mode: &mut GameMode,
    save_state: &mut SaveState,
    solution_state: &mut SolutionState,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    block_entities: &Query<Entity, With<BlockEntity>>,
    render_manager: &WorldRenderManager,
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
        mode,
        save_state,
        solution_state,
    );
    despawn_world(commands, block_entities);
    factory_structures.clear();
    movement_influence.clear();
    pusher_state.clear();
    factory_structures.ensure_current_world(world);
    rebuild_world_for_debug_state(
        commands,
        meshes,
        world,
        render_manager,
        debug,
        factory_structures,
    );
}

pub(crate) fn reset_current_solution(
    world: &mut WorldBlocks,
    simulation: &mut SimulationState,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    block_entities: &Query<Entity, With<BlockEntity>>,
    render_manager: &WorldRenderManager,
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
        despawn_world(commands, block_entities);
        rebuild_world_for_debug_state(
            commands,
            meshes,
            world,
            render_manager,
            debug,
            factory_structures,
        );
    }
}

pub(crate) fn open_loaded_world(
    name: &str,
    entry: WorldEntryMode,
    world: &mut WorldBlocks,
    builder_mode: &mut BuilderMode,
    carried: &mut CarriedItem,
    placement: &mut PlacementState,
    save_state: &mut SaveState,
    solution_state: &mut SolutionState,
    simulation: &mut SimulationState,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    block_entities: &Query<Entity, With<BlockEntity>>,
    render_manager: &WorldRenderManager,
    debug: &DebugState,
    factory_structures: &mut FactoryStructureState,
    movement_influence: &mut MovementInfluenceCache,
    pusher_state: &mut PusherState,
    mode: &mut GameMode,
) -> bool {
    let Some(loaded) = load_world(world, name) else {
        return false;
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
    let mut inventory = InventoryItems::for_mode(*builder_mode);
    if let Some(hotbar) = loaded.hotbar {
        inventory.set_hotbar(hotbar);
    }
    commands.insert_resource(inventory);
    placement.selected = 0;

    save_state.current = Some(name.to_string());
    save_state.current_kind = Some(match entry {
        WorldEntryMode::EditPuzzle => SaveKind::Puzzle,
        WorldEntryMode::PlaySolution => SaveKind::Solution,
    });
    save_state.select_puzzle(None, None);

    solution_state.entry = entry;
    solution_state.dirty = false;
    solution_state.puzzle_name = match entry {
        WorldEntryMode::EditPuzzle => None,
        WorldEntryMode::PlaySolution => loaded
            .puzzle_name
            .or_else(|| save_state.selected_puzzle.clone()),
    };
    solution_state.puzzle_snapshot = match entry {
        WorldEntryMode::EditPuzzle => None,
        WorldEntryMode::PlaySolution => loaded.puzzle_snapshot.or_else(|| Some(loaded.world)),
    };

    refresh_static_generated_markers(world);
    factory_structures.clear();
    movement_influence.clear();
    pusher_state.clear();
    factory_structures.ensure_current_world(world);
    despawn_world(commands, block_entities);
    rebuild_world_for_debug_state(
        commands,
        meshes,
        world,
        render_manager,
        debug,
        factory_structures,
    );
    *mode = GameMode::Playing;
    true
}

pub(crate) fn clear_loaded_world(
    world: &mut WorldBlocks,
    placement: &mut PlacementState,
    save_state: &mut SaveState,
    solution_state: &mut SolutionState,
    simulation: &mut SimulationState,
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
    solution_state.puzzle_name = None;
    solution_state.dirty = false;
    solution_state.entry = WorldEntryMode::EditPuzzle;
    factory_structures.clear();
    movement_influence.clear();
    pusher_state.clear();
}

pub(crate) fn switch_to_edit_mode(
    world: &mut WorldBlocks,
    builder_mode: &mut BuilderMode,
    inventory: &mut InventoryItems,
    carried: &mut CarriedItem,
    placement: &mut PlacementState,
    mode: &mut GameMode,
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
    solution_state.puzzle_name = None;
    *mode = GameMode::Paused;
}
pub(crate) fn primary_click(click: &mut On<Pointer<Click>>) -> bool {
    click.event.button == PointerButton::Primary
}

fn push_rename_char(buffer: &mut String, ch: char) {
    if buffer.chars().count() >= 24 || ch.is_control() {
        return;
    }
    buffer.push(ch);
}

pub(crate) fn push_text_input(buffer: &mut String, event: &KeyboardInput) {
    let Some(text) = event.text.as_deref() else {
        return;
    };
    for ch in text.chars() {
        push_rename_char(buffer, ch);
    }
}

pub(crate) fn toggle_block_dropdown(
    open_dropdown: &mut OpenBlockPanelDropdown,
    dropdown: BlockPanelDropdown,
) {
    open_dropdown.0 = if open_dropdown.0 == Some(dropdown) {
        None
    } else {
        Some(dropdown)
    };
}
