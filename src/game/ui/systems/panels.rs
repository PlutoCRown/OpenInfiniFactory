pub fn update_panel_visibility(
    mode: Res<GameMode>,
    save_state: Res<SaveState>,
    solution_state: Res<SolutionState>,
    settings_tab: Res<SettingsTab>,
    ui_runtime: Res<UiRuntime>,
    world: Res<WorldBlocks>,
    mut open_block_dropdown: ResMut<OpenBlockPanelDropdown>,
    mut controlled_panels: Query<(&PanelVisibility, &mut Node)>,
    mut pause_buttons: Query<(&MenuAction, &mut Node), With<Button>>,
    mut bound_panels: Query<(&UiPanelBinding, &mut Node)>,
    confirm_dialog: Res<ConfirmDialogState>,
) {
    let active_panel = ui_runtime.active_panel();
    for (visibility, mut style) in &mut controlled_panels {
        style.display = display_for(panel_visible(
            *visibility,
            *mode,
            *settings_tab,
            &ui_runtime,
            &confirm_dialog,
        ));
    }

    for (action, mut style) in &mut pause_buttons {
        style.display = if pause_action_visible(&save_state, &solution_state, *action) {
            Display::Flex
        } else {
            Display::None
        };
    }

    if open_block_dropdown.0.is_some() && !active_block_has_panel(&ui_runtime, &world, active_panel)
    {
        open_block_dropdown.0 = None;
    }
    for (binding, mut style) in &mut bound_panels {
        style.display = display_for(active_panel == Some(binding.0));
    }
}

fn active_block_has_panel(
    ui_runtime: &UiRuntime,
    world: &WorldBlocks,
    active_panel: Option<UiPanelId>,
) -> bool {
    let Some(pos) = ui_runtime.active_block_pos() else {
        return false;
    };
    world
        .system_blocks
        .get(&pos)
        .and_then(|block| block.kind.ui_panel())
        == active_panel
}

fn panel_visible(
    visibility: PanelVisibility,
    mode: GameMode,
    settings_tab: SettingsTab,
    ui_runtime: &UiRuntime,
    confirm_dialog: &ConfirmDialogState,
) -> bool {
    match visibility {
        PanelVisibility::GameMode(target_mode) => mode == target_mode,
        PanelVisibility::SettingsTab(tab) => ui_runtime.is_settings_open() && settings_tab == tab,
        PanelVisibility::ConfirmDialog => confirm_dialog.kind.is_some(),
        PanelVisibility::ModalScrim => {
            ui_runtime.has_modal_panel() || confirm_dialog.kind.is_some()
        }
    }
}

fn display_for(visible: bool) -> Display {
    if visible {
        Display::Flex
    } else {
        Display::None
    }
}

pub fn update_ui_layers(
    ui_runtime: Res<UiRuntime>,
    confirm_dialog: Res<ConfirmDialogState>,
    mut layered_nodes: Query<(
        &mut GlobalZIndex,
        Option<&UiPanelBinding>,
        Option<&PanelVisibility>,
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

    for (mut z, binding, visibility) in &mut layered_nodes {
        z.0 = if visibility == Some(&PanelVisibility::ModalScrim) {
            scrim_z
        } else if visibility == Some(&PanelVisibility::ConfirmDialog) {
            confirm_z
        } else if let Some(binding) = binding {
            ui_runtime
                .panel_layer(binding.0)
                .map(panel_layer_z)
                .unwrap_or(PANEL_LAYER_BASE)
        } else if visibility.is_some() {
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
