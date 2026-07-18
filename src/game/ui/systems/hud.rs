pub fn update_hud_visibility(
    mode: Res<State<GameMode>>,
    playing_ui: Res<PlayingUiState>,
    builder_mode: Res<BuilderMode>,
    simulation: Res<SimulationState>,
    save_state: Res<SaveState>,
    mut primed: Local<bool>,
    added_hud: Query<
        (),
        Or<(
            Added<InGameHudStyle>,
            Added<Crosshair>,
            Added<InGameHudVisibility>,
            Added<GameplayHudVisibility>,
        )>,
    >,
    mut hud_style: Query<&mut Node, With<InGameHudStyle>>,
    mut visibility_sets: ParamSet<(
        Query<&mut Visibility, With<Crosshair>>,
        Query<&mut Visibility, (With<InGameHudVisibility>, Without<Crosshair>)>,
        Query<&mut Visibility, With<GameplayHudVisibility>>,
    )>,
) {
    let dirty = !*primed
        || mode.is_changed()
        || playing_ui.is_changed()
        || builder_mode.is_changed()
        || simulation.is_changed()
        || save_state.is_changed()
        || !added_hud.is_empty();
    if !dirty {
        return;
    }
    *primed = true;

    let has_world = save_state.current.is_some();
    let hide_gameplay_hud = *builder_mode == BuilderMode::Play && simulation.is_active();
    let active_play = playing_ui.active_play();

    let hud_display = if has_world {
        Display::Flex
    } else {
        Display::None
    };
    for mut style in &mut hud_style {
        if style.display != hud_display {
            style.display = hud_display;
        }
    }

    let crosshair = if has_world && *mode.get() == GameMode::Playing && active_play {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    for mut visibility in &mut visibility_sets.p0() {
        visibility.set_if_neq(crosshair);
    }

    let in_game = if has_world {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    for mut visibility in &mut visibility_sets.p1() {
        visibility.set_if_neq(in_game);
    }

    let gameplay = if has_world && !hide_gameplay_hud {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    for mut visibility in &mut visibility_sets.p2() {
        visibility.set_if_neq(gameplay);
    }
}
