use bevy::prelude::*;
use bevy_scene::{bsn, prelude::EntityCommandsSceneExt};

use crate::game::ui::components::{
    default_button_size, default_font_size, raised_border, HoverButton, BUTTON_BG,
};
use crate::game::ui::types::{LocalizedText, PauseMenuAction};

pub(super) fn spawn_menu_button(
    parent: &mut ChildSpawnerCommands,
    height: f32,
    font_size: f32,
    action: PauseMenuAction,
    text_key: &'static str,
) {
    parent
        .spawn((Button, HoverButton, action))
        .queue_apply_scene(menu_button_visual_scene(height))
        .queue_spawn_related_scenes::<Children>(menu_button_label_scene(text_key, font_size));
}

fn menu_button_visual_scene(height: f32) -> impl bevy_scene::Scene {
    bsn! {
        Node {
            width: Val::Percent(100.0),
            flex_grow: 1.0,
            height: Val::Px(default_button_size(height)),
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

fn menu_button_label_scene(text_key: &'static str, font_size: f32) -> impl bevy_scene::SceneList {
    bsn! {
        (
            Text({text_key})
            TextFont {
                font_size: {default_font_size(font_size)}
            }
            TextColor(Color::WHITE)
            LocalizedText {
                key: {text_key}
            }
            Pickable::IGNORE
        )
    }
}
