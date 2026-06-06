pub fn update_localized_ui(
    _ui_thread: UiMainThread,
    revision: Res<I18nRevision>,
    mut labels: ParamSet<(
        Query<(&LocalizedText, &mut Text)>,
        Query<(&LocalizedText, &mut Text), Added<LocalizedText>>,
    )>,
) {
    if revision.is_changed() {
        for (localized, mut text) in &mut labels.p0() {
            text.0 = i18n.t(localized.key);
        }
        return;
    }

    for (localized, mut text) in &mut labels.p1() {
        text.0 = i18n.t(localized.key);
    }
}
