pub fn update_panel_visibility(
    mode: Res<GameMode>,
    save_state: Res<SaveState>,
    solution_state: Res<SolutionState>,
    settings_tab: Res<SettingsTab>,
    ui_runtime: Res<UiRuntime>,
    mut open_block_dropdown: ResMut<OpenBlockPanelDropdown>,
    mut style_sets: ParamSet<(
        Query<&mut Node, (With<MainMenuPanel>, Without<PauseAction>)>,
        Query<&mut Node, (With<SaveListPanel>, Without<PauseAction>)>,
        Query<
            &mut Node,
            (
                With<SettingsGameplayGroup>,
                Without<PauseAction>,
                Without<UiPanelBinding>,
            ),
        >,
        Query<
            &mut Node,
            (
                With<SettingsKeyBindingsGroup>,
                Without<PauseAction>,
                Without<UiPanelBinding>,
            ),
        >,
        Query<&mut Node, (With<BackpackPanel>, Without<PauseAction>)>,
        Query<&mut Node, (With<PausePanel>, Without<PauseAction>)>,
        Query<&mut Node, (With<ConfirmDialogPanel>, Without<PauseAction>)>,
        Query<&mut Node, (With<ModalScrim>, Without<PauseAction>)>,
    )>,
    mut pause_buttons: Query<(&PauseAction, &mut Node), With<Button>>,
    mut bound_panels: Query<
        (&UiPanelBinding, &mut Node),
        (
            Without<PauseAction>,
            Without<MainMenuPanel>,
            Without<SaveListPanel>,
            Without<SettingsGameplayGroup>,
            Without<SettingsKeyBindingsGroup>,
            Without<BackpackPanel>,
            Without<PausePanel>,
            Without<ConfirmDialogPanel>,
            Without<ModalScrim>,
        ),
    >,
    confirm_dialog: Res<ConfirmDialogState>,
) {
    for mut style in &mut style_sets.p0() {
        style.display = if *mode == GameMode::MainMenu {
            Display::Flex
        } else {
            Display::None
        };
    }

    for mut style in &mut style_sets.p1() {
        style.display = if *mode == GameMode::SaveListMain {
            Display::Flex
        } else {
            Display::None
        };
    }

    let settings_open = ui_runtime.is_settings_open();

    for mut style in &mut style_sets.p2() {
        style.display = if settings_open && *settings_tab == SettingsTab::Gameplay {
            Display::Flex
        } else {
            Display::None
        };
    }

    for mut style in &mut style_sets.p3() {
        style.display = if settings_open && *settings_tab == SettingsTab::KeyBindings {
            Display::Flex
        } else {
            Display::None
        };
    }

    for mut style in &mut style_sets.p4() {
        style.display = if *mode == GameMode::Inventory {
            Display::Flex
        } else {
            Display::None
        };
    }

    for mut style in &mut style_sets.p5() {
        style.display = if *mode == GameMode::Paused {
            Display::Flex
        } else {
            Display::None
        };
    }

    for mut style in &mut style_sets.p6() {
        style.display = if confirm_dialog.kind.is_some() {
            Display::Flex
        } else {
            Display::None
        };
    }

    let active_panel = ui_runtime.active_panel();
    for mut style in &mut style_sets.p7() {
        style.display = if ui_runtime.has_modal_panel() || confirm_dialog.kind.is_some() {
            Display::Flex
        } else {
            Display::None
        };
    }

    for (action, mut style) in &mut pause_buttons {
        style.display = if pause_action_visible(&save_state, &solution_state, *action) {
            Display::Flex
        } else {
            Display::None
        };
    }

    if !block_dropdown_matches_panel(open_block_dropdown.0, active_panel) {
        open_block_dropdown.0 = None;
    }
    for (binding, mut style) in &mut bound_panels {
        let visible = active_panel == Some(binding.0);
        style.display = if visible {
            Display::Flex
        } else {
            Display::None
        };
    }
}

pub fn update_ui_layers(
    ui_runtime: Res<UiRuntime>,
    confirm_dialog: Res<ConfirmDialogState>,
    mut layered_nodes: Query<(
        &mut GlobalZIndex,
        Option<&UiPanelBinding>,
        Has<MainMenuPanel>,
        Has<SaveListPanel>,
        Has<PausePanel>,
        Has<BackpackPanel>,
        Has<ConfirmDialogPanel>,
        Has<ModalScrim>,
    )>,
) {
    const BASE_LAYER: i32 = 100;

    let top_panel_z = ui_runtime
        .top_modal_layer()
        .map(panel_layer_z)
        .unwrap_or(PANEL_LAYER_BASE);
    let confirm_z = if confirm_dialog.kind.is_some() {
        top_panel_z + CONFIRM_LAYER_STEP
    } else {
        PANEL_LAYER_BASE
    };
    let scrim_z = if confirm_dialog.kind.is_some() {
        confirm_z + SCRIM_OFFSET
    } else {
        ui_runtime
            .top_modal_layer()
            .map(|layer| panel_layer_z(layer) + SCRIM_OFFSET)
            .unwrap_or(PANEL_LAYER_BASE + SCRIM_OFFSET)
    };

    for (
        mut z,
        binding,
        main_menu,
        save_list,
        pause_panel,
        backpack_panel,
        confirm_panel,
        modal_scrim,
    ) in &mut layered_nodes
    {
        z.0 = if modal_scrim {
            scrim_z
        } else if confirm_panel {
            confirm_z
        } else if let Some(binding) = binding {
            ui_runtime
                .panel_layer(binding.0)
                .map(panel_layer_z)
                .unwrap_or(PANEL_LAYER_BASE)
        } else if main_menu || save_list || pause_panel || backpack_panel {
            BASE_LAYER
        } else {
            z.0
        };
    }
}

const PANEL_LAYER_BASE: i32 = 1_000;
const PANEL_LAYER_STEP: i32 = 20;
const SCRIM_OFFSET: i32 = -1;
const CONFIRM_LAYER_STEP: i32 = 20;

fn panel_layer_z(layer: usize) -> i32 {
    PANEL_LAYER_BASE + layer as i32 * PANEL_LAYER_STEP
}

fn block_dropdown_matches_panel(
    dropdown: Option<BlockPanelDropdown>,
    panel: Option<UiPanelId>,
) -> bool {
    matches!(
        (dropdown, panel),
        (None, _)
            | (
                Some(BlockPanelDropdown::GeneratorMaterial),
                Some(UiPanelId::Generator)
            )
            | (
                Some(BlockPanelDropdown::GoalMaterial),
                Some(UiPanelId::Goal)
            )
            | (
                Some(BlockPanelDropdown::LabelerColor),
                Some(UiPanelId::Labeler)
            )
            | (
                Some(BlockPanelDropdown::ConverterInput),
                Some(UiPanelId::Converter)
            )
            | (
                Some(BlockPanelDropdown::ConverterOutput),
                Some(UiPanelId::Converter)
            )
            | (
                Some(BlockPanelDropdown::TeleportPair),
                Some(UiPanelId::Teleport)
            )
    )
}
