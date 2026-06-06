use bevy::prelude::*;

use crate::game::ui::access::{bind_ui_scope, i18n};

use super::components::{
    absolute_text_bundle, default_button_size, flex_row, full_width_button, localized_text,
    panel_bundle, panel_content, panel_title_bar, panel_title_label, raised_border, root_node,
    spawn_panel, styled_button, text, transparent_node, PanelOptions, STATUS_TEXT, BUTTON_BG,
};
use crate::game::blocks::panels::{spawn_all_overlays, spawn_all_panels};
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
use super::widgets::spawn_confirm_dialog_button;
use super::types::{
    Crosshair, GameplayHudVisibility, InGameHudVisibility, PanelVisibility,
    PlayingUiRoot, StatusText, StatusTextKind, UiPanelBinding, UiRoot,
};

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
