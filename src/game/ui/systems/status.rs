fn pause_action_visible(
    save_state: &SaveState,
    solution_state: &SolutionState,
    action: PauseMenuAction,
) -> bool {
    match action {
        PauseMenuAction::ToggleBuilderMode => solution_state.entry != WorldEntryMode::PlaySolution,
        PauseMenuAction::SaveAsNewPuzzle => save_state.current_kind == Some(SaveKind::Puzzle),
        PauseMenuAction::ResetSolution => save_state.current_kind == Some(SaveKind::Solution),
        _ => true,
    }
}

fn builder_mode_name(mode: BuilderMode, i18n: &I18n) -> String {
    match mode {
        BuilderMode::Edit => i18n.text("mode.edit"),
        BuilderMode::Play => i18n.text("mode.play"),
    }
}

pub fn update_status_ui(
    placement: Res<PlacementState>,
    inventory: Res<InventoryItems>,
    builder_mode: Res<BuilderMode>,
    simulation: Res<SimulationState>,
    save_state: Res<SaveState>,
    config: Res<GameConfig>,
    i18n: Res<I18n>,
    added_status_texts: Query<(), Added<StatusText>>,
    added_panel_texts: Query<(), Added<PanelText>>,
    mut gameplay_ui_changed: MessageReader<GameplayUiChanged>,
    mut inventory_changed: MessageReader<InventoryChanged>,
    mut language_changed: MessageReader<LanguageChanged>,
    mut save_list_changed: MessageReader<SaveListChanged>,
    mut settings_changed: MessageReader<SettingsChanged>,
    mut texts: ParamSet<(
        Query<(&StatusText, &mut Text)>,
        Query<(&PanelText, &mut Text)>,
    )>,
) {
    if gameplay_ui_changed.read().next().is_none()
        && inventory_changed.read().next().is_none()
        && !simulation.is_changed()
        && save_list_changed.read().next().is_none()
        && settings_changed.read().next().is_none()
        && added_status_texts.is_empty()
        && added_panel_texts.is_empty()
        && language_changed.read().next().is_none()
    {
        return;
    }

    for (status, mut text) in &mut texts.p0() {
        text.0 = status_text_value(
            status.0,
            &placement,
            &inventory,
            *builder_mode,
            &simulation,
            &save_state,
            &config,
            &i18n,
        );
    }

    for (panel_text, mut text) in &mut texts.p1() {
        if panel_text.0 == PanelTextKind::InventoryTitle {
            text.0 = i18n.fmt(
                "inventory.title",
                &[("mode", builder_mode_name(*builder_mode, &i18n))],
            );
        }
    }
}

fn status_text_value(
    kind: StatusTextKind,
    placement: &PlacementState,
    inventory: &InventoryItems,
    builder_mode: BuilderMode,
    simulation: &SimulationState,
    save_state: &SaveState,
    config: &GameConfig,
    i18n: &I18n,
) -> String {
    match kind {
        StatusTextKind::Hotbar => {
            let selected_item = inventory.hotbar[placement.selected];
            let selected = selected_item
                .map(|item| i18n.text(item.name_key()))
                .unwrap_or_else(|| i18n.text("empty"));
            i18n.fmt(
                "status.hotbar",
                &[
                    ("mode", builder_mode_name(builder_mode, i18n)),
                    ("selected", selected),
                ],
            )
        }
        StatusTextKind::CurrentSave => save_state
            .current
            .as_ref()
            .map(|name| i18n.fmt("save.world", &[("name", name.clone())]))
            .unwrap_or_else(|| i18n.text("save.no_world_loaded")),
        StatusTextKind::Simulation => i18n.fmt(
            "status.simulation",
            &[
                ("mode", builder_mode_name(builder_mode, i18n)),
                ("turns", simulation.turn.to_string()),
                (
                    "state",
                    i18n.text(if simulation.running {
                        "state.playing"
                    } else {
                        "state.paused"
                    }),
                ),
                ("speed", format!("{:.0}", simulation.speed)),
            ],
        ),
        StatusTextKind::SimulationOverlay => {
            if builder_mode == BuilderMode::Play {
                simulation_status_text_value(simulation, config, i18n)
            } else {
                String::new()
            }
        }
    }
}

fn simulation_status_text_value(
    simulation: &SimulationState,
    config: &GameConfig,
    i18n: &I18n,
) -> String {
    let start = config.input(ConfigAction::Simulate).name().to_string();
    let fast = config
        .input(ConfigAction::SimulationFast)
        .name()
        .to_string();
    let step = config
        .input(ConfigAction::SimulationStep)
        .name()
        .to_string();
    let rollback = config
        .input(ConfigAction::SimulationRollback)
        .name()
        .to_string();

    let (state_key, controls_key, controls_args): (&str, &str, Vec<(&str, String)>) =
        if !simulation.is_active() {
            (
                "simulation_state.ready",
                "simulation_controls.ready",
                vec![("start", start)],
            )
        } else if simulation.running && simulation.speed > 1.0 {
            (
                "simulation_state.fast",
                "simulation_controls.fast",
                vec![("fast", fast), ("step", step), ("rollback", rollback)],
            )
        } else if simulation.running {
            (
                "simulation_state.playing",
                "simulation_controls.playing",
                vec![("step", step), ("fast", fast), ("rollback", rollback)],
            )
        } else {
            (
                "simulation_state.paused",
                "simulation_controls.paused",
                vec![("step", step), ("start", start), ("rollback", rollback)],
            )
        };
    let controls = i18n.fmt(controls_key, &controls_args);

    i18n.fmt(
        "status.simulation_overlay",
        &[
            ("state", i18n.text(state_key)),
            ("turns", simulation.turn.to_string()),
            ("controls", controls),
        ],
    )
}
