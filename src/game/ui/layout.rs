use bevy::prelude::*;

use crate::shared::config::ConfigAction;
use crate::shared::i18n::I18n;
use crate::shared::save::SAVE_SLOTS;

use super::components::{
    default_button_size, default_font_size, flex_row, localized_text, root_node, text,
    transparent_node,
};
use super::theme::{absolute_text_bundle, panel_bundle, STATUS_TEXT, TITLE_TEXT};
use super::types::{
    BackpackPanel, BlockPanelDropdown, CarriedIcon, CarriedLabel, ConfirmDialogAction,
    ConfirmDialogMessage, ConfirmDialogPanel, ConfirmDialogPrimaryLabel,
    ConfirmDialogSecondaryLabel, ConfirmDialogTitle, ConverterAction, ConverterInputRow,
    ConverterPanel, Crosshair, CurrentSaveText, GeneratorAction, GeneratorPanel,
    GeneratorPeriodText, GoalAction, GoalPanel, HotbarText, InGameHudStyle, InGameHudVisibility,
    InventoryTitle, InventoryTooltip, InventoryTooltipText, LabelerAction, LabelerPanel,
    MainMenuAction, MainMenuPanel, PauseAction, PausePanel, SaveListAction, SaveListPanel,
    SaveListTitle, SettingsAction, SettingsDropdown, SettingsGameplayGroup,
    SettingsKeyBindingsGroup, SettingsPanel, SettingsSlider, SettingsDropdownRow,
    SimulationStatusText, SimulationText, SlotArea, TeleportAction, TeleportNameText,
    TeleportPanel, UiPanelBinding, UiPanelId, BACKPACK_SLOTS, HOTBAR_SLOTS,
};
use super::widgets::{
    scroll_container, scroll_content, spawn_block_panel_dropdown, spawn_close_button,
    spawn_confirm_dialog_button, spawn_generator_button, spawn_localized_main_button,
    spawn_localized_pause_button, spawn_localized_settings_button, spawn_save_back_button,
    spawn_save_row_button, spawn_save_slot_button, spawn_settings_dropdown, spawn_settings_slider,
    spawn_settings_slider_value, spawn_settings_tab, spawn_slot, spawn_teleport_button,
};
use crate::game::world::blocks::{MaterialKind, StampColor};

pub fn setup_ui(mut commands: Commands, i18n: Res<I18n>) {
    commands.spawn(root_node()).with_children(|root| {
        spawn_status_overlays(root);
        spawn_hotbar(root);
        spawn_inventory_panel(root, &i18n);
        spawn_generator_panel(root, &i18n);
        spawn_goal_panel(root, &i18n);
        spawn_labeler_panel(root, &i18n);
        spawn_converter_panel(root, &i18n);
        spawn_teleport_panel(root, &i18n);
        spawn_pause_panel(root, &i18n);
        spawn_settings_panel(root, &i18n);
        spawn_confirm_dialog(root);
        spawn_main_menu(root, &i18n);
        spawn_save_list(root);
        spawn_carried_label(root);
        spawn_inventory_tooltip(root);
    });
}

fn spawn_generator_panel(root: &mut ChildSpawnerCommands, i18n: &I18n) {
    root.spawn((
        panel_bundle(430.0, 230.0, -215.0, -115.0),
        GeneratorPanel,
        UiPanelBinding(UiPanelId::Generator),
    ))
    .with_children(|panel| {
        panel.spawn(localized_text(i18n, "generator.title", 26.0, TITLE_TEXT));
        spawn_close_button(panel, GeneratorAction::Close);
        spawn_panel_row(panel, i18n, "panel.period", |row| {
            spawn_generator_button(row, GeneratorAction::PeriodDown);
            row.spawn((text("", 18.0, Color::WHITE), GeneratorPeriodText));
            spawn_generator_button(row, GeneratorAction::PeriodUp);
        });
        spawn_panel_row(panel, i18n, "panel.material", |row| {
            spawn_block_panel_dropdown(
                row,
                BlockPanelDropdown::GeneratorMaterial,
                GeneratorAction::ToggleMaterialDropdown,
                material_options(i18n)
                    .map(|(label, material)| (label, GeneratorAction::SetMaterial(material))),
            );
        });
    });
}

fn spawn_goal_panel(root: &mut ChildSpawnerCommands, i18n: &I18n) {
    root.spawn((
        panel_bundle(430.0, 190.0, -215.0, -95.0),
        GoalPanel,
        UiPanelBinding(UiPanelId::Goal),
    ))
    .with_children(|panel| {
        panel.spawn(localized_text(i18n, "goal.title", 26.0, TITLE_TEXT));
        spawn_close_button(panel, GoalAction::Close);
        spawn_panel_row(panel, i18n, "panel.material", |row| {
            spawn_block_panel_dropdown(
                row,
                BlockPanelDropdown::GoalMaterial,
                GoalAction::ToggleMaterialDropdown,
                material_options(i18n)
                    .map(|(label, material)| (label, GoalAction::SetMaterial(material))),
            );
        });
    });
}

fn spawn_teleport_panel(root: &mut ChildSpawnerCommands, i18n: &I18n) {
    root.spawn((
        panel_bundle(460.0, 230.0, -230.0, -115.0),
        TeleportPanel,
        UiPanelBinding(UiPanelId::Teleport),
    ))
    .with_children(|panel| {
        panel.spawn(localized_text(i18n, "teleport.title", 26.0, TITLE_TEXT));
        spawn_close_button(panel, TeleportAction::Close);
        spawn_panel_row(panel, i18n, "panel.name", |row| {
            spawn_teleport_button(row, TeleportAction::Rename);
            row.spawn((text("", 18.0, Color::WHITE), TeleportNameText));
        });
        spawn_panel_row(panel, i18n, "panel.pair", |row| {
            spawn_block_panel_dropdown(
                row,
                BlockPanelDropdown::TeleportPair,
                TeleportAction::TogglePairDropdown,
                std::iter::empty::<(String, TeleportAction)>(),
            );
        });
    });
}

fn spawn_converter_panel(root: &mut ChildSpawnerCommands, i18n: &I18n) {
    root.spawn((
        panel_bundle(460.0, 260.0, -230.0, -130.0),
        ConverterPanel,
        UiPanelBinding(UiPanelId::Converter),
    ))
    .with_children(|panel| {
        panel.spawn(localized_text(i18n, "converter.title", 26.0, TITLE_TEXT));
        spawn_close_button(panel, ConverterAction::Close);
        panel
            .spawn((panel_row_node(), ConverterInputRow))
            .with_children(|row| {
                spawn_panel_label(row, i18n, "panel.input");
                spawn_block_panel_dropdown(
                    row,
                    BlockPanelDropdown::ConverterInput,
                    ConverterAction::ToggleInputDropdown,
                    material_options(i18n)
                        .map(|(label, material)| (label, ConverterAction::SetInput(material))),
                );
            });
        spawn_panel_row(panel, i18n, "panel.output", |row| {
            spawn_block_panel_dropdown(
                row,
                BlockPanelDropdown::ConverterOutput,
                ConverterAction::ToggleOutputDropdown,
                material_options(i18n)
                    .map(|(label, material)| (label, ConverterAction::SetOutput(material))),
            );
        });
    });
}

fn spawn_labeler_panel(root: &mut ChildSpawnerCommands, i18n: &I18n) {
    root.spawn((
        panel_bundle(420.0, 190.0, -210.0, -95.0),
        LabelerPanel,
        UiPanelBinding(UiPanelId::Labeler),
    ))
    .with_children(|panel| {
        panel.spawn(localized_text(i18n, "labeler.title", 26.0, TITLE_TEXT));
        spawn_close_button(panel, LabelerAction::Close);
        spawn_panel_row(panel, i18n, "panel.color", |row| {
            spawn_block_panel_dropdown(
                row,
                BlockPanelDropdown::LabelerColor,
                LabelerAction::ToggleColorDropdown,
                stamp_color_options(i18n)
                    .map(|(label, color)| (label, LabelerAction::SetColor(color))),
            );
        });
    });
}

fn panel_row_node() -> impl Bundle {
    transparent_node(Node {
        width: Val::Percent(100.0),
        height: Val::Px(default_button_size(40.0)),
        display: Display::Flex,
        align_items: AlignItems::Center,
        column_gap: Val::Px(10.0),
        ..default()
    })
}

fn spawn_panel_row(
    panel: &mut ChildSpawnerCommands,
    i18n: &I18n,
    label_key: &'static str,
    controls: impl FnOnce(&mut ChildSpawnerCommands),
) {
    panel.spawn(panel_row_node()).with_children(|row| {
        spawn_panel_label(row, i18n, label_key);
        controls(row);
    });
}

fn spawn_panel_label(row: &mut ChildSpawnerCommands, i18n: &I18n, label_key: &'static str) {
    row.spawn((
        localized_text(i18n, label_key, 16.0, Color::srgb(0.86, 0.88, 0.86)),
        Node {
            width: Val::Px(110.0),
            ..default()
        },
    ));
}

fn material_options(i18n: &I18n) -> impl Iterator<Item = (String, MaterialKind)> + '_ {
    MaterialKind::ALL
        .into_iter()
        .map(|material| (i18n.text(material.name_key()), material))
}

fn stamp_color_options(i18n: &I18n) -> impl Iterator<Item = (String, StampColor)> + '_ {
    StampColor::ALL
        .into_iter()
        .map(|color| (i18n.text(color.name_key()), color))
}

fn spawn_status_overlays(root: &mut ChildSpawnerCommands) {
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
    root.spawn((
        absolute_text_bundle(
            "",
            16.0,
            STATUS_TEXT,
            None,
            Some(Val::Px(18.0)),
            None,
            Some(Val::Px(18.0)),
        ),
        SimulationStatusText,
        InGameHudVisibility,
    ));
}

fn spawn_hotbar(root: &mut ChildSpawnerCommands) {
    root.spawn((
        Node {
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
        BackgroundColor(Color::srgba(0.04, 0.04, 0.04, 0.38)),
        InGameHudStyle,
    ))
    .with_children(|bar| {
        for index in 0..HOTBAR_SLOTS {
            spawn_slot(bar, SlotArea::Hotbar, index);
        }
    });
}

fn spawn_inventory_panel(root: &mut ChildSpawnerCommands, i18n: &I18n) {
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

fn spawn_pause_panel(root: &mut ChildSpawnerCommands, i18n: &I18n) {
    root.spawn((panel_bundle(420.0, 560.0, -210.0, -280.0), PausePanel))
        .with_children(|panel| {
            panel.spawn(localized_text(i18n, "state.paused", 30.0, TITLE_TEXT));
            for action in [
                PauseAction::Resume,
                PauseAction::ToggleBuilderMode,
                PauseAction::SaveWorld,
                PauseAction::ResetSolution,
                PauseAction::OpenSettings,
                PauseAction::BackToMainMenu,
            ] {
                spawn_localized_pause_button(panel, action);
            }
        });
}

fn spawn_confirm_dialog(root: &mut ChildSpawnerCommands) {
    root.spawn((
        panel_bundle(460.0, 250.0, -230.0, -125.0),
        ConfirmDialogPanel,
    ))
    .with_children(|panel| {
        panel.spawn((text("", 24.0, TITLE_TEXT), ConfirmDialogTitle));
        panel.spawn((
            text("", 15.0, STATUS_TEXT),
            TextLayout::new_with_justify(Justify::Center),
            Node {
                min_height: Val::Px(54.0),
                align_self: AlignSelf::Stretch,
                ..default()
            },
            ConfirmDialogMessage,
        ));
        panel.spawn(flex_row(40.0, 8.0)).with_children(|row| {
            spawn_confirm_dialog_button(
                row,
                ConfirmDialogAction::Primary,
                ConfirmDialogPrimaryLabel,
            );
            spawn_confirm_dialog_button(
                row,
                ConfirmDialogAction::Secondary,
                ConfirmDialogSecondaryLabel,
            );
            spawn_confirm_dialog_button(row, ConfirmDialogAction::Cancel, ());
        });
    });
}

fn spawn_settings_panel(root: &mut ChildSpawnerCommands, i18n: &I18n) {
    root.spawn((
        panel_bundle(840.0, 660.0, -420.0, -330.0),
        SettingsPanel,
        UiPanelBinding(UiPanelId::Settings),
    ))
    .with_children(|panel| {
        panel.spawn(localized_text(i18n, "settings.title", 30.0, TITLE_TEXT));
        spawn_settings_tabs(panel);
        spawn_gameplay_settings(panel, i18n);
        spawn_key_bindings(panel, i18n);
    });
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
            transparent_node(Node {
                width: Val::Percent(100.0),
                min_height: Val::Px(default_button_size(54.0)),
                display: Display::Flex,
                align_items: AlignItems::Center,
                column_gap: Val::Px(18.0),
                ..default()
            }),
            SettingsGameplayGroup,
            SettingsDropdownRow(dropdown),
            ZIndex(300),
        ))
        .with_children(|row| {
            row.spawn((
                localized_text(i18n, label_key, 15.0, Color::srgb(0.82, 0.88, 0.90)),
                Node {
                    width: Val::Px(220.0),
                    align_self: AlignSelf::Center,
                    ..default()
                },
            ));
            row.spawn(transparent_node(Node {
                width: Val::Px(530.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                justify_content: JustifyContent::Center,
                ..default()
            }))
            .with_children(|controls| {
                spawn_settings_dropdown(controls, dropdown);
            });
        });
}

fn spawn_settings_slider_row(
    panel: &mut ChildSpawnerCommands,
    i18n: &I18n,
    label_key: &'static str,
    slider: SettingsSlider,
) {
    panel
        .spawn(transparent_node(Node {
            width: Val::Percent(100.0),
            min_height: Val::Px(default_button_size(54.0)),
            display: Display::Flex,
            align_items: AlignItems::Center,
            column_gap: Val::Px(18.0),
            ..default()
        }))
        .insert(SettingsGameplayGroup)
        .with_children(|row| {
            row.spawn((
                localized_text(i18n, label_key, 15.0, Color::srgb(0.82, 0.88, 0.90)),
                Node {
                    width: Val::Px(220.0),
                    align_self: AlignSelf::Center,
                    ..default()
                },
            ));
            row.spawn(transparent_node(Node {
                width: Val::Px(360.0),
                justify_content: JustifyContent::Center,
                ..default()
            }))
            .with_children(|controls| {
                spawn_settings_slider(controls, slider);
            });
            spawn_settings_slider_value(row, slider);
        });
}

fn spawn_gameplay_settings(panel: &mut ChildSpawnerCommands, i18n: &I18n) {
    panel
        .spawn(scroll_container(500.0))
        .insert(SettingsGameplayGroup)
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
                    spawn_settings_slider_row(content, i18n, "settings.fov", SettingsSlider::Fov);
                    spawn_settings_slider_row(
                        content,
                        i18n,
                        "settings.ui_scale_label",
                        SettingsSlider::UiScale,
                    );
                    spawn_settings_slider_row(
                        content,
                        i18n,
                        "settings.gravity",
                        SettingsSlider::Gravity,
                    );
                    spawn_settings_dropdown_row(
                        content,
                        i18n,
                        "settings.language",
                        SettingsDropdown::Language,
                    );
                    spawn_settings_dropdown_row(
                        content,
                        i18n,
                        "settings.place_selection_mode",
                        SettingsDropdown::PlaceSelectionMode,
                    );
                    spawn_settings_dropdown_row(
                        content,
                        i18n,
                        "settings.delete_selection_mode",
                        SettingsDropdown::DeleteSelectionMode,
                    );
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
        .insert(SettingsKeyBindingsGroup)
        .with_children(|container| {
            container
                .spawn((key_bindings_grid_bundle(), scroll_content()))
                .with_children(|grid| {
                    grid.spawn(localized_text(
                        i18n,
                        "settings.group.general",
                        18.0,
                        Color::WHITE,
                    ));
                    grid.spawn(transparent_node(Node::default()));
                    for action in ConfigAction::GENERAL {
                        spawn_localized_settings_button(grid, SettingsAction::Bind(action));
                    }
                    grid.spawn(localized_text(
                        i18n,
                        "settings.group.simulation",
                        18.0,
                        Color::WHITE,
                    ));
                    grid.spawn(transparent_node(Node::default()));
                    for action in ConfigAction::SIMULATION {
                        spawn_localized_settings_button(grid, SettingsAction::Bind(action));
                    }
                });
        });
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

fn spawn_main_menu(root: &mut ChildSpawnerCommands, i18n: &I18n) {
    root.spawn((panel_bundle(420.0, 340.0, -210.0, -170.0), MainMenuPanel))
        .with_children(|panel| {
            panel.spawn(localized_text(i18n, "main.title", 30.0, Color::WHITE));
            for action in [
                MainMenuAction::EditPuzzle,
                MainMenuAction::Play,
                MainMenuAction::OpenSettings,
                MainMenuAction::Quit,
            ] {
                spawn_localized_main_button(panel, action);
            }
        });
}

fn spawn_save_list(root: &mut ChildSpawnerCommands) {
    root.spawn((panel_bundle(900.0, 620.0, -450.0, -310.0), SaveListPanel))
        .with_children(|panel| {
            panel.spawn((text("", 26.0, Color::WHITE), SaveListTitle));
            panel.spawn(flex_row(470.0, 12.0)).with_children(|columns| {
                columns
                    .spawn(transparent_node(Node {
                        width: Val::Px(420.0),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(6.0),
                        ..default()
                    }))
                    .with_children(|left| {
                        for index in 0..SAVE_SLOTS {
                            left.spawn(flex_row(32.0, 6.0)).with_children(|row| {
                                spawn_save_row_button(
                                    row,
                                    SaveListAction::LoadPuzzle(index),
                                    260.0,
                                );
                                spawn_save_row_button(
                                    row,
                                    SaveListAction::DeletePuzzle(index),
                                    80.0,
                                );
                            });
                        }
                        spawn_save_slot_button(left, SaveListAction::NewPuzzle);
                    });
                columns
                    .spawn(transparent_node(Node {
                        width: Val::Px(420.0),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(6.0),
                        ..default()
                    }))
                    .with_children(|right| {
                        for index in 0..SAVE_SLOTS {
                            right.spawn(flex_row(32.0, 6.0)).with_children(|row| {
                                spawn_save_row_button(
                                    row,
                                    SaveListAction::LoadSolution(index),
                                    260.0,
                                );
                                spawn_save_row_button(
                                    row,
                                    SaveListAction::DeleteSolution(index),
                                    80.0,
                                );
                            });
                        }
                        spawn_save_slot_button(right, SaveListAction::NewSolution);
                    });
            });
            spawn_save_back_button(panel);
        });
}

fn spawn_carried_label(root: &mut ChildSpawnerCommands) {
    root.spawn((
        Node {
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
        BorderColor::all(Color::srgb(1.0, 1.0, 1.0)),
        BackgroundColor(Color::srgba(0.18, 0.18, 0.19, 0.86)),
        ZIndex(100),
        CarriedIcon,
    ))
    .with_children(|icon| {
        icon.spawn((
            ImageNode::default(),
            Node {
                width: Val::Px(default_button_size(64.0)),
                height: Val::Px(default_button_size(64.0)),
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                top: Val::Percent(50.0),
                margin: UiRect {
                    left: Val::Px(-default_button_size(32.0)),
                    top: Val::Px(-default_button_size(32.0)),
                    ..default()
                },
                ..default()
            },
        ));
        icon.spawn((
            Text::new(""),
            TextFont {
                font_size: default_font_size(12.0),
                ..default()
            },
            TextColor(Color::WHITE),
            TextLayout::new_with_justify(Justify::Center),
            Node {
                margin: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            CarriedLabel,
        ));
    });
}

fn spawn_inventory_tooltip(root: &mut ChildSpawnerCommands) {
    root.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            display: Display::None,
            padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
            border: UiRect::all(Val::Px(1.0)),
            ..default()
        },
        BorderColor::all(Color::srgba(0.72, 0.82, 0.88, 0.75)),
        BackgroundColor(Color::srgba(0.05, 0.06, 0.07, 0.92)),
        ZIndex(140),
        InventoryTooltip,
    ))
    .with_children(|tooltip| {
        tooltip.spawn((
            text("", 14.0, Color::WHITE),
            TextLayout::new_with_justify(Justify::Center),
            InventoryTooltipText,
        ));
    });
}

fn inventory_panel_bundle() -> impl Bundle {
    (
        Node {
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
        BackgroundColor(Color::srgba(0.12, 0.12, 0.13, 0.94)),
    )
}

fn inventory_grid_bundle() -> impl Bundle {
    transparent_node(Node {
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
