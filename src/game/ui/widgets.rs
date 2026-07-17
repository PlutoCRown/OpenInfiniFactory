use bevy::prelude::*;

use crate::game::blocks::BlockPresent;
use crate::game::state::GameSettings;

use super::components::{
    default_button_size, default_font_size, full_width_button, inset_border, localized_text,
    menu_button, raised_border, slider_bundle, slider_fill, slider_knob, styled_button,
    text_button, BUTTON_BG,
};
use super::types::{
    AreaKind, InventoryItem, InventorySlot, KeyBindingButton, SettingsAction, SettingsDropdown,
    SettingsDropdownLabel, SettingsDropdownList, SettingsField, SettingsSliderFill,
    SettingsSliderKnob, SettingsText, SettingsTextKind, SettingsValueText, SlotArea, UiActionLabel,
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

fn plain_node(style: Node) -> impl Bundle {
    (style, BackgroundColor(Color::NONE))
}

pub(super) fn spawn_slot(parent: &mut ChildSpawnerCommands, area: SlotArea, index: usize) {
    const SLOT_BORDER: f32 = 3.0;

    parent
        .spawn((
            styled_button(
                Node {
                    width: Val::Px(default_button_size(54.0)),
                    height: Val::Px(default_button_size(54.0)),
                    border: UiRect::all(Val::Px(SLOT_BORDER)),
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
                Pickable::IGNORE,
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
                Pickable::IGNORE,
                TextLayout::justify(Justify::Center),
                Node {
                    margin: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
            ));
        });
}

pub(super) fn spawn_localized_settings_button(
    parent: &mut ChildSpawnerCommands,
    action: SettingsAction,
) {
    let is_binding = matches!(action, SettingsAction::Bind(_));
    let is_debug_http = matches!(action, SettingsAction::StartDebugHttp);
    let key = action.label_key();
    let mut button = parent.spawn((full_width_button(36.0), action));
    if let SettingsAction::Bind(action) = action {
        button.insert(KeyBindingButton(action));
    }
    button.with_children(|button| {
        if is_binding {
            button.spawn((
                label_text("", 14.0, Color::WHITE),
                SettingsText(SettingsTextKind::KeyBinding),
            ));
        } else if is_debug_http {
            button.spawn((
                label_text("", 14.0, Color::WHITE),
                SettingsText(SettingsTextKind::DebugHttp),
            ));
        } else {
            button.spawn(localized_text(key, 14.0, Color::WHITE));
        }
    });
}

pub(super) fn spawn_settings_tab(parent: &mut ChildSpawnerCommands, action: SettingsAction) {
    let key = action.label_key();
    parent
        .spawn((
            text_button(
                Node {
                    min_width: Val::Px(default_button_size(150.0)),
                    height: Val::Px(default_button_size(38.0)),
                    ..default()
                },
                raised_border(),
                BUTTON_BG,
            ),
            action,
        ))
        .with_children(|tab| {
            tab.spawn(localized_text(key, 15.0, Color::WHITE));
        });
}

pub(super) fn spawn_settings_slider(
    parent: &mut ChildSpawnerCommands,
    field: SettingsField,
    settings: &GameSettings,
) {
    let percent = field.percent(settings);
    parent
        .spawn(slider_bundle(SettingsAction::Field(field), percent))
        .with_children(|track| {
            track.spawn((slider_fill(percent), SettingsSliderFill(field)));
            track.spawn((slider_knob(percent), SettingsSliderKnob(field)));
        });
}

pub(super) fn spawn_settings_slider_value(parent: &mut ChildSpawnerCommands, field: SettingsField) {
    parent
        .spawn(plain_node(Node {
            width: Val::Px(130.0),
            min_width: Val::Px(130.0),
            height: Val::Percent(100.0),
            flex_shrink: 0.0,
            display: Display::Flex,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::FlexEnd,
            ..default()
        }))
        .with_children(|cell| {
            cell.spawn((
                label_text("", 13.0, Color::srgb(0.88, 0.94, 0.96)),
                TextLayout::justify(Justify::Right),
                SettingsValueText(field),
            ));
        });
}

const DROPDOWN_CHEVRON_COLOR: Color = Color::srgb(0.72, 0.80, 0.84);

pub(super) fn spawn_settings_dropdown(
    parent: &mut ChildSpawnerCommands,
    dropdown: SettingsDropdown,
) {
    parent
        .spawn((
            plain_node(Node {
                width: Val::Px(260.0),
                position_type: PositionType::Relative,
                ..default()
            }),
            ZIndex(300),
        ))
        .with_children(|container| {
            container
                .spawn((
                    text_button(
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(default_button_size(36.0)),
                            ..default()
                        },
                        raised_border(),
                        BUTTON_BG,
                    ),
                    SettingsAction::ToggleDropdown(dropdown),
                ))
                .with_children(|button| {
                    button
                        .spawn(plain_node(Node {
                            width: Val::Percent(100.0),
                            display: Display::Flex,
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::SpaceBetween,
                            column_gap: Val::Px(10.0),
                            ..default()
                        }))
                        .with_children(|row| {
                            row.spawn((
                                label_text("", 14.0, Color::WHITE),
                                SettingsDropdownLabel(dropdown),
                                Node {
                                    flex_grow: 1.0,
                                    flex_shrink: 1.0,
                                    min_width: Val::Px(0.0),
                                    overflow: Overflow::clip(),
                                    ..default()
                                },
                            ));
                            spawn_dropdown_chevron(row);
                        });
                });
        });
}

fn spawn_dropdown_chevron(parent: &mut ChildSpawnerCommands) {
    parent
        .spawn(plain_node(Node {
            width: Val::Px(12.0),
            height: Val::Px(8.0),
            flex_shrink: 0.0,
            position_type: PositionType::Relative,
            ..default()
        }))
        .with_children(|chevron| {
            chevron.spawn((
                dropdown_chevron_line(Val::Px(0.0), Val::Px(1.0), 45.0),
                Pickable::IGNORE,
            ));
            chevron.spawn((
                dropdown_chevron_line(Val::Auto, Val::Px(1.0), -45.0),
                Pickable::IGNORE,
            ));
        });
}

fn dropdown_chevron_line(anchor: Val, top: Val, degrees: f32) -> impl Bundle {
    let mut node = Node {
        position_type: PositionType::Absolute,
        top,
        width: Val::Px(7.0),
        height: Val::Px(2.0),
        ..default()
    };
    if matches!(anchor, Val::Auto) {
        node.right = Val::Px(0.0);
    } else {
        node.left = anchor;
    }

    (
        node,
        BackgroundColor(DROPDOWN_CHEVRON_COLOR),
        UiTransform::from_rotation(Rot2::degrees(degrees)),
    )
}

pub(super) fn spawn_settings_dropdown_list(
    parent: &mut ChildSpawnerCommands,
    dropdown: SettingsDropdown,
    options: impl IntoIterator<Item = (String, SettingsAction)>,
) {
    parent
        .spawn((
            dropdown_list_node(260.0),
            GlobalZIndex(20_000),
            SettingsDropdownList(dropdown),
        ))
        .with_children(|list| {
            for (label, action) in options {
                spawn_dropdown_option(list, label, action);
            }
        });
}

fn dropdown_list_node(width: f32) -> impl Bundle {
    (
        Node {
            width: Val::Px(width),
            display: Display::None,
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(3.0),
            padding: UiRect::all(Val::Px(4.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.10, 0.11, 0.12, 0.98)),
    )
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

pub(super) fn slot_color(item: InventoryItem) -> Color {
    match item {
        InventoryItem::Area(AreaKind::Selection) => Color::srgb(0.22, 0.66, 0.62),
        InventoryItem::LightPanel => Color::srgb(0.92, 0.82, 0.28),
        InventoryItem::Block(kind) => kind.item_slot_color(),
    }
}

pub(super) fn short_item_name(item: InventoryItem) -> &'static str {
    match item {
        InventoryItem::Area(AreaKind::Selection) => "short.area.selection",
        InventoryItem::LightPanel => "short.item.light_panel",
        InventoryItem::Block(kind) => kind.short_name_key(),
    }
}
