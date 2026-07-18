use bevy::prelude::*;

use crate::game::state::{GameMode, SolutionState, StartMenuScreen, WorldEntryMode};
use crate::game::ui::access::{UiMainThread, i18n};
use crate::game::ui::components::{
    BUTTON_BG, BUTTON_HOVER_BG, hover_border, inset_border, pressed_border, raised_border,
};
use crate::game::ui::screens::{spawn_save_management_row, spawn_save_select_row};
use crate::game::ui::types::{
    SaveListAction, SaveListCloseButton, SaveListCreateButton, SaveListPrompt,
    SaveListPuzzleColumn, SaveListPuzzleRows, SaveListRenderState, SaveListSolutionColumn,
    SaveListSolutionRows, SaveListTitleText, UiHoverState,
};
use crate::shared::save::SaveState;

use super::view::{SaveListColumn, SaveListViewCtx, save_list_puzzle_rows, save_list_title};

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
        Query<
            (&mut Node, &mut Visibility),
            (With<SaveListSolutionColumn>, Without<SaveListPuzzleColumn>),
        >,
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
    added_titles: Query<(), Added<SaveListTitleText>>,
    puzzle_rows_query: Query<Entity, (With<SaveListPuzzleRows>, Without<SaveListSolutionRows>)>,
    solution_rows_query: Query<Entity, (With<SaveListSolutionRows>, Without<SaveListPuzzleRows>)>,
    children_query: Query<&Children>,
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
    // 存档列表未显示时不刷
    if *mode.get() != GameMode::StartMenu || *start_menu_screen != StartMenuScreen::SaveList {
        return;
    }

    let play_flow = solution_state.save_list_entry == WorldEntryMode::PlaySolution;
    let edit_flow = solution_state.save_list_entry == WorldEntryMode::EditPuzzle;
    let puzzle_rows = save_list_puzzle_rows(&save_state);
    let solution_rows = save_state
        .selected_puzzle_solutions()
        .iter()
        .filter_map(|entry| entry.slot.solution.clone())
        .collect::<Vec<_>>();
    let show_solutions = play_flow && save_state.selected_puzzle.is_some();

    let structure_changed = mode.is_changed()
        || start_menu_screen.is_changed()
        || save_state.is_changed()
        || solution_state.is_changed()
        || render_state.paint_buttons
        || !added_titles.is_empty();
    let mut rebuilt_rows = false;

    let entry = solution_state.save_list_entry;
    let puzzle_column = if edit_flow {
        SaveListColumn::PuzzleEdit
    } else {
        SaveListColumn::PuzzlePlay
    };
    // 按需挂载后行容器是空壳；或挂载当帧实体尚未出现时，不能信 keys 缓存
    let puzzle_rows_stale =
        row_hosts_stale(puzzle_rows_query.iter(), &children_query, puzzle_rows.len())
            || render_state.entry != Some(entry)
            || render_state.puzzle_keys != puzzle_rows;
    let solution_rows_stale = row_hosts_stale(
        solution_rows_query.iter(),
        &children_query,
        solution_rows.len(),
    ) || render_state.entry != Some(entry)
        || render_state.solution_keys != solution_rows;

    if structure_changed {
        let title = save_list_title(
            *mode.get(),
            *start_menu_screen,
            solution_state.save_list_entry,
        );
        for mut text in &mut texts.p0() {
            if text.0 != title {
                text.0 = title.clone();
            }
        }

        for mut node in &mut column_nodes.p0() {
            if node.display != Display::Flex {
                node.display = Display::Flex;
            }
        }
        let solution_display = if show_solutions {
            Display::Flex
        } else {
            Display::None
        };
        let solution_visibility = if show_solutions {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        for (mut node, mut visibility) in &mut column_nodes.p1() {
            if node.display != solution_display {
                node.display = solution_display;
            }
            visibility.set_if_neq(solution_visibility);
        }
        for (action, mut node) in &mut column_nodes.p2() {
            let next = match action {
                SaveListAction::NewPuzzle => {
                    if edit_flow {
                        Display::Flex
                    } else {
                        Display::None
                    }
                }
                SaveListAction::NewSolution => {
                    // 方案面板本身会显隐；按钮保持可见即可
                    Display::Flex
                }
                _ => node.display,
            };
            if node.display != next {
                node.display = next;
            }
        }

        let prompt = if play_flow && save_state.selected_puzzle.is_none() {
            i18n.t("save.choose_puzzle_prompt")
        } else {
            String::new()
        };
        for mut text in &mut texts.p2() {
            if text.0 != prompt {
                text.0 = prompt.clone();
            }
        }
    }

    if puzzle_rows_stale {
        if puzzle_rows_query.is_empty() {
            // 挂载命令尚未生效，下一帧再重建；勿写 cache
            render_state.paint_buttons = true;
        } else {
            for entity in &puzzle_rows_query {
                rebuild_rows(&mut commands, entity, puzzle_column, &puzzle_rows);
            }
            render_state.puzzle_keys = puzzle_rows.clone();
            rebuilt_rows = true;
        }
    }
    if solution_rows_stale {
        if solution_rows_query.is_empty() {
            render_state.paint_buttons = true;
        } else {
            for entity in &solution_rows_query {
                rebuild_rows(
                    &mut commands,
                    entity,
                    SaveListColumn::Solution,
                    &solution_rows,
                );
            }
            render_state.solution_keys = solution_rows;
            rebuilt_rows = true;
        }
    }
    if rebuilt_rows || structure_changed {
        render_state.entry = Some(entry);
    }
    if rebuilt_rows {
        render_state.paint_buttons = true;
    }

    let style_changed = structure_changed || hover.is_changed() || render_state.paint_buttons;
    if !style_changed {
        return;
    }
    let paint_labels = structure_changed || render_state.paint_buttons;
    // 本帧刚排队重建时实体尚未生成，留到下一帧再清标记
    if !rebuilt_rows {
        render_state.paint_buttons = false;
    }

    let ctx = SaveListViewCtx {
        save_state: &save_state,
        edit_flow,
        play_flow,
    };
    let hover_only = !structure_changed && !paint_labels && hover.is_changed();
    if hover_only {
        let prev = render_state.last_hover;
        let next = hover.entity;
        render_state.last_hover = next;
        for entity in [prev, next].into_iter().flatten() {
            let Ok((_, action, _, mut background, mut border)) = buttons.get_mut(entity) else {
                continue;
            };
            let view = action.button_view(&ctx);
            let hovered = view.enabled && next == Some(entity);
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
        }
        return;
    }

    render_state.last_hover = hover.entity;
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

        if paint_labels {
            for child in children.iter() {
                if let Ok(mut text) = texts.p1().get_mut(child) {
                    if text.0 != view.label {
                        text.0 = view.label.clone();
                    }
                }
            }
        }
    }
}

/// 行容器缺失，或子节点数量与期望不一致（含刚挂载的空壳）
fn row_hosts_stale(
    hosts: impl IntoIterator<Item = Entity>,
    children: &Query<&Children>,
    expected_len: usize,
) -> bool {
    let mut any = false;
    for entity in hosts {
        any = true;
        let count = children.get(entity).map(|c| c.len()).unwrap_or(0);
        if count != expected_len {
            return true;
        }
    }
    // 期望有行但宿主还没出现
    !any && expected_len > 0
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
