use bevy::prelude::*;

use crate::shared::config::{ConfigAction, ConfigSelectionMode, GameConfig};
use crate::shared::i18n::I18n;

use crate::game::ui::components::{
    default_button_size, flex_row, localized_text, scroll_container, scroll_content, spawn_panel,
    transparent_node, PanelOptions,
};
use crate::game::ui::types::{
    ButtonSpec, PanelVisibility, SettingsAction, SettingsControl, SettingsDropdownRow,
    SettingsDropdownSpec, SettingsDropdownValue, SettingsItem, SettingsTab, UiPanelBinding,
    UiPanelId, GAMEPLAY_SETTINGS,
};

mod actions;
mod systems;
mod widgets;

pub(crate) use actions::{settings_action_clicked, settings_menu_actions};
pub(crate) use systems::{
    update_settings_dropdowns_ui, update_settings_slider_drag_ui, update_settings_sliders_ui,
    update_settings_tabs_ui, update_settings_text_ui,
};

use widgets::{
    spawn_localized_settings_button, spawn_settings_dropdown, spawn_settings_dropdown_list,
    spawn_settings_slider, spawn_settings_slider_value, spawn_settings_tab,
};

pub fn spawn_settings_panel(root: &mut ChildSpawnerCommands, i18n: &I18n) {
    spawn_panel(
        root,
        i18n,
        PanelOptions::new(840.0, "settings.title")
            .closable()
            .title_size(30.0),
        UiPanelBinding(UiPanelId::Settings),
        |panel| {
            spawn_settings_tabs(panel);
            spawn_gameplay_settings(panel, i18n);
            spawn_key_bindings(panel, i18n);
        },
    );
    spawn_settings_dropdown_layers(root, i18n);
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
            spawn_settings_tab(tabs, SettingsAction::TabGameplay, "button.gameplay");
            spawn_settings_tab(tabs, SettingsAction::TabKeyBindings, "button.key_bindings");
        });
}

fn spawn_settings_dropdown_row(
    panel: &mut ChildSpawnerCommands,
    i18n: &I18n,
    text_key: &'static str,
    dropdown: SettingsDropdownSpec,
) {
    panel
        .spawn((
            settings_row_node(),
            PanelVisibility::SettingsTab(SettingsTab::Gameplay),
            SettingsDropdownRow(dropdown.id),
            ZIndex(300),
        ))
        .with_children(|row| {
            spawn_settings_label(row, i18n, text_key);
            row.spawn(transparent_node(Node {
                width: Val::Px(530.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                justify_content: JustifyContent::Center,
                ..default()
            }))
            .with_children(|controls| {
                spawn_settings_dropdown(controls, dropdown.id);
            });
        });
}

fn spawn_settings_dropdown_layers(root: &mut ChildSpawnerCommands, i18n: &I18n) {
    spawn_settings_dropdown_list(root, SettingsDropdownSpec::LANGUAGE.id, language_options());
    spawn_settings_dropdown_list(
        root,
        SettingsDropdownSpec::PLACE_SELECTION_MODE.id,
        place_selection_options(i18n),
    );
    spawn_settings_dropdown_list(
        root,
        SettingsDropdownSpec::DELETE_SELECTION_MODE.id,
        delete_selection_options(i18n),
    );
}

fn spawn_settings_slider_row(
    panel: &mut ChildSpawnerCommands,
    i18n: &I18n,
    text_key: &'static str,
    item: SettingsItem,
) {
    panel
        .spawn(settings_row_node())
        .insert(PanelVisibility::SettingsTab(SettingsTab::Gameplay))
        .with_children(|row| {
            spawn_settings_label(row, i18n, text_key);
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

fn spawn_settings_label(row: &mut ChildSpawnerCommands, i18n: &I18n, text_key: &'static str) {
    row.spawn((
        localized_text(i18n, text_key, 15.0, Color::srgb(0.82, 0.88, 0.90)),
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
            spawn_settings_slider_row(panel, i18n, item.text_key, item)
        }
        SettingsControl::Dropdown(dropdown) => {
            spawn_settings_dropdown_row(panel, i18n, item.text_key, dropdown)
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
                .spawn((key_bindings_columns_bundle(), scroll_content()))
                .with_children(|columns| {
                    spawn_key_group(
                        columns,
                        i18n,
                        "settings.group.general",
                        GENERAL_KEY_BINDINGS,
                    );
                    spawn_key_group(
                        columns,
                        i18n,
                        "settings.group.simulation",
                        SIMULATION_KEY_BINDINGS,
                    );
                });
        });
}

fn spawn_key_group(
    columns: &mut ChildSpawnerCommands,
    i18n: &I18n,
    text_key: &'static str,
    bindings: &[ButtonSpec<ConfigAction>],
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
            group.spawn(localized_text(i18n, text_key, 18.0, Color::WHITE));
            for binding in bindings {
                spawn_localized_settings_button(
                    group,
                    SettingsAction::Bind(binding.on_click),
                    binding.text,
                );
            }
        });
}

fn spawn_settings_footer(panel: &mut ChildSpawnerCommands) {
    panel.spawn(flex_row(42.0, 8.0)).with_children(|row| {
        for button in SETTINGS_FOOTER {
            spawn_localized_settings_button(row, button.on_click, button.text);
        }
    });
}

fn key_bindings_columns_bundle() -> impl Bundle {
    transparent_node(Node {
        position_type: PositionType::Absolute,
        width: Val::Percent(100.0),
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::FlexStart,
        column_gap: Val::Px(14.0),
        ..default()
    })
}

fn language_options() -> Vec<(String, SettingsAction)> {
    crate::shared::i18n::Language::ALL
        .into_iter()
        .map(|language| {
            (
                language.native_name().to_string(),
                SettingsAction::SetLanguage(language),
            )
        })
        .collect()
}

fn place_selection_options(i18n: &I18n) -> Vec<(String, SettingsAction)> {
    ConfigSelectionMode::ALL
        .into_iter()
        .map(|mode| {
            (
                i18n.text(selection_mode_text_key(mode)),
                SettingsAction::SetPlaceSelectionMode(mode),
            )
        })
        .collect()
}

fn delete_selection_options(i18n: &I18n) -> Vec<(String, SettingsAction)> {
    ConfigSelectionMode::ALL
        .into_iter()
        .map(|mode| {
            (
                i18n.text(selection_mode_text_key(mode)),
                SettingsAction::SetDeleteSelectionMode(mode),
            )
        })
        .collect()
}

const GENERAL_KEY_BINDINGS: &[ButtonSpec<ConfigAction>] = &[
    ButtonSpec::new("action.pause", ConfigAction::Pause),
    ButtonSpec::new("action.inventory", ConfigAction::Inventory),
    ButtonSpec::new("action.alternate", ConfigAction::Alternate),
    ButtonSpec::new("action.rotate", ConfigAction::RotateOrRollback),
    ButtonSpec::new("action.debug", ConfigAction::Debug),
    ButtonSpec::new("action.forward", ConfigAction::Forward),
    ButtonSpec::new("action.backward", ConfigAction::Backward),
    ButtonSpec::new("action.left", ConfigAction::Left),
    ButtonSpec::new("action.right", ConfigAction::Right),
    ButtonSpec::new("action.jump_or_fly_up", ConfigAction::JumpOrFlyUp),
    ButtonSpec::new("action.fly_down", ConfigAction::FlyDown),
    ButtonSpec::new("action.place", ConfigAction::Place),
    ButtonSpec::new("action.delete", ConfigAction::Delete),
    ButtonSpec::new("action.pick", ConfigAction::Pick),
];

const SIMULATION_KEY_BINDINGS: &[ButtonSpec<ConfigAction>] = &[
    ButtonSpec::new("action.simulation_start", ConfigAction::Simulate),
    ButtonSpec::new("action.simulation_step", ConfigAction::SimulationStep),
    ButtonSpec::new("action.simulation_fast", ConfigAction::SimulationFast),
    ButtonSpec::new(
        "action.simulation_rollback",
        ConfigAction::SimulationRollback,
    ),
];

const SETTINGS_FOOTER: &[ButtonSpec<SettingsAction>] = &[
    ButtonSpec::new("button.reset_defaults", SettingsAction::ResetDefaults),
    ButtonSpec::new("button.open_config_folder", SettingsAction::OpenFolder),
    ButtonSpec::new("button.back", SettingsAction::Back),
];

pub(crate) fn config_action_text_key(action: ConfigAction) -> &'static str {
    GENERAL_KEY_BINDINGS
        .iter()
        .chain(SIMULATION_KEY_BINDINGS)
        .find_map(|binding| (binding.on_click == action).then_some(binding.text))
        .unwrap_or("")
}

pub(crate) fn selection_mode_text_key(mode: ConfigSelectionMode) -> &'static str {
    match mode {
        ConfigSelectionMode::Point => "selection_mode.point",
        ConfigSelectionMode::Line => "selection_mode.line",
        ConfigSelectionMode::Plane => "selection_mode.plane",
    }
}

pub(crate) fn settings_dropdown_value_text(
    dropdown: SettingsDropdownSpec,
    config: &GameConfig,
    i18n: &I18n,
) -> String {
    match dropdown.value {
        SettingsDropdownValue::Language => i18n.language().native_name().to_string(),
        SettingsDropdownValue::PlaceSelectionMode => {
            i18n.text(selection_mode_text_key(config.place_selection_mode))
        }
        SettingsDropdownValue::DeleteSelectionMode => {
            i18n.text(selection_mode_text_key(config.delete_selection_mode))
        }
    }
}

pub(crate) fn settings_dropdown_spec_by_id(
    id: crate::game::ui::types::SettingsDropdownId,
) -> Option<SettingsDropdownSpec> {
    [
        SettingsDropdownSpec::LANGUAGE,
        SettingsDropdownSpec::PLACE_SELECTION_MODE,
        SettingsDropdownSpec::DELETE_SELECTION_MODE,
    ]
    .into_iter()
    .find(|spec| spec.id == id)
}
