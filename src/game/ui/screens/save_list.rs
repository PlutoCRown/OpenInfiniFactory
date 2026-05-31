use bevy::prelude::*;

use crate::game::state::GameMode;
use crate::shared::i18n::I18n;
use crate::shared::save::SAVE_SLOTS;

use super::super::components::{
    default_button_size, flex_row, full_width_button, panel_bundle, panel_content, panel_title_bar,
    panel_title_button, panel_title_label, raised_border, styled_button, text,
    transparent_node, BUTTON_BG,
};
use super::super::types::{PanelText, PanelTextKind, PanelVisibility, SaveListAction, SaveListRow};

pub fn spawn_save_list(root: &mut ChildSpawnerCommands, i18n: &I18n) {
    root.spawn((
        panel_bundle(900.0),
        GlobalZIndex(0),
        PanelVisibility::GameMode(GameMode::SaveListMain),
    ))
    .with_children(|panel| {
        panel.spawn(panel_title_bar()).with_children(|title| {
            title.spawn((
                panel_title_label(i18n.text("save.title.default"), 26.0),
                PanelText(PanelTextKind::SaveListTitle),
            ));
            title
                .spawn((panel_title_button(), SaveListAction::Back))
                .with_children(|button| {
                    button.spawn(text("x", 12.0, Color::WHITE));
                });
        });
        panel.spawn(panel_content()).with_children(|panel| {
            panel.spawn(flex_row(470.0, 12.0)).with_children(|columns| {
                spawn_save_column(
                    columns,
                    SaveListRow::Puzzle,
                    SaveListAction::LoadPuzzle,
                    SaveListAction::DeletePuzzle,
                    SaveListAction::NewPuzzle,
                );
                spawn_save_column(
                    columns,
                    SaveListRow::Solution,
                    SaveListAction::LoadSolution,
                    SaveListAction::DeleteSolution,
                    SaveListAction::NewSolution,
                );
            });
        });
    });
}

fn spawn_save_column(
    columns: &mut ChildSpawnerCommands,
    row: fn(usize) -> SaveListRow,
    load: fn(usize) -> SaveListAction,
    delete: fn(usize) -> SaveListAction,
    create: SaveListAction,
) {
    columns
        .spawn(transparent_node(Node {
            width: Val::Px(420.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(6.0),
            ..default()
        }))
        .with_children(|column| {
            for index in 0..SAVE_SLOTS {
                column
                    .spawn((flex_row(32.0, 6.0), row(index)))
                    .with_children(|row| {
                        spawn_save_row_button(row, load(index), 260.0);
                        spawn_save_row_button(row, delete(index), 80.0);
                    });
            }
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

fn spawn_save_row_button(
    parent: &mut ChildSpawnerCommands,
    action: SaveListAction,
    width: f32,
) {
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
