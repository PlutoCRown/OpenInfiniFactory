use bevy::prelude::*;

use crate::game::ui::access::{I18nRevision, UiMainThread};
use crate::shared::save::SaveState;

use super::types::MenuAction;

pub fn update_menu_labels(
    _ui_thread: UiMainThread,
    i18n_revision: Res<I18nRevision>,
    save_state: Res<SaveState>,
    menu_buttons: Query<(&MenuAction, &Children), With<Button>>,
    mut texts: Query<&mut Text>,
) {
    if !i18n_revision.is_changed() && !save_state.is_changed() {
        return;
    }

    for (action, children) in &menu_buttons {
        let label = action.label(&save_state);
        for child in children.iter() {
            if let Ok(mut text) = texts.get_mut(child) {
                text.0 = label.clone();
            }
        }
    }
}
