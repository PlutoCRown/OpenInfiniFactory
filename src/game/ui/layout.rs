use bevy::prelude::*;

use crate::game::ui::access::bind_ui_scope;

use super::components::{
    auto_width_button, default_button_size, flex_row_auto, panel_bundle_auto, panel_content,
    panel_title_bar, panel_title_label, raised_border, root_node, styled_button, text, BUTTON_BG,
    STATUS_TEXT,
};
use super::screens::{
    spawn_carried_label, spawn_hotbar, spawn_inventory_panel, spawn_inventory_tooltip,
    spawn_main_menu, spawn_pause_panel, spawn_save_list,
};
use super::types::{
    Crosshair, InGameHudVisibility, PanelVisibility, PlayingUiRoot, StatusText, StatusTextKind,
    UiRoot,
};
use super::widgets::spawn_confirm_dialog_button;
use crate::game::blocks::panels::{spawn_all_overlays, spawn_all_panels};
use crate::game::ui::core::confirm_dialog::{
    ConfirmButtonId, ConfirmMessageText, ConfirmTitleText,
};
use crate::game::ui::core::host::{PlayingUiRootEntity, UiRootEntity};
use crate::game::ui::core::text_prompt::{TextPromptButtonId, TextPromptRoot, TextPromptText};

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
            spawn_all_panels(root);
            spawn_pause_panel(root);
            spawn_carried_label(root);
            spawn_inventory_tooltip(root);
            spawn_all_overlays(root);
        })
        .id();
    commands.insert_resource(PlayingUiRootEntity(root));
}

pub fn setup_playing_ui_system(world: &mut World) {
    bind_ui_scope(world);
    let mut commands = world.commands();
    setup_playing_ui(&mut commands);
}

const CROSSHAIR_ARM: f32 = 12.0;
const CROSSHAIR_THICKNESS: f32 = 2.0;

fn spawn_crosshair(root: &mut ChildSpawnerCommands) {
    let offset = (CROSSHAIR_ARM - CROSSHAIR_THICKNESS) * 0.5;

    root.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            display: Display::Flex,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        Crosshair,
        InGameHudVisibility,
        Pickable {
            should_block_lower: false,
            is_hoverable: false,
        },
    ))
    .with_children(|overlay| {
        overlay
            .spawn(Node {
                width: Val::Px(CROSSHAIR_ARM),
                height: Val::Px(CROSSHAIR_ARM),
                ..default()
            })
            .with_children(|mark| {
                mark.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(0.0),
                        top: Val::Px(offset),
                        width: Val::Px(CROSSHAIR_ARM),
                        height: Val::Px(CROSSHAIR_THICKNESS),
                        ..default()
                    },
                    BackgroundColor(Color::WHITE),
                ));
                mark.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(offset),
                        top: Val::Px(0.0),
                        width: Val::Px(CROSSHAIR_THICKNESS),
                        height: Val::Px(CROSSHAIR_ARM),
                        ..default()
                    },
                    BackgroundColor(Color::WHITE),
                ));
            });
    });
}

fn spawn_status_overlays(root: &mut ChildSpawnerCommands) {
    spawn_crosshair(root);
    root.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(18.0),
            top: Val::Px(18.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(6.0),
            ..default()
        },
        InGameHudVisibility,
    ))
    .with_children(|column| {
        column.spawn((
            text("", 15.0, STATUS_TEXT),
            StatusText(StatusTextKind::Summary),
        ));
        column.spawn((
            text("", 15.0, Color::srgb(0.82, 0.92, 1.0)),
            StatusText(StatusTextKind::TargetBlock),
        ));
    });
    root.spawn((
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(18.0),
            top: Val::Px(18.0),
            max_width: Val::Px(440.0),
            ..default()
        },
        InGameHudVisibility,
    ))
    .with_children(|panel| {
        panel.spawn((
            text("", 14.0, STATUS_TEXT),
            StatusText(StatusTextKind::TargetMovement),
        ));
    });
}

fn spawn_confirm_dialog(root: &mut ChildSpawnerCommands) {
    root.spawn((
        panel_bundle_auto(560.0),
        GlobalZIndex(0),
        PanelVisibility::ConfirmDialog,
    ))
    .with_children(|panel| {
        panel.spawn(panel_title_bar()).with_children(|title| {
            title.spawn((panel_title_label("", 24.0), ConfirmTitleText));
        });
        panel.spawn(panel_content()).with_children(|panel| {
            panel.spawn((
                text("", 15.0, STATUS_TEXT),
                TextLayout::new_with_justify(Justify::Center),
                Node {
                    width: Val::Auto,
                    max_width: Val::Px(520.0),
                    min_height: Val::Px(54.0),
                    align_self: AlignSelf::Center,
                    ..default()
                },
                ConfirmMessageText,
            ));
            panel.spawn(flex_row_auto(34.0, 8.0)).with_children(|row| {
                spawn_confirm_dialog_button(row, ConfirmButtonId::Confirm);
                spawn_confirm_dialog_button(row, ConfirmButtonId::Extra);
                spawn_confirm_dialog_button(row, ConfirmButtonId::Cancel);
            });
        });
    });
}

fn spawn_text_prompt(root: &mut ChildSpawnerCommands) {
    root.spawn((
        panel_bundle_auto(420.0),
        GlobalZIndex(30_000),
        TextPromptRoot,
    ))
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
            content
                .spawn(flex_row_auto(34.0, 8.0))
                .with_children(|row| {
                    row.spawn((auto_width_button(34.0), TextPromptButtonId::Save))
                        .with_children(|button| {
                            button.spawn((
                                text("", 15.0, Color::WHITE),
                                TextLayout::new_with_no_wrap(),
                            ));
                        });
                    row.spawn((auto_width_button(34.0), TextPromptButtonId::Cancel))
                        .with_children(|button| {
                            button.spawn((
                                text("", 15.0, Color::WHITE),
                                TextLayout::new_with_no_wrap(),
                            ));
                        });
                });
        });
    });
}
