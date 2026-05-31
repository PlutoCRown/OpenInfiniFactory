pub fn update_hud_visibility(
    mode: Res<GameMode>,
    save_state: Res<SaveState>,
    mut hud_style: Query<&mut Node, With<InGameHudStyle>>,
    mut visibility_sets: ParamSet<(
        Query<&mut Visibility, With<Crosshair>>,
        Query<&mut Visibility, With<InGameHudVisibility>>,
    )>,
) {
    let has_world = save_state.current.is_some();

    for mut style in &mut hud_style {
        style.display = if has_world {
            Display::Flex
        } else {
            Display::None
        };
    }

    for mut visibility in &mut visibility_sets.p0() {
        *visibility = if has_world && *mode == GameMode::Playing {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    for mut visibility in &mut visibility_sets.p1() {
        *visibility = if has_world {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}
