use bevy::prelude::*;

use crate::game::state::GameMode;
use crate::shared::i18n::I18n;

use crate::game::ui::components::{
    default_button_size, flex_row, full_width_button, panel_bundle, panel_content, panel_title_bar,
    panel_title_button, panel_title_label, raised_border, styled_button, text, transparent_node,
    BUTTON_BG,
};
use crate::game::ui::types::{
    PanelText, PanelTextKind, PanelVisibility, SaveListAction, SaveListCloseButton, SaveListPanel,
    SaveListPrompt, SaveListPuzzleColumn, SaveListSolutionColumn, TextPromptAction, TextPromptRoot,
    TextPromptText,
};

mod actions;
mod systems;

pub(crate) use actions::{save_list_actions, text_prompt_actions, text_prompt_input};
pub use systems::{update_save_list_ui, update_text_prompt_ui};

pub fn spawn_save_list(root: &mut ChildSpawnerCommands, i18n: &I18n) {
    root.spawn((
        panel_bundle(900.0),
        GlobalZIndex(0),
        PanelVisibility::GameMode(GameMode::SaveListMain),
        SaveListPanel,
    ))
    .with_children(|panel| {
        panel.spawn(panel_title_bar()).with_children(|title| {
            title.spawn((
                panel_title_label(i18n.text("save.title.default"), 26.0),
                PanelText(PanelTextKind::SaveListTitle),
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
    spawn_text_prompt(root);
}

fn save_columns_row() -> impl Bundle {
    transparent_node(Node {
        width: Val::Auto,
        display: Display::Flex,
        align_items: AlignItems::FlexStart,
        column_gap: Val::Px(12.0),
        ..default()
    })
}

fn spawn_save_column(
    columns: &mut ChildSpawnerCommands,
    create: SaveListAction,
    marker: impl Component + Copy,
) {
    columns
        .spawn((
            transparent_node(Node {
                width: Val::Px(SAVE_LIST_EDIT_COLUMN_WIDTH),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.0),
                ..default()
            }),
            marker,
        ))
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
    width: f32,
) {
    let side_width = 82.0;
    let load_width = (width - side_width * 2.0 - 12.0).max(180.0);
    parent.spawn(flex_row(32.0, 6.0)).with_children(|row| {
        spawn_save_row_button(row, load, load_width);
        spawn_save_row_button(row, rename, side_width);
        spawn_save_row_button(row, delete, side_width);
    });
}

pub fn spawn_save_select_row(parent: &mut ChildSpawnerCommands, load: SaveListAction) {
    parent
        .spawn(full_width_button(32.0))
        .insert(load)
        .with_children(|button| {
            button.spawn(text("", 13.0, Color::WHITE));
        });
}

fn spawn_save_row_button(parent: &mut ChildSpawnerCommands, action: SaveListAction, width: f32) {
    parent
        .spawn((
            styled_button(
                Node {
                    width: Val::Px(default_button_size(width)),
                    min_width: Val::Px(default_button_size(width)),
                    height: Val::Px(default_button_size(30.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                raised_border(),
                BUTTON_BG,
            ),
            action,
        ))
        .with_children(|button| {
            button.spawn(text("", 13.0, Color::WHITE));
        });
}

fn spawn_text_prompt(root: &mut ChildSpawnerCommands) {
    root.spawn((panel_bundle(420.0), GlobalZIndex(30_000), TextPromptRoot))
        .with_children(|panel| {
            panel.spawn(panel_title_bar()).with_children(|title| {
                title.spawn((panel_title_label("", 20.0), TextPromptText::Title));
            });
            panel.spawn(panel_content()).with_children(|content| {
                content
                    .spawn(styled_button(
                        Node {
                            width: Val::Percent(100.0),
                            min_height: Val::Px(default_button_size(38.0)),
                            padding: UiRect::horizontal(Val::Px(12.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        raised_border(),
                        BUTTON_BG,
                    ))
                    .with_children(|input| {
                        input.spawn((text("", 16.0, Color::WHITE), TextPromptText::Value));
                    });
                content.spawn(flex_row(36.0, 8.0)).with_children(|row| {
                    spawn_prompt_button(row, TextPromptAction::Confirm);
                    spawn_prompt_button(row, TextPromptAction::Cancel);
                });
            });
        });
}

fn spawn_prompt_button(parent: &mut ChildSpawnerCommands, action: TextPromptAction) {
    parent
        .spawn((full_width_button(34.0), action))
        .with_children(|button| {
            button.spawn(text("", 15.0, Color::WHITE));
        });
}

const SAVE_LIST_EDIT_COLUMN_WIDTH: f32 = 466.0;
