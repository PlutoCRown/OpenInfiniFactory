use bevy::prelude::*;

use crate::game::ui::access::{bind_ui_scope, i18n};

use super::components::{
    absolute_text_bundle, default_button_size, flex_row, full_width_button, localized_text,
    panel_bundle, panel_content, panel_title_bar, panel_title_label, raised_border, root_node,
    spawn_panel, styled_button, text, transparent_node, PanelOptions, STATUS_TEXT, BUTTON_BG,
};
use crate::game::block_editing::{BlockPanelAction, BlockPanelDropdown, BlockPanelText, BlockPanelTextKind};
use crate::game::ui::core::confirm_dialog::{
    ConfirmButtonId, ConfirmMessageText, ConfirmTitleText,
};
use crate::game::ui::core::host::{PlayingUiRootEntity, UiRootEntity};
use crate::game::ui::core::text_prompt::{
    TextPromptButtonId, TextPromptRoot, TextPromptText,
};
use super::screens::{
    spawn_carried_label, spawn_hotbar, spawn_inventory_panel, spawn_inventory_tooltip,
    spawn_main_menu, spawn_pause_panel, spawn_save_list,
};
use crate::game::state::UiPanelId;
use super::types::{
    ConverterInputRow, Crosshair, GameplayHudVisibility, InGameHudVisibility, PanelVisibility,
    PlayingUiRoot, StatusText, StatusTextKind, UiPanelBinding, UiRoot,
};
use super::widgets::{
    spawn_block_panel_button, spawn_block_panel_dropdown, spawn_block_panel_dropdown_list,
    spawn_confirm_dialog_button, spawn_material_icon_dropdown_list, spawn_material_icon_slot,
};
use crate::game::world::blocks::{MaterialKind, StampColor};

pub fn setup_menu_ui(world: &mut World) {
    bind_ui_scope(world);
    let mut commands = world.commands();
    let root = commands
        .spawn((root_node(), UiRoot))
        .with_children(|root| {
            spawn_confirm_dialog(root);
            spawn_text_prompt(root);
            spawn_main_menu(root);
            spawn_save_list(root);
        })
        .id();
    commands.insert_resource(UiRootEntity(root));
}

pub fn setup_playing_ui(commands: &mut Commands) {
    let root = commands
        .spawn((root_node(), PlayingUiRoot))
        .with_children(|root| {
            spawn_status_overlays(root);
            spawn_hotbar(root);
            spawn_inventory_panel(root);
            spawn_generator_panel(root);
            spawn_goal_panel(root);
            spawn_labeler_panel(root);
            spawn_converter_panel(root);
            spawn_teleport_panel(root);
            spawn_pause_panel(root);
            spawn_carried_label(root);
            spawn_inventory_tooltip(root);
            spawn_block_dropdown_layers(root);
        })
        .id();
    commands.insert_resource(PlayingUiRootEntity(root));
}

pub fn setup_playing_ui_system(world: &mut World) {
    bind_ui_scope(world);
    let mut commands = world.commands();
    setup_playing_ui(&mut commands);
}

fn spawn_generator_panel(root: &mut ChildSpawnerCommands) {
    spawn_panel(
        root,
        PanelOptions::new(430.0, "generator.title").closable(),
        UiPanelBinding(UiPanelId::Generator),
        |panel| {
            spawn_panel_row(panel, "panel.period", |row| {
                spawn_block_panel_button(row, BlockPanelAction::PeriodDown);
                row.spawn((
                    text("", 18.0, Color::WHITE),
                    BlockPanelText(BlockPanelTextKind::GeneratorPeriod),
                ));
                spawn_block_panel_button(row, BlockPanelAction::PeriodUp);
            });
            spawn_panel_row(panel, "panel.material", |row| {
                spawn_material_icon_slot(
                    row,
                    BlockPanelDropdown::GeneratorMaterial,
                    BlockPanelAction::ToggleMaterialDropdown,
                );
            });
        },
    );
}

fn spawn_goal_panel(root: &mut ChildSpawnerCommands) {
    spawn_panel(
        root,
        PanelOptions::new(430.0, "goal.title").closable(),
        UiPanelBinding(UiPanelId::Goal),
        |panel| {
            spawn_panel_row(panel, "panel.material", |row| {
                spawn_material_icon_slot(
                    row,
                    BlockPanelDropdown::GoalMaterial,
                    BlockPanelAction::ToggleMaterialDropdown,
                );
            });
        },
    );
}

fn spawn_teleport_panel(root: &mut ChildSpawnerCommands) {
    spawn_panel(
        root,
        PanelOptions::new(460.0, "teleport.title").closable(),
        UiPanelBinding(UiPanelId::Teleport),
        |panel| {
            spawn_panel_row(panel, "panel.name", |row| {
                spawn_block_panel_button(row, BlockPanelAction::StartTeleportRename);
                row.spawn((
                    text("", 18.0, Color::WHITE),
                    BlockPanelText(BlockPanelTextKind::TeleportName),
                ));
            });
            spawn_panel_row(panel, "panel.pair", |row| {
                spawn_block_panel_dropdown(
                    row,
                    BlockPanelDropdown::TeleportPair,
                    BlockPanelAction::ToggleTeleportPairDropdown,
                );
            });
        },
    );
}

fn spawn_converter_panel(root: &mut ChildSpawnerCommands) {
    spawn_panel(
        root,
        PanelOptions::new(460.0, "converter.title").closable(),
        UiPanelBinding(UiPanelId::Converter),
        |panel| {
            panel
                .spawn((panel_row_node(), ConverterInputRow))
                .with_children(|row| {
                    spawn_panel_label(row, "panel.input");
                    spawn_material_icon_slot(
                        row,
                        BlockPanelDropdown::ConverterInput,
                        BlockPanelAction::ToggleInputDropdown,
                    );
                });
            spawn_panel_row(panel, "panel.output", |row| {
                spawn_material_icon_slot(
                    row,
                    BlockPanelDropdown::ConverterOutput,
                    BlockPanelAction::ToggleOutputDropdown,
                );
            });
        },
    );
}

fn spawn_labeler_panel(root: &mut ChildSpawnerCommands) {
    spawn_panel(
        root,
        PanelOptions::new(420.0, "labeler.title").closable().dynamic_title(),
        UiPanelBinding(UiPanelId::Labeler),
        |panel| {
            spawn_panel_row(panel, "panel.color", |row| {
                spawn_block_panel_dropdown(
                    row,
                    BlockPanelDropdown::LabelerColor,
                    BlockPanelAction::ToggleColorDropdown,
                );
            });
        },
    );
}

fn spawn_block_dropdown_layers(root: &mut ChildSpawnerCommands) {
    spawn_material_icon_dropdown_list(
        root,
        BlockPanelDropdown::GeneratorMaterial,
        material_options().map(|material| (material, BlockPanelAction::SetMaterial(material))),
    );
    spawn_material_icon_dropdown_list(
        root,
        BlockPanelDropdown::GoalMaterial,
        material_options().map(|material| (material, BlockPanelAction::SetMaterial(material))),
    );
    spawn_material_icon_dropdown_list(
        root,
        BlockPanelDropdown::ConverterInput,
        material_options().map(|material| (material, BlockPanelAction::SetInput(material))),
    );
    spawn_material_icon_dropdown_list(
        root,
        BlockPanelDropdown::ConverterOutput,
        material_options().map(|material| (material, BlockPanelAction::SetOutput(material))),
    );
    spawn_block_panel_dropdown_list(
        root,
        BlockPanelDropdown::LabelerColor,
        stamp_color_options().map(|(label, color)| (label, BlockPanelAction::SetColor(color))),
    );
    spawn_block_panel_dropdown_list(
        root,
        BlockPanelDropdown::TeleportPair,
        std::iter::empty::<(String, BlockPanelAction)>(),
    );
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
    label_key: &'static str,
    controls: impl FnOnce(&mut ChildSpawnerCommands),
) {
    panel.spawn(panel_row_node()).with_children(|row| {
        spawn_panel_label(row, label_key);
        controls(row);
    });
}

fn spawn_panel_label(row: &mut ChildSpawnerCommands, label_key: &'static str) {
    row.spawn((
        localized_text(label_key, 16.0, Color::srgb(0.86, 0.88, 0.86)),
        Node {
            width: Val::Px(110.0),
            ..default()
        },
    ));
}

fn material_options() -> impl Iterator<Item = MaterialKind> {
    MaterialKind::ALL.into_iter()
}

fn stamp_color_options() -> impl Iterator<Item = (String, StampColor)> {
    StampColor::ALL
        .into_iter()
        .map(|color| (i18n.t(color.name_key()), color))
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
        StatusText(StatusTextKind::Hotbar),
        InGameHudVisibility,
        GameplayHudVisibility,
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
        StatusText(StatusTextKind::CurrentSave),
        InGameHudVisibility,
        GameplayHudVisibility,
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
        StatusText(StatusTextKind::Simulation),
        InGameHudVisibility,
        GameplayHudVisibility,
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
        StatusText(StatusTextKind::SimulationOverlay),
        InGameHudVisibility,
    ));
}

fn spawn_confirm_dialog(root: &mut ChildSpawnerCommands) {
    root.spawn((
        panel_bundle(460.0),
        GlobalZIndex(0),
        PanelVisibility::ConfirmDialog,
    ))
    .with_children(|panel| {
        panel.spawn(panel_title_bar()).with_children(|title| {
            title.spawn((
                panel_title_label("", 24.0),
                ConfirmTitleText,
            ));
        });
        panel.spawn(panel_content()).with_children(|panel| {
            panel.spawn((
                text("", 15.0, STATUS_TEXT),
                TextLayout::new_with_justify(Justify::Center),
                Node {
                    min_height: Val::Px(54.0),
                    align_self: AlignSelf::Stretch,
                    ..default()
                },
                ConfirmMessageText,
            ));
            panel.spawn(flex_row(40.0, 8.0)).with_children(|row| {
                spawn_confirm_dialog_button(row, ConfirmButtonId::Confirm);
                spawn_confirm_dialog_button(row, ConfirmButtonId::Extra);
                spawn_confirm_dialog_button(row, ConfirmButtonId::Cancel);
            });
        });
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
                    row.spawn((full_width_button(34.0), TextPromptButtonId::Save))
                        .with_children(|button| {
                            button.spawn(text("", 15.0, Color::WHITE));
                        });
                    row.spawn((full_width_button(34.0), TextPromptButtonId::Cancel))
                        .with_children(|button| {
                            button.spawn(text("", 15.0, Color::WHITE));
                        });
                });
            });
        });
}
