use bevy::prelude::*;

use crate::shared::i18n::I18n;
use crate::shared::save::SaveState;

use super::types::MenuAction;

pub fn update_menu_labels(
    i18n: Res<I18n>,
    save_state: Res<SaveState>,
    menu_buttons: Query<(&MenuAction, &Children), With<Button>>,
    mut texts: Query<&mut Text>,
) {
    if !i18n.is_changed() && !save_state.is_changed() {
        return;
    }

    for (action, children) in &menu_buttons {
        let label = action.label(&save_state, &i18n);
        for child in children.iter() {
            if let Ok(mut text) = texts.get_mut(child) {
                text.0 = label.clone();
            }
        }
    }
}
