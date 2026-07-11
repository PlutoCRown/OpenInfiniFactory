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
    inventory: Res<InventoryItems>,
    builder_mode: Res<BuilderMode>,
    simulation: Res<SimulationState>,
    save_state: Res<SaveState>,
    config: Res<GameConfig>,
    mut texts: Query<(&StatusText, &mut Text)>,
) {
    for (status, mut text) in &mut texts {
        let next = status_text_value(
            status.0,
            &placement,
            &world,
            &inventory,
            *builder_mode,
            &simulation,
            &save_state,
            &config,
        );
        if text.0 != next {
            text.0 = next;
        }
    }
}

fn status_text_value(
    kind: StatusTextKind,
    placement: &PlacementState,
    world: &WorldBlocks,
    inventory: &InventoryItems,
    builder_mode: BuilderMode,
    simulation: &SimulationState,
    save_state: &SaveState,
    config: &GameConfig,
) -> String {
    match kind {
        StatusTextKind::TargetBlock => target_status_line(placement, world),
        StatusTextKind::Hotbar => {
            let selected_item = inventory.hotbar[placement.selected];
            let selected = selected_item
                .map(|item| i18n.t(item.name_key()))
                .unwrap_or_else(|| i18n.t("empty"));
            i18n.fmt(
                "status.hotbar",
                &[
                    ("mode", builder_mode_name(builder_mode)),
                    ("selected", selected),
                ],
            )
        }
        StatusTextKind::CurrentSave => save_state
            .current
            .as_ref()
            .map(|slot| i18n.fmt("save.world", &[("name", slot.display_name())]))
            .unwrap_or_else(|| i18n.t("save.no_world_loaded")),
        StatusTextKind::Simulation => i18n.fmt(
            "status.simulation",
            &[
                ("mode", builder_mode_name(builder_mode)),
                ("turns", simulation.turn.to_string()),
                (
                    "state",
                    i18n.t(if simulation.running {
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
                simulation_status_text_value(simulation, config)
            } else {
                String::new()
            }
        }
    }
}

fn simulation_status_text_value(simulation: &SimulationState, config: &GameConfig) -> String {
    let start = config.input(ActionKeyName::Simulate).name().to_string();
    let fast = config
        .input(ActionKeyName::SimulationFast)
        .name()
        .to_string();
    let step = config
        .input(ActionKeyName::SimulationStep)
        .name()
        .to_string();
    let rollback = config
        .input(ActionKeyName::SimulationRollback)
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
            ("state", i18n.t(state_key)),
            ("turns", simulation.turn.to_string()),
            ("controls", controls),
        ],
    )
}
