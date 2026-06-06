use bevy::input::keyboard::KeyboardInput;
use bevy::picking::prelude::{Click, Pointer};
use bevy::prelude::*;

use crate::game::state::{
    BuilderMode, GameMode, PlacementState, PlayingUiState, SimulationState, SolutionState,
    StartMenuScreen, WorldEntryMode,
};
use crate::game::ui::core::text_input::{primary_click, push_text_input};
use crate::game::ui::core::world_menu::{
    confirm_text_prompt, open_loaded_world, open_text_prompt, reset_current_solution,
    return_to_main_menu, save_current_world, switch_to_edit_mode_and_rebuild, WorldMenuParams,
};
use crate::game::ui::types::{CarriedItem, InventoryItems};
use crate::shared::save::{delete_save, SaveState};

use super::types::{
    ConfirmDialogAction, ConfirmDialogKind, ConfirmDialogState, SaveListAction, TextPromptAction,
    TextPromptKind, TextPromptState,
};

pub fn save_list_actions(
    mut click: On<Pointer<Click>>,
    mode: Res<State<GameMode>>,
    mut start_menu_screen: ResMut<StartMenuScreen>,
    mut next_state: ResMut<NextState<GameMode>>,
    mut builder_mode: ResMut<BuilderMode>,
    mut inventory: ResMut<InventoryItems>,
    mut carried: ResMut<CarriedItem>,
    mut placement: ResMut<PlacementState>,
    mut save_state: ResMut<SaveState>,
    mut solution_state: ResMut<SolutionState>,
    mut simulation: ResMut<SimulationState>,
    mut world_menu: WorldMenuParams,
    mut confirm_dialog: ResMut<ConfirmDialogState>,
    mut text_prompt: ResMut<TextPromptState>,
    actions: Query<&SaveListAction>,
) {
    if !primary_click(&mut click)
        || *mode.get() != GameMode::StartMenu
        || *start_menu_screen != StartMenuScreen::SaveList
    {
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
            open_text_prompt(&mut text_prompt, TextPromptKind::NewPuzzle, "puzzle");
        }
        SaveListAction::NewSolution => {
            if solution_state.save_list_entry != WorldEntryMode::PlaySolution {
                return;
            }
            let Some(puzzle_name) = save_state.selected_puzzle.clone() else {
                return;
            };
            open_text_prompt(
                &mut text_prompt,
                TextPromptKind::NewSolution {
                    puzzle: puzzle_name,
                },
                "solution",
            );
        }
        SaveListAction::LoadPuzzle(name) => {
            if solution_state.save_list_entry == WorldEntryMode::EditPuzzle {
                if !save_state.puzzles().iter().any(|entry| entry.name == *name) {
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
                    world_menu.render_assets.as_deref(),
                    &world_menu.debug,
                    &mut world_menu.factory_structures,
                    &mut world_menu.movement_influence,
                    &mut world_menu.pusher_state,
                    *mode.get(),
                    &mut next_state,
                );
            } else {
                let Some(choice) = save_state
                    .puzzle_choices()
                    .into_iter()
                    .find(|choice| choice.name == *name)
                else {
                    return;
                };
                save_state.select_puzzle(Some(choice.name), Some(choice.source));
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
                world_menu.render_assets.as_deref(),
                &world_menu.debug,
                &mut world_menu.factory_structures,
                &mut world_menu.movement_influence,
                &mut world_menu.pusher_state,
                *mode.get(),
                &mut next_state,
            );
        }
        SaveListAction::RenamePuzzle(name) => {
            if solution_state.save_list_entry != WorldEntryMode::EditPuzzle {
                return;
            }
            if save_state.puzzles().iter().any(|entry| entry.name == *name) {
                open_text_prompt(
                    &mut text_prompt,
                    TextPromptKind::RenamePuzzle { name: name.clone() },
                    &name,
                );
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
                open_text_prompt(
                    &mut text_prompt,
                    TextPromptKind::RenameSolution { name: name.clone() },
                    &name,
                );
            }
        }
        SaveListAction::DeletePuzzle(name) => {
            if solution_state.save_list_entry != WorldEntryMode::EditPuzzle {
                return;
            }
            if save_state.puzzles().iter().any(|entry| entry.name == *name) {
                confirm_dialog.kind = Some(ConfirmDialogKind::DeleteSave { name: name.clone() });
            }
        }
        SaveListAction::DeleteSolution(name) => {
            if save_state
                .selected_puzzle_solutions()
                .iter()
                .any(|entry| entry.name == *name)
            {
                confirm_dialog.kind = Some(ConfirmDialogKind::DeleteSave { name: name.clone() });
            }
        }
        SaveListAction::Back => {
            *start_menu_screen = StartMenuScreen::Main;
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
    mut playing_ui: ResMut<PlayingUiState>,
    mut next_state: ResMut<NextState<GameMode>>,
    mut start_menu_screen: ResMut<StartMenuScreen>,
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
                save_state.select_puzzle(None, None);
            }
        }
        (ConfirmDialogKind::ResetSolution, ConfirmDialogAction::Primary) => {
            reset_current_solution(
                &mut world_menu.world,
                &mut simulation,
                &mut world_menu.commands,
                &mut world_menu.meshes,
                &world_menu.block_entities,
                world_menu.render_assets.as_deref(),
                &world_menu.debug,
                &mut world_menu.factory_structures,
                &mut world_menu.movement_influence,
                &mut world_menu.pusher_state,
                &solution_state,
            );
            playing_ui.paused = true;
        }
        (ConfirmDialogKind::ReturnToMain, ConfirmDialogAction::Primary) => {
            save_current_world(
                &world_menu.world,
                &inventory,
                &mut save_state,
                &mut solution_state,
                &simulation,
            );
            return_to_main_menu(
                &mut world_menu.world,
                &mut placement,
                &mut save_state,
                &mut solution_state,
                &mut simulation,
                &mut world_menu.commands,
                &mut world_menu.meshes,
                world_menu.render_assets.as_deref(),
                &world_menu.block_entities,
                &world_menu.debug,
                &mut world_menu.factory_structures,
                &mut world_menu.movement_influence,
                &mut world_menu.pusher_state,
                &mut next_state,
                &mut start_menu_screen,
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
                world_menu.render_assets.as_deref(),
                &world_menu.block_entities,
                &world_menu.debug,
                &mut world_menu.factory_structures,
                &mut world_menu.movement_influence,
                &mut world_menu.pusher_state,
                &mut next_state,
                &mut start_menu_screen,
            );
        }
        (ConfirmDialogKind::SaveSolutionBeforeEdit, ConfirmDialogAction::Primary) => {
            save_current_world(
                &world_menu.world,
                &inventory,
                &mut save_state,
                &mut solution_state,
                &simulation,
            );
            switch_to_edit_mode_and_rebuild(
                &mut world_menu.world,
                &mut builder_mode,
                &mut inventory,
                &mut carried,
                &mut placement,
                &mut playing_ui,
                &mut save_state,
                &mut solution_state,
                &mut world_menu.commands,
                &mut world_menu.meshes,
                &world_menu.block_entities,
                world_menu.render_assets.as_deref(),
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
                &mut playing_ui,
                &mut save_state,
                &mut solution_state,
                &mut world_menu.commands,
                &mut world_menu.meshes,
                &world_menu.block_entities,
                world_menu.render_assets.as_deref(),
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

pub fn text_prompt_actions(
    mut click: On<Pointer<Click>>,
    mut prompt: ResMut<TextPromptState>,
    mode: Res<State<GameMode>>,
    mut next_state: ResMut<NextState<GameMode>>,
    mut builder_mode: ResMut<BuilderMode>,
    mut inventory: ResMut<InventoryItems>,
    mut carried: ResMut<CarriedItem>,
    mut placement: ResMut<PlacementState>,
    mut save_state: ResMut<SaveState>,
    mut solution_state: ResMut<SolutionState>,
    mut simulation: ResMut<SimulationState>,
    mut world_menu: WorldMenuParams,
    actions: Query<&TextPromptAction>,
) {
    if !primary_click(&mut click) {
        return;
    }
    let Ok(action) = actions.get(click.entity).copied() else {
        return;
    };
    click.propagate(false);
    match action {
        TextPromptAction::Confirm => confirm_text_prompt(
            &mut prompt,
            *mode.get(),
            &mut next_state,
            &mut builder_mode,
            &mut inventory,
            &mut carried,
            &mut placement,
            &mut save_state,
            &mut solution_state,
            &mut simulation,
            &mut world_menu,
        ),
        TextPromptAction::Cancel => {
            prompt.kind = None;
            prompt.value.clear();
        }
    }
}

pub fn text_prompt_input(
    mut prompt: ResMut<TextPromptState>,
    mode: Res<State<GameMode>>,
    mut next_state: ResMut<NextState<GameMode>>,
    mut builder_mode: ResMut<BuilderMode>,
    mut inventory: ResMut<InventoryItems>,
    mut carried: ResMut<CarriedItem>,
    mut placement: ResMut<PlacementState>,
    mut save_state: ResMut<SaveState>,
    mut solution_state: ResMut<SolutionState>,
    mut simulation: ResMut<SimulationState>,
    mut world_menu: WorldMenuParams,
    mut keyboard_input: MessageReader<KeyboardInput>,
) {
    if prompt.kind.is_none() {
        return;
    }
    let mut confirm = false;
    let mut cancel = false;
    for event in keyboard_input.read() {
        if event.state != bevy::input::ButtonState::Pressed {
            continue;
        }
        match &event.logical_key {
            bevy::input::keyboard::Key::Enter => confirm = true,
            bevy::input::keyboard::Key::Escape => cancel = true,
            bevy::input::keyboard::Key::Backspace => {
                prompt.value.pop();
            }
            _ => push_text_input(&mut prompt.value, event),
        }
    }
    if confirm {
        confirm_text_prompt(
            &mut prompt,
            *mode.get(),
            &mut next_state,
            &mut builder_mode,
            &mut inventory,
            &mut carried,
            &mut placement,
            &mut save_state,
            &mut solution_state,
            &mut simulation,
            &mut world_menu,
        );
    } else if cancel {
        prompt.kind = None;
        prompt.value.clear();
    }
}
