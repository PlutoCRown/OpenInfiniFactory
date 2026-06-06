pub fn update_panel_visibility(
    mode: Res<State<GameMode>>,
    start_menu_screen: Res<StartMenuScreen>,
    playing_ui: Res<PlayingUiState>,
    save_state: Res<SaveState>,
    solution_state: Res<SolutionState>,
    settings_tab: Res<SettingsTab>,
    ui_runtime: Res<UiRuntime>,
    world: Res<WorldBlocks>,
    mut open_block_dropdown: ResMut<OpenBlockPanelDropdown>,
    mut nodes: ParamSet<(
        Query<(&PanelVisibility, &mut Node)>,
        Query<(&MenuAction, &mut Node), With<Button>>,
        Query<(&UiPanelBinding, &mut Node)>,
        Query<
            (&mut Node, &mut Visibility, &mut PanelPosition),
            (With<PanelWindow>, Without<TextPromptRoot>),
        >,
    )>,
    confirm_dialog: Res<ConfirmDialogState>,
) {
    let active_panel = ui_runtime.active_panel();
    let mode = mode.get();
    for (visibility, mut style) in &mut nodes.p0() {
        style.display = display_for(panel_visible(
            *visibility,
            *mode,
            *start_menu_screen,
            &playing_ui,
            *settings_tab,
            &ui_runtime,
            &confirm_dialog,
        ));
    }

    for (action, mut style) in &mut nodes.p1() {
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
    for (binding, mut style) in &mut nodes.p2() {
        style.display = display_for(active_panel == Some(binding.0));
    }

    for (mut style, mut visibility, mut position) in &mut nodes.p3() {
        if style.display == Display::None {
            position.dragged = false;
            reset_panel_centering(&mut style);
            *visibility = Visibility::Hidden;
        } else if position.dragged {
            *visibility = Visibility::Visible;
        } else {
            reset_panel_centering(&mut style);
            *visibility = Visibility::Visible;
        }
    }
}

pub fn panel_close_clicked(
    mut click: On<Pointer<Click>>,
    mut ui_runtime: ResMut<UiRuntime>,
    mut open_block_dropdown: ResMut<OpenBlockPanelDropdown>,
    mut open_settings_dropdown: ResMut<OpenSettingsDropdown>,
    mut pending_key_bind: ResMut<PendingKeyBind>,
    mut inline_edit: ResMut<InlineTextEditState>,
    mut drag: ResMut<PanelDragState>,
    close_buttons: Query<(), With<PanelCloseButton>>,
) {
    if click.event.button != PointerButton::Primary || close_buttons.get(click.entity).is_err() {
        return;
    }
    click.propagate(false);

    if ui_runtime.is_settings_open() {
        open_settings_dropdown.0 = None;
        pending_key_bind.0 = None;
    }
    open_block_dropdown.0 = None;
    inline_edit.clear();
    ui_runtime.close_active();
    drag.clear();
}

pub fn panel_drag_started(
    mut drag_start: On<Pointer<DragStart>>,
    title_bars: Query<&ChildOf, With<PanelTitleBar>>,
    mut panels: Query<
        (
            &mut Node,
            &ComputedNode,
            &UiGlobalTransform,
            &mut PanelPosition,
        ),
        With<PanelWindow>,
    >,
    mut drag: ResMut<PanelDragState>,
) {
    if drag_start.event.button != PointerButton::Primary {
        return;
    }
    let Ok(panel) = title_bars.get(drag_start.entity) else {
        return;
    };
    let panel_entity = panel.parent();
    let Ok((mut style, computed, transform, mut position)) = panels.get_mut(panel_entity) else {
        return;
    };
    drag_start.propagate(false);
    let top_left = panel_logical_top_left(computed, transform);
    // Margin-auto centering does not populate `Node.left/top`; pin the laid-out position
    // before switching to pointer-driven coordinates so the panel does not jump.
    style.left = Val::Px(top_left.x);
    style.top = Val::Px(top_left.y);
    style.right = Val::Auto;
    style.bottom = Val::Auto;
    style.margin = UiRect::ZERO;
    position.dragged = true;
    drag.panel = Some(panel_entity);
    drag.grab_offset = drag_start.pointer_location.position - top_left;
}

pub fn panel_dragged(
    mut drag_event: On<Pointer<Drag>>,
    title_bars: Query<(), With<PanelTitleBar>>,
    mut drag: ResMut<PanelDragState>,
    mut panels: Query<(&mut Node, &mut PanelPosition), With<PanelWindow>>,
) {
    if drag_event.event.button != PointerButton::Primary
        || title_bars.get(drag_event.entity).is_err()
    {
        return;
    }
    let Some(panel) = drag.panel else {
        return;
    };
    let Ok((mut style, mut position)) = panels.get_mut(panel) else {
        drag.clear();
        return;
    };
    drag_event.propagate(false);
    let next = drag_event.pointer_location.position - drag.grab_offset;
    style.left = Val::Px(next.x.max(10.0));
    style.top = Val::Px(next.y.max(10.0));
    style.right = Val::Auto;
    style.bottom = Val::Auto;
    style.margin = UiRect::ZERO;
    position.dragged = true;
}

pub fn panel_drag_ended(
    mut drag_end: On<Pointer<DragEnd>>,
    title_bars: Query<(), With<PanelTitleBar>>,
    mut drag: ResMut<PanelDragState>,
) {
    if drag_end.event.button != PointerButton::Primary || title_bars.get(drag_end.entity).is_err() {
        return;
    }
    drag_end.propagate(false);
    drag.clear();
}

fn panel_visible(
    visibility: PanelVisibility,
    mode: GameMode,
    start_menu_screen: StartMenuScreen,
    playing_ui: &PlayingUiState,
    settings_tab: SettingsTab,
    ui_runtime: &UiRuntime,
    confirm_dialog: &ConfirmDialogState,
) -> bool {
    match visibility {
        PanelVisibility::StartMenuScreen(screen) => {
            mode == GameMode::StartMenu && start_menu_screen == screen
        }
        PanelVisibility::PauseMenu => mode == GameMode::Playing && playing_ui.paused,
        PanelVisibility::Inventory => mode == GameMode::Playing && playing_ui.inventory_open,
        PanelVisibility::SettingsTab(tab) => ui_runtime.is_settings_open() && settings_tab == tab,
        PanelVisibility::ConfirmDialog => confirm_dialog.kind.is_some(),
        PanelVisibility::ModalScrim => {
            ui_runtime.has_modal_panel() || confirm_dialog.kind.is_some()
        }
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

fn display_for(visible: bool) -> Display {
    if visible {
        Display::Flex
    } else {
        Display::None
    }
}

fn panel_logical_top_left(computed: &ComputedNode, transform: &UiGlobalTransform) -> Vec2 {
    let node_rect = Rect::from_center_size(transform.translation.trunc(), computed.size());
    node_rect.min * computed.inverse_scale_factor()
}

fn reset_panel_centering(style: &mut Node) {
    style.left = Val::Auto;
    style.right = Val::Auto;
    style.top = Val::Auto;
    style.bottom = Val::Auto;
    style.margin = UiRect::all(Val::Auto);
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
