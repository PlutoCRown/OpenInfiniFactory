use bevy::prelude::*;

use crate::game::ui::access::{UiContext, i18n};
use crate::game::ui::types::LocalizedText;

pub fn update_localized_ui(
    ui_context: UiContext,
    mut labels: Query<(Ref<LocalizedText>, &mut Text)>,
) {
    let _ui_scope = ui_context.enter();
    for (localized, mut text) in &mut labels {
        if !ui_context.locale_changed() && !localized.is_added() {
            continue;
        }
        let next = i18n.t(localized.key);
        if text.0 != next {
            text.0 = next;
        }
    }
}
