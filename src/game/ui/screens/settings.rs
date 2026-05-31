use bevy::prelude::*;

use crate::shared::config::{ConfigAction, ConfigSelectionMode};
use crate::shared::i18n::I18n;

use super::super::components::{
    default_button_size, flex_row, localized_text, scroll_container, scroll_content, spawn_panel,
    transparent_node, PanelOptions,
};
use super::super::types::{
    PanelVisibility, SettingsAction, SettingsControl, SettingsDropdown, SettingsDropdownRow,
    SettingsItem, SettingsTab, UiPanelBinding, UiPanelId, GAMEPLAY_SETTINGS,
};
use super::super::widgets::{
    spawn_localized_settings_button, spawn_settings_dropdown, spawn_settings_slider,
    spawn_settings_slider_value, spawn_settings_tab,
};

pub fn spawn_settings_panel(root: &mut ChildSpawnerCommands, i18n: &I18n) {
    spawn_panel(
        root,
        i18n,
        PanelOptions::new(840.0, "settings.title").closable().title_size(30.0),
        UiPanelBinding(UiPanelId::Settings),
        |panel| {
            spawn_settings_tabs(panel);
            spawn_gameplay_settings(panel, i18n);
            spawn_key_bindings(panel, i18n);
        },
    );
}

fn spawn_settings_tabs(panel: &mut ChildSpawnerCommands) {
    panel
        .spawn(transparent_node(Node {
            width: Val::Percent(100.0),
            height: Val::Px(default_button_size(42.0)),
            display: Display::Flex,
            column_gap: Val::Px(6.0),
            ..default()
        }))
        .with_children(|tabs| {
            spawn_settings_tab(tabs, SettingsAction::TabGameplay);
            spawn_settings_tab(tabs, SettingsAction::TabKeyBindings);
        });
}

fn spawn_settings_dropdown_row(
    panel: &mut ChildSpawnerCommands,
    i18n: &I18n,
    label_key: &'static str,
    dropdown: SettingsDropdown,
) {
    panel
        .spawn((
            settings_row_node(),
            PanelVisibility::SettingsTab(SettingsTab::Gameplay),
            SettingsDropdownRow(dropdown),
            ZIndex(300),
        ))
        .with_children(|row| {
            spawn_settings_label(row, i18n, label_key);
            row.spawn(transparent_node(Node {
                width: Val::Px(530.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                justify_content: JustifyContent::Center,
                ..default()
            }))
            .with_children(|controls| {
                spawn_settings_dropdown(controls, dropdown, settings_dropdown_options(i18n, dropdown));
            });
        });
}

fn spawn_settings_slider_row(
    panel: &mut ChildSpawnerCommands,
    i18n: &I18n,
    label_key: &'static str,
    item: SettingsItem,
) {
    panel
        .spawn(settings_row_node())
        .insert(PanelVisibility::SettingsTab(SettingsTab::Gameplay))
        .with_children(|row| {
            spawn_settings_label(row, i18n, label_key);
            row.spawn(transparent_node(Node {
                width: Val::Px(360.0),
                justify_content: JustifyContent::Center,
                ..default()
            }))
            .with_children(|controls| {
                if let SettingsControl::Slider { field, .. } = item.control {
                    spawn_settings_slider(controls, field);
                }
            });
            if let SettingsControl::Slider { field, .. } = item.control {
                spawn_settings_slider_value(row, field);
            }
        });
}

fn settings_row_node() -> impl Bundle {
    transparent_node(Node {
        width: Val::Percent(100.0),
        min_height: Val::Px(default_button_size(54.0)),
        display: Display::Flex,
        align_items: AlignItems::Center,
        column_gap: Val::Px(18.0),
        ..default()
    })
}

fn spawn_settings_label(
    row: &mut ChildSpawnerCommands,
    i18n: &I18n,
    label_key: &'static str,
) {
    row.spawn((
        localized_text(i18n, label_key, 15.0, Color::srgb(0.82, 0.88, 0.90)),
        Node {
            width: Val::Px(220.0),
            align_self: AlignSelf::Center,
            ..default()
        },
    ));
}

fn spawn_settings_item(panel: &mut ChildSpawnerCommands, i18n: &I18n, item: SettingsItem) {
    match item.control {
        SettingsControl::Slider { .. } => {
            spawn_settings_slider_row(panel, i18n, item.label_key, item)
        }
        SettingsControl::Dropdown(dropdown) => {
            spawn_settings_dropdown_row(panel, i18n, item.label_key, dropdown)
        }
    }
}

fn spawn_gameplay_settings(panel: &mut ChildSpawnerCommands, i18n: &I18n) {
    panel
        .spawn(scroll_container(500.0))
        .insert(PanelVisibility::SettingsTab(SettingsTab::Gameplay))
        .with_children(|container| {
            container
                .spawn((
                    transparent_node(Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(8.0),
                        ..default()
                    }),
                    scroll_content(),
                ))
                .with_children(|content| {
                    for item in GAMEPLAY_SETTINGS {
                        spawn_settings_item(content, i18n, *item);
                    }
                    content.spawn(transparent_node(Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(120.0),
                        ..default()
                    }));
                    spawn_settings_footer(content);
                });
        });
}

fn spawn_key_bindings(panel: &mut ChildSpawnerCommands, i18n: &I18n) {
    panel
        .spawn(scroll_container(360.0))
        .insert(PanelVisibility::SettingsTab(SettingsTab::KeyBindings))
        .with_children(|container| {
            container
                .spawn((key_bindings_grid_bundle(), scroll_content()))
                .with_children(|grid| {
                    spawn_key_group(grid, i18n, "settings.group.general", &ConfigAction::GENERAL);
                    spawn_key_group(
                        grid,
                        i18n,
                        "settings.group.simulation",
                        &ConfigAction::SIMULATION,
                    );
                });
        });
}

fn spawn_key_group(
    grid: &mut ChildSpawnerCommands,
    i18n: &I18n,
    label_key: &'static str,
    actions: &[ConfigAction],
) {
    grid.spawn(localized_text(i18n, label_key, 18.0, Color::WHITE));
    grid.spawn(transparent_node(Node::default()));
    for action in actions {
        spawn_localized_settings_button(grid, SettingsAction::Bind(*action));
    }
}

fn spawn_settings_footer(panel: &mut ChildSpawnerCommands) {
    panel.spawn(flex_row(42.0, 8.0)).with_children(|row| {
        for action in [
            SettingsAction::ResetDefaults,
            SettingsAction::OpenFolder,
            SettingsAction::Back,
        ] {
            spawn_localized_settings_button(row, action);
        }
    });
}

fn key_bindings_grid_bundle() -> impl Bundle {
    transparent_node(Node {
        display: Display::Grid,
        grid_template_columns: RepeatedGridTrack::flex(2, 1.0),
        grid_template_rows: RepeatedGridTrack::flex(11, 1.0),
        position_type: PositionType::Absolute,
        width: Val::Percent(100.0),
        height: Val::Px(462.0),
        row_gap: Val::Px(6.0),
        column_gap: Val::Px(8.0),
        ..default()
    })
}

fn settings_dropdown_options(
    i18n: &I18n,
    dropdown: SettingsDropdown,
) -> Vec<(String, SettingsAction)> {
    match dropdown {
        SettingsDropdown::Language => crate::shared::i18n::Language::ALL
            .into_iter()
            .map(|language| {
                (
                    language.native_name().to_string(),
                    SettingsAction::SetLanguage(language),
                )
            })
            .collect(),
        SettingsDropdown::PlaceSelectionMode => ConfigSelectionMode::ALL
            .into_iter()
            .map(|mode| {
                (
                    i18n.text(mode.label_key()),
                    SettingsAction::SetPlaceSelectionMode(mode),
                )
            })
            .collect(),
        SettingsDropdown::DeleteSelectionMode => ConfigSelectionMode::ALL
            .into_iter()
            .map(|mode| {
                (
                    i18n.text(mode.label_key()),
                    SettingsAction::SetDeleteSelectionMode(mode),
                )
            })
            .collect(),
    }
}
