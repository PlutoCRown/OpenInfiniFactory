use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::ButtonState;
use bevy::picking::prelude::{Click, Pointer};
use bevy::prelude::*;

use crate::game::state::{
    BuilderMode, GameMode, PlacementState, SimulationState, SolutionState, WorldEntryMode,
};
use crate::game::systems::world_flow::{
    delete_save_dialog, open_loaded_world, open_loaded_world_from_menu, primary_click,
    push_text_input, puzzle_has_solutions, WorldMenuParams,
};
use crate::game::ui::{
    CarriedItem, CloseUiModal, ConfirmDialogButtonSpec, ConfirmDialogEffect, ConfirmDialogMessage,
    ConfirmDialogSpec, InventoryChanged, InventoryItems, OpenConfirmDialog, OpenTextPrompt,
    SaveListAction, SaveListChanged, TextPromptAction, TextPromptKind, UiModalKind, UiModalOpened,
    UiRuntime,
};
use crate::game::world::grid::seed_demo_world;
use crate::shared::save::{
    load_world, next_named_save, rename_save, save_solution_with_puzzle, save_world, SaveKind,
    SaveState,
};

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
    mut open_confirm: MessageWriter<OpenConfirmDialog>,
    mut open_prompt: MessageWriter<OpenTextPrompt>,
    mut inventory_changed: MessageWriter<InventoryChanged>,
    mut save_list_changed: MessageWriter<SaveListChanged>,
    actions: Query<&SaveListAction>,
) {
    if !primary_click(&mut click) || *mode != GameMode::SaveListMain {
        return;
    }
    let Ok(action) = actions.get(click.entity).cloned() else {
        return;
    };
    click.propagate(false);

    match action {
        SaveListAction::NewPuzzle => {
            if solution_state.save_list_entry != WorldEntryMode::EditPuzzle {
                return;
            }
            open_prompt.write(OpenTextPrompt::new(TextPromptKind::NewPuzzle, "puzzle"));
        }
        SaveListAction::NewSolution => {
            if solution_state.save_list_entry != WorldEntryMode::PlaySolution {
                return;
            }
            let Some(puzzle_name) = save_state.selected_puzzle.clone() else {
                return;
            };
            open_prompt.write(OpenTextPrompt::new(
                TextPromptKind::NewSolution {
                    puzzle: puzzle_name,
                },
                "solution",
            ));
        }
        SaveListAction::LoadPuzzle(name) => {
            if solution_state.save_list_entry == WorldEntryMode::EditPuzzle {
                if !save_state.puzzles().iter().any(|entry| entry.name == *name) {
                    return;
                }
                if puzzle_has_solutions(&mut save_state, &name) {
                    open_confirm.write(OpenConfirmDialog(ConfirmDialogSpec::new(
                        ConfirmDialogMessage::Named {
                            key: "confirm.edit_puzzle_with_solutions",
                            name: name.clone(),
                        },
                        ConfirmDialogButtonSpec::new(
                            "button.edit_puzzle",
                            ConfirmDialogEffect::OpenPuzzleForEdit { name: name.clone() },
                        ),
                        None,
                    )));
                    return;
                }
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
                    &world_menu.render_manager,
                    &world_menu.debug,
                    &mut world_menu.factory_structures,
                    &mut world_menu.movement_influence,
                    &mut world_menu.pusher_state,
                    &mut mode,
                );
                inventory_changed.write(InventoryChanged);
                save_list_changed.write(SaveListChanged);
            } else {
                let Some(choice) = save_state
                    .puzzle_choices()
                    .into_iter()
                    .find(|choice| choice.name == *name)
                else {
                    return;
                };
                save_state.select_puzzle(Some(choice.name), Some(choice.source));
                save_list_changed.write(SaveListChanged);
            }
        }
        SaveListAction::LoadSolution(name) => {
            if solution_state.save_list_entry != WorldEntryMode::PlaySolution {
                return;
            }
            if save_state.selected_puzzle.is_none() {
                return;
            }
            if !save_state
                .selected_puzzle_solutions()
                .iter()
                .any(|entry| entry.name == *name)
            {
                return;
            }
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
                &world_menu.render_manager,
                &world_menu.debug,
                &mut world_menu.factory_structures,
                &mut world_menu.movement_influence,
                &mut world_menu.pusher_state,
                &mut mode,
            );
            inventory_changed.write(InventoryChanged);
            save_list_changed.write(SaveListChanged);
        }
        SaveListAction::RenamePuzzle(name) => {
            if solution_state.save_list_entry != WorldEntryMode::EditPuzzle {
                return;
            }
            if save_state.puzzles().iter().any(|entry| entry.name == *name) {
                open_prompt.write(OpenTextPrompt::new(
                    TextPromptKind::RenamePuzzle { name: name.clone() },
                    name.clone(),
                ));
            }
        }
        SaveListAction::RenameSolution(name) => {
            if solution_state.save_list_entry != WorldEntryMode::PlaySolution {
                return;
            }
            if save_state
                .selected_puzzle_solutions()
                .iter()
                .any(|entry| entry.name == *name)
            {
                open_prompt.write(OpenTextPrompt::new(
                    TextPromptKind::RenameSolution { name: name.clone() },
                    name.clone(),
                ));
            }
        }
        SaveListAction::DeletePuzzle(name) => {
            if solution_state.save_list_entry != WorldEntryMode::EditPuzzle {
                return;
            }
            if save_state.puzzles().iter().any(|entry| entry.name == *name) {
                open_confirm.write(OpenConfirmDialog(delete_save_dialog(name.clone())));
            }
        }
        SaveListAction::DeleteSolution(name) => {
            if save_state
                .selected_puzzle_solutions()
                .iter()
                .any(|entry| entry.name == *name)
            {
                open_confirm.write(OpenConfirmDialog(delete_save_dialog(name.clone())));
            }
        }
        SaveListAction::Back => {
            *mode = GameMode::MainMenu;
        }
    }
}

pub fn text_prompt_actions(
    mut click: On<Pointer<Click>>,
    mut ui_runtime: ResMut<UiRuntime>,
    mut close_modal: MessageWriter<CloseUiModal>,
    mut mode: ResMut<GameMode>,
    mut builder_mode: ResMut<BuilderMode>,
    mut inventory: ResMut<InventoryItems>,
    mut carried: ResMut<CarriedItem>,
    mut placement: ResMut<PlacementState>,
    mut save_state: ResMut<SaveState>,
    mut solution_state: ResMut<SolutionState>,
    mut simulation: ResMut<SimulationState>,
    mut world_menu: WorldMenuParams,
    actions: Query<&TextPromptAction>,
    mut inventory_changed: MessageWriter<InventoryChanged>,
    mut save_list_changed: MessageWriter<SaveListChanged>,
) {
    if !primary_click(&mut click) {
        return;
    }
    let Ok(action) = actions.get(click.entity).copied() else {
        return;
    };
    click.propagate(false);
    match action {
        TextPromptAction::Confirm => confirm_active_text_prompt(
            &mut ui_runtime,
            &mut close_modal,
            &mut mode,
            &mut builder_mode,
            &mut inventory,
            &mut carried,
            &mut placement,
            &mut save_state,
            &mut solution_state,
            &mut simulation,
            &mut world_menu,
            &mut inventory_changed,
            &mut save_list_changed,
        ),
        TextPromptAction::Cancel => {
            close_modal.write(CloseUiModal);
        }
    }
}

pub fn text_prompt_input(
    mut ui_runtime: ResMut<UiRuntime>,
    mut mode: ResMut<GameMode>,
    mut builder_mode: ResMut<BuilderMode>,
    mut inventory: ResMut<InventoryItems>,
    mut carried: ResMut<CarriedItem>,
    mut placement: ResMut<PlacementState>,
    mut save_state: ResMut<SaveState>,
    mut solution_state: ResMut<SolutionState>,
    mut simulation: ResMut<SimulationState>,
    mut world_menu: WorldMenuParams,
    mut close_modal: MessageWriter<CloseUiModal>,
    mut keyboard_input: MessageReader<KeyboardInput>,
    mut modal_opened: MessageWriter<UiModalOpened>,
    mut inventory_changed: MessageWriter<InventoryChanged>,
    mut save_list_changed: MessageWriter<SaveListChanged>,
) {
    if ui_runtime.text_prompt().is_none() {
        return;
    }
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
                if let Some(prompt) = ui_runtime.text_prompt_mut() {
                    prompt.value.pop();
                    modal_opened.write(UiModalOpened {
                        kind: UiModalKind::TextPrompt,
                    });
                }
            }
            _ => {
                if let Some(prompt) = ui_runtime.text_prompt_mut() {
                    push_text_input(&mut prompt.value, event);
                    modal_opened.write(UiModalOpened {
                        kind: UiModalKind::TextPrompt,
                    });
                }
            }
        }
    }
    if confirm {
        confirm_active_text_prompt(
            &mut ui_runtime,
            &mut close_modal,
            &mut mode,
            &mut builder_mode,
            &mut inventory,
            &mut carried,
            &mut placement,
            &mut save_state,
            &mut solution_state,
            &mut simulation,
            &mut world_menu,
            &mut inventory_changed,
            &mut save_list_changed,
        );
    } else if cancel {
        close_modal.write(CloseUiModal);
    }
}

fn confirm_active_text_prompt(
    ui_runtime: &mut UiRuntime,
    close_modal: &mut MessageWriter<CloseUiModal>,
    mode: &mut GameMode,
    builder_mode: &mut BuilderMode,
    inventory: &mut InventoryItems,
    carried: &mut CarriedItem,
    placement: &mut PlacementState,
    save_state: &mut SaveState,
    solution_state: &mut SolutionState,
    simulation: &mut SimulationState,
    world_menu: &mut WorldMenuParams,
    inventory_changed: &mut MessageWriter<InventoryChanged>,
    save_list_changed: &mut MessageWriter<SaveListChanged>,
) {
    let Some(prompt) = ui_runtime.text_prompt().cloned() else {
        return;
    };
    close_modal.write(CloseUiModal);
    let kind = prompt.kind;
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
            *inventory = InventoryItems::for_mode(BuilderMode::Edit);
            if save_world(&world_menu.world, &name, SaveKind::Puzzle, inventory) {
                save_state.refresh();
                inventory_changed.write(InventoryChanged);
                save_list_changed.write(SaveListChanged);
                open_loaded_world_from_menu(
                    &name,
                    WorldEntryMode::EditPuzzle,
                    mode,
                    builder_mode,
                    inventory,
                    carried,
                    placement,
                    save_state,
                    solution_state,
                    simulation,
                    world_menu,
                );
                inventory_changed.write(InventoryChanged);
                save_list_changed.write(SaveListChanged);
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
            *inventory = InventoryItems::for_mode(BuilderMode::Play);
            if save_solution_with_puzzle(
                &world_menu.world,
                &name,
                &puzzle,
                &puzzle_snapshot,
                inventory,
            ) {
                save_state.refresh();
                inventory_changed.write(InventoryChanged);
                save_list_changed.write(SaveListChanged);
                open_loaded_world_from_menu(
                    &name,
                    WorldEntryMode::PlaySolution,
                    mode,
                    builder_mode,
                    inventory,
                    carried,
                    placement,
                    save_state,
                    solution_state,
                    simulation,
                    world_menu,
                );
                inventory_changed.write(InventoryChanged);
                save_list_changed.write(SaveListChanged);
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
                save_list_changed.write(SaveListChanged);
            }
        }
        TextPromptKind::SaveAsNewPuzzle => {
            let world = simulation.authoring_world(&world_menu.world);
            if save_world(world, &name, SaveKind::Puzzle, inventory) {
                save_state.current = Some(name);
                save_state.current_kind = Some(SaveKind::Puzzle);
                solution_state.dirty = false;
                save_state.refresh();
                save_list_changed.write(SaveListChanged);
            }
        }
    }
}
