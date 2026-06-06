use bevy::prelude::*;

use crate::game::state::StartMenuScreen;
use crate::game::ui::access::i18n;

use super::super::components::{
    default_button_size, flex_row, full_width_button, panel_bundle_responsive, panel_content,
    panel_title_bar, panel_title_button, panel_title_label, raised_border, styled_button, text,
    transparent_node, BUTTON_BG,
};
use super::super::types::{
    PanelVisibility, SaveListAction, SaveListCloseButton, SaveListPanel,
    SaveListPrompt, SaveListPuzzleColumn, SaveListSolutionColumn, SaveListTitleText,
};

pub const SAVE_LIST_PANEL_WIDTH_PERCENT: f32 = 92.0;
pub const SAVE_LIST_PANEL_MAX_WIDTH: f32 = 980.0;
const SAVE_LIST_ACTION_BUTTON_WIDTH: f32 = 72.0;

pub fn spawn_save_list(root: &mut ChildSpawnerCommands) {
    root.spawn((
        panel_bundle_responsive(SAVE_LIST_PANEL_WIDTH_PERCENT, SAVE_LIST_PANEL_MAX_WIDTH),
        GlobalZIndex(0),
        PanelVisibility::StartMenuScreen(StartMenuScreen::SaveList),
        SaveListPanel,
    ))
    .with_children(|panel| {
        panel.spawn(panel_title_bar()).with_children(|title| {
            title.spawn((
                panel_title_label(i18n.t("save.title.default"), 26.0),
                SaveListTitleText,
            ));
            title
                .spawn((
                    panel_title_button(),
                    SaveListAction::Back,
                    SaveListCloseButton,
                ))
                .with_children(|button| {
                    button.spawn(text("x", 12.0, Color::WHITE));
                });
        });
        panel.spawn(panel_content()).with_children(|panel| {
            panel.spawn(save_columns_row()).with_children(|columns| {
                spawn_save_column(columns, SaveListAction::NewPuzzle, SaveListPuzzleColumn);
                spawn_save_column(columns, SaveListAction::NewSolution, SaveListSolutionColumn);
            });
            panel.spawn((
                text("", 16.0, Color::srgb(0.82, 0.86, 0.88)),
                SaveListPrompt,
            ));
        });
    });
}

fn save_columns_row() -> impl Bundle {
    transparent_node(Node {
        width: Val::Percent(100.0),
        display: Display::Flex,
        align_items: AlignItems::FlexStart,
        column_gap: Val::Px(12.0),
        ..default()
    })
}

fn save_column_node() -> Node {
    Node {
        flex_grow: 1.0,
        flex_shrink: 1.0,
        flex_basis: Val::Px(0.0),
        min_width: Val::Px(0.0),
        flex_direction: FlexDirection::Column,
        row_gap: Val::Px(6.0),
        ..default()
    }
}

fn spawn_save_column(
    columns: &mut ChildSpawnerCommands,
    create: SaveListAction,
    marker: impl Component + Copy,
) {
    columns
        .spawn((transparent_node(save_column_node()), marker))
        .with_children(|column| {
            spawn_save_slot_button(column, create);
        });
}

fn spawn_save_slot_button(parent: &mut ChildSpawnerCommands, action: SaveListAction) {
    parent
        .spawn((full_width_button(34.0), action))
        .with_children(|button| {
            button.spawn(text("", 15.0, Color::WHITE));
        });
}

pub fn spawn_save_management_row(
    parent: &mut ChildSpawnerCommands,
    load: SaveListAction,
    rename: SaveListAction,
    delete: SaveListAction,
) {
    parent.spawn(flex_row(32.0, 6.0)).with_children(|row| {
        spawn_save_row_button(row, load, SaveRowButtonLayout::Flex);
        spawn_save_row_button(row, rename, SaveRowButtonLayout::Fixed(SAVE_LIST_ACTION_BUTTON_WIDTH));
        spawn_save_row_button(row, delete, SaveRowButtonLayout::Fixed(SAVE_LIST_ACTION_BUTTON_WIDTH));
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
        TextLayout::new_with_no_wrap(),
        Node {
            max_width: Val::Percent(100.0),
            overflow: Overflow::clip(),
            ..default()
        },
    )
}
