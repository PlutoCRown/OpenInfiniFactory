pub fn update_hud_visibility(
    mode: Res<State<GameMode>>,
    playing_ui: Res<PlayingUiState>,
    builder_mode: Res<BuilderMode>,
    simulation: Res<SimulationState>,
    save_state: Res<SaveState>,
    mut hud_style: Query<&mut Node, With<InGameHudStyle>>,
    mut visibility_sets: ParamSet<(
        Query<&mut Visibility, With<Crosshair>>,
        Query<&mut Visibility, With<InGameHudVisibility>>,
        Query<&mut Visibility, With<GameplayHudVisibility>>,
    )>,
) {
    let has_world = save_state.current.is_some();
    let hide_gameplay_hud = *builder_mode == BuilderMode::Play && simulation.running;
    let active_play = playing_ui.active_play();

    for mut style in &mut hud_style {
        style.display = if has_world {
            Display::Flex
        } else {
            Display::None
        };
    }

    for mut visibility in &mut visibility_sets.p0() {
        *visibility = if has_world && *mode.get() == GameMode::Playing && active_play {
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

    for mut visibility in &mut visibility_sets.p2() {
        *visibility = if has_world && !hide_gameplay_hud {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}
