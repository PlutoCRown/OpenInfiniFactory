use bevy::prelude::*;

use crate::game::state::{GameMode, SolutionState, StartMenuScreen, WorldEntryMode};
use crate::game::ui::components::{
    full_width_button, hover_border, inset_border, pressed_border, raised_border, text,
    BUTTON_BG, BUTTON_HOVER_BG,
};
use crate::game::ui::screens::{
    spawn_save_management_row, spawn_save_select_row, SAVE_LIST_EDIT_COLUMN_WIDTH,
    SAVE_LIST_PLAY_COLUMN_WIDTH,
};
use crate::game::ui::types::{
    SaveListAction, SaveListCloseButton, SaveListPanel,
    SaveListPrompt, SaveListPuzzleColumn, SaveListRenderState, SaveListSolutionColumn,
    SaveListTitleText, UiHoverState,
};
use crate::shared::i18n::I18n;
use crate::shared::save::SaveState;

use super::view::{
    save_list_puzzle_rows, save_list_title, SaveListColumn, SaveListViewCtx,
};

pub fn update_save_list_ui(
    mode: Res<State<GameMode>>,
    start_menu_screen: Res<StartMenuScreen>,
    save_state: Res<SaveState>,
    solution_state: Res<SolutionState>,
    i18n: Res<I18n>,
    hover: Res<UiHoverState>,
    mut render_state: ResMut<SaveListRenderState>,
    mut commands: Commands,
    mut texts: ParamSet<(
        Query<&mut Text, With<SaveListTitleText>>,
        Query<&mut Text, (Without<SaveListTitleText>, Without<SaveListPrompt>)>,
        Query<&mut Text, With<SaveListPrompt>>,
    )>,
    mut puzzle_columns: Query<
        (Entity, &mut Node, &Children),
        (With<SaveListPuzzleColumn>, Without<SaveListSolutionColumn>),
    >,
    mut solution_columns: Query<
        (Entity, &mut Node, &Children),
        (With<SaveListSolutionColumn>, Without<SaveListPuzzleColumn>),
    >,
    mut panels: Query<
        &mut Node,
        (
            With<SaveListPanel>,
            Without<SaveListPuzzleColumn>,
            Without<SaveListSolutionColumn>,
        ),
    >,
    mut buttons: Query<
        (
            Entity,
            &SaveListAction,
            &Children,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        (With<Button>, Without<SaveListCloseButton>),
    >,
) {
    let play_flow = solution_state.save_list_entry == WorldEntryMode::PlaySolution;
    let edit_flow = solution_state.save_list_entry == WorldEntryMode::EditPuzzle;
    let puzzle_rows = save_list_puzzle_rows(&save_state, edit_flow);
    let solution_rows = save_state
        .selected_puzzle_solutions()
        .iter()
        .map(|entry| entry.name.clone())
        .collect::<Vec<_>>();
    let show_solutions = play_flow && save_state.selected_puzzle.is_some();

    for mut text in &mut texts.p0() {
        text.0 = save_list_title(
            *mode.get(),
            *start_menu_screen,
            solution_state.save_list_entry,
            &i18n,
        );
    }

    update_save_list_columns(
        &mut commands,
        &mut render_state,
        &mut panels,
        &mut puzzle_columns,
        &mut solution_columns,
        &puzzle_rows,
        &solution_rows,
        solution_state.save_list_entry,
        edit_flow,
        show_solutions,
    );

    for mut text in &mut texts.p2() {
        text.0 = if play_flow && save_state.selected_puzzle.is_none() {
            i18n.text("save.choose_puzzle_prompt")
        } else {
            String::new()
        };
    }

    let ctx = SaveListViewCtx {
        save_state: &save_state,
        edit_flow,
        play_flow,
        i18n: &i18n,
    };
    for (entity, action, children, mut background, mut border) in &mut buttons {
        let view = action.button_view(&ctx);
        let hovered = view.enabled && hover.entity == Some(entity);

        *background = if hovered {
            BUTTON_HOVER_BG.into()
        } else if view.enabled && view.selected {
            Color::srgba(0.22, 0.35, 0.32, 0.96).into()
        } else if view.enabled {
            BUTTON_BG.into()
        } else {
            Color::srgba(0.12, 0.12, 0.13, 0.82).into()
        };
        *border = if hovered {
            hover_border()
        } else if view.selected {
            pressed_border()
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

fn update_save_list_columns(
    commands: &mut Commands,
    render_state: &mut SaveListRenderState,
    panels: &mut Query<
        &mut Node,
        (
            With<SaveListPanel>,
            Without<SaveListPuzzleColumn>,
            Without<SaveListSolutionColumn>,
        ),
    >,
    puzzle_columns: &mut Query<
        (Entity, &mut Node, &Children),
        (With<SaveListPuzzleColumn>, Without<SaveListSolutionColumn>),
    >,
    solution_columns: &mut Query<
        (Entity, &mut Node, &Children),
        (With<SaveListSolutionColumn>, Without<SaveListPuzzleColumn>),
    >,
    puzzle_rows: &[String],
    solution_rows: &[String],
    entry: WorldEntryMode,
    edit_flow: bool,
    show_solutions: bool,
) {
    let puzzle_column = if edit_flow {
        SaveListColumn::PuzzleEdit
    } else {
        SaveListColumn::PuzzlePlay
    };
    let puzzle_width = if edit_flow {
        SAVE_LIST_EDIT_COLUMN_WIDTH
    } else {
        SAVE_LIST_PLAY_COLUMN_WIDTH
    };
    let solution_width = SAVE_LIST_EDIT_COLUMN_WIDTH;
    let panel_width = if show_solutions {
        puzzle_width + solution_width + 44.0
    } else {
        puzzle_width + 32.0
    };
    for mut panel in panels {
        panel.width = Val::Px(panel_width);
    }

    for (entity, mut node, children) in puzzle_columns {
        node.display = Display::Flex;
        node.width = Val::Px(puzzle_width);
        if render_state.entry != Some(entry) || render_state.puzzle_keys != puzzle_rows {
            rebuild_column(commands, entity, children, puzzle_column, puzzle_rows);
        }
    }
    if render_state.entry != Some(entry) || render_state.puzzle_keys != puzzle_rows {
        render_state.puzzle_keys = puzzle_rows.to_vec();
    }

    for (entity, mut node, children) in solution_columns {
        node.display = if show_solutions {
            Display::Flex
        } else {
            Display::None
        };
        node.width = Val::Px(solution_width);
        if render_state.entry != Some(entry) || render_state.solution_keys != solution_rows {
            rebuild_column(
                commands,
                entity,
                children,
                SaveListColumn::Solution,
                solution_rows,
            );
        }
    }
    if render_state.entry != Some(entry) || render_state.solution_keys != solution_rows {
        render_state.solution_keys = solution_rows.to_vec();
    }
    render_state.entry = Some(entry);
}

fn rebuild_column(
    commands: &mut Commands,
    column_entity: Entity,
    children: &Children,
    column: SaveListColumn,
    names: &[String],
) {
    for child in children.iter() {
        commands.entity(child).despawn();
    }
    commands.entity(column_entity).with_children(|parent| {
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
        if let Some(create_action) = column.create_action() {
            parent
                .spawn((full_width_button(34.0), create_action))
                .with_children(|button| {
                    button.spawn(text("", 15.0, Color::WHITE));
                });
        }
    });
}
