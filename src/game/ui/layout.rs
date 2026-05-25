use bevy::prelude::*;

use crate::shared::config::ConfigAction;
use crate::shared::i18n::I18n;
use crate::shared::save::SAVE_SLOTS;

use super::components::{flex_row, localized_text, root_node, text, transparent_node};
use super::theme::{absolute_text_bundle, panel_bundle, STATUS_TEXT};
use super::types::{
    BackpackPanel, CarriedLabel, Crosshair, CurrentSaveText, FovText, HotbarText, InventoryTitle,
    MainMenuAction, MainMenuPanel, PauseAction, PausePanel, SaveListPanel, SaveListTitle,
    SettingsAction, SettingsGameplayGroup, SettingsKeyBindingsGroup, SettingsPanel,
    SettingsStatusText, SimulationAction, SimulationText, SlotArea, BACKPACK_SLOTS, HOTBAR_SLOTS,
};
use super::widgets::{
    spawn_language_settings_button, spawn_localized_main_button, spawn_localized_pause_button,
    spawn_localized_settings_button, spawn_localized_sim_button, spawn_save_back_button,
    spawn_save_slot_button, spawn_slot,
};

pub fn setup_ui(mut commands: Commands, i18n: Res<I18n>) {
    commands.spawn(root_node()).with_children(|root| {
        spawn_status_overlays(root);
        spawn_simulation_buttons(root, &i18n);
        spawn_hotbar(root);
        spawn_inventory_panel(root, &i18n);
        spawn_pause_panel(root, &i18n);
        spawn_settings_panel(root, &i18n);
        spawn_main_menu(root, &i18n);
        spawn_save_list(root);
        spawn_carried_label(root);
    });
}

fn spawn_status_overlays(root: &mut ChildBuilder) {
    root.spawn((
        absolute_text_bundle(
            "+",
            30.0,
            Color::WHITE,
            Some(Val::Percent(50.0)),
            None,
            Some(Val::Percent(50.0)),
            None,
        ),
        Crosshair,
    ));
    root.spawn((
        absolute_text_bundle(
            "",
            16.0,
            Color::WHITE,
            Some(Val::Px(18.0)),
            None,
            None,
            Some(Val::Px(92.0)),
        ),
        HotbarText,
    ));
    root.spawn((
        absolute_text_bundle(
            "",
            15.0,
            STATUS_TEXT,
            Some(Val::Px(18.0)),
            None,
            Some(Val::Px(18.0)),
            None,
        ),
        CurrentSaveText,
    ));
    root.spawn((
        absolute_text_bundle(
            "",
            16.0,
            STATUS_TEXT,
            None,
            Some(Val::Px(18.0)),
            Some(Val::Px(118.0)),
            None,
        ),
        SimulationText,
    ));
}

fn spawn_simulation_buttons(root: &mut ChildBuilder, i18n: &I18n) {
    root.spawn(transparent_node(Style {
        width: Val::Px(260.0),
        height: Val::Px(38.0),
        position_type: PositionType::Absolute,
        right: Val::Px(18.0),
        top: Val::Px(182.0),
        display: Display::Flex,
        column_gap: Val::Px(6.0),
        ..default()
    }))
    .with_children(|bar| {
        spawn_localized_sim_button(
            bar,
            i18n.text("button.sim_play"),
            "button.sim_play",
            SimulationAction::ToggleRun,
        );
        spawn_localized_sim_button(
            bar,
            i18n.text("button.rollback"),
            "button.rollback",
            SimulationAction::Rollback,
        );
    });
}

fn spawn_hotbar(root: &mut ChildBuilder) {
    root.spawn(NodeBundle {
        style: Style {
            width: Val::Px(540.0),
            height: Val::Px(58.0),
            position_type: PositionType::Absolute,
            left: Val::Percent(50.0),
            bottom: Val::Px(22.0),
            margin: UiRect {
                left: Val::Px(-270.0),
                ..default()
            },
            display: Display::Flex,
            justify_content: JustifyContent::Center,
            column_gap: Val::Px(4.0),
            ..default()
        },
        background_color: Color::srgba(0.04, 0.04, 0.04, 0.38).into(),
        ..default()
    })
    .with_children(|bar| {
        for index in 0..HOTBAR_SLOTS {
            spawn_slot(bar, SlotArea::Hotbar, index);
        }
    });
}

fn spawn_inventory_panel(root: &mut ChildBuilder, i18n: &I18n) {
    root.spawn((inventory_panel_bundle(), BackpackPanel))
        .with_children(|panel| {
            panel.spawn((
                text("", 24.0, Color::srgb(0.94, 0.94, 0.92)),
                InventoryTitle,
            ));
            panel.spawn(inventory_grid_bundle()).with_children(|grid| {
                for index in 0..BACKPACK_SLOTS {
                    spawn_slot(grid, SlotArea::Backpack, index);
                }
            });
            panel.spawn(localized_text(
                i18n,
                "inventory.help",
                15.0,
                Color::srgb(0.78, 0.78, 0.76),
            ));
        });
}

fn spawn_pause_panel(root: &mut ChildBuilder, i18n: &I18n) {
    root.spawn((panel_bundle(380.0, 450.0, -190.0, -225.0), PausePanel))
        .with_children(|panel| {
            panel.spawn(localized_text(i18n, "state.paused", 30.0, Color::WHITE));
            for (key, action) in [
                ("button.resume", PauseAction::Resume),
                ("button.toggle_builder_mode", PauseAction::ToggleBuilderMode),
                ("button.save_world", PauseAction::SaveWorld),
                ("button.switch_save", PauseAction::OpenSaveList),
                ("button.settings", PauseAction::OpenSettings),
                ("button.back_to_main_menu", PauseAction::BackToMainMenu),
                ("button.quit_game", PauseAction::Quit),
            ] {
                spawn_localized_pause_button(panel, i18n.text(key), key, action);
            }
        });
}

fn spawn_settings_panel(root: &mut ChildBuilder, i18n: &I18n) {
    root.spawn((panel_bundle(760.0, 560.0, -380.0, -280.0), SettingsPanel))
        .with_children(|panel| {
            panel.spawn(localized_text(i18n, "settings.title", 30.0, Color::WHITE));
            spawn_settings_tabs(panel, i18n);
            panel.spawn((
                text("", 16.0, Color::srgb(0.84, 0.92, 1.0)),
                SettingsStatusText,
            ));
            spawn_gameplay_settings(panel, i18n);
            spawn_key_bindings(panel, i18n);
            spawn_settings_footer(panel, i18n);
        });
}

fn spawn_settings_tabs(panel: &mut ChildBuilder, i18n: &I18n) {
    panel.spawn(flex_row(40.0, 8.0)).with_children(|tabs| {
        spawn_localized_settings_button(
            tabs,
            i18n.text("button.gameplay"),
            "button.gameplay",
            SettingsAction::TabGameplay,
        );
        spawn_localized_settings_button(
            tabs,
            i18n.text("button.key_bindings"),
            "button.key_bindings",
            SettingsAction::TabKeyBindings,
        );
    });
}

fn spawn_gameplay_settings(panel: &mut ChildBuilder, i18n: &I18n) {
    panel
        .spawn(flex_row(40.0, 8.0))
        .insert(SettingsGameplayGroup)
        .with_children(|row| {
            spawn_localized_settings_button(
                row,
                i18n.text("button.fov_down"),
                "button.fov_down",
                SettingsAction::FovDown,
            );
            row.spawn((text("", 18.0, Color::WHITE), FovText));
            spawn_localized_settings_button(
                row,
                i18n.text("button.fov_up"),
                "button.fov_up",
                SettingsAction::FovUp,
            );
            spawn_language_settings_button(
                row,
                i18n.fmt(
                    "button.language",
                    &[("language", i18n.language().native_name().to_string())],
                ),
                SettingsAction::LanguageNext,
            );
        });
}

fn spawn_key_bindings(panel: &mut ChildBuilder, i18n: &I18n) {
    panel
        .spawn(key_bindings_grid_bundle())
        .insert(SettingsKeyBindingsGroup)
        .with_children(|grid| {
            for action in ConfigAction::ALL {
                spawn_localized_settings_button(
                    grid,
                    i18n.text(action.label_key()),
                    action.label_key(),
                    SettingsAction::Bind(action),
                );
            }
        });
}

fn spawn_settings_footer(panel: &mut ChildBuilder, i18n: &I18n) {
    panel.spawn(flex_row(42.0, 8.0)).with_children(|row| {
        for (key, action) in [
            ("button.reset_defaults", SettingsAction::ResetDefaults),
            ("button.open_config_folder", SettingsAction::OpenFolder),
            ("button.back", SettingsAction::Back),
        ] {
            spawn_localized_settings_button(row, i18n.text(key), key, action);
        }
    });
}

fn spawn_main_menu(root: &mut ChildBuilder, i18n: &I18n) {
    root.spawn((panel_bundle(360.0, 260.0, -180.0, -130.0), MainMenuPanel))
        .with_children(|panel| {
            panel.spawn(localized_text(i18n, "main.title", 30.0, Color::WHITE));
            for (key, action) in [
                ("button.create_new_world", MainMenuAction::NewWorld),
                ("button.load_save", MainMenuAction::OpenSaveList),
                ("button.quit_game", MainMenuAction::Quit),
            ] {
                spawn_localized_main_button(panel, i18n.text(key), key, action);
            }
        });
}

fn spawn_save_list(root: &mut ChildBuilder) {
    root.spawn((panel_bundle(460.0, 460.0, -230.0, -230.0), SaveListPanel))
        .with_children(|panel| {
            panel.spawn((text("", 26.0, Color::WHITE), SaveListTitle));
            for index in 0..SAVE_SLOTS {
                spawn_save_slot_button(panel, index);
            }
            spawn_save_back_button(panel);
        });
}

fn spawn_carried_label(root: &mut ChildBuilder) {
    root.spawn((
        TextBundle {
            text: Text::from_section(
                "",
                TextStyle {
                    font_size: 18.0,
                    color: Color::WHITE,
                    ..default()
                },
            ),
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                top: Val::Percent(50.0),
                margin: UiRect {
                    left: Val::Px(18.0),
                    top: Val::Px(18.0),
                    ..default()
                },
                ..default()
            },
            ..default()
        },
        CarriedLabel,
    ));
}

fn inventory_panel_bundle() -> NodeBundle {
    NodeBundle {
        style: Style {
            width: Val::Px(540.0),
            height: Val::Px(350.0),
            position_type: PositionType::Absolute,
            left: Val::Percent(50.0),
            top: Val::Percent(50.0),
            margin: UiRect {
                left: Val::Px(-270.0),
                top: Val::Px(-175.0),
                ..default()
            },
            padding: UiRect::all(Val::Px(18.0)),
            display: Display::None,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(12.0),
            ..default()
        },
        background_color: Color::srgba(0.12, 0.12, 0.13, 0.94).into(),
        ..default()
    }
}

fn inventory_grid_bundle() -> NodeBundle {
    transparent_node(Style {
        display: Display::Grid,
        grid_template_columns: RepeatedGridTrack::flex(9, 1.0),
        grid_template_rows: RepeatedGridTrack::flex(3, 1.0),
        row_gap: Val::Px(4.0),
        column_gap: Val::Px(4.0),
        width: Val::Px(504.0),
        height: Val::Px(164.0),
        ..default()
    })
}

fn key_bindings_grid_bundle() -> NodeBundle {
    transparent_node(Style {
        display: Display::Grid,
        grid_template_columns: RepeatedGridTrack::flex(2, 1.0),
        grid_template_rows: RepeatedGridTrack::flex(6, 1.0),
        width: Val::Percent(100.0),
        height: Val::Px(300.0),
        row_gap: Val::Px(6.0),
        column_gap: Val::Px(8.0),
        ..default()
    })
}
