use bevy::prelude::*;

use crate::game::ui::components::{
    default_button_size, inset_border, label_text, menu_button, styled_button, transparent_node,
};
use crate::game::ui::types::{
    BlockEditAction, BlockMaterialIcon, BlockMaterialIconSlot, BlockPanelDropdown,
    BlockPanelDropdownLabel, BlockPanelDropdownList, LocalizedText,
};
use crate::game::world::blocks::MaterialKind;

pub fn spawn_block_edit_button(
    parent: &mut ChildSpawnerCommands,
    action: BlockEditAction,
    text_key: &'static str,
) {
    parent
        .spawn((menu_button(36.0), action))
        .with_children(|button| {
            button.spawn((
                label_text(text_key, 14.0, Color::WHITE),
                LocalizedText { key: text_key },
            ));
        });
}

pub fn spawn_block_panel_dropdown<A>(
    parent: &mut ChildSpawnerCommands,
    dropdown: BlockPanelDropdown,
    toggle_action: A,
) where
    A: Component + Copy,
{
    parent
        .spawn((
            transparent_node(Node {
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

pub fn spawn_block_panel_dropdown_list<A>(
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

pub fn spawn_material_icon_slot<A>(
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
        .with_children(spawn_material_icon);
}

pub fn spawn_material_icon_dropdown_list<A>(
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
                .with_children(spawn_material_icon);
            }
        });
}

fn spawn_material_icon(slot: &mut ChildSpawnerCommands) {
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
