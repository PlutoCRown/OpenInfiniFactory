pub fn update_localized_ui(
    i18n: Res<I18n>,
    save_state: Res<SaveState>,
    added_localized_text: Query<(), Added<LocalizedText>>,
    mut language_changed: MessageReader<LanguageChanged>,
    mut save_list_changed: MessageReader<SaveListChanged>,
    mut localized_text: Query<(&LocalizedText, &mut Text)>,
) {
    if language_changed.read().next().is_none()
        && save_list_changed.read().next().is_none()
        && added_localized_text.is_empty()
    {
        return;
    }

    for (localized, mut text) in &mut localized_text {
        text.0 = if localized.key == "button.save_world" {
            match save_state.current_kind {
                Some(SaveKind::Solution) => i18n.text("button.save_solution"),
                _ => i18n.text("button.save_puzzle"),
            }
        } else if localized.key == "button.save_as_new_puzzle" {
            i18n.text("button.save_as_new_puzzle")
        } else {
            i18n.text(localized.key)
        };
    }
}
