use bevy::prelude::*;

use crate::shared::i18n::I18n;

use super::components::{
    absolute_text_bundle, default_button_size, localized_text, panel_bundle, panel_content,
    panel_title_bar, panel_title_label, root_node, spawn_panel, text, transparent_node,
    PanelOptions, STATUS_TEXT,
};
use super::screens::{
    spawn_carried_label, spawn_hotbar, spawn_inventory_panel, spawn_inventory_tooltip,
    spawn_main_menu, spawn_pause_panel, spawn_save_list, spawn_settings_panel,
};
use super::types::{
    BlockEditAction, BlockPanelDropdown, BlockPanelText, BlockPanelTextKind, ConfirmDialogAction,
    ConverterInputRow, Crosshair, GameplayHudVisibility, InGameHudVisibility, PanelText,
    PanelTextKind, PanelVisibility, StatusText, StatusTextKind, TeleportAction, UiPanelBinding,
    UiPanelId,
};
use super::widgets::{
    spawn_block_edit_button, spawn_block_panel_dropdown, spawn_block_panel_dropdown_list,
    spawn_confirm_dialog_button, spawn_material_icon_dropdown_list, spawn_material_icon_slot,
    spawn_teleport_button,
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
        spawn_modal_scrim(root);
        spawn_main_menu(root, &i18n);
        spawn_save_list(root, &i18n);
        spawn_carried_label(root);
        spawn_inventory_tooltip(root);
        spawn_block_dropdown_layers(root, &i18n);
    });
}

fn spawn_modal_scrim(root: &mut ChildSpawnerCommands) {
    root.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            display: Display::None,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.16)),
        Pickable {
            should_block_lower: true,
            is_hoverable: false,
        },
        GlobalZIndex(0),
        PanelVisibility::ModalScrim,
    ));
}

fn spawn_generator_panel(root: &mut ChildSpawnerCommands, i18n: &I18n) {
    spawn_panel(
        root,
        i18n,
        PanelOptions::new(430.0, "generator.title").closable(),
        UiPanelBinding(UiPanelId::Generator),
        |panel| {
            spawn_panel_row(panel, i18n, "panel.period", |row| {
                spawn_block_edit_button(row, BlockEditAction::PeriodDown, "button.period_down");
                row.spawn((
                    text("", 18.0, Color::WHITE),
                    BlockPanelText(BlockPanelTextKind::GeneratorPeriod),
                ));
                spawn_block_edit_button(row, BlockEditAction::PeriodUp, "button.period_up");
            });
            spawn_panel_row(panel, i18n, "panel.material", |row| {
                spawn_material_icon_slot(
                    row,
                    BlockPanelDropdown::GeneratorMaterial,
                    BlockEditAction::ToggleMaterialDropdown,
                );
            });
        },
    );
}

fn spawn_goal_panel(root: &mut ChildSpawnerCommands, i18n: &I18n) {
    spawn_panel(
        root,
        i18n,
        PanelOptions::new(430.0, "goal.title").closable(),
        UiPanelBinding(UiPanelId::Goal),
        |panel| {
            spawn_panel_row(panel, i18n, "panel.material", |row| {
                spawn_material_icon_slot(
                    row,
                    BlockPanelDropdown::GoalMaterial,
                    BlockEditAction::ToggleMaterialDropdown,
                );
            });
        },
    );
}

fn spawn_teleport_panel(root: &mut ChildSpawnerCommands, i18n: &I18n) {
    spawn_panel(
        root,
        i18n,
        PanelOptions::new(460.0, "teleport.title").closable(),
        UiPanelBinding(UiPanelId::Teleport),
        |panel| {
            spawn_panel_row(panel, i18n, "panel.name", |row| {
                spawn_teleport_button(row, TeleportAction::Rename, "button.teleport_rename");
                row.spawn((
                    text("", 18.0, Color::WHITE),
                    BlockPanelText(BlockPanelTextKind::TeleportName),
                ));
            });
            spawn_panel_row(panel, i18n, "panel.pair", |row| {
                spawn_block_panel_dropdown(
                    row,
                    BlockPanelDropdown::TeleportPair,
                    TeleportAction::TogglePairDropdown,
                );
            });
        },
    );
}

fn spawn_converter_panel(root: &mut ChildSpawnerCommands, i18n: &I18n) {
    spawn_panel(
        root,
        i18n,
        PanelOptions::new(460.0, "converter.title").closable(),
        UiPanelBinding(UiPanelId::Converter),
        |panel| {
            panel
                .spawn((panel_row_node(), ConverterInputRow))
                .with_children(|row| {
                    spawn_panel_label(row, i18n, "panel.input");
                    spawn_material_icon_slot(
                        row,
                        BlockPanelDropdown::ConverterInput,
                        BlockEditAction::ToggleInputDropdown,
                    );
                });
            spawn_panel_row(panel, i18n, "panel.output", |row| {
                spawn_material_icon_slot(
                    row,
                    BlockPanelDropdown::ConverterOutput,
                    BlockEditAction::ToggleOutputDropdown,
                );
            });
        },
    );
}

fn spawn_labeler_panel(root: &mut ChildSpawnerCommands, i18n: &I18n) {
    spawn_panel(
        root,
        i18n,
        PanelOptions::new(420.0, "labeler.title").closable(),
        UiPanelBinding(UiPanelId::Labeler),
        |panel| {
            spawn_panel_row(panel, i18n, "panel.color", |row| {
                spawn_block_panel_dropdown(
                    row,
                    BlockPanelDropdown::LabelerColor,
                    BlockEditAction::ToggleColorDropdown,
                );
            });
        },
    );
}

fn spawn_block_dropdown_layers(root: &mut ChildSpawnerCommands, i18n: &I18n) {
    spawn_material_icon_dropdown_list(
        root,
        BlockPanelDropdown::GeneratorMaterial,
        material_options().map(|material| (material, BlockEditAction::SetMaterial(material))),
    );
    spawn_material_icon_dropdown_list(
        root,
        BlockPanelDropdown::GoalMaterial,
        material_options().map(|material| (material, BlockEditAction::SetMaterial(material))),
    );
    spawn_material_icon_dropdown_list(
        root,
        BlockPanelDropdown::ConverterInput,
        material_options().map(|material| (material, BlockEditAction::SetInput(material))),
    );
    spawn_material_icon_dropdown_list(
        root,
        BlockPanelDropdown::ConverterOutput,
        material_options().map(|material| (material, BlockEditAction::SetOutput(material))),
    );
    spawn_block_panel_dropdown_list(
        root,
        BlockPanelDropdown::LabelerColor,
        stamp_color_options(i18n).map(|(label, color)| (label, BlockEditAction::SetColor(color))),
    );
    spawn_block_panel_dropdown_list(
        root,
        BlockPanelDropdown::TeleportPair,
        std::iter::empty::<(String, TeleportAction)>(),
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
    i18n: &I18n,
    text_key: &'static str,
    controls: impl FnOnce(&mut ChildSpawnerCommands),
) {
    panel.spawn(panel_row_node()).with_children(|row| {
        spawn_panel_label(row, i18n, text_key);
        controls(row);
    });
}

fn spawn_panel_label(row: &mut ChildSpawnerCommands, i18n: &I18n, text_key: &'static str) {
    row.spawn((
        localized_text(i18n, text_key, 16.0, Color::srgb(0.86, 0.88, 0.86)),
        Node {
            width: Val::Px(110.0),
            ..default()
        },
    ));
}

fn material_options() -> impl Iterator<Item = MaterialKind> {
    MaterialKind::ALL.into_iter()
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
        panel_bundle(620.0),
        GlobalZIndex(0),
        PanelVisibility::ConfirmDialog,
    ))
    .with_children(|panel| {
        panel.spawn(panel_title_bar()).with_children(|title| {
            title.spawn((
                panel_title_label("", 24.0),
                PanelText(PanelTextKind::ConfirmTitle),
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
                PanelText(PanelTextKind::ConfirmMessage),
            ));
            panel
                .spawn(confirm_dialog_actions_row())
                .with_children(|row| {
                    spawn_confirm_dialog_button(
                        row,
                        ConfirmDialogAction::Primary,
                        "button.confirm",
                    );
                    spawn_confirm_dialog_button(
                        row,
                        ConfirmDialogAction::Secondary,
                        "button.confirm",
                    );
                    spawn_confirm_dialog_button(row, ConfirmDialogAction::Cancel, "button.cancel");
                });
        });
    });
}

fn confirm_dialog_actions_row() -> impl Bundle {
    transparent_node(Node {
        width: Val::Percent(100.0),
        height: Val::Px(default_button_size(40.0)),
        display: Display::Flex,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        column_gap: Val::Px(8.0),
        ..default()
    })
}
