use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;

use super::components::{
    default_button_size, default_font_size, full_width_button, inset_border, menu_button,
    raised_border, slider_bundle, slider_fill, slider_knob, styled_button,
};
use crate::game::block_editing::{BlockPanelAction, BlockPanelDropdown, BlockMaterialIcon, BlockMaterialIconSlot, BlockPanelDropdownLabel, BlockPanelDropdownList};
use super::types::{
    AreaKind, ConfirmDialogAction, InventoryItem,
    InventorySlot, KeyBindingButton, MenuAction, SettingsAction, SettingsDropdown,
    SettingsDropdownLabel, SettingsDropdownList, SettingsField, SettingsSliderFill,
    SettingsSliderKnob, SettingsText, SettingsTextKind, SettingsValueText, SlotArea,
    UiActionLabel,
};
use crate::game::world::blocks::MaterialKind;

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
    spawn_full_width_localized_button(parent, height, font_size, action);
}

pub(super) fn spawn_block_panel_button(parent: &mut ChildSpawnerCommands, action: BlockPanelAction) {
    spawn_localized_button(parent, 36.0, 14.0, action);
}

pub(super) fn spawn_block_panel_dropdown<A>(
    parent: &mut ChildSpawnerCommands,
    dropdown: BlockPanelDropdown,
    toggle_action: A,
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
        });
}

pub(super) fn spawn_block_panel_dropdown_list<A>(
    parent: &mut ChildSpawnerCommands,
    dropdown: BlockPanelDropdown,
    options: impl IntoIterator<Item = (String, A)>,
) where
    A: Component + Copy,
{
    parent
        .spawn((
            dropdown_list_node(230.0),
            GlobalZIndex(20_000),
            BlockPanelDropdownList(dropdown),
        ))
        .with_children(|list| {
            for (label, action) in options {
                spawn_dropdown_option(list, label, action);
            }
        });
}

pub(super) fn spawn_material_icon_slot<A>(
    parent: &mut ChildSpawnerCommands,
    dropdown: BlockPanelDropdown,
    toggle_action: A,
) where
    A: Component + Copy,
{
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
            BlockMaterialIconSlot(dropdown),
            toggle_action,
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
        });
}

pub(super) fn spawn_material_icon_dropdown_list<A>(
    parent: &mut ChildSpawnerCommands,
    dropdown: BlockPanelDropdown,
    options: impl IntoIterator<Item = (MaterialKind, A)>,
) where
    A: Component + Copy,
{
    parent
        .spawn((
            icon_dropdown_list_node(),
            GlobalZIndex(20_000),
            BlockPanelDropdownList(dropdown),
        ))
        .with_children(|list| {
            for (material, action) in options {
                list.spawn((
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
                    BlockMaterialIcon(material),
                    action,
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
                });
            }
        });
}

pub(super) fn spawn_confirm_dialog_button(
    parent: &mut ChildSpawnerCommands,
    action: ConfirmDialogAction,
) {
    let key = action.label_key();
    parent
        .spawn((full_width_button(34.0), action))
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
    let mut button = parent.spawn((full_width_button(36.0), action));
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
        .spawn(slider_bundle(SettingsAction::Field(field)))
        .with_children(|track| {
            track.spawn((slider_fill(), SettingsSliderFill(field)));
            track.spawn((slider_knob(), SettingsSliderKnob(field)));
        });
}

pub(super) fn spawn_settings_slider_value(parent: &mut ChildSpawnerCommands, field: SettingsField) {
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
        });
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

fn icon_dropdown_list_node() -> impl Bundle {
    (
        Node {
            width: Val::Px(192.0),
            display: Display::None,
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            row_gap: Val::Px(4.0),
            column_gap: Val::Px(4.0),
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

fn spawn_full_width_localized_button<'a, A>(
    parent: &'a mut ChildSpawnerCommands,
    height: f32,
    font_size: f32,
    action: A,
) -> EntityCommands<'a>
where
    A: Bundle + Copy + UiActionLabel,
{
    let key = action.label_key();
    let mut entity = parent.spawn((full_width_button(height), action));
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
