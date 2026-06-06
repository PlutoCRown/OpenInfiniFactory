use bevy::prelude::*;

use crate::game::blocks::{BlockKind, MaterialKind};
use crate::game::ui::components::{
    default_button_size, default_font_size, localized_text, menu_button, styled_button,
    ui_logical_bounds,
};
use crate::game::ui::types::UiActionLabel;
use crate::game::world::rendering::BlockIconAssets;

pub fn spawn_labeled_panel_button<A>(
    parent: &mut ChildSpawnerCommands,
    action: A,
) where
    A: Component + Copy + UiActionLabel,
{
    parent
        .spawn((menu_button(36.0), action))
        .with_children(|button| {
            button.spawn(localized_text(action.label_key(), 14.0, Color::WHITE));
        });
}

pub fn spawn_text_dropdown_toggle<A, L>(
    parent: &mut ChildSpawnerCommands,
    toggle_action: A,
    label_marker: L,
) where
    A: Component + Copy,
    L: Component + Copy,
{
    parent
        .spawn((
            Node {
                width: Val::Px(230.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                position_type: PositionType::Relative,
                ..default()
            },
            BackgroundColor(Color::NONE),
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
                        Text::new(""),
                        TextFont {
                            font_size: default_font_size(14.0),
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        label_marker,
                    ));
                    button.spawn((
                        Text::new("v"),
                        TextFont {
                            font_size: default_font_size(12.0),
                            ..default()
                        },
                        TextColor(Color::srgb(0.72, 0.80, 0.84)),
                    ));
                });
        });
}

pub fn spawn_text_dropdown_list<A, L>(
    parent: &mut ChildSpawnerCommands,
    list_marker: L,
    options: impl IntoIterator<Item = (String, A)>,
) where
    A: Component + Copy,
    L: Component + Copy,
{
    parent
        .spawn((
            dropdown_list_node(230.0),
            GlobalZIndex(20_000),
            list_marker,
        ))
        .with_children(|list| {
            for (label, action) in options {
                spawn_text_option(list, label, action);
            }
        });
}

pub fn spawn_material_icon_toggle<A, S>(
    parent: &mut ChildSpawnerCommands,
    slot_marker: S,
    toggle_action: A,
) where
    A: Component + Copy,
    S: Component + Copy,
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
                Color::srgb(0.255, 0.251, 0.251),
                Color::srgba(0.0, 0.0, 0.0, 0.35),
            ),
            slot_marker,
            toggle_action,
        ))
        .with_children(|slot| {
            slot.spawn(material_icon_node());
        });
}

pub fn spawn_material_icon_list<A, O, L>(
    parent: &mut ChildSpawnerCommands,
    list_marker: L,
    options: impl IntoIterator<Item = (MaterialKind, A)>,
    option_marker: fn(MaterialKind) -> O,
) where
    A: Component + Copy,
    O: Component + Copy,
    L: Component + Copy,
{
    parent
        .spawn((
            icon_dropdown_list_node(),
            GlobalZIndex(20_000),
            list_marker,
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
                        Color::srgb(0.255, 0.251, 0.251),
                        Color::srgba(0.0, 0.0, 0.0, 0.35),
                    ),
                    option_marker(material),
                    action,
                ))
                .with_children(|slot| {
                    slot.spawn(material_icon_node());
                });
            }
        });
}

pub fn update_material_icon(
    children: &Children,
    material: Option<MaterialKind>,
    block_icons: &BlockIconAssets,
    icon_query: &mut Query<&mut ImageNode>,
) {
    let icon = material
        .and_then(BlockKind::material_block_kind)
        .and_then(|kind| block_icons.get(kind));
    for child in children.iter() {
        if let Ok(mut image) = icon_query.get_mut(child) {
            *image = icon.clone().map(ImageNode::new).unwrap_or_default();
        }
    }
}

pub fn position_dropdown_from_trigger(
    trigger_node: &ComputedNode,
    transform: &UiGlobalTransform,
    list_node: &ComputedNode,
    viewport: Vec2,
) -> Option<(f32, f32)> {
    let trigger = ui_logical_bounds(trigger_node, transform);
    let list_size = list_node.size() * list_node.inverse_scale_factor();
    let below = trigger.max.y + 4.0;
    let above = trigger.min.y - list_size.y - 4.0;
    let top = if below + list_size.y <= viewport.y - 10.0 || above < 10.0 {
        below
    } else {
        above.max(10.0)
    };
    let top = top.clamp(10.0, (viewport.y - list_size.y - 10.0).max(10.0));
    let left = trigger
        .min
        .x
        .clamp(10.0, (viewport.x - list_size.x - 10.0).max(10.0));
    Some((left, top))
}

fn material_icon_node() -> impl Bundle {
    (
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
    )
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

fn spawn_text_option<A>(parent: &mut ChildSpawnerCommands, label: String, action: A)
where
    A: Component + Copy,
{
    parent
        .spawn((menu_button(32.0), action))
        .with_children(|button| {
            button.spawn((
                Text::new(label),
                TextFont {
                    font_size: default_font_size(13.0),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}
