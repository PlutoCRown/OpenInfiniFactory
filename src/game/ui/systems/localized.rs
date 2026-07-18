pub fn update_localized_ui(
    _ui_thread: UiMainThread,
    mut labels: Query<(&LocalizedText, &mut Text), Added<LocalizedText>>,
) {
    for (localized, mut text) in &mut labels {
        text.0 = i18n.t(localized.key);
    }
}
