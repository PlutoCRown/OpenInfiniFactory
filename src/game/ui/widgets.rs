use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use bevy::ui_widgets::{Slider, SliderRange, SliderThumb, SliderValue, TrackClick};

use super::components::{
    default_button_size, default_font_size, inset_border, menu_button, raised_border,
};
use super::types::{
    AreaKind, BlockEditAction, BlockPanelDropdown, BlockPanelDropdownLabel, BlockPanelDropdownList,
    ConfirmDialogAction, InventoryItem, InventorySlot, KeyBindingButton,
    MenuAction, SaveListAction, ScrollContainer,
    ScrollContent, SettingsAction, SettingsDropdown, SettingsDropdownLabel, SettingsDropdownList,
    SettingsDropdownRoot, SettingsField, SettingsSliderFill, SettingsSliderKnob, SettingsText,
    SettingsTextKind, SettingsValueText, SlotArea, TeleportAction, UiActionLabel,
};

fn label_text(value: impl Into<String>, font_size: f32, color: Color) -> impl Bundle {
    (
        Text::new(value),
        TextFont {
            font_size: default_font_size(font_size),
            ..default()
        },
        TextColor(color),
    )
}

fn styled_button(style: Node, border: impl Into<BorderColor>, background: Color) -> impl Bundle {
    (Button, style, border.into(), BackgroundColor(background))
}

fn plain_node(style: Node) -> impl Bundle {
    (style, BackgroundColor(Color::NONE))
}

pub(super) fn spawn_slot(parent: &mut ChildSpawnerCommands, area: SlotArea, index: usize) {
    parent
        .spawn((
            styled_button(
                Node {
                    width: Val::Px(default_button_size(54.0)),
                    height: Val::Px(default_button_size(54.0)),
                    border: UiRect {
                        left: Val::Px(4.0),
                        right: Val::Px(4.0),
                        top: Val::Px(5.0),
                        bottom: Val::Px(5.0),
                    },
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                inset_border(),
                Color::srgb(0.255, 0.251, 0.251),
            ),
            BoxShadow::new(
                Color::srgba(0.0, 0.0, 0.0, 0.50),
                Val::Px(0.0),
                Val::Px(0.0),
                Val::Px(0.0),
                Val::Px(3.0),
            ),
            InventorySlot { area, index },
        ))
        .with_children(|slot| {
            slot.spawn((
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
            slot.spawn((
                label_text("", 12.0, Color::WHITE),
                TextLayout::new_with_justify(Justify::Center),
                Node {
                    margin: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
            ));
        });
}

pub(super) fn spawn_menu_button(
    parent: &mut ChildSpawnerCommands,
    height: f32,
    font_size: f32,
    action: MenuAction,
) {
    spawn_localized_button(parent, height, font_size, action);
}

pub(super) fn spawn_block_edit_button(parent: &mut ChildSpawnerCommands, action: BlockEditAction) {
    spawn_localized_button(parent, 36.0, 14.0, action);
}

pub(super) fn spawn_teleport_button(parent: &mut ChildSpawnerCommands, action: TeleportAction) {
    spawn_localized_button(parent, 36.0, 14.0, action);
}

pub(super) fn spawn_close_button<A>(parent: &mut ChildSpawnerCommands, action: A)
where
    A: Component + Copy + UiActionLabel,
{
    let key = action.label_key();
    parent
        .spawn((
            styled_button(
                Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(10.0),
                    top: Val::Px(10.0),
                    width: Val::Px(34.0),
                    height: Val::Px(34.0),
                    border: UiRect::all(Val::Px(2.0)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                raised_border(),
                Color::srgb(0.255, 0.251, 0.251),
            ),
            action,
        ))
        .with_children(|button| {
            button.spawn(label_text(key, 12.0, Color::WHITE));
        });
}

pub(super) fn spawn_block_panel_dropdown<A>(
    parent: &mut ChildSpawnerCommands,
    dropdown: BlockPanelDropdown,
    toggle_action: A,
    options: impl IntoIterator<Item = (String, A)>,
) where
    A: Component + Copy,
{
    parent
        .spawn((
            plain_node(Node {
                width: Val::Px(230.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                position_type: PositionType::Relative,
                ..default()
            }),
            ZIndex(300),
        ))
        .with_children(|container| {
            container
                .spawn((
                    styled_button(
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(default_button_size(34.0)),
                            padding: UiRect::horizontal(Val::Px(12.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::SpaceBetween,
                            ..default()
                        },
                        Color::srgb(0.38, 0.39, 0.40),
                        Color::srgba(0.18, 0.20, 0.22, 0.96),
                    ),
                    toggle_action,
                ))
                .with_children(|button| {
                    button.spawn((
                        label_text("", 14.0, Color::WHITE),
                        BlockPanelDropdownLabel(dropdown),
                    ));
                    button.spawn(label_text("v", 12.0, Color::srgb(0.72, 0.80, 0.84)));
                });
            container
                .spawn((
                    (
                        Node {
                            width: Val::Percent(100.0),
                            display: Display::None,
                            position_type: PositionType::Absolute,
                            top: Val::Px(default_button_size(38.0)),
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(3.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.10, 0.11, 0.12, 0.98)),
                        ZIndex(500),
                    ),
                    BlockPanelDropdownList(dropdown),
                ))
                .with_children(|list| {
                    for (label, action) in options {
                        spawn_dropdown_option(list, label, action);
                    }
                });
        });
}

pub(super) fn spawn_confirm_dialog_button(
    parent: &mut ChildSpawnerCommands,
    action: ConfirmDialogAction,
) {
    let key = action.label_key();
    parent
        .spawn((menu_button(34.0), action))
        .with_children(|button| {
            button.spawn((
                label_text(key, 15.0, Color::WHITE),
                super::types::LocalizedText { key },
            ));
        });
}

pub(super) fn spawn_localized_settings_button(
    parent: &mut ChildSpawnerCommands,
    action: SettingsAction,
) {
    let is_binding = matches!(action, SettingsAction::Bind(_));
    let key = action.label_key();
    let mut button = parent.spawn((menu_button(36.0), action));
    if let SettingsAction::Bind(action) = action {
        button.insert(KeyBindingButton(action));
    }
    button.with_children(|button| {
        let mut label_entity = button.spawn((
            label_text(key, 14.0, Color::WHITE),
            super::types::LocalizedText { key },
        ));
        if is_binding {
            label_entity.insert(SettingsText(SettingsTextKind::KeyBinding));
        }
    });
}

pub(super) fn spawn_settings_tab(parent: &mut ChildSpawnerCommands, action: SettingsAction) {
    let key = action.label_key();
    parent
        .spawn((
            styled_button(
                Node {
                    min_width: Val::Px(default_button_size(150.0)),
                    height: Val::Px(default_button_size(38.0)),
                    padding: UiRect::horizontal(Val::Px(18.0)),
                    border: UiRect {
                        left: Val::Px(1.0),
                        right: Val::Px(1.0),
                        top: Val::Px(1.0),
                        bottom: Val::Px(3.0),
                    },
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                raised_border(),
                Color::srgb(0.255, 0.251, 0.251),
            ),
            BoxShadow::new(
                Color::srgba(0.0, 0.0, 0.0, 0.45),
                Val::Px(0.0),
                Val::Px(0.0),
                Val::Px(0.0),
                Val::Px(3.0),
            ),
            action,
        ))
        .with_children(|tab| {
            tab.spawn((
                label_text(key, 15.0, Color::WHITE),
                super::types::LocalizedText { key },
            ));
        });
}

pub(super) fn spawn_settings_slider(parent: &mut ChildSpawnerCommands, field: SettingsField) {
    parent
        .spawn((
            styled_button(
                Node {
                    width: Val::Px(360.0),
                    height: Val::Px(default_button_size(22.0)),
                    padding: UiRect::all(Val::Px(3.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    align_items: AlignItems::Center,
                    ..default()
                },
                inset_border(),
                Color::srgb(0.255, 0.251, 0.251),
            ),
            Slider {
                track_click: TrackClick::Snap,
            },
            SliderValue(0.0),
            SliderRange::new(0.0, 100.0),
            SettingsAction::Field(field),
        ))
        .with_children(|track| {
            track.spawn((
                (
                    Node {
                        width: Val::Percent(50.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.32, 0.62, 0.72)),
                ),
                SettingsSliderFill(field),
                Pickable::IGNORE,
            ));
            track.spawn((
                (
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Percent(50.0),
                        top: Val::Px(3.0),
                        bottom: Val::Px(3.0),
                        width: Val::Px(6.0),
                        margin: UiRect {
                            left: Val::Px(-3.0),
                            ..default()
                        },
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.90, 0.96, 1.0)),
                ),
                SettingsSliderKnob(field),
                SliderThumb,
                Pickable::IGNORE,
            ));
        });
}

pub(super) fn spawn_settings_slider_value(
    parent: &mut ChildSpawnerCommands,
    field: SettingsField,
) {
    parent.spawn((
        label_text("", 13.0, Color::srgb(0.88, 0.94, 0.96)),
        TextLayout::new_with_justify(Justify::Right),
        Node {
            width: Val::Px(130.0),
            align_self: AlignSelf::Center,
            ..default()
        },
        SettingsValueText(field),
    ));
}

pub(super) fn spawn_settings_dropdown(
    parent: &mut ChildSpawnerCommands,
    dropdown: SettingsDropdown,
) {
    parent
        .spawn((
            plain_node(Node {
                width: Val::Px(260.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                position_type: PositionType::Relative,
                ..default()
            }),
            ZIndex(300),
            SettingsDropdownRoot(dropdown),
        ))
        .with_children(|container| {
            container
                .spawn((
                    styled_button(
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(default_button_size(36.0)),
                            padding: UiRect::horizontal(Val::Px(12.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::SpaceBetween,
                            ..default()
                        },
                        Color::srgb(0.38, 0.39, 0.40),
                        Color::srgba(0.18, 0.20, 0.22, 0.96),
                    ),
                    SettingsAction::ToggleDropdown(dropdown),
                ))
                .with_children(|button| {
                    button.spawn((
                        label_text("", 14.0, Color::WHITE),
                        SettingsDropdownLabel(dropdown),
                    ));
                    button.spawn(label_text("v", 12.0, Color::srgb(0.72, 0.80, 0.84)));
                });
            container
                .spawn((
                    (
                        Node {
                            width: Val::Percent(100.0),
                            display: Display::None,
                            position_type: PositionType::Absolute,
                            top: Val::Px(default_button_size(40.0)),
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(3.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.10, 0.11, 0.12, 0.98)),
                        ZIndex(500),
                    ),
                    SettingsDropdownList(dropdown),
                ))
                .with_children(|list| match dropdown {
                    SettingsDropdown::Language => {
                        for language in crate::shared::i18n::Language::ALL {
                            spawn_dropdown_option(
                                list,
                                language.native_name().to_string(),
                                SettingsAction::SetLanguage(language),
                            );
                        }
                    }
                    SettingsDropdown::PlaceSelectionMode => {
                        for mode in crate::shared::config::ConfigSelectionMode::ALL {
                            spawn_localized_dropdown_option(
                                list,
                                mode.label_key(),
                                SettingsAction::SetPlaceSelectionMode(mode),
                            );
                        }
                    }
                    SettingsDropdown::DeleteSelectionMode => {
                        for mode in crate::shared::config::ConfigSelectionMode::ALL {
                            spawn_localized_dropdown_option(
                                list,
                                mode.label_key(),
                                SettingsAction::SetDeleteSelectionMode(mode),
                            );
                        }
                    }
                });
        });
}

fn spawn_dropdown_option<A>(parent: &mut ChildSpawnerCommands, label: String, action: A)
where
    A: Component + Copy,
{
    parent
        .spawn((menu_button(32.0), action))
        .with_children(|button| {
            button.spawn(label_text(label, 13.0, Color::WHITE));
        });
}

fn spawn_localized_dropdown_option(
    parent: &mut ChildSpawnerCommands,
    key: &'static str,
    action: SettingsAction,
) {
    parent
        .spawn((menu_button(32.0), action))
        .with_children(|button| {
            button.spawn((
                label_text(key, 13.0, Color::WHITE),
                super::types::LocalizedText { key },
            ));
        });
}

pub(super) fn scroll_container(height: f32) -> (impl Bundle, ScrollContainer) {
    (
        (
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(height),
                position_type: PositionType::Relative,
                overflow: Overflow::clip_y(),
                ..default()
            },
            BackgroundColor(Color::NONE),
        ),
        ScrollContainer {
            offset: 0.0,
            max_offset: 0.0,
        },
    )
}

pub(super) fn scroll_content() -> ScrollContent {
    ScrollContent
}

pub(super) fn spawn_save_slot_button(parent: &mut ChildSpawnerCommands, action: SaveListAction) {
    parent
        .spawn((menu_button(34.0), action))
        .with_children(|button| {
            button.spawn(label_text("", 15.0, Color::WHITE));
        });
}

pub(super) fn spawn_save_row_button(
    parent: &mut ChildSpawnerCommands,
    action: SaveListAction,
    width: f32,
) {
    parent
        .spawn((
            styled_button(
                Node {
                    width: Val::Px(default_button_size(width)),
                    min_width: Val::Px(default_button_size(width)),
                    height: Val::Px(default_button_size(30.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                super::components::BUTTON_BORDER,
                super::components::BUTTON_BG,
            ),
            action,
        ))
        .with_children(|button| {
            button.spawn(label_text("", 13.0, Color::WHITE));
        });
}

pub(super) fn spawn_save_back_button(parent: &mut ChildSpawnerCommands) {
    parent
        .spawn((menu_button(38.0), SaveListAction::Back))
        .with_children(|button| {
            button.spawn(label_text("", 16.0, Color::WHITE));
        });
}

fn spawn_localized_button<'a, A>(
    parent: &'a mut ChildSpawnerCommands,
    height: f32,
    font_size: f32,
    action: A,
) -> EntityCommands<'a>
where
    A: Bundle + Copy + UiActionLabel,
{
    let key = action.label_key();
    let mut entity = parent.spawn((menu_button(height), action));
    entity.with_children(|button| {
        button.spawn((
            label_text(key, font_size, Color::WHITE),
            super::types::LocalizedText { key },
        ));
    });
    entity
}

pub(super) fn slot_color(item: InventoryItem) -> Color {
    match item {
        InventoryItem::Area(AreaKind::Selection) => Color::srgb(0.22, 0.66, 0.62),
        InventoryItem::Block(kind) => kind.slot_color(),
    }
}

pub(super) fn short_item_name(item: InventoryItem) -> &'static str {
    match item {
        InventoryItem::Area(AreaKind::Selection) => "short.area.selection",
        InventoryItem::Block(kind) => kind.short_name_key(),
    }
}
