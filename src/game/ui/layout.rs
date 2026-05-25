use bevy::prelude::*;

use crate::shared::config::ConfigAction;
use crate::shared::save::SAVE_SLOTS;

use super::theme::{absolute_text_bundle, panel_bundle, row_bundle, text_section, STATUS_TEXT};
use super::types::{
    BackpackPanel, CarriedLabel, Crosshair, CurrentSaveText, FovText, HotbarText, InventoryTitle,
    MainMenuAction, MainMenuPanel, PauseAction, PausePanel, SaveListPanel, SaveListTitle,
    SettingsAction, SettingsGameplayGroup, SettingsKeyBindingsGroup, SettingsPanel,
    SettingsStatusText, SimulationAction, SimulationText, SlotArea, BACKPACK_SLOTS, HOTBAR_SLOTS,
};
use super::widgets::{
    spawn_main_button, spawn_pause_button, spawn_save_back_button, spawn_save_slot_button,
    spawn_settings_button, spawn_sim_button, spawn_slot,
};

pub fn setup_ui(mut commands: Commands) {
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                ..default()
            },
            background_color: Color::NONE.into(),
            ..default()
        })
        .with_children(|root| {
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

            root.spawn(NodeBundle {
                style: Style {
                    width: Val::Px(260.0),
                    height: Val::Px(38.0),
                    position_type: PositionType::Absolute,
                    right: Val::Px(18.0),
                    top: Val::Px(182.0),
                    display: Display::Flex,
                    column_gap: Val::Px(6.0),
                    ..default()
                },
                background_color: Color::NONE.into(),
                ..default()
            })
            .with_children(|bar| {
                spawn_sim_button(bar, "Play", SimulationAction::ToggleRun);
                spawn_sim_button(bar, "Rollback", SimulationAction::Rollback);
            });

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

            root.spawn((
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
                },
                BackpackPanel,
            ))
            .with_children(|panel| {
                panel.spawn((
                    TextBundle::from_section(
                        "",
                        TextStyle {
                            font_size: 24.0,
                            color: Color::srgb(0.94, 0.94, 0.92),
                            ..default()
                        },
                    ),
                    InventoryTitle,
                ));

                panel
                    .spawn(NodeBundle {
                        style: Style {
                            display: Display::Grid,
                            grid_template_columns: RepeatedGridTrack::flex(9, 1.0),
                            grid_template_rows: RepeatedGridTrack::flex(3, 1.0),
                            row_gap: Val::Px(4.0),
                            column_gap: Val::Px(4.0),
                            width: Val::Px(504.0),
                            height: Val::Px(164.0),
                            ..default()
                        },
                        background_color: Color::NONE.into(),
                        ..default()
                    })
                    .with_children(|grid| {
                        for index in 0..BACKPACK_SLOTS {
                            spawn_slot(grid, SlotArea::Backpack, index);
                        }
                    });

                panel.spawn(TextBundle::from_section(
                    "Click a slot to pick up or swap. Number keys select the hotbar.",
                    TextStyle {
                        font_size: 15.0,
                        color: Color::srgb(0.78, 0.78, 0.76),
                        ..default()
                    },
                ));
            });

            root.spawn((panel_bundle(380.0, 450.0, -190.0, -225.0), PausePanel))
                .with_children(|panel| {
                    panel.spawn(text_section("Paused", 30.0, Color::WHITE));
                    spawn_pause_button(panel, "Resume", PauseAction::Resume);
                    spawn_pause_button(
                        panel,
                        "Toggle Edit/Play Mode",
                        PauseAction::ToggleBuilderMode,
                    );
                    spawn_pause_button(panel, "Save World", PauseAction::SaveWorld);
                    spawn_pause_button(panel, "Switch Save", PauseAction::OpenSaveList);
                    spawn_pause_button(panel, "Settings", PauseAction::OpenSettings);
                    spawn_pause_button(panel, "Back to Main Menu", PauseAction::BackToMainMenu);
                    spawn_pause_button(panel, "Quit Game", PauseAction::Quit);
                });

            root.spawn((panel_bundle(760.0, 560.0, -380.0, -280.0), SettingsPanel))
                .with_children(|panel| {
                    panel.spawn(text_section("Settings", 30.0, Color::WHITE));

                    panel
                        .spawn(NodeBundle {
                            style: Style {
                                width: Val::Percent(100.0),
                                height: Val::Px(40.0),
                                display: Display::Flex,
                                column_gap: Val::Px(8.0),
                                ..default()
                            },
                            background_color: Color::NONE.into(),
                            ..default()
                        })
                        .with_children(|tabs| {
                            spawn_settings_button(tabs, "Gameplay", SettingsAction::TabGameplay);
                            spawn_settings_button(
                                tabs,
                                "Key Bindings",
                                SettingsAction::TabKeyBindings,
                            );
                        });

                    panel.spawn((
                        TextBundle::from_section(
                            "",
                            TextStyle {
                                font_size: 16.0,
                                color: Color::srgb(0.84, 0.92, 1.0),
                                ..default()
                            },
                        ),
                        SettingsStatusText,
                    ));

                    panel
                        .spawn(row_bundle(40.0))
                        .insert(SettingsGameplayGroup)
                        .with_children(|row| {
                            spawn_settings_button(row, "FOV -", SettingsAction::FovDown);
                            row.spawn((
                                TextBundle::from_section(
                                    "",
                                    TextStyle {
                                        font_size: 18.0,
                                        color: Color::WHITE,
                                        ..default()
                                    },
                                ),
                                FovText,
                            ));
                            spawn_settings_button(row, "FOV +", SettingsAction::FovUp);
                        });

                    panel
                        .spawn(NodeBundle {
                            style: Style {
                                display: Display::Grid,
                                grid_template_columns: RepeatedGridTrack::flex(2, 1.0),
                                grid_template_rows: RepeatedGridTrack::flex(6, 1.0),
                                width: Val::Percent(100.0),
                                height: Val::Px(300.0),
                                row_gap: Val::Px(6.0),
                                column_gap: Val::Px(8.0),
                                ..default()
                            },
                            background_color: Color::NONE.into(),
                            ..default()
                        })
                        .insert(SettingsKeyBindingsGroup)
                        .with_children(|grid| {
                            for action in ConfigAction::ALL {
                                spawn_settings_button(
                                    grid,
                                    action.label(),
                                    SettingsAction::Bind(action),
                                );
                            }
                        });

                    panel
                        .spawn(NodeBundle {
                            style: Style {
                                width: Val::Percent(100.0),
                                height: Val::Px(42.0),
                                display: Display::Flex,
                                column_gap: Val::Px(8.0),
                                ..default()
                            },
                            background_color: Color::NONE.into(),
                            ..default()
                        })
                        .with_children(|row| {
                            spawn_settings_button(
                                row,
                                "Reset Defaults",
                                SettingsAction::ResetDefaults,
                            );
                            spawn_settings_button(
                                row,
                                "Open Config Folder",
                                SettingsAction::OpenFolder,
                            );
                            spawn_settings_button(row, "Back", SettingsAction::Back);
                        });
                });

            root.spawn((panel_bundle(360.0, 260.0, -180.0, -130.0), MainMenuPanel))
                .with_children(|panel| {
                    panel.spawn(text_section("OpenInfiniFactory", 30.0, Color::WHITE));
                    spawn_main_button(panel, "Create New World", MainMenuAction::NewWorld);
                    spawn_main_button(panel, "Load Save", MainMenuAction::OpenSaveList);
                    spawn_main_button(panel, "Quit Game", MainMenuAction::Quit);
                });

            root.spawn((panel_bundle(460.0, 460.0, -230.0, -230.0), SaveListPanel))
                .with_children(|panel| {
                    panel.spawn((
                        TextBundle::from_section(
                            "",
                            TextStyle {
                                font_size: 26.0,
                                color: Color::WHITE,
                                ..default()
                            },
                        ),
                        SaveListTitle,
                    ));
                    for index in 0..SAVE_SLOTS {
                        spawn_save_slot_button(panel, index);
                    }
                    spawn_save_back_button(panel);
                });

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
        });
}
