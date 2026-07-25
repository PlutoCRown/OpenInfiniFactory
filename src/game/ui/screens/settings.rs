use bevy::prelude::*;

use crate::shared::config::{ActionKeyName, ConfigSelectionMode, ConfigWindowMode};
use crate::shared::platform::StoragePlatform;

use super::super::components::{
    PanelOptions, default_button_size, flex_row, localized_text, scroll_container, scroll_content,
    spawn_panel, transparent_node,
};
use super::super::types::{
    GAMEPLAY_SETTINGS, GRAPHICS_SETTINGS, PanelVisibility, SettingsAction, SettingsControl,
    SettingsDropdown, SettingsDropdownRow, SettingsItem, SettingsTab, UiPanelBinding,
};
use super::super::widgets::{
    spawn_localized_settings_button, spawn_settings_dropdown, spawn_settings_dropdown_list,
    spawn_settings_radio_group, spawn_settings_slider, spawn_settings_slider_value,
    spawn_settings_tab,
};
use crate::game::state::{GameSettings, UiPanelId};
use crate::game::ui::access::i18n;

/// 相对窗口逻辑像素的外边距（与存档列表一致）
pub const SETTINGS_MARGIN: f32 = 48.0;

/// 按窗口逻辑尺寸算出设置弹窗宽高（仅初始化用；不超过可用区域）
pub fn settings_panel_size(window_w: f32, window_h: f32, ui_scale: f32) -> (f32, f32) {
    let scale = ui_scale.max(0.01);
    (
        (window_w / scale - SETTINGS_MARGIN * 2.0).max(1.0),
        (window_h / scale - SETTINGS_MARGIN * 2.0).max(1.0),
    )
}

pub fn spawn_settings_panel(
    root: &mut ChildSpawnerCommands,
    settings: &GameSettings,
    panel_w: f32,
    panel_h: f32,
) {
    spawn_panel(
        root,
        PanelOptions::new(panel_w, "settings.title")
            .with_height(panel_h)
            .closable(),
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
            flex_shrink: 0.0,
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
                let mut cell = settings_control_cell();
                cell.flex_direction = FlexDirection::Column;
                cell.justify_content = JustifyContent::Center;
                cell
            }))
            .with_children(|controls| {
                spawn_settings_dropdown(controls, dropdown);
            });
        });
}

fn spawn_settings_radio_row(
    panel: &mut ChildSpawnerCommands,
    label_key: &'static str,
    group: SettingsDropdown,
    tab: SettingsTab,
) {
    panel
        .spawn((settings_row_node(), PanelVisibility::SettingsTab(tab)))
        .with_children(|row| {
            spawn_settings_label(row, label_key);
            row.spawn(transparent_node(settings_control_cell()))
                .with_children(|controls| {
                    spawn_settings_radio_group(controls, settings_choice_options(group));
                });
        });
}

fn spawn_settings_dropdown_layers(root: &mut ChildSpawnerCommands) {
    // 仅保留仍为下拉的项；单选组不再挂列表层
    let mut dropdowns = vec![SettingsDropdown::Language, SettingsDropdown::Ssao];
    if StoragePlatform::current() == StoragePlatform::Desktop {
        dropdowns.push(SettingsDropdown::WindowMode);
    }
    for dropdown in dropdowns {
        spawn_settings_dropdown_list(root, dropdown, settings_choice_options(dropdown));
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
                let mut cell = settings_control_cell();
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

fn settings_control_cell() -> Node {
    Node {
        width: Val::Percent(100.0),
        flex_grow: 1.0,
        flex_shrink: 1.0,
        min_width: Val::Px(0.0),
        height: Val::Percent(100.0),
        display: Display::Flex,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::FlexStart,
        ..default()
    }
}

fn settings_label_cell() -> Node {
    Node {
        width: Val::Px(220.0),
        min_width: Val::Px(220.0),
        height: Val::Percent(100.0),
        flex_shrink: 0.0,
        display: Display::Flex,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::FlexStart,
        ..default()
    }
}

fn spawn_settings_label(row: &mut ChildSpawnerCommands, label_key: &'static str) {
    row.spawn(transparent_node(settings_label_cell()))
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
        SettingsControl::Radio(group) => {
            spawn_settings_radio_row(panel, item.label_key, group, tab)
        }
    }
}

fn spawn_gameplay_settings(panel: &mut ChildSpawnerCommands, settings: &GameSettings) {
    panel
        .spawn(scroll_container())
        .insert(PanelVisibility::SettingsTab(SettingsTab::Gameplay))
        .with_children(|container| {
            container.spawn(scroll_content()).with_children(|content| {
                for item in GAMEPLAY_SETTINGS {
                    spawn_settings_item(content, *item, settings, SettingsTab::Gameplay);
                }
                spawn_settings_footer(content);
            });
        });
}

fn spawn_graphics_settings(panel: &mut ChildSpawnerCommands, settings: &GameSettings) {
    panel
        .spawn(scroll_container())
        .insert(PanelVisibility::SettingsTab(SettingsTab::Graphics))
        .with_children(|container| {
            container.spawn(scroll_content()).with_children(|content| {
                for item in GRAPHICS_SETTINGS {
                    if matches!(
                        item.control,
                        SettingsControl::Dropdown(SettingsDropdown::WindowMode)
                    ) && StoragePlatform::current() != StoragePlatform::Desktop
                    {
                        continue;
                    }
                    spawn_settings_item(content, *item, settings, SettingsTab::Graphics);
                }
            });
        });
}

fn spawn_key_bindings(panel: &mut ChildSpawnerCommands) {
    panel
        .spawn(scroll_container())
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
        if StoragePlatform::current() == StoragePlatform::Desktop {
            actions.push(SettingsAction::OpenFolder);
        }
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

fn settings_choice_options(dropdown: SettingsDropdown) -> Vec<(String, SettingsAction)> {
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
        SettingsDropdown::Ssao => crate::shared::config::ConfigSsaoQuality::ALL
            .into_iter()
            .map(|quality| {
                (
                    i18n.t(quality.label_key()),
                    SettingsAction::SetSsaoQuality(quality),
                )
            })
            .collect(),
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
        SettingsDropdown::WindowMode => ConfigWindowMode::ALL
            .into_iter()
            .map(|mode| {
                (
                    i18n.t(mode.label_key()),
                    SettingsAction::SetWindowMode(mode),
                )
            })
            .collect(),
    }
}
