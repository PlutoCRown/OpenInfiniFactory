use bevy::prelude::*;

use crate::game::state::{GameMode, SolutionState, StartMenuScreen, WorldEntryMode};
use crate::game::ui::access::{i18n, UiMainThread};
use crate::game::ui::components::{
    hover_border, inset_border, pressed_border, raised_border, BUTTON_BG, BUTTON_HOVER_BG,
};
use crate::game::ui::screens::{spawn_save_management_row, spawn_save_select_row};
use crate::game::ui::types::{
    SaveListAction, SaveListCloseButton, SaveListCreateButton, SaveListPrompt,
    SaveListPuzzleColumn, SaveListPuzzleRows, SaveListRenderState, SaveListSolutionColumn,
    SaveListSolutionRows, SaveListTitleText, UiHoverState,
};
use crate::shared::save::SaveState;

use super::view::{save_list_puzzle_rows, save_list_title, SaveListColumn, SaveListViewCtx};

pub fn update_save_list_ui(
    _ui_thread: UiMainThread,
    mode: Res<State<GameMode>>,
    start_menu_screen: Res<StartMenuScreen>,
    save_state: Res<SaveState>,
    solution_state: Res<SolutionState>,
    hover: Res<UiHoverState>,
    mut render_state: ResMut<SaveListRenderState>,
    mut commands: Commands,
    mut texts: ParamSet<(
        Query<&mut Text, With<SaveListTitleText>>,
        Query<&mut Text, (Without<SaveListTitleText>, Without<SaveListPrompt>)>,
        Query<&mut Text, With<SaveListPrompt>>,
    )>,
    mut column_nodes: ParamSet<(
        Query<&mut Node, (With<SaveListPuzzleColumn>, Without<SaveListSolutionColumn>)>,
        Query<&mut Node, (With<SaveListSolutionColumn>, Without<SaveListPuzzleColumn>)>,
        Query<
            (&SaveListAction, &mut Node),
            (
                With<SaveListCreateButton>,
                With<Button>,
                Without<SaveListPuzzleColumn>,
                Without<SaveListSolutionColumn>,
            ),
        >,
    )>,
    puzzle_rows_query: Query<Entity, (With<SaveListPuzzleRows>, Without<SaveListSolutionRows>)>,
    solution_rows_query: Query<Entity, (With<SaveListSolutionRows>, Without<SaveListPuzzleRows>)>,
    mut buttons: Query<
        (
            Entity,
            &SaveListAction,
            &Children,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        (
            With<Button>,
            Without<SaveListCloseButton>,
            Without<SaveListPuzzleColumn>,
            Without<SaveListSolutionColumn>,
            Without<SaveListPuzzleRows>,
            Without<SaveListSolutionRows>,
        ),
    >,
) {
    let play_flow = solution_state.save_list_entry == WorldEntryMode::PlaySolution;
    let edit_flow = solution_state.save_list_entry == WorldEntryMode::EditPuzzle;
    let puzzle_rows = save_list_puzzle_rows(&save_state);
    let solution_rows = save_state
        .selected_puzzle_solutions()
        .iter()
        .filter_map(|entry| entry.slot.solution.clone())
        .collect::<Vec<_>>();
    let show_solutions = play_flow && save_state.selected_puzzle.is_some();

    for mut text in &mut texts.p0() {
        text.0 = save_list_title(
            *mode.get(),
            *start_menu_screen,
            solution_state.save_list_entry,
        );
    }

    for mut node in &mut column_nodes.p0() {
        node.display = Display::Flex;
    }
    for mut node in &mut column_nodes.p1() {
        node.display = if show_solutions {
            Display::Flex
        } else {
            Display::None
        };
    }
    for (action, mut node) in &mut column_nodes.p2() {
        node.display = match action {
            SaveListAction::NewPuzzle => {
                if edit_flow {
                    Display::Flex
                } else {
                    Display::None
                }
            }
            SaveListAction::NewSolution => {
                if show_solutions {
                    Display::Flex
                } else {
                    Display::None
                }
            }
            _ => node.display,
        };
    }

    let entry = solution_state.save_list_entry;
    let puzzle_column = if edit_flow {
        SaveListColumn::PuzzleEdit
    } else {
        SaveListColumn::PuzzlePlay
    };

    if render_state.entry != Some(entry) || render_state.puzzle_keys != puzzle_rows {
        for entity in &puzzle_rows_query {
            rebuild_rows(&mut commands, entity, puzzle_column, &puzzle_rows);
        }
        render_state.puzzle_keys = puzzle_rows.clone();
    }
    if render_state.entry != Some(entry) || render_state.solution_keys != solution_rows {
        for entity in &solution_rows_query {
            rebuild_rows(
                &mut commands,
                entity,
                SaveListColumn::Solution,
                &solution_rows,
            );
        }
        render_state.solution_keys = solution_rows;
    }
    render_state.entry = Some(entry);

    for mut text in &mut texts.p2() {
        text.0 = if play_flow && save_state.selected_puzzle.is_none() {
            i18n.t("save.choose_puzzle_prompt")
        } else {
            String::new()
        };
    }

    let ctx = SaveListViewCtx {
        save_state: &save_state,
        edit_flow,
        play_flow,
    };
    for (entity, action, children, mut background, mut border) in &mut buttons {
        let view = action.button_view(&ctx);
        let hovered = view.enabled && hover.entity == Some(entity);

        *background = if view.enabled && view.selected {
            Color::srgba(0.22, 0.35, 0.32, 0.96).into()
        } else if hovered {
            BUTTON_HOVER_BG.into()
        } else if view.enabled {
            BUTTON_BG.into()
        } else {
            Color::srgba(0.12, 0.12, 0.13, 0.82).into()
        };
        *border = if view.selected {
            pressed_border()
        } else if hovered {
            hover_border()
        } else if view.enabled {
            raised_border()
        } else {
            inset_border()
        };

        for child in children.iter() {
            if let Ok(mut text) = texts.p1().get_mut(child) {
                text.0 = view.label.clone();
            }
        }
    }
}

fn rebuild_rows(
    commands: &mut Commands,
    rows_entity: Entity,
    column: SaveListColumn,
    names: &[String],
) {
    // 空行容器可能尚无 Children，重建时直接清子树再挂行
    commands.entity(rows_entity).despawn_related::<Children>();
    commands.entity(rows_entity).with_children(|parent| {
        for name in names {
            if column.is_management() {
                spawn_save_management_row(
                    parent,
                    column.load(name.clone()),
                    column.rename(name.clone()),
                    column.delete(name.clone()),
                );
            } else {
                spawn_save_select_row(parent, column.load(name.clone()));
            }
        }
    });
}
