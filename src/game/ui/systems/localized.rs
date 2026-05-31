pub fn update_localized_ui(
    i18n: Res<I18n>,
    save_state: Res<SaveState>,
    mut localized_text: Query<(&LocalizedText, &mut Text)>,
) {
    if !i18n.is_changed() && !save_state.is_changed() {
        return;
    }

    for (localized, mut text) in &mut localized_text {
        text.0 = if localized.key == "button.save_world" {
            match save_state.current_kind {
                Some(SaveKind::Solution) => i18n.text("button.save_solution"),
                _ => i18n.text("button.save_puzzle"),
            }
        } else {
            i18n.text(localized.key)
        };
    }
}
