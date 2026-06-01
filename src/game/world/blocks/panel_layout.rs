use bevy::prelude::*;

use crate::game::ui::components::{
    default_button_size, localized_text, spawn_panel, text, transparent_node, PanelOptions,
};
use crate::game::ui::types::UiPanelBinding;
use crate::game::ui::UiPanelId;
use crate::shared::i18n::I18n;

pub fn spawn_block_panel(
    root: &mut ChildSpawnerCommands,
    i18n: &I18n,
    width: f32,
    title_key: &'static str,
    panel: UiPanelId,
    content: impl FnOnce(&mut ChildSpawnerCommands),
) {
    spawn_panel(
        root,
        i18n,
        PanelOptions::new(width, title_key).closable(),
        UiPanelBinding(panel),
        content,
    );
}

pub fn panel_row_node() -> impl Bundle {
    transparent_node(Node {
        width: Val::Percent(100.0),
        height: Val::Px(default_button_size(40.0)),
        display: Display::Flex,
        align_items: AlignItems::Center,
        column_gap: Val::Px(10.0),
        ..default()
    })
}

pub fn spawn_panel_row(
    panel: &mut ChildSpawnerCommands,
    i18n: &I18n,
    text_key: &'static str,
    controls: impl FnOnce(&mut ChildSpawnerCommands),
) {
    panel.spawn(panel_row_node()).with_children(|row| {
        spawn_panel_label(row, i18n, text_key);
        controls(row);
    });
}

pub fn spawn_panel_label(row: &mut ChildSpawnerCommands, i18n: &I18n, text_key: &'static str) {
    row.spawn((
        localized_text(i18n, text_key, 16.0, Color::srgb(0.86, 0.88, 0.86)),
        Node {
            width: Val::Px(110.0),
            ..default()
        },
    ));
}

pub fn panel_text(value: impl Into<String>, font_size: f32, color: Color) -> impl Bundle {
    text(value, font_size, color)
}
