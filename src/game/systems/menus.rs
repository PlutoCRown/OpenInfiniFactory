use bevy::ecs::system::SystemParam;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::ButtonState;
use bevy::picking::pointer::PointerButton;
use bevy::picking::prelude::{Click, Pointer};
use bevy::prelude::*;
use bevy::ui_widgets::{CoreSliderDragState, Slider, SliderRange, SliderValue};

use crate::game::simulation::factory_activity::FactoryStructureState;
use crate::game::simulation::markers::refresh_static_generated_markers;
use crate::game::simulation::movement::PusherState;
use crate::game::simulation::structures::MovementInfluenceCache;
use crate::game::state::{
    BuilderMode, GameMode, GameSettings, PlacementState, SimulationState, SolutionState,
    TeleportRenameState, WorldEntryMode,
};
use crate::game::systems::debug::DebugState;
use crate::game::ui::{
    ActiveSettingsSlider, BlockEditAction, BlockPanelDropdown, CarriedItem, ConfirmDialogAction,
    ConfirmDialogKind, ConfirmDialogState, InventoryItems, MenuAction, OpenBlockPanelDropdown,
    OpenSettingsDropdown, PendingKeyBind, SaveListAction, SettingsAction, SettingsSliderTrigger,
    SettingsTab, TeleportAction, UiPanelContext, UiPanelId, UiRuntime,
};
use crate::game::world::grid::{seed_demo_world, WorldBlocks};
use crate::game::world::rendering::{
    despawn_world, rebuild_world_for_debug_state, BlockEntity, WorldRenderAssets,
};
use crate::game::{GRAVITY_SCALE_MAX, GRAVITY_SCALE_MIN, UI_SCALE_MAX, UI_SCALE_MIN};
use crate::shared::config::{input_from_buttons, open_config_folder, save_config, GameConfig};
use crate::shared::i18n::{resolve_language, I18n};
use crate::shared::save::{
    delete_save, load_world, next_world_name, reset_solution_world, save_solution_with_puzzle,
    save_world, SaveKind, SaveState,
};

#[derive(SystemParam)]
pub struct WorldMenuParams<'w, 's> {
    pub commands: Commands<'w, 's>,
    pub meshes: ResMut<'w, Assets<Mesh>>,
    pub world: ResMut<'w, WorldBlocks>,
    pub render_assets: Res<'w, WorldRenderAssets>,
    pub debug: Res<'w, DebugState>,
    pub factory_structures: ResMut<'w, FactoryStructureState>,
    pub movement_influence: ResMut<'w, MovementInfluenceCache>,
    pub pusher_state: ResMut<'w, PusherState>,
    pub block_entities: Query<'w, 's, Entity, With<BlockEntity>>,
}

pub fn menu_actions(
    mut click: On<Pointer<Click>>,
    mut builder_mode: ResMut<BuilderMode>,
    mut simulation: ResMut<SimulationState>,
    mut inventory: ResMut<InventoryItems>,
    mut carried: ResMut<CarriedItem>,
    mut placement: ResMut<PlacementState>,
    mut mode: ResMut<GameMode>,
    mut save_state: ResMut<SaveState>,
    mut solution_state: ResMut<SolutionState>,
    mut ui_runtime: ResMut<UiRuntime>,
    mut world_menu: WorldMenuParams,
    mut confirm_dialog: ResMut<ConfirmDialogState>,
    mut app_exit_messages: ResMut<Messages<AppExit>>,
    actions: Query<&MenuAction>,
) {
    if !primary_click(&mut click) {
        return;
    }
    let Ok(action) = actions.get(click.entity).copied() else {
        return;
    };
    click.propagate(false);

    match (*mode, action) {
        (GameMode::MainMenu, MenuAction::EditPuzzle) => {
            save_state.refresh();
            save_state.select_puzzle(None);
            solution_state.save_list_entry = WorldEntryMode::EditPuzzle;
            *mode = GameMode::SaveListMain;
        }
        (GameMode::MainMenu, MenuAction::Play) => {
            save_state.refresh();
            save_state.select_puzzle(None);
            solution_state.save_list_entry = WorldEntryMode::PlaySolution;
            *mode = GameMode::SaveListMain;
        }
        (GameMode::MainMenu, MenuAction::OpenSettings) => {
            ui_runtime.open(
                UiPanelId::Settings,
                UiPanelContext::ReturnTo(GameMode::MainMenu),
            );
        }
        (GameMode::MainMenu, MenuAction::Quit) => {
            app_exit_messages.write(AppExit::Success);
        }
        (GameMode::Paused, MenuAction::Resume) => *mode = GameMode::Playing,
        (GameMode::Paused, MenuAction::ToggleBuilderMode) => {
            if solution_state.entry == WorldEntryMode::PlaySolution {
                return;
            }
            *builder_mode = match *builder_mode {
                BuilderMode::Edit => {
                    simulation.running = false;
                    simulation.step_requested = false;
                    simulation.accumulator = 0.0;
                    simulation.start_snapshot = None;
                    if save_state.current_kind == Some(SaveKind::Puzzle) {
                        solution_state.puzzle_snapshot = Some(world_menu.world.clone());
                    }
                    save_state.current_kind = Some(SaveKind::Solution);
                    BuilderMode::Play
                }
                BuilderMode::Play => {
                    confirm_dialog.kind = Some(ConfirmDialogKind::SaveSolutionBeforeEdit);
                    return;
                }
            };
            *inventory = InventoryItems::for_mode(*builder_mode);
            carried.clear();
            placement.selected = 0;
            *mode = GameMode::Playing;
        }
        (GameMode::Paused, MenuAction::SaveWorld) => {
            save_current_world(
                &world_menu.world,
                &inventory,
                &mut save_state,
                &mut solution_state,
            );
        }
        (GameMode::Paused, MenuAction::ResetSolution) => {
            confirm_dialog.kind = Some(ConfirmDialogKind::ResetSolution);
        }
        (GameMode::Paused, MenuAction::OpenSettings) => {
            ui_runtime.open(
                UiPanelId::Settings,
                UiPanelContext::ReturnTo(GameMode::Paused),
            );
        }
        (GameMode::Paused, MenuAction::BackToMainMenu) => {
            if solution_state.dirty {
                confirm_dialog.kind = Some(ConfirmDialogKind::ReturnToMain);
            } else {
                clear_loaded_world(
                    &mut world_menu.world,
                    &mut placement,
                    &mut save_state,
                    &mut solution_state,
                    &mut simulation,
                    &mut world_menu.commands,
                    &mut world_menu.meshes,
                    &world_menu.block_entities,
                    &world_menu.render_assets,
                    &world_menu.debug,
                    &mut world_menu.factory_structures,
                    &mut world_menu.movement_influence,
                    &mut world_menu.pusher_state,
                );
                *mode = GameMode::MainMenu;
            }
        }
        _ => {}
    }
}

pub fn save_list_actions(
    mut click: On<Pointer<Click>>,
    mut mode: ResMut<GameMode>,
    mut builder_mode: ResMut<BuilderMode>,
    mut inventory: ResMut<InventoryItems>,
    mut carried: ResMut<CarriedItem>,
    mut placement: ResMut<PlacementState>,
    mut save_state: ResMut<SaveState>,
    mut solution_state: ResMut<SolutionState>,
    mut simulation: ResMut<SimulationState>,
    mut world_menu: WorldMenuParams,
    mut confirm_dialog: ResMut<ConfirmDialogState>,
    actions: Query<&SaveListAction>,
) {
    if !primary_click(&mut click) || *mode != GameMode::SaveListMain {
        return;
    }
    let Ok(action) = actions.get(click.entity).copied() else {
        return;
    };
    click.propagate(false);

    match action {
        SaveListAction::NewPuzzle => {
            if solution_state.save_list_entry != WorldEntryMode::EditPuzzle {
                return;
            }
            let name = next_world_name(&save_state.slots);
            world_menu.world.clear();
            seed_demo_world(&mut world_menu.world);
            *inventory = InventoryItems::for_mode(BuilderMode::Edit);
            save_world(&world_menu.world, &name, SaveKind::Puzzle, &inventory);
            save_state.refresh();
            open_loaded_world(
                &name,
                WorldEntryMode::EditPuzzle,
                &mut world_menu.world,
                &mut builder_mode,
                &mut inventory,
                &mut carried,
                &mut placement,
                &mut save_state,
                &mut solution_state,
                &mut simulation,
                &mut world_menu.commands,
                &mut world_menu.meshes,
                &world_menu.block_entities,
                &world_menu.render_assets,
                &world_menu.debug,
                &mut world_menu.factory_structures,
                &mut world_menu.movement_influence,
                &mut world_menu.pusher_state,
                &mut mode,
            );
        }
        SaveListAction::NewSolution => {
            if solution_state.save_list_entry != WorldEntryMode::PlaySolution {
                return;
            }
            let Some(puzzle_name) = save_state.selected_puzzle.clone() else {
                return;
            };
            if load_world(&mut world_menu.world, &puzzle_name).is_none() {
                return;
            }
            let name = next_world_name(&save_state.slots);
            let puzzle_snapshot = world_menu.world.clone();
            *inventory = InventoryItems::for_mode(BuilderMode::Play);
            save_solution_with_puzzle(&world_menu.world, &name, &puzzle_snapshot, &inventory);
            save_state.refresh();
            open_loaded_world(
                &name,
                WorldEntryMode::PlaySolution,
                &mut world_menu.world,
                &mut builder_mode,
                &mut inventory,
                &mut carried,
                &mut placement,
                &mut save_state,
                &mut solution_state,
                &mut simulation,
                &mut world_menu.commands,
                &mut world_menu.meshes,
                &world_menu.block_entities,
                &world_menu.render_assets,
                &world_menu.debug,
                &mut world_menu.factory_structures,
                &mut world_menu.movement_influence,
                &mut world_menu.pusher_state,
                &mut mode,
            );
        }
        SaveListAction::LoadPuzzle(index) => {
            let puzzles = save_state.puzzles();
            let Some(entry) = puzzles.get(index) else {
                return;
            };
            let name = entry.name.clone();
            if solution_state.save_list_entry == WorldEntryMode::EditPuzzle {
                open_loaded_world(
                    &name,
                    WorldEntryMode::EditPuzzle,
                    &mut world_menu.world,
                    &mut builder_mode,
                    &mut inventory,
                    &mut carried,
                    &mut placement,
                    &mut save_state,
                    &mut solution_state,
                    &mut simulation,
                    &mut world_menu.commands,
                    &mut world_menu.meshes,
                    &world_menu.block_entities,
                    &world_menu.render_assets,
                    &world_menu.debug,
                    &mut world_menu.factory_structures,
                    &mut world_menu.movement_influence,
                    &mut world_menu.pusher_state,
                    &mut mode,
                );
            } else {
                save_state.select_puzzle(Some(name));
            }
        }
        SaveListAction::LoadSolution(index) => {
            if solution_state.save_list_entry != WorldEntryMode::PlaySolution {
                return;
            }
            if save_state.selected_puzzle.is_none() {
                return;
            }
            let solutions = save_state.selected_puzzle_solutions();
            let Some(entry) = solutions.get(index) else {
                return;
            };
            let name = entry.name.clone();
            open_loaded_world(
                &name,
                WorldEntryMode::PlaySolution,
                &mut world_menu.world,
                &mut builder_mode,
                &mut inventory,
                &mut carried,
                &mut placement,
                &mut save_state,
                &mut solution_state,
                &mut simulation,
                &mut world_menu.commands,
                &mut world_menu.meshes,
                &world_menu.block_entities,
                &world_menu.render_assets,
                &world_menu.debug,
                &mut world_menu.factory_structures,
                &mut world_menu.movement_influence,
                &mut world_menu.pusher_state,
                &mut mode,
            );
        }
        SaveListAction::DeletePuzzle(index) => {
            if let Some(entry) = save_state.puzzles().get(index) {
                confirm_dialog.kind = Some(ConfirmDialogKind::DeleteSave {
                    name: entry.name.clone(),
                });
            }
        }
        SaveListAction::DeleteSolution(index) => {
            if let Some(entry) = save_state.selected_puzzle_solutions().get(index) {
                confirm_dialog.kind = Some(ConfirmDialogKind::DeleteSave {
                    name: entry.name.clone(),
                });
            }
        }
        SaveListAction::Back => {
            *mode = GameMode::MainMenu;
        }
    }
}

pub fn confirm_dialog_actions(
    mut click: On<Pointer<Click>>,
    mut builder_mode: ResMut<BuilderMode>,
    mut simulation: ResMut<SimulationState>,
    mut inventory: ResMut<InventoryItems>,
    mut carried: ResMut<CarriedItem>,
    mut placement: ResMut<PlacementState>,
    mut mode: ResMut<GameMode>,
    mut save_state: ResMut<SaveState>,
    mut solution_state: ResMut<SolutionState>,
    mut world_menu: WorldMenuParams,
    mut confirm_dialog: ResMut<ConfirmDialogState>,
    actions: Query<&ConfirmDialogAction>,
) {
    if !primary_click(&mut click) {
        return;
    }
    let Some(kind) = confirm_dialog.kind.clone() else {
        return;
    };
    let Ok(action) = actions.get(click.entity).copied() else {
        return;
    };
    click.propagate(false);

    match (kind, action) {
        (_, ConfirmDialogAction::Cancel) => {}
        (ConfirmDialogKind::DeleteSave { name }, ConfirmDialogAction::Primary) => {
            delete_save(&name);
            save_state.refresh();
            if save_state.selected_puzzle.as_deref() == Some(name.as_str()) {
                save_state.select_puzzle(None);
            }
        }
        (ConfirmDialogKind::ResetSolution, ConfirmDialogAction::Primary) => {
            reset_current_solution(
                &mut world_menu.world,
                &mut simulation,
                &mut world_menu.commands,
                &mut world_menu.meshes,
                &world_menu.block_entities,
                &world_menu.render_assets,
                &world_menu.debug,
                &mut world_menu.factory_structures,
                &mut world_menu.movement_influence,
                &mut world_menu.pusher_state,
                &solution_state,
            );
            *mode = GameMode::Paused;
        }
        (ConfirmDialogKind::ReturnToMain, ConfirmDialogAction::Primary) => {
            save_current_world(
                &world_menu.world,
                &inventory,
                &mut save_state,
                &mut solution_state,
            );
            return_to_main_menu(
                &mut world_menu.world,
                &mut placement,
                &mut save_state,
                &mut solution_state,
                &mut simulation,
                &mut world_menu.commands,
                &mut world_menu.meshes,
                &world_menu.block_entities,
                &world_menu.render_assets,
                &world_menu.debug,
                &mut world_menu.factory_structures,
                &mut world_menu.movement_influence,
                &mut world_menu.pusher_state,
                &mut mode,
            );
        }
        (ConfirmDialogKind::ReturnToMain, ConfirmDialogAction::Secondary) => {
            return_to_main_menu(
                &mut world_menu.world,
                &mut placement,
                &mut save_state,
                &mut solution_state,
                &mut simulation,
                &mut world_menu.commands,
                &mut world_menu.meshes,
                &world_menu.block_entities,
                &world_menu.render_assets,
                &world_menu.debug,
                &mut world_menu.factory_structures,
                &mut world_menu.movement_influence,
                &mut world_menu.pusher_state,
                &mut mode,
            );
        }
        (ConfirmDialogKind::SaveSolutionBeforeEdit, ConfirmDialogAction::Primary) => {
            save_current_world(
                &world_menu.world,
                &inventory,
                &mut save_state,
                &mut solution_state,
            );
            switch_to_edit_mode_and_rebuild(
                &mut world_menu.world,
                &mut builder_mode,
                &mut inventory,
                &mut carried,
                &mut placement,
                &mut mode,
                &mut save_state,
                &mut solution_state,
                &mut world_menu.commands,
                &mut world_menu.meshes,
                &world_menu.block_entities,
                &world_menu.render_assets,
                &world_menu.debug,
                &mut world_menu.factory_structures,
                &mut world_menu.movement_influence,
                &mut world_menu.pusher_state,
            );
        }
        (ConfirmDialogKind::SaveSolutionBeforeEdit, ConfirmDialogAction::Secondary) => {
            switch_to_edit_mode_and_rebuild(
                &mut world_menu.world,
                &mut builder_mode,
                &mut inventory,
                &mut carried,
                &mut placement,
                &mut mode,
                &mut save_state,
                &mut solution_state,
                &mut world_menu.commands,
                &mut world_menu.meshes,
                &world_menu.block_entities,
                &world_menu.render_assets,
                &world_menu.debug,
                &mut world_menu.factory_structures,
                &mut world_menu.movement_influence,
                &mut world_menu.pusher_state,
            );
        }
        (_, ConfirmDialogAction::Secondary) => {}
    }

    confirm_dialog.kind = None;
}

fn save_current_world(
    world: &WorldBlocks,
    inventory: &InventoryItems,
    save_state: &mut SaveState,
    solution_state: &mut SolutionState,
) {
    let kind = save_state.current_kind.unwrap_or(SaveKind::Puzzle);
    let name = save_state
        .current
        .clone()
        .unwrap_or_else(|| next_world_name(&save_state.slots));
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

fn switch_to_edit_mode_and_rebuild(
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
    render_assets: &WorldRenderAssets,
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
        render_assets,
        debug,
        factory_structures,
    );
}

fn reset_current_solution(
    world: &mut WorldBlocks,
    simulation: &mut SimulationState,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    block_entities: &Query<Entity, With<BlockEntity>>,
    render_assets: &WorldRenderAssets,
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
        factory_structures.clear();
        movement_influence.clear();
        pusher_state.clear();
        factory_structures.ensure_current_world(world);
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

fn return_to_main_menu(
    world: &mut WorldBlocks,
    placement: &mut PlacementState,
    save_state: &mut SaveState,
    solution_state: &mut SolutionState,
    simulation: &mut SimulationState,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    block_entities: &Query<Entity, With<BlockEntity>>,
    render_assets: &WorldRenderAssets,
    debug: &DebugState,
    factory_structures: &mut FactoryStructureState,
    movement_influence: &mut MovementInfluenceCache,
    pusher_state: &mut PusherState,
    mode: &mut GameMode,
) {
    clear_loaded_world(
        world,
        placement,
        save_state,
        solution_state,
        simulation,
        commands,
        meshes,
        block_entities,
        render_assets,
        debug,
        factory_structures,
        movement_influence,
        pusher_state,
    );
    *mode = GameMode::MainMenu;
}

fn open_loaded_world(
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
    render_assets: &WorldRenderAssets,
    debug: &DebugState,
    factory_structures: &mut FactoryStructureState,
    movement_influence: &mut MovementInfluenceCache,
    pusher_state: &mut PusherState,
    mode: &mut GameMode,
) {
    let Some(loaded) = load_world(world, name) else {
        return;
    };

    simulation.running = false;
    simulation.step_requested = false;
    simulation.turn = 0;
    simulation.accumulator = 0.0;
    simulation.start_snapshot = None;
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
        render_assets,
        debug,
        factory_structures,
    );
    *mode = GameMode::Playing;
}

fn clear_loaded_world(
    world: &mut WorldBlocks,
    placement: &mut PlacementState,
    save_state: &mut SaveState,
    solution_state: &mut SolutionState,
    simulation: &mut SimulationState,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    block_entities: &Query<Entity, With<BlockEntity>>,
    render_assets: &WorldRenderAssets,
    debug: &DebugState,
    factory_structures: &mut FactoryStructureState,
    movement_influence: &mut MovementInfluenceCache,
    pusher_state: &mut PusherState,
) {
    simulation.running = false;
    simulation.step_requested = false;
    simulation.accumulator = 0.0;
    simulation.start_snapshot = None;
    placement.selection.clear();
    placement.edit_gesture = None;
    world.clear();
    save_state.current = None;
    save_state.current_kind = None;
    save_state.select_puzzle(None);
    solution_state.puzzle_snapshot = None;
    solution_state.dirty = false;
    solution_state.entry = WorldEntryMode::EditPuzzle;
    factory_structures.clear();
    movement_influence.clear();
    pusher_state.clear();
    factory_structures.ensure_current_world(world);
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

fn switch_to_edit_mode(
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
    *mode = GameMode::Paused;
}

pub fn settings_menu_actions(
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut settings: ResMut<GameSettings>,
    mut ui_scale: ResMut<UiScale>,
    mut config: ResMut<GameConfig>,
    mut open_dropdown: ResMut<OpenSettingsDropdown>,
    mut pending_key_bind: ResMut<PendingKeyBind>,
    mut active_slider: ResMut<ActiveSettingsSlider>,
    ui_runtime: Res<UiRuntime>,
    slider_values: Query<(&SettingsAction, &SliderValue, &SliderRange), With<Slider>>,
    slider_changes: Query<
        (
            &SettingsAction,
            Ref<SliderValue>,
            &SliderRange,
            &CoreSliderDragState,
        ),
        (
            With<Slider>,
            Or<(Changed<SliderValue>, Changed<CoreSliderDragState>)>,
        ),
    >,
) {
    if !ui_runtime.is_settings_open() {
        pending_key_bind.0 = None;
        open_dropdown.0 = None;
        active_slider.0 = None;
        return;
    }

    if let Some(action) = pending_key_bind.0 {
        if let Some(input) = input_from_buttons(&keys, &mouse_buttons) {
            config.set_input(action, input);
            save_config(&config);
            pending_key_bind.0 = None;
        }
    }

    update_settings_sliders_from_input(
        &slider_changes,
        &mut active_slider,
        &mut settings,
        &mut ui_scale,
        &mut config,
    );

    if mouse_buttons.just_released(MouseButton::Left) {
        commit_active_settings_slider(
            &slider_values,
            &mut active_slider,
            &mut settings,
            &mut ui_scale,
            &mut config,
        );
    }
}

pub fn settings_action_clicked(
    mut click: On<Pointer<Click>>,
    mut mode: ResMut<GameMode>,
    mut settings: ResMut<GameSettings>,
    mut ui_scale: ResMut<UiScale>,
    mut config: ResMut<GameConfig>,
    mut i18n: ResMut<I18n>,
    mut settings_tab: ResMut<SettingsTab>,
    mut open_dropdown: ResMut<OpenSettingsDropdown>,
    mut pending_key_bind: ResMut<PendingKeyBind>,
    mut active_slider: ResMut<ActiveSettingsSlider>,
    mut ui_runtime: ResMut<UiRuntime>,
    actions: Query<&SettingsAction>,
) {
    if !primary_click(&mut click) || !ui_runtime.is_settings_open() {
        return;
    }
    let Ok(action) = actions.get(click.entity).copied() else {
        return;
    };
    click.propagate(false);

    match action {
        SettingsAction::TabGameplay => {
            *settings_tab = SettingsTab::Gameplay;
            open_dropdown.0 = None;
        }
        SettingsAction::TabKeyBindings => {
            *settings_tab = SettingsTab::KeyBindings;
            open_dropdown.0 = None;
        }
        SettingsAction::Field(field) => {
            active_slider.0 = Some(field);
        }
        SettingsAction::SetPlaceSelectionMode(selection_mode) => {
            config.place_selection_mode = selection_mode;
            open_dropdown.0 = None;
            save_config(&config);
        }
        SettingsAction::SetDeleteSelectionMode(selection_mode) => {
            config.delete_selection_mode = selection_mode;
            open_dropdown.0 = None;
            save_config(&config);
        }
        SettingsAction::SetLanguage(language) => {
            i18n.set_language(language);
            config.language = Some(language);
            open_dropdown.0 = None;
            save_config(&config);
        }
        SettingsAction::ToggleDropdown(dropdown) => {
            open_dropdown.0 = if open_dropdown.0 == Some(dropdown) {
                None
            } else {
                Some(dropdown)
            };
        }
        SettingsAction::Bind(action) => {
            pending_key_bind.0 = Some(action);
        }
        SettingsAction::ResetDefaults => {
            *config = GameConfig::default();
            settings.fov_degrees = config.fov_degrees;
            settings.ui_scale = config.ui_scale.clamp(UI_SCALE_MIN, UI_SCALE_MAX);
            settings.gravity_scale = config
                .gravity_scale
                .clamp(GRAVITY_SCALE_MIN, GRAVITY_SCALE_MAX);
            ui_scale.0 = settings.ui_scale;
            i18n.set_language(resolve_language(config.language));
            open_dropdown.0 = None;
            pending_key_bind.0 = None;
            save_config(&config);
        }
        SettingsAction::OpenFolder => {
            open_config_folder();
        }
        SettingsAction::Back => {
            open_dropdown.0 = None;
            pending_key_bind.0 = None;
            let return_mode = settings_return_mode(&ui_runtime, *mode);
            ui_runtime.close_active();
            *mode = return_mode;
        }
    }
}

fn settings_return_mode(ui_runtime: &UiRuntime, fallback: GameMode) -> GameMode {
    ui_runtime
        .active()
        .and_then(|session| match session.context {
            UiPanelContext::ReturnTo(mode) => Some(mode),
            _ => None,
        })
        .unwrap_or(fallback)
}

fn update_settings_sliders_from_input(
    slider_changes: &Query<
        (
            &SettingsAction,
            Ref<SliderValue>,
            &SliderRange,
            &CoreSliderDragState,
        ),
        (
            With<Slider>,
            Or<(Changed<SliderValue>, Changed<CoreSliderDragState>)>,
        ),
    >,
    active_slider: &mut ActiveSettingsSlider,
    settings: &mut GameSettings,
    ui_scale: &mut UiScale,
    config: &mut GameConfig,
) {
    for (action, value, range, drag_state) in slider_changes {
        let SettingsAction::Field(field) = *action else {
            continue;
        };
        let percent = range.thumb_position(value.0).clamp(0.0, 1.0);

        if drag_state.dragging {
            active_slider.0 = Some(field);
            if field
                .slider()
                .is_some_and(|slider| slider.trigger == SettingsSliderTrigger::Live)
            {
                field.apply_percent(percent, settings, ui_scale, config);
            }
            continue;
        }

        if active_slider.0 == Some(field) || value.is_changed() {
            field.apply_percent(percent, settings, ui_scale, config);
            save_config(config);
            if active_slider.0 == Some(field) {
                active_slider.0 = None;
            }
        }
    }
}

fn commit_active_settings_slider(
    slider_values: &Query<(&SettingsAction, &SliderValue, &SliderRange), With<Slider>>,
    active_slider: &mut ActiveSettingsSlider,
    settings: &mut GameSettings,
    ui_scale: &mut UiScale,
    config: &mut GameConfig,
) {
    let Some(field) = active_slider.0.take() else {
        return;
    };

    for (action, value, range) in slider_values {
        if *action != SettingsAction::Field(field) {
            continue;
        }
        let percent = range.thumb_position(value.0).clamp(0.0, 1.0);
        field.apply_percent(percent, settings, ui_scale, config);
        save_config(config);
        return;
    }
}

fn primary_click(click: &mut On<Pointer<Click>>) -> bool {
    click.event.button == PointerButton::Primary
}

pub fn block_edit_actions(
    mut click: On<Pointer<Click>>,
    mut ui_runtime: ResMut<UiRuntime>,
    mut open_dropdown: ResMut<OpenBlockPanelDropdown>,
    mut solution_state: ResMut<SolutionState>,
    mut world_menu: WorldMenuParams,
    actions: Query<&BlockEditAction>,
) {
    if !primary_click(&mut click) {
        return;
    }
    let Some(pos) = ui_runtime.active_block_pos() else {
        return;
    };
    let Some(block) = world_menu.world.system_blocks.get(&pos).copied() else {
        ui_runtime.close_current();
        return;
    };
    let Ok(action) = actions.get(click.entity).copied() else {
        return;
    };
    click.propagate(false);
    block.kind.handle_edit_action(
        pos,
        action,
        &mut world_menu.world,
        &mut solution_state,
        &mut open_dropdown,
    );
    if !block_edit_action_mutates_world(action) {
        return;
    }
    despawn_world(&mut world_menu.commands, &world_menu.block_entities);
    world_menu.factory_structures.clear();
    world_menu.movement_influence.clear();
    world_menu.pusher_state.clear();
    world_menu
        .factory_structures
        .ensure_current_world(&world_menu.world);
    rebuild_world_for_debug_state(
        &mut world_menu.commands,
        &mut world_menu.meshes,
        &world_menu.world,
        &world_menu.render_assets,
        &world_menu.debug,
        &mut world_menu.factory_structures,
    );
}

fn block_edit_action_mutates_world(action: BlockEditAction) -> bool {
    !matches!(
        action,
        BlockEditAction::ToggleMaterialDropdown
            | BlockEditAction::ToggleColorDropdown
            | BlockEditAction::ToggleInputDropdown
            | BlockEditAction::ToggleOutputDropdown
    )
}

pub fn teleport_menu_actions(
    mut click: On<Pointer<Click>>,
    mut ui_runtime: ResMut<UiRuntime>,
    mut open_dropdown: ResMut<OpenBlockPanelDropdown>,
    mut rename_state: ResMut<TeleportRenameState>,
    mut world: ResMut<WorldBlocks>,
    mut solution_state: ResMut<SolutionState>,
    actions: Query<&TeleportAction>,
) {
    if !primary_click(&mut click) || ui_runtime.active_panel() != Some(UiPanelId::Teleport) {
        return;
    }

    let Some(pos) = ui_runtime.active_block_pos() else {
        ui_runtime.close_current();
        return;
    };

    let Ok(action) = actions.get(click.entity).copied() else {
        return;
    };
    click.propagate(false);

    match action {
        TeleportAction::TogglePairDropdown => {
            toggle_block_dropdown(&mut open_dropdown, BlockPanelDropdown::TeleportPair);
        }
        TeleportAction::SetPair(pair) => {
            let mut settings = world.teleport_settings(pos);
            settings.pair = pair;
            world.set_teleport_settings(pos, settings);
            solution_state.dirty = true;
            open_dropdown.0 = None;
        }
        TeleportAction::Rename => {
            let settings = world.teleport_settings(pos);
            rename_state.editing = Some(pos);
            rename_state.buffer = settings.name;
        }
    }
}

pub fn teleport_rename_input(
    ui_runtime: Res<UiRuntime>,
    mut rename_state: ResMut<TeleportRenameState>,
    mut world: ResMut<WorldBlocks>,
    mut solution_state: ResMut<SolutionState>,
    mut keyboard_input: MessageReader<KeyboardInput>,
) {
    if ui_runtime.active_panel() != Some(UiPanelId::Teleport) || rename_state.editing.is_none() {
        keyboard_input.clear();
        return;
    }

    let pos = rename_state.editing.expect("checked above");
    let mut confirm = false;
    let mut cancel = false;

    for event in keyboard_input.read() {
        if event.state != ButtonState::Pressed {
            continue;
        }
        match &event.logical_key {
            Key::Enter => confirm = true,
            Key::Escape => cancel = true,
            Key::Backspace => {
                rename_state.buffer.pop();
            }
            Key::Space => push_rename_char(&mut rename_state.buffer, ' '),
            Key::Character(text) => {
                for ch in text.chars() {
                    push_rename_char(&mut rename_state.buffer, ch);
                }
            }
            _ => {}
        }
    }

    if confirm {
        let mut settings = world.teleport_settings(pos);
        let trimmed = rename_state.buffer.trim();
        if !trimmed.is_empty() {
            settings.name = trimmed.chars().take(24).collect();
            world.set_teleport_settings(pos, settings);
            solution_state.dirty = true;
        }
        rename_state.editing = None;
    } else if cancel {
        rename_state.editing = None;
    }
}

fn push_rename_char(buffer: &mut String, ch: char) {
    if buffer.chars().count() >= 24 || ch.is_control() {
        return;
    }
    buffer.push(ch);
}

fn toggle_block_dropdown(open_dropdown: &mut OpenBlockPanelDropdown, dropdown: BlockPanelDropdown) {
    open_dropdown.0 = if open_dropdown.0 == Some(dropdown) {
        None
    } else {
        Some(dropdown)
    };
}
