use bevy::picking::prelude::{Click, Pointer};
use bevy::prelude::*;

use crate::game::state::{
    BuilderMode, GameMode, PlacementState, SimulationState, SolutionState, WorldEntryMode,
};
use crate::game::systems::world_flow::{
    open_loaded_world_from_menu, primary_click, reset_current_solution, return_to_main_menu,
    save_current_world, switch_to_edit_mode_and_rebuild, WorldMenuParams,
};
use crate::game::ui::{
    CarriedItem, CloseUiModal, ConfirmDialogAction, ConfirmDialogEffect, ConfirmDialogResult,
    GameplayUiChanged, InventoryChanged, InventoryItems, OpenTextPrompt, SaveListChanged,
    TextPromptKind, UiRuntime,
};
use crate::shared::save::{delete_save, SaveState};

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
    ui_runtime: Res<UiRuntime>,
    mut close_modal: MessageWriter<CloseUiModal>,
    mut open_prompt: MessageWriter<OpenTextPrompt>,
    mut gameplay_ui_changed: MessageWriter<GameplayUiChanged>,
    mut inventory_changed: MessageWriter<InventoryChanged>,
    mut save_list_changed: MessageWriter<SaveListChanged>,
    actions: Query<&ConfirmDialogAction>,
) {
    if !primary_click(&mut click) {
        return;
    }
    let Some(dialog) = ui_runtime.confirm_dialog().cloned() else {
        return;
    };
    let Ok(action) = actions.get(click.entity).copied() else {
        return;
    };
    click.propagate(false);

    let result = action.result();
    close_modal.write(CloseUiModal);
    let effect = match result {
        ConfirmDialogResult::Primary => dialog.primary_effect,
        ConfirmDialogResult::Secondary => {
            dialog.secondary_effect.unwrap_or(ConfirmDialogEffect::None)
        }
        ConfirmDialogResult::Cancel => ConfirmDialogEffect::None,
    };

    match effect {
        ConfirmDialogEffect::None => {}
        ConfirmDialogEffect::DeleteSave { name } => {
            delete_save(&name);
            save_state.refresh();
            if save_state.selected_puzzle.as_deref() == Some(name.as_str()) {
                save_state.select_puzzle(None, None);
            }
            save_list_changed.write(SaveListChanged);
        }
        ConfirmDialogEffect::ResetSolution => {
            reset_current_solution(
                &mut world_menu.world,
                &mut simulation,
                &mut world_menu.commands,
                &mut world_menu.meshes,
                &world_menu.block_entities,
                &world_menu.render_manager,
                &world_menu.debug,
                &mut world_menu.factory_structures,
                &mut world_menu.movement_influence,
                &mut world_menu.pusher_state,
                &solution_state,
            );
            inventory_changed.write(InventoryChanged);
            gameplay_ui_changed.write(GameplayUiChanged);
            *mode = GameMode::Paused;
        }
        ConfirmDialogEffect::ReturnToMain { save_first } => {
            if save_first {
                save_current_world(
                    &world_menu.world,
                    &inventory,
                    &mut save_state,
                    &mut solution_state,
                    &simulation,
                );
                save_list_changed.write(SaveListChanged);
            }
            return_to_main_menu(
                &mut world_menu.world,
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
            save_list_changed.write(SaveListChanged);
            gameplay_ui_changed.write(GameplayUiChanged);
        }
        ConfirmDialogEffect::SwitchToEditMode { save_first } => {
            if save_first {
                save_current_world(
                    &world_menu.world,
                    &inventory,
                    &mut save_state,
                    &mut solution_state,
                    &simulation,
                );
                save_list_changed.write(SaveListChanged);
            }
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
                &world_menu.render_manager,
                &world_menu.debug,
                &mut world_menu.factory_structures,
                &mut world_menu.movement_influence,
                &mut world_menu.pusher_state,
            );
            inventory_changed.write(InventoryChanged);
            save_list_changed.write(SaveListChanged);
            gameplay_ui_changed.write(GameplayUiChanged);
        }
        ConfirmDialogEffect::OpenPuzzleForEdit { name } => {
            open_loaded_world_from_menu(
                &name,
                WorldEntryMode::EditPuzzle,
                &mut mode,
                &mut builder_mode,
                &mut inventory,
                &mut carried,
                &mut placement,
                &mut save_state,
                &mut solution_state,
                &mut simulation,
                &mut world_menu,
            );
            inventory_changed.write(InventoryChanged);
            save_list_changed.write(SaveListChanged);
            gameplay_ui_changed.write(GameplayUiChanged);
        }
        ConfirmDialogEffect::SaveCurrentWorld => {
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
        ConfirmDialogEffect::SaveAsNewPuzzle { default_name } => {
            open_prompt.write(OpenTextPrompt::new(
                TextPromptKind::SaveAsNewPuzzle,
                default_name,
            ));
        }
    }
}
