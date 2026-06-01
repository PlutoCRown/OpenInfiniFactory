use bevy::prelude::*;

use crate::game::ui::components::{full_width_button, label_text};
use crate::game::ui::types::{LocalizedText, MainMenuAction};

pub(super) fn spawn_menu_button(
    parent: &mut ChildSpawnerCommands,
    height: f32,
    font_size: f32,
    action: MainMenuAction,
    text_key: &'static str,
) {
    parent
        .spawn((full_width_button(height), action))
        .with_children(|button| {
            button.spawn((
                label_text(text_key, font_size, Color::WHITE),
                LocalizedText { key: text_key },
            ));
        });
}
