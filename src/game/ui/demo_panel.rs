use bevy::prelude::*;
use bevy_scene::{bsn, prelude::EntityCommandsSceneExt};

use crate::game::ui::components::{spawn_panel, PanelOptions};
use crate::game::ui::{
    OpenUiPanel, UiPanelBinding, UiPanelContext, UiPanelDescriptor, UiPanelKey, UiPanelRegistry,
};
use crate::shared::i18n::I18n;

pub const PANEL_DEMO: UiPanelKey = UiPanelKey("demo.panel");

pub fn register_demo_panel(mut registry: ResMut<UiPanelRegistry>) {
    registry.register(UiPanelDescriptor::new(
        PANEL_DEMO,
        "demo.title",
        true,
        spawn_demo_panel,
    ));
}

pub fn open_demo_panel_shortcut(
    keys: Res<ButtonInput<KeyCode>>,
    mut open: MessageWriter<OpenUiPanel>,
) {
    if keys.just_pressed(KeyCode::KeyT) {
        open.write(OpenUiPanel::new(PANEL_DEMO, UiPanelContext::None));
    }
}

pub fn spawn_demo_panel(root: &mut ChildSpawnerCommands, i18n: &I18n) -> Entity {
    spawn_panel(
        root,
        i18n,
        PanelOptions::new(320.0, "demo.title").closable(),
        UiPanelBinding(PANEL_DEMO),
        |panel| {
            panel
                .spawn_empty()
                .queue_spawn_related_scenes::<Children>(demo_panel_scene());
        },
    )
}

fn demo_panel_scene() -> impl bevy_scene::SceneList {
    bsn! {
        (
            Text("Hello panel")
            TextFont {
                font_size: FontSize::Px(24.0)
            }
            TextColor(Color::srgb(0.90, 0.84, 0.76))
        )
    }
}
