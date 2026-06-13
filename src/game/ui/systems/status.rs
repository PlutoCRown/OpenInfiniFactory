fn builder_mode_name(mode: BuilderMode) -> String {
    i18n.t(match mode {
        BuilderMode::Edit => "mode.edit",
        BuilderMode::Play => "mode.play",
    })
}

pub fn update_status_ui(
    _ui_thread: UiMainThread,
    placement: Res<PlacementState>,
    world: Res<WorldBlocks>,
    builder_mode: Res<BuilderMode>,
    save_state: Res<SaveState>,
    mut texts: Query<(&StatusText, &mut Text)>,
) {
    for (status, mut text) in &mut texts {
        text.0 = status_text_value(status.0, &placement, &world, *builder_mode, &save_state);
    }
}

fn status_text_value(
    kind: StatusTextKind,
    placement: &PlacementState,
    world: &WorldBlocks,
    builder_mode: BuilderMode,
    save_state: &SaveState,
) -> String {
    match kind {
        StatusTextKind::Summary => {
            let world_name = save_state
                .current
                .as_ref()
                .map(|name| name.clone())
                .unwrap_or_else(|| i18n.t("save.no_world_loaded"));
            i18n.fmt(
                "status.world_mode",
                &[
                    ("world", world_name),
                    ("mode", builder_mode_name(builder_mode)),
                ],
            )
        }
        StatusTextKind::TargetBlock => target_status_line(placement, world),
    }
}
