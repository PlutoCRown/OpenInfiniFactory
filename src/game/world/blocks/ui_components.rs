use bevy::prelude::*;
use bevy_scene::{bsn, prelude::EntityCommandsSceneExt};

use crate::game::ui::components::{
    default_button_size, default_font_size, inset_border, raised_border, HoverButton, BUTTON_BG,
};
use crate::game::ui::types::{BlockMaterialIcon, BlockPanelDropdown, BlockPanelDropdownList};
use crate::game::world::blocks::MaterialKind;

pub fn spawn_block_panel_dropdown_list<A>(
    parent: &mut ChildSpawnerCommands,
    dropdown: BlockPanelDropdown,
    options: impl IntoIterator<Item = (String, A)>,
) where
    A: Component + Copy,
{
    parent
        .spawn((
            BlockPanelDropdownList(dropdown),
            GlobalZIndex(20_000),
            Visibility::Visible,
        ))
        .queue_apply_scene(dropdown_list_scene(230.0))
        .with_children(|list| {
            for (label, action) in options {
                spawn_dropdown_option(list, label, action);
            }
        });
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
            BlockPanelDropdownList(dropdown),
            GlobalZIndex(20_000),
            Visibility::Visible,
        ))
        .queue_apply_scene(icon_dropdown_list_scene())
        .with_children(|list| {
            for (material, action) in options {
                list.spawn((Button, HoverButton, BlockMaterialIcon(material), action))
                    .queue_apply_scene(material_icon_option_scene())
                    .queue_spawn_related_scenes::<Children>(material_icon_scene());
            }
        });
}

fn dropdown_list_scene(width: f32) -> impl bevy_scene::Scene {
    bsn! {
        Node {
            width: Val::Px(width),
            display: Display::None,
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(3.0),
            padding: UiRect::all(Val::Px(4.0)),
        }
        BackgroundColor(Color::srgba(0.10, 0.11, 0.12, 0.98))
    }
}

fn icon_dropdown_list_scene() -> impl bevy_scene::Scene {
    bsn! {
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
        }
        BackgroundColor(Color::srgba(0.10, 0.11, 0.12, 0.98))
    }
}

fn material_icon_option_scene() -> impl bevy_scene::Scene {
    bsn! {
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
        }
        BorderColor {
            top: {inset_border().top},
            right: {inset_border().right},
            bottom: {inset_border().bottom},
            left: {inset_border().left},
        }
        BackgroundColor(Color::srgb(0.255, 0.251, 0.251))
    }
}

fn material_icon_scene() -> impl bevy_scene::SceneList {
    bsn! {
        (
            ImageNode::default()
            Node {
                width: Val::Px(default_button_size(64.0)),
                height: Val::Px(default_button_size(64.0)),
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                top: Val::Percent(50.0),
                margin: UiRect {
                    left: {Val::Px(-default_button_size(32.0))},
                    top: {Val::Px(-default_button_size(32.0))},
                },
            }
        )
    }
}

fn spawn_dropdown_option<A>(parent: &mut ChildSpawnerCommands, label: String, action: A)
where
    A: Component + Copy,
{
    parent
        .spawn((Button, HoverButton, action))
        .queue_apply_scene(dropdown_option_button_scene())
        .queue_spawn_related_scenes::<Children>(dropdown_option_label_scene(label));
}

fn dropdown_option_button_scene() -> impl bevy_scene::Scene {
    bsn! {
        Node {
            height: Val::Px(default_button_size(32.0)),
            border: UiRect {
                left: Val::Px(3.0),
                right: Val::Px(3.0),
                top: Val::Px(4.0),
                bottom: Val::Px(5.0),
            },
            padding: UiRect::horizontal(Val::Px(14.0)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
        }
        BorderColor {
            top: {raised_border().top},
            right: {raised_border().right},
            bottom: {raised_border().bottom},
            left: {raised_border().left},
        }
        BackgroundColor(BUTTON_BG)
        BoxShadow::new(
            Color::srgba(0.0, 0.0, 0.0, 0.62),
            Val::Px(0.0),
            Val::Px(0.0),
            Val::Px(0.0),
            Val::Px(4.0),
        )
    }
}

fn dropdown_option_label_scene(label: String) -> impl bevy_scene::SceneList {
    bsn! {
        (
            Text({label})
            TextFont {
                font_size: {default_font_size(13.0)}
            }
            TextColor(Color::WHITE)
        )
    }
}
