use bevy::prelude::*;

use crate::game::state::GameSettings;
use crate::game::ui::access::i18n;
use crate::game::ui::core::confirm_dialog::{ConfirmProps, ConfirmResult};
use crate::game::{GRAVITY_SCALE_MAX, GRAVITY_SCALE_MIN, MOUSE_SENSITIVITY_MAX, MOUSE_SENSITIVITY_MIN, UI_SCALE_MAX, UI_SCALE_MIN};
use crate::shared::config::{save_config, GameConfig};
use crate::shared::i18n::resolve_language;

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

    *world.resource_mut::<GameConfig>() = GameConfig::default();

    let (fov, ui_scale, gravity, mouse_sensitivity_x, mouse_sensitivity_y, language) = {
        let config = world.resource::<GameConfig>();
        (
            config.fov_degrees,
            config.ui_scale.clamp(UI_SCALE_MIN, UI_SCALE_MAX),
            config
                .gravity_scale
                .clamp(GRAVITY_SCALE_MIN, GRAVITY_SCALE_MAX),
            config
                .mouse_sensitivity_x
                .clamp(MOUSE_SENSITIVITY_MIN, MOUSE_SENSITIVITY_MAX),
            config
                .mouse_sensitivity_y
                .clamp(MOUSE_SENSITIVITY_MIN, MOUSE_SENSITIVITY_MAX),
            config.language,
        )
    };

    {
        let mut settings = world.resource_mut::<GameSettings>();
        settings.fov_degrees = fov;
        settings.ui_scale = ui_scale;
        settings.gravity_scale = gravity;
        settings.mouse_sensitivity_x = mouse_sensitivity_x;
        settings.mouse_sensitivity_y = mouse_sensitivity_y;
    }

    world.resource_mut::<UiScale>().0 = ui_scale;
    i18n.set_language(resolve_language(language));
    world.resource_mut::<OpenSettingsDropdown>().0 = None;
    world.resource_mut::<PendingKeyBind>().0 = None;
    save_config(world.resource::<GameConfig>());
}
