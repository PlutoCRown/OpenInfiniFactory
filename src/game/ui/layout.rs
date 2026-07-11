use bevy::prelude::*;
use bevy::text::{EditableText, TextCursorStyle};
use bevy::ui::widget::TextScroll;

use crate::game::ui::access::bind_ui_scope;

use super::components::{
    absolute_text_bundle, auto_width_button, default_button_size, default_font_size, flex_row_auto,
    panel_bundle_auto, panel_content, panel_title_bar, panel_title_label, raised_border, root_node,
    text, BUTTON_BG, STATUS_TEXT,
};
use super::screens::{
    spawn_carried_label, spawn_hotbar, spawn_inventory_panel, spawn_inventory_tooltip,
    spawn_main_menu, spawn_pause_panel, spawn_save_list,
};
use super::types::{
    Crosshair, GameplayHudVisibility, InGameHudVisibility, PanelVisibility, PlayingUiRoot,
    StatusText, StatusTextKind, UiRoot,
};
use crate::game::blocks::panels::{spawn_all_overlays, spawn_all_panels};
use crate::game::ui::core::confirm_dialog::{
    ConfirmButtonId, ConfirmMessageText, ConfirmTitleText,
};
use crate::game::cameras::{GameplayViewBackdrop, GameplayViewImage};
use crate::game::ui::core::host::{PlayingUiRootEntity, UiRootEntity};
use crate::game::ui::core::text_prompt::{
    TextPromptButtonId, TextPromptInput, TextPromptRoot, TextPromptTitle,
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


fn spawn_gameplay_view_backdrop(root: &mut ChildSpawnerCommands, image: Handle<Image>) {
    root.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            ..default()
        },
        ImageNode::new(image),
        GameplayViewBackdrop,
        Pickable::IGNORE,
    ));
}

pub fn setup_playing_ui(commands: &mut Commands, view_image: Handle<Image>) {
    let root = commands
        .spawn((root_node(), PlayingUiRoot))
        .with_children(|root| {
            spawn_gameplay_view_backdrop(root, view_image);
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
    let Some(view) = world.get_resource::<GameplayViewImage>() else {
        return;
    };
    let image = view.0.clone();
    let mut commands = world.commands();
    setup_playing_ui(&mut commands, image);
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
            15.0,
            Color::srgb(0.82, 0.92, 1.0),
            Some(Val::Px(18.0)),
            None,
            Some(Val::Px(156.0)),
            None,
        ),
        StatusText(StatusTextKind::TargetBlock),
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
        StatusText(StatusTextKind::SimulationOverlay),
        InGameHudVisibility,
    ));
}

fn spawn_confirm_dialog(root: &mut ChildSpawnerCommands) {
    root.spawn((
        panel_bundle_auto(560.0),
        GlobalZIndex(0),
        PanelVisibility::ConfirmDialog,
        children![
            (
                panel_title_bar(),
                children![(panel_title_label("", 24.0), ConfirmTitleText)]
            ),
            (
                panel_content(),
                children![
                    (
                        text("", 15.0, STATUS_TEXT),
                        TextLayout::justify(Justify::Center),
                        Node {
                            width: Val::Auto,
                            max_width: Val::Px(520.0),
                            min_height: Val::Px(54.0),
                            align_self: AlignSelf::Center,
                            ..default()
                        },
                        ConfirmMessageText,
                    ),
                    (
                        flex_row_auto(34.0, 8.0),
                        children![
                            (
                                auto_width_button(34.0),
                                ConfirmButtonId::Confirm,
                                children![(text("", 15.0, Color::WHITE), TextLayout::no_wrap())]
                            ),
                            (
                                auto_width_button(34.0),
                                ConfirmButtonId::Extra,
                                children![(text("", 15.0, Color::WHITE), TextLayout::no_wrap())]
                            ),
                            (
                                auto_width_button(34.0),
                                ConfirmButtonId::Cancel,
                                children![(text("", 15.0, Color::WHITE), TextLayout::no_wrap())]
                            ),
                        ]
                    ),
                ]
            ),
        ],
    ));
}

fn spawn_text_prompt(root: &mut ChildSpawnerCommands) {
    root.spawn((
        panel_bundle_auto(420.0),
        GlobalZIndex(30_000),
        TextPromptRoot,
        children![
            (
                panel_title_bar(),
                children![(panel_title_label("", 20.0), TextPromptTitle)]
            ),
            (
                panel_content(),
                children![
                    (
                        Node {
                            width: Val::Percent(100.0),
                            min_height: Val::Px(default_button_size(38.0)),
                            padding: UiRect::horizontal(Val::Px(12.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            align_items: AlignItems::Center,
                            overflow: Overflow::clip(),
                            ..default()
                        },
                        raised_border(),
                        BackgroundColor(BUTTON_BG),
                        EditableText {
                            max_characters: Some(24),
                            allow_newlines: false,
                            visible_lines: Some(1.0),
                            ..EditableText::new("")
                        },
                        TextScroll::default(),
                        TextFont {
                            font_size: default_font_size(16.0),
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        TextLayout::no_wrap(),
                        TextCursorStyle::default(),
                        TextPromptInput,
                    ),
                    (
                        flex_row_auto(34.0, 8.0),
                        children![
                            (
                                auto_width_button(34.0),
                                TextPromptButtonId::Save,
                                children![(text("", 15.0, Color::WHITE), TextLayout::no_wrap())]
                            ),
                            (
                                auto_width_button(34.0),
                                TextPromptButtonId::Cancel,
                                children![(text("", 15.0, Color::WHITE), TextLayout::no_wrap())]
                            ),
                        ]
                    ),
                ]
            ),
        ],
    ));
}
