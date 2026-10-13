use bevy::prelude::*;

use crate::game::ui::access::i18n;
use crate::game::ui::core::confirm_dialog::{ConfirmProps, ConfirmResult};
use crate::shared::config::{GameConfig, save_config};
use crate::shared::touch_profile::TouchProfile;

use super::types::{OpenSettingsDropdown, PendingKeyBind};

pub fn reset_defaults_spec() -> ConfirmProps {
    ConfirmProps {
        title: i18n.t("confirm.title"),
        message: i18n.t("confirm.reset_defaults"),
        confirm_text: i18n.t("button.confirm_reset_defaults"),
        cancel_text: i18n.t("button.cancel"),
        extra: None,
    }
}

pub fn on_reset_defaults(result: ConfirmResult, world: &mut World) {
    if !matches!(result, ConfirmResult::Confirmed) {
        return;
    }

    let config = GameConfig::default().normalized();
    let effective_ui_scale = world
        .resource::<TouchProfile>()
        .effective_ui_scale(config.ui_scale);
    *world.resource_mut::<GameConfig>() = config;
    world.resource_mut::<UiScale>().0 = effective_ui_scale;
    world
        .resource_mut::<super::types::ActiveSettingsSlider>()
        .clear();
    world.resource_mut::<OpenSettingsDropdown>().0 = None;
    world.resource_mut::<PendingKeyBind>().0 = None;
    save_config(world.resource::<GameConfig>());
}
