use bevy::prelude::*;

use crate::shared::config::ConfigAction;
use crate::shared::i18n::I18n;
use crate::shared::save::SAVE_SLOTS;

use super::components::{
    default_button_size, default_font_size, flex_row, localized_text, root_node, text,
    transparent_node,
};
use super::theme::{absolute_text_bundle, panel_bundle, STATUS_TEXT};
use super::types::{
    BackpackPanel, CarriedIcon, CarriedLabel, ConverterAction, ConverterInputRow,
    ConverterInputText, ConverterModeText, ConverterOutputText, ConverterPanel, Crosshair,
    CurrentSaveText, GeneratorAction, GeneratorMaterialText, GeneratorPanel, GeneratorPeriodText,
    HotbarText, InGameHudStyle, InGameHudVisibility, InventoryTitle, LabelerAction,
    LabelerColorText, LabelerPanel, MainMenuAction, MainMenuPanel, PauseAction, PausePanel,
    SaveListPanel, SaveListTitle, SettingsAction, SettingsDropdown, SettingsGameplayGroup,
    SettingsKeyBindingsGroup, SettingsPanel, SettingsSlider, SettingsStatusText, SimulationText,
    SlotArea, TeleportAction, TeleportNameText, TeleportPairText, TeleportPanel, BACKPACK_SLOTS,
    HOTBAR_SLOTS,
};
use super::widgets::{
    scroll_container, scroll_content, spawn_converter_button, spawn_generator_button,
    spawn_labeler_button, spawn_localized_main_button, spawn_localized_pause_button,
    spawn_localized_settings_button, spawn_save_back_button, spawn_save_slot_button,
    spawn_settings_dropdown, spawn_settings_slider, spawn_settings_tab, spawn_slot,
    spawn_teleport_button,
};

pub fn setup_ui(mut commands: Commands, i18n: Res<I18n>) {
    commands.spawn(root_node()).with_children(|root| {
        spawn_status_overlays(root);
        spawn_hotbar(root);
        spawn_inventory_panel(root, &i18n);
        spawn_generator_panel(root, &i18n);
        spawn_labeler_panel(root, &i18n);
        spawn_converter_panel(root, &i18n);
        spawn_teleport_panel(root, &i18n);
        spawn_pause_panel(root, &i18n);
        spawn_settings_panel(root, &i18n);
        spawn_main_menu(root, &i18n);
        spawn_save_list(root);
        spawn_carried_label(root);
    });
}

fn spawn_generator_panel(root: &mut ChildBuilder, i18n: &I18n) {
    root.spawn((panel_bundle(480.0, 320.0, -240.0, -160.0), GeneratorPanel))
        .with_children(|panel| {
            panel.spawn(localized_text(i18n, "generator.title", 26.0, Color::WHITE));
            panel.spawn(flex_row(40.0, 8.0)).with_children(|row| {
                spawn_generator_button(
                    row,
                    i18n.text("button.period_down"),
                    "button.period_down",
                    GeneratorAction::PeriodDown,
                );
                row.spawn((text("", 18.0, Color::WHITE), GeneratorPeriodText));
                spawn_generator_button(
                    row,
                    i18n.text("button.period_up"),
                    "button.period_up",
                    GeneratorAction::PeriodUp,
                );
            });
            panel.spawn(flex_row(40.0, 8.0)).with_children(|row| {
                spawn_generator_button(
                    row,
                    i18n.text("button.material_next"),
                    "button.material_next",
                    GeneratorAction::MaterialNext,
                );
                row.spawn((text("", 18.0, Color::WHITE), GeneratorMaterialText));
            });
            spawn_generator_button(
                panel,
                i18n.text("button.close"),
                "button.close",
                GeneratorAction::Close,
            );
        });
}

fn spawn_teleport_panel(root: &mut ChildBuilder, i18n: &I18n) {
    root.spawn((panel_bundle(460.0, 280.0, -230.0, -140.0), TeleportPanel))
        .with_children(|panel| {
            panel.spawn(localized_text(i18n, "teleport.title", 26.0, Color::WHITE));
            panel.spawn((text("", 18.0, Color::WHITE), TeleportNameText));
            panel.spawn((text("", 18.0, Color::WHITE), TeleportPairText));
            panel.spawn(flex_row(40.0, 8.0)).with_children(|row| {
                spawn_teleport_button(
                    row,
                    i18n.text("button.teleport_pair"),
                    "button.teleport_pair",
                    TeleportAction::CyclePair,
                );
                spawn_teleport_button(
                    row,
                    i18n.text("button.teleport_rename"),
                    "button.teleport_rename",
                    TeleportAction::Rename,
                );
            });
            spawn_teleport_button(
                panel,
                i18n.text("button.close"),
                "button.close",
                TeleportAction::Close,
            );
        });
}

fn spawn_converter_panel(root: &mut ChildBuilder, i18n: &I18n) {
    root.spawn((panel_bundle(460.0, 320.0, -230.0, -160.0), ConverterPanel))
        .with_children(|panel| {
            panel.spawn(localized_text(i18n, "converter.title", 26.0, Color::WHITE));
            panel.spawn(flex_row(40.0, 8.0)).with_children(|row| {
                spawn_converter_button(
                    row,
                    i18n.text("button.converter_mode"),
                    "button.converter_mode",
                    ConverterAction::ToggleMode,
                );
                row.spawn((text("", 18.0, Color::WHITE), ConverterModeText));
            });
            panel
                .spawn((flex_row(40.0, 8.0), ConverterInputRow))
                .with_children(|row| {
                    spawn_converter_button(
                        row,
                        i18n.text("button.input_material"),
                        "button.input_material",
                        ConverterAction::InputNext,
                    );
                    row.spawn((text("", 18.0, Color::WHITE), ConverterInputText));
                });
            panel.spawn(flex_row(40.0, 8.0)).with_children(|row| {
                spawn_converter_button(
                    row,
                    i18n.text("button.output_material"),
                    "button.output_material",
                    ConverterAction::OutputNext,
                );
                row.spawn((text("", 18.0, Color::WHITE), ConverterOutputText));
            });
            spawn_converter_button(
                panel,
                i18n.text("button.close"),
                "button.close",
                ConverterAction::Close,
            );
        });
}

fn spawn_labeler_panel(root: &mut ChildBuilder, i18n: &I18n) {
    root.spawn((panel_bundle(420.0, 240.0, -210.0, -120.0), LabelerPanel))
        .with_children(|panel| {
            panel.spawn(localized_text(i18n, "labeler.title", 26.0, Color::WHITE));
            panel.spawn(flex_row(40.0, 8.0)).with_children(|row| {
                spawn_labeler_button(
                    row,
                    i18n.text("button.previous_color"),
                    "button.previous_color",
                    LabelerAction::PreviousColor,
                );
                row.spawn((text("", 18.0, Color::WHITE), LabelerColorText));
                spawn_labeler_button(
                    row,
                    i18n.text("button.next_color"),
                    "button.next_color",
                    LabelerAction::NextColor,
                );
            });
            spawn_labeler_button(
                panel,
                i18n.text("button.close"),
                "button.close",
                LabelerAction::Close,
            );
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
        InGameHudVisibility,
    ));
    root.spawn((
        absolute_text_bundle(
            "",
            16.0,
            Color::WHITE,
            Some(Val::Px(18.0)),
            None,
            Some(Val::Px(62.0)),
            None,
        ),
        HotbarText,
        InGameHudVisibility,
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
        InGameHudVisibility,
    ));
    root.spawn((
        absolute_text_bundle(
            "",
            16.0,
            STATUS_TEXT,
            Some(Val::Px(18.0)),
            None,
            Some(Val::Px(112.0)),
            None,
        ),
        SimulationText,
        InGameHudVisibility,
    ));
}

fn spawn_hotbar(root: &mut ChildBuilder) {
    root.spawn((
        NodeBundle {
            style: Style {
                width: Val::Px(default_button_size(540.0)),
                height: Val::Px(default_button_size(58.0)),
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                bottom: Val::Px(22.0),
                margin: UiRect {
                    left: Val::Px(-default_button_size(270.0)),
                    ..default()
                },
                display: Display::Flex,
                justify_content: JustifyContent::Center,
                column_gap: Val::Px(4.0),
                ..default()
            },
            background_color: Color::srgba(0.04, 0.04, 0.04, 0.38).into(),
            ..default()
        },
        InGameHudStyle,
    ))
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
    root.spawn((panel_bundle(420.0, 560.0, -210.0, -280.0), PausePanel))
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
    root.spawn((panel_bundle(840.0, 660.0, -420.0, -330.0), SettingsPanel))
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
    panel
        .spawn(transparent_node(Style {
            width: Val::Percent(100.0),
            height: Val::Px(default_button_size(42.0)),
            display: Display::Flex,
            column_gap: Val::Px(6.0),
            ..default()
        }))
        .with_children(|tabs| {
            spawn_settings_tab(
                tabs,
                i18n.text("button.gameplay"),
                "button.gameplay",
                SettingsAction::TabGameplay,
            );
            spawn_settings_tab(
                tabs,
                i18n.text("button.key_bindings"),
                "button.key_bindings",
                SettingsAction::TabKeyBindings,
            );
        });
}

fn spawn_settings_row(
    panel: &mut ChildBuilder,
    i18n: &I18n,
    label_key: &'static str,
    label_marker: impl Bundle,
    controls: impl FnOnce(&mut ChildBuilder),
) {
    panel
        .spawn(transparent_node(Style {
            width: Val::Percent(100.0),
            min_height: Val::Px(default_button_size(54.0)),
            display: Display::Flex,
            align_items: AlignItems::FlexStart,
            column_gap: Val::Px(18.0),
            ..default()
        }))
        .insert(SettingsGameplayGroup)
        .with_children(|row| {
            row.spawn((
                localized_text(i18n, label_key, 15.0, Color::srgb(0.82, 0.88, 0.90)),
                label_marker,
            ));
            row.spawn(transparent_node(Style {
                width: Val::Px(430.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                ..default()
            }))
            .with_children(controls);
        });
}

fn spawn_gameplay_settings(panel: &mut ChildBuilder, i18n: &I18n) {
    spawn_settings_row(panel, i18n, "settings.fov", (), |controls| {
        spawn_settings_slider(controls, SettingsSlider::Fov);
    });
    spawn_settings_row(panel, i18n, "settings.ui_scale_label", (), |controls| {
        spawn_settings_slider(controls, SettingsSlider::UiScale);
    });
    spawn_settings_row(panel, i18n, "settings.language", (), |controls| {
        spawn_settings_dropdown(controls, SettingsDropdown::Language);
    });
    spawn_settings_row(
        panel,
        i18n,
        "settings.place_selection_mode",
        (),
        |controls| {
            spawn_settings_dropdown(controls, SettingsDropdown::PlaceSelectionMode);
        },
    );
    spawn_settings_row(
        panel,
        i18n,
        "settings.delete_selection_mode",
        (),
        |controls| {
            spawn_settings_dropdown(controls, SettingsDropdown::DeleteSelectionMode);
        },
    );
}

fn spawn_key_bindings(panel: &mut ChildBuilder, i18n: &I18n) {
    panel
        .spawn(scroll_container(360.0))
        .insert(SettingsKeyBindingsGroup)
        .with_children(|container| {
            container
                .spawn((key_bindings_grid_bundle(), scroll_content()))
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
    root.spawn((panel_bundle(420.0, 340.0, -210.0, -170.0), MainMenuPanel))
        .with_children(|panel| {
            panel.spawn(localized_text(i18n, "main.title", 30.0, Color::WHITE));
            for (key, action) in [
                ("button.create_new_world", MainMenuAction::NewWorld),
                ("button.load_save", MainMenuAction::OpenSaveList),
                ("button.settings", MainMenuAction::OpenSettings),
                ("button.quit_game", MainMenuAction::Quit),
            ] {
                spawn_localized_main_button(panel, i18n.text(key), key, action);
            }
        });
}

fn spawn_save_list(root: &mut ChildBuilder) {
    root.spawn((panel_bundle(520.0, 560.0, -260.0, -280.0), SaveListPanel))
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
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Px(default_button_size(46.0)),
                height: Val::Px(default_button_size(46.0)),
                display: Display::None,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            border_color: Color::srgb(1.0, 1.0, 1.0).into(),
            background_color: Color::srgba(0.18, 0.18, 0.19, 0.86).into(),
            z_index: ZIndex::Global(100),
            ..default()
        },
        CarriedIcon,
    ))
    .with_children(|icon| {
        icon.spawn((
            TextBundle {
                text: Text::from_section(
                    "",
                    TextStyle {
                        font_size: default_font_size(12.0),
                        color: Color::WHITE,
                        ..default()
                    },
                )
                .with_justify(JustifyText::Center),
                style: Style {
                    margin: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                ..default()
            },
            CarriedLabel,
        ));
    });
}

fn inventory_panel_bundle() -> NodeBundle {
    NodeBundle {
        style: Style {
            width: Val::Px(640.0),
            height: Val::Px(430.0),
            position_type: PositionType::Absolute,
            left: Val::Percent(50.0),
            top: Val::Percent(50.0),
            margin: UiRect {
                left: Val::Px(-320.0),
                top: Val::Px(-215.0),
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
        width: Val::Px(605.0),
        height: Val::Px(197.0),
        ..default()
    })
}

fn key_bindings_grid_bundle() -> NodeBundle {
    transparent_node(Style {
        display: Display::Grid,
        grid_template_columns: RepeatedGridTrack::flex(2, 1.0),
        grid_template_rows: RepeatedGridTrack::flex(6, 1.0),
        position_type: PositionType::Absolute,
        width: Val::Percent(100.0),
        height: Val::Px(360.0),
        row_gap: Val::Px(6.0),
        column_gap: Val::Px(8.0),
        ..default()
    })
}
