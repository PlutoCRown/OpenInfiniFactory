use bevy::picking::prelude::{Click, Pointer};
use bevy::prelude::*;

use crate::game::state::{
    BuilderMode, GameMode, PlacementState, SimulationState, SolutionState, WorldEntryMode,
};
use crate::game::systems::world_flow::{
    primary_click, puzzle_has_solutions, return_to_main_menu_from_menu, save_current_world,
    WorldMenuParams,
};
use crate::game::ui::{
    CarriedItem, ConfirmDialogButtonSpec, ConfirmDialogEffect, ConfirmDialogMessage,
    ConfirmDialogSpec, GameplayUiChanged, InventoryChanged, InventoryItems, OpenConfirmDialog,
    OpenTextPrompt, OpenUiPanel, PauseMenuAction, SaveListChanged, TextPromptKind, UiPanelContext,
    UiPanelKey,
};
use crate::shared::save::{next_named_save, SaveKind, SaveState};

pub fn pause_menu_actions(
    mut click: On<Pointer<Click>>,
    mut builder_mode: ResMut<BuilderMode>,
    mut simulation: ResMut<SimulationState>,
    mut inventory: ResMut<InventoryItems>,
    mut carried: ResMut<CarriedItem>,
    mut placement: ResMut<PlacementState>,
    mut mode: ResMut<GameMode>,
    mut save_state: ResMut<SaveState>,
    mut solution_state: ResMut<SolutionState>,
    mut open_confirm: MessageWriter<OpenConfirmDialog>,
    mut open_prompt: MessageWriter<OpenTextPrompt>,
    mut open_panel: MessageWriter<OpenUiPanel>,
    mut gameplay_ui_changed: MessageWriter<GameplayUiChanged>,
    mut inventory_changed: MessageWriter<InventoryChanged>,
    mut save_list_changed: MessageWriter<SaveListChanged>,
    mut world_menu: WorldMenuParams,
    actions: Query<&PauseMenuAction>,
) {
    if !primary_click(&mut click) || *mode != GameMode::Paused {
        return;
    }
    let Ok(action) = actions.get(click.entity).copied() else {
        return;
    };
    click.propagate(false);

    match action {
        PauseMenuAction::Resume => *mode = GameMode::Playing,
        PauseMenuAction::ToggleBuilderMode => {
            if solution_state.entry == WorldEntryMode::PlaySolution {
                return;
            }
            *builder_mode = match *builder_mode {
                BuilderMode::Edit => {
                    simulation.running = false;
                    simulation.step_requested = false;
                    simulation.accumulator = 0.0;
                    simulation.start_snapshot = None;
                    simulation.start_factory_structures = None;
                    solution_state.puzzle_snapshot = Some(world_menu.world.clone());
                    solution_state.puzzle_name = save_state.current.clone();
                    save_state.current = Some(next_named_save(
                        &save_state
                            .entries
                            .iter()
                            .map(|entry| entry.name.clone())
                            .collect::<Vec<_>>(),
                        save_state.current.as_deref().unwrap_or("solution"),
                    ));
                    save_state.current_kind = Some(SaveKind::Solution);
                    BuilderMode::Play
                }
                BuilderMode::Play => {
                    open_confirm.write(OpenConfirmDialog(ConfirmDialogSpec::new(
                        ConfirmDialogMessage::TextKey("confirm.save_solution_before_edit"),
                        ConfirmDialogButtonSpec::new(
                            "button.save_solution_and_edit",
                            ConfirmDialogEffect::SwitchToEditMode { save_first: true },
                        ),
                        Some(ConfirmDialogButtonSpec::new(
                            "button.discard_solution_and_edit",
                            ConfirmDialogEffect::SwitchToEditMode { save_first: false },
                        )),
                    )));
                    return;
                }
            };
            *inventory = InventoryItems::for_mode(*builder_mode);
            carried.clear();
            placement.selected = 0;
            inventory_changed.write(InventoryChanged);
            save_list_changed.write(SaveListChanged);
            gameplay_ui_changed.write(GameplayUiChanged);
            *mode = GameMode::Playing;
        }
        PauseMenuAction::SaveWorld => {
            if let (Some(SaveKind::Puzzle), Some(name)) =
                (save_state.current_kind, save_state.current.clone())
            {
                if puzzle_has_solutions(&mut save_state, &name) {
                    open_confirm.write(OpenConfirmDialog(ConfirmDialogSpec::new(
                        ConfirmDialogMessage::Named {
                            key: "confirm.save_puzzle_with_solutions",
                            name: name.clone(),
                        },
                        ConfirmDialogButtonSpec::new(
                            "button.save_puzzle",
                            ConfirmDialogEffect::SaveCurrentWorld,
                        ),
                        Some(ConfirmDialogButtonSpec::new(
                            "button.save_as_new_puzzle",
                            ConfirmDialogEffect::SaveAsNewPuzzle { default_name: name },
                        )),
                    )));
                    return;
                }
            }
            save_current_world(
                &world_menu.world,
                &inventory,
                &mut save_state,
                &mut solution_state,
                &simulation,
            );
            save_list_changed.write(SaveListChanged);
            gameplay_ui_changed.write(GameplayUiChanged);
        }
        PauseMenuAction::SaveAsNewPuzzle => {
            open_prompt.write(OpenTextPrompt::new(
                TextPromptKind::SaveAsNewPuzzle,
                "puzzle",
            ));
        }
        PauseMenuAction::ResetSolution => {
            open_confirm.write(OpenConfirmDialog(ConfirmDialogSpec::new(
                ConfirmDialogMessage::TextKey("confirm.reset_solution"),
                ConfirmDialogButtonSpec::new(
                    "button.confirm_reset_solution",
                    ConfirmDialogEffect::ResetSolution,
                ),
                None,
            )));
        }
        PauseMenuAction::OpenSettings => {
            open_panel.write(OpenUiPanel::new(
                UiPanelKey::SETTINGS,
                UiPanelContext::ReturnTo(GameMode::Paused),
            ));
        }
        PauseMenuAction::BackToMainMenu => {
            if solution_state.dirty {
                open_confirm.write(OpenConfirmDialog(ConfirmDialogSpec::new(
                    ConfirmDialogMessage::TextKey("confirm.return_to_main"),
                    ConfirmDialogButtonSpec::new(
                        "button.save_and_back",
                        ConfirmDialogEffect::ReturnToMain { save_first: true },
                    ),
                    Some(ConfirmDialogButtonSpec::new(
                        "button.discard_and_back",
                        ConfirmDialogEffect::ReturnToMain { save_first: false },
                    )),
                )));
            } else {
                return_to_main_menu_from_menu(
                    &mut placement,
                    &mut save_state,
                    &mut solution_state,
                    &mut simulation,
                    &mut mode,
                    &mut world_menu,
                );
                save_list_changed.write(SaveListChanged);
                gameplay_ui_changed.write(GameplayUiChanged);
            }
        }
    }
}
