use bevy::prelude::*;

use crate::game::state::{GameMode, StartMenuScreen, WorldEntryMode};
use crate::game::ui::access::i18n;
use crate::game::ui::features::save::save_list_title;

use super::super::components::{
    BUTTON_BG, default_button_size, flex_row, full_width_button, panel_bundle_responsive_flow,
    panel_content, panel_title_bar, panel_title_button, panel_title_label, raised_border,
    spawn_close_icon, styled_button, text, text_button, transparent_node,
};
use super::super::types::{
    LocalizedText, PanelVisibility, SaveListAction, SaveListCloseButton, SaveListCreateButton,
    SaveListPanel, SaveListPrompt, SaveListPuzzleColumn, SaveListPuzzleRows,
    SaveListSolutionColumn, SaveListSolutionRows, SaveListTitleText,
};

pub const SAVE_LIST_PANEL_WIDTH_PERCENT: f32 = 46.0;
pub const SAVE_LIST_PANEL_MAX_WIDTH: f32 = 480.0;
const SAVE_LIST_ACTION_BUTTON_WIDTH: f32 = 72.0;

/// 挂载存档选择：Puzzle / Solution 各一块独立面板（须已 bind_ui_scope）
pub fn spawn_save_list(root: &mut ChildSpawnerCommands, entry: WorldEntryMode) {
    let title = save_list_title(GameMode::StartMenu, StartMenuScreen::SaveList, entry);
    let edit_flow = entry == WorldEntryMode::EditPuzzle;

    root.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            display: Display::Flex,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            column_gap: Val::Px(16.0),
            padding: UiRect::all(Val::Px(12.0)),
            ..default()
        },
        BackgroundColor(Color::NONE),
        Pickable::IGNORE,
        PanelVisibility::StartMenuScreen(StartMenuScreen::SaveList),
    ))
    .with_children(|row| {
        spawn_puzzle_panel(row, title, edit_flow);
        spawn_solution_panel(row);
    });
}

fn spawn_puzzle_panel(row: &mut ChildSpawnerCommands, title: String, edit_flow: bool) {
    row.spawn((
        panel_bundle_responsive_flow(
            SAVE_LIST_PANEL_WIDTH_PERCENT,
            SAVE_LIST_PANEL_MAX_WIDTH,
            false,
        ),
        GlobalZIndex(0),
        SaveListPanel,
        SaveListPuzzleColumn,
    ))
    .with_children(|panel| {
        panel.spawn(panel_title_bar()).with_children(|bar| {
            bar.spawn((panel_title_label(title, 26.0), SaveListTitleText));
            bar.spawn((
                panel_title_button(),
                SaveListAction::Back,
                SaveListCloseButton,
            ))
            .with_children(spawn_close_icon);
        });
        panel.spawn(panel_content()).with_children(|content| {
            content
                .spawn((
                    create_list_button(
                        34.0,
                        if edit_flow {
                            Display::Flex
                        } else {
                            Display::None
                        },
                    ),
                    SaveListAction::NewPuzzle,
                    SaveListCreateButton,
                ))
                .with_children(|button| {
                    button.spawn(text("", 15.0, Color::WHITE));
                });
            content.spawn((transparent_node(save_rows_node()), SaveListPuzzleRows));
            content.spawn((
                text("", 16.0, Color::srgb(0.82, 0.86, 0.88)),
                SaveListPrompt,
            ));
        });
    });
}

fn spawn_solution_panel(row: &mut ChildSpawnerCommands) {
    // 默认 Hidden：编辑流永不打开；游玩流等选中谜题后再由 update 打开
    row.spawn((
        panel_bundle_responsive_flow(
            SAVE_LIST_PANEL_WIDTH_PERCENT,
            SAVE_LIST_PANEL_MAX_WIDTH,
            true,
        ),
        GlobalZIndex(0),
        SaveListSolutionColumn,
    ))
    .with_children(|panel| {
        panel.spawn(panel_title_bar()).with_children(|bar| {
            bar.spawn((
                panel_title_label(i18n.t("save.title.select_solution"), 26.0),
                LocalizedText {
                    key: "save.title.select_solution",
                },
            ));
        });
        panel.spawn(panel_content()).with_children(|content| {
            content
                .spawn((
                    create_list_button(34.0, Display::Flex),
                    SaveListAction::NewSolution,
                    SaveListCreateButton,
                ))
                .with_children(|button| {
                    button.spawn(text("", 15.0, Color::WHITE));
                });
            content.spawn((transparent_node(save_rows_node()), SaveListSolutionRows));
        });
    });
}

fn save_rows_node() -> Node {
    Node {
        width: Val::Percent(100.0),
        flex_direction: FlexDirection::Column,
        row_gap: Val::Px(6.0),
        flex_grow: 0.0,
        ..default()
    }
}

fn create_list_button(height: f32, display: Display) -> impl Bundle {
    text_button(
        Node {
            width: Val::Percent(100.0),
            flex_grow: 0.0,
            flex_shrink: 0.0,
            height: Val::Px(default_button_size(height)),
            display,
            ..default()
        },
        raised_border(),
        BUTTON_BG,
    )
}

pub fn spawn_save_management_row(
    parent: &mut ChildSpawnerCommands,
    load: SaveListAction,
    rename: SaveListAction,
    delete: SaveListAction,
) {
    parent.spawn(flex_row(32.0, 6.0)).with_children(|row| {
        spawn_save_row_button(row, load, SaveRowButtonLayout::Flex);
        spawn_save_row_button(
            row,
            rename,
            SaveRowButtonLayout::Fixed(SAVE_LIST_ACTION_BUTTON_WIDTH),
        );
        spawn_save_row_button(
            row,
            delete,
            SaveRowButtonLayout::Fixed(SAVE_LIST_ACTION_BUTTON_WIDTH),
        );
    });
}

pub fn spawn_save_select_row(parent: &mut ChildSpawnerCommands, load: SaveListAction) {
    parent
        .spawn(full_width_button(32.0))
        .insert(load)
        .with_children(|button| {
            button.spawn(save_row_label("", 13.0));
        });
}

#[derive(Clone, Copy)]
enum SaveRowButtonLayout {
    Flex,
    Fixed(f32),
}

fn spawn_save_row_button(
    parent: &mut ChildSpawnerCommands,
    action: SaveListAction,
    layout: SaveRowButtonLayout,
) {
    let mut style = Node {
        height: Val::Px(default_button_size(30.0)),
        border: UiRect::all(Val::Px(1.0)),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        overflow: Overflow::clip(),
        ..default()
    };
    match layout {
        SaveRowButtonLayout::Flex => {
            style.flex_grow = 1.0;
            style.flex_shrink = 1.0;
            style.min_width = Val::Px(0.0);
        }
        SaveRowButtonLayout::Fixed(width) => {
            let width = default_button_size(width);
            style.width = Val::Px(width);
            style.min_width = Val::Px(width);
            style.flex_shrink = 0.0;
        }
    }

    parent
        .spawn((styled_button(style, raised_border(), BUTTON_BG), action))
        .with_children(|button| {
            button.spawn(save_row_label("", 13.0));
        });
}

fn save_row_label(value: impl Into<String>, font_size: f32) -> impl Bundle {
    (
        text(value, font_size, Color::WHITE),
        TextLayout::no_wrap(),
        Node {
            max_width: Val::Percent(100.0),
            overflow: Overflow::clip(),
            ..default()
        },
    )
}
