mod actions;
mod confirm;
pub mod types;
mod update;

use bevy::prelude::*;

pub(crate) use actions::{dispatch_settings_actions, settings_menu_actions};
pub use actions::{emit_settings_actions, settings_slider_changed, settings_slider_released};
pub use types::*;
pub use update::{
    update_settings_dropdowns_ui, update_settings_sliders_ui, update_settings_tabs_ui,
    update_settings_text_ui,
};

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            sync_settings_tab_panels
                .in_set(crate::game::schedule::GameSet::UiChrome)
                .before(crate::game::ui::update_panel_visibility),
        );

        app.add_message::<crate::game::ui::core::host::UiAction<types::SettingsAction>>();
        app.insert_resource(SettingsTab::default())
            .insert_resource(OpenSettingsDropdown::default())
            .insert_resource(PendingKeyBind::default())
            .insert_resource(ActiveSettingsSlider::default())
            .add_observer(emit_settings_actions)
            .add_observer(settings_slider_changed)
            .add_observer(settings_slider_released)
            .add_systems(
                Update,
                dispatch_settings_actions.in_set(crate::game::schedule::GameSet::Menus),
            )
            .add_systems(
                Update,
                (
                    update_settings_text_ui,
                    update_settings_sliders_ui,
                    update_settings_dropdowns_ui,
                    update_settings_tabs_ui,
                )
                    .run_if(
                        |ui_navigation: Res<crate::game::ui::core::runtime::UiNavigation>| {
                            ui_navigation.is_settings_open()
                        },
                    )
                    .in_set(crate::game::schedule::GameSet::UiFeat),
            );
    }
}

/// 根据设置页自身状态切换标签对应面板，挂载时也进行初始化。
fn sync_settings_tab_panels(
    tab: Res<types::SettingsTab>,
    navigation: Res<crate::game::ui::core::UiNavigation>,
    mut panels: Query<(&types::SettingsTabPanel, &mut Node)>,
) {
    for (selector, mut node) in &mut panels {
        let next = if navigation.is_settings_open() && selector.0 == *tab {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != next {
            node.display = next;
        }
    }
}
