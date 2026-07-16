use bevy::prelude::*;

use crate::shared::config::{ActionKeyName, ConfigSelectionMode};

use super::super::components::{
    default_button_size, flex_row, localized_text, scroll_container, scroll_content, spawn_panel,
    transparent_node, PanelOptions, ScrollContent,
};
use super::super::types::{
    PanelVisibility, SettingsAction, SettingsControl, SettingsDropdown, SettingsDropdownRow,
    SettingsItem, SettingsTab, UiPanelBinding, GAMEPLAY_SETTINGS, GRAPHICS_SETTINGS,
};
use super::super::widgets::{
    spawn_localized_settings_button, spawn_settings_dropdown, spawn_settings_dropdown_list,
    spawn_settings_slider, spawn_settings_slider_value, spawn_settings_tab,
};
use crate::game::state::{GameSettings, UiPanelId};
use crate::game::ui::access::i18n;

pub fn spawn_settings_panel(root: &mut ChildSpawnerCommands, settings: &GameSettings) {
    spawn_panel(
        root,
        PanelOptions::new(840.0, "settings.title")
            .closable()
            .title_size(30.0),
        UiPanelBinding(UiPanelId::Settings),
        |panel| {
            spawn_settings_tabs(panel);
            spawn_gameplay_settings(panel, settings);
            spawn_graphics_settings(panel, settings);
            spawn_key_bindings(panel);
        },
    );
    spawn_settings_dropdown_layers(root);
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
            spawn_settings_tab(tabs, SettingsAction::TabGraphics);
            spawn_settings_tab(tabs, SettingsAction::TabKeyBindings);
        });
}

fn spawn_settings_dropdown_row(
    panel: &mut ChildSpawnerCommands,
    label_key: &'static str,
    dropdown: SettingsDropdown,
    tab: SettingsTab,
) {
    panel
        .spawn((
            settings_row_node(),
            PanelVisibility::SettingsTab(tab),
            SettingsDropdownRow(dropdown),
            ZIndex(300),
        ))
        .with_children(|row| {
            spawn_settings_label(row, label_key);
            row.spawn(transparent_node({
                let mut cell = settings_cell(530.0);
                cell.flex_direction = FlexDirection::Column;
                cell.justify_content = JustifyContent::Center;
                cell
            }))
            .with_children(|controls| {
                spawn_settings_dropdown(controls, dropdown);
            });
        });
}

fn spawn_settings_dropdown_layers(root: &mut ChildSpawnerCommands) {
    for dropdown in [
        SettingsDropdown::Language,
        SettingsDropdown::PlaceSelectionMode,
        SettingsDropdown::DeleteSelectionMode,
        SettingsDropdown::Shadows,
        SettingsDropdown::Vsync,
        SettingsDropdown::Skybox,
    ] {
        spawn_settings_dropdown_list(root, dropdown, settings_dropdown_options(dropdown));
    }
}

fn spawn_settings_slider_row(
    panel: &mut ChildSpawnerCommands,
    label_key: &'static str,
    item: SettingsItem,
    settings: &GameSettings,
    tab: SettingsTab,
) {
    panel
        .spawn(settings_row_node())
        .insert(PanelVisibility::SettingsTab(tab))
        .with_children(|row| {
            spawn_settings_label(row, label_key);
            row.spawn(transparent_node({
                let mut cell = settings_cell(360.0);
                cell.justify_content = JustifyContent::Center;
                cell
            }))
            .with_children(|controls| {
                if let SettingsControl::Slider { field, .. } = item.control {
                    spawn_settings_slider(controls, field, settings);
                }
            });
            if let SettingsControl::Slider { field, .. } = item.control {
                spawn_settings_slider_value(row, field);
            }
        });
}

fn settings_row_height() -> f32 {
    default_button_size(40.0)
}

fn settings_row_node() -> impl Bundle {
    transparent_node(Node {
        width: Val::Percent(100.0),
        height: Val::Px(settings_row_height()),
        display: Display::Flex,
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        column_gap: Val::Px(18.0),
        flex_shrink: 0.0,
        ..default()
    })
}

fn settings_cell(width: f32) -> Node {
    Node {
        width: Val::Px(width),
        min_width: Val::Px(width),
        height: Val::Percent(100.0),
        flex_shrink: 0.0,
        display: Display::Flex,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::FlexStart,
        ..default()
    }
}

fn spawn_settings_label(row: &mut ChildSpawnerCommands, label_key: &'static str) {
    row.spawn(transparent_node(settings_cell(220.0)))
        .with_children(|cell| {
            cell.spawn(localized_text(
                label_key,
                15.0,
                Color::srgb(0.82, 0.88, 0.90),
            ));
        });
}

fn spawn_settings_item(
    panel: &mut ChildSpawnerCommands,
    item: SettingsItem,
    settings: &GameSettings,
    tab: SettingsTab,
) {
    match item.control {
        SettingsControl::Slider { .. } => {
            spawn_settings_slider_row(panel, item.label_key, item, settings, tab)
        }
        SettingsControl::Dropdown(dropdown) => {
            spawn_settings_dropdown_row(panel, item.label_key, dropdown, tab)
        }
    }
}

fn spawn_gameplay_settings(panel: &mut ChildSpawnerCommands, settings: &GameSettings) {
    panel
        .spawn(scroll_container(500.0))
        .insert(PanelVisibility::SettingsTab(SettingsTab::Gameplay))
        .with_children(|container| {
            container
                .spawn((
                    ScrollContent,
                    Node {
                        width: Val::Percent(100.0),
                        position_type: PositionType::Absolute,
                        left: Val::Px(0.0),
                        top: Val::Px(0.0),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(8.0),
                        flex_shrink: 0.0,
                        overflow: Overflow::clip(),
                        ..default()
                    },
                    BackgroundColor(Color::NONE),
                ))
                .with_children(|content| {
                    for item in GAMEPLAY_SETTINGS {
                        spawn_settings_item(content, *item, settings, SettingsTab::Gameplay);
                    }
                    spawn_settings_footer(content);
                });
        });
}

fn spawn_graphics_settings(panel: &mut ChildSpawnerCommands, settings: &GameSettings) {
    panel
        .spawn(scroll_container(200.0))
        .insert(PanelVisibility::SettingsTab(SettingsTab::Graphics))
        .with_children(|container| {
            container.spawn(scroll_content()).with_children(|content| {
                for item in GRAPHICS_SETTINGS {
                    spawn_settings_item(content, *item, settings, SettingsTab::Graphics);
                }
            });
        });
}

fn spawn_key_bindings(panel: &mut ChildSpawnerCommands) {
    panel
        .spawn(scroll_container(360.0))
        .insert(PanelVisibility::SettingsTab(SettingsTab::KeyBindings))
        .with_children(|container| {
            container.spawn(scroll_content()).with_children(|content| {
                content
                    .spawn(key_bindings_columns_bundle())
                    .with_children(|columns| {
                        spawn_key_group(columns, "settings.group.general", &ActionKeyName::GENERAL);
                        spawn_key_group(
                            columns,
                            "settings.group.simulation",
                            &ActionKeyName::SIMULATION,
                        );
                        spawn_key_group(columns, "settings.group.mouse", &ActionKeyName::MOUSE);
                    });
            });
        });
}

fn spawn_key_group(
    columns: &mut ChildSpawnerCommands,
    label_key: &'static str,
    actions: &[ActionKeyName],
) {
    columns
        .spawn(transparent_node(Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(12.0),
            flex_grow: 1.0,
            flex_basis: Val::Px(0.0),
            ..default()
        }))
        .with_children(|group| {
            group.spawn(localized_text(label_key, 18.0, Color::WHITE));
            for action in actions {
                spawn_localized_settings_button(group, SettingsAction::Bind(*action));
            }
        });
}

fn spawn_settings_footer(panel: &mut ChildSpawnerCommands) {
    panel.spawn(flex_row(42.0, 8.0)).with_children(|row| {
        let mut actions = Vec::new();
        #[cfg(not(target_arch = "wasm32"))]
        actions.push(SettingsAction::StartDebugHttp);
        actions.push(SettingsAction::ResetDefaults);
        if crate::shared::platform::StoragePlatform::current()
            == crate::shared::platform::StoragePlatform::Desktop
        {
            actions.push(SettingsAction::OpenFolder);
        }
        actions.push(SettingsAction::Back);
        for action in actions {
            spawn_localized_settings_button(row, action);
        }
    });
}

fn key_bindings_columns_bundle() -> impl Bundle {
    transparent_node(Node {
        width: Val::Percent(100.0),
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::FlexStart,
        column_gap: Val::Px(14.0),
        flex_shrink: 0.0,
        ..default()
    })
}

fn settings_dropdown_options(dropdown: SettingsDropdown) -> Vec<(String, SettingsAction)> {
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
                    i18n.t(mode.label_key()),
                    SettingsAction::SetPlaceSelectionMode(mode),
                )
            })
            .collect(),
        SettingsDropdown::DeleteSelectionMode => ConfigSelectionMode::ALL
            .into_iter()
            .map(|mode| {
                (
                    i18n.t(mode.label_key()),
                    SettingsAction::SetDeleteSelectionMode(mode),
                )
            })
            .collect(),
        SettingsDropdown::Shadows => vec![
            (
                i18n.t("settings.option_on"),
                SettingsAction::SetShadowsEnabled(true),
            ),
            (
                i18n.t("settings.option_off"),
                SettingsAction::SetShadowsEnabled(false),
            ),
        ],
        SettingsDropdown::Vsync => vec![
            (
                i18n.t("settings.option_on"),
                SettingsAction::SetVsyncEnabled(true),
            ),
            (
                i18n.t("settings.option_off"),
                SettingsAction::SetVsyncEnabled(false),
            ),
        ],
        SettingsDropdown::Skybox => vec![
            (
                i18n.t("settings.option_on"),
                SettingsAction::SetSkyboxEnabled(true),
            ),
            (
                i18n.t("settings.option_off"),
                SettingsAction::SetSkyboxEnabled(false),
            ),
        ],
    }
}
