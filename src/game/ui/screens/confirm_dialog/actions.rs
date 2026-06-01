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
    CarriedItem, ConfirmDialogAction, ConfirmDialogEffect, ConfirmDialogResult, InventoryItems,
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
    mut ui_runtime: ResMut<UiRuntime>,
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
    ui_runtime.close_modal();
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
        }
        ConfirmDialogEffect::SaveCurrentWorld => {
            save_current_world(
                &world_menu.world,
                &inventory,
                &mut save_state,
                &mut solution_state,
                &simulation,
            );
        }
        ConfirmDialogEffect::SaveAsNewPuzzle { default_name } => {
            ui_runtime.open_text_prompt(TextPromptKind::SaveAsNewPuzzle, &default_name);
        }
    }
}
