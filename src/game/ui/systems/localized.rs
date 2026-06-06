pub fn update_localized_ui(
    i18n: Res<I18n>,
    mut localized_text: Query<(&LocalizedText, &mut Text)>,
) {
    if !i18n.is_changed() {
        return;
    }

    for (localized, mut text) in &mut localized_text {
        text.0 = i18n.text(localized.key);
    }
}
