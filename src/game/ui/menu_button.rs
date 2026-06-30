//! 同质菜单按钮：声明式 `label_key` + `on_click`，spawn 与 handler 共用同一张表。

use bevy::picking::prelude::{Click, Pointer};
use bevy::prelude::*;

use crate::game::ui::components::{full_width_button, localized_text};
use crate::game::ui::core::text_input::primary_click;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Message)]
pub struct MenuButtonClick {
    pub set: MenuButtonSet,
    pub index: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum MenuButtonSet {
    StartMenu,
    PauseMenu,
}

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct MenuButtonMarker {
    pub set: MenuButtonSet,
    pub index: u8,
}

pub fn spawn_menu_button(
    parent: &mut ChildSpawnerCommands,
    set: MenuButtonSet,
    index: u8,
    label_key: &'static str,
) {
    const HEIGHT: f32 = 38.0;
    const FONT_SIZE: f32 = 16.0;

    parent
        .spawn((
            full_width_button(HEIGHT),
            MenuButtonMarker { set, index },
        ))
        .with_children(|button| {
            button.spawn(localized_text(label_key, FONT_SIZE, Color::WHITE));
        });
}

pub fn on_menu_button_click(
    mut click: On<Pointer<Click>>,
    mut writer: MessageWriter<MenuButtonClick>,
    ui_host: Res<crate::game::ui::core::host::UiHost>,
    markers: Query<&MenuButtonMarker>,
) {
    if ui_host.modal_open() || !primary_click(&mut click) {
        return;
    }
    let Ok(marker) = markers.get(click.entity) else {
        return;
    };
    click.propagate(false);
    writer.write(MenuButtonClick {
        set: marker.set,
        index: marker.index,
    });
}

pub fn register_menu_button_clicks(app: &mut App) {
    app.add_message::<MenuButtonClick>()
        .add_observer(on_menu_button_click);
}
