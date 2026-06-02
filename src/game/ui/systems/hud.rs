pub fn update_hud_visibility(
    mode: Res<GameMode>,
    builder_mode: Res<BuilderMode>,
    simulation: Res<SimulationState>,
    added_hud_nodes: Query<(), Added<InGameHudStyle>>,
    added_crosshair: Query<(), Added<Crosshair>>,
    added_in_game_visibility: Query<(), Added<InGameHudVisibility>>,
    added_gameplay_visibility: Query<(), Added<GameplayHudVisibility>>,
    mut gameplay_ui_changed: MessageReader<GameplayUiChanged>,
    mut save_list_changed: MessageReader<SaveListChanged>,
    mut hud_style: Query<&mut Node, With<InGameHudStyle>>,
    mut visibility_sets: ParamSet<(
        Query<&mut Visibility, With<Crosshair>>,
        Query<&mut Visibility, With<InGameHudVisibility>>,
        Query<&mut Visibility, With<GameplayHudVisibility>>,
    )>,
) {
    if gameplay_ui_changed.read().next().is_none()
        && save_list_changed.read().next().is_none()
        && !simulation.is_changed()
        && added_hud_nodes.is_empty()
        && added_crosshair.is_empty()
        && added_in_game_visibility.is_empty()
        && added_gameplay_visibility.is_empty()
    {
        return;
    }

    let has_world = matches!(*mode, GameMode::Playing | GameMode::Inventory | GameMode::Paused);
    let hide_gameplay_hud = *builder_mode == BuilderMode::Play && simulation.running;

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

    for mut visibility in &mut visibility_sets.p2() {
        *visibility = if has_world && !hide_gameplay_hud {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}
