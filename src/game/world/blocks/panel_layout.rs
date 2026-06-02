use bevy::prelude::*;
use bevy_scene::bsn;

use crate::game::ui::components::{default_button_size, spawn_panel, PanelOptions};
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
) -> Entity {
    spawn_panel(
        root,
        i18n,
        PanelOptions::new(width, title_key).closable(),
        UiPanelBinding::from(panel),
        content,
    )
}

pub fn panel_row_scene() -> impl bevy_scene::Scene {
    bsn! {
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(default_button_size(40.0)),
            display: Display::Flex,
            align_items: AlignItems::Center,
            column_gap: Val::Px(10.0),
        }
        BackgroundColor(Color::NONE)
    }
}
