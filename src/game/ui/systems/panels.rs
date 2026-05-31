pub fn update_panel_visibility(
    mode: Res<GameMode>,
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
            (&Node, &mut Visibility, &mut PanelPosition),
            (With<PanelWindow>, Without<TextPromptRoot>),
        >,
    )>,
    confirm_dialog: Res<ConfirmDialogState>,
) {
    let active_panel = ui_runtime.active_panel();
    for (visibility, mut style) in &mut nodes.p0() {
        style.display = display_for(panel_visible(
            *visibility,
            *mode,
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

    for (style, mut visibility, mut position) in &mut nodes.p3() {
        if style.display == Display::None {
            position.centered = false;
            position.dragged = false;
            *visibility = Visibility::Hidden;
        } else if position.centered || position.dragged {
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}

pub fn panel_close_clicked(
    mut click: On<Pointer<Click>>,
    mut mode: ResMut<GameMode>,
    mut ui_runtime: ResMut<UiRuntime>,
    mut open_block_dropdown: ResMut<OpenBlockPanelDropdown>,
    mut open_settings_dropdown: ResMut<OpenSettingsDropdown>,
    mut pending_key_bind: ResMut<PendingKeyBind>,
    mut teleport_rename: ResMut<TeleportRenameState>,
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
    teleport_rename.editing = None;
    let return_mode = panel_close_return_mode(&ui_runtime);
    ui_runtime.close_active();
    if let Some(return_mode) = return_mode {
        *mode = return_mode;
    }
    drag.clear();
}

pub fn panel_drag_started(
    mut drag_start: On<Pointer<DragStart>>,
    title_bars: Query<&ChildOf, With<PanelTitleBar>>,
    panels: Query<&Node, With<PanelWindow>>,
    mut drag: ResMut<PanelDragState>,
) {
    if drag_start.event.button != PointerButton::Primary {
        return;
    }
    let Ok(panel) = title_bars.get(drag_start.entity) else {
        return;
    };
    let panel_entity = panel.parent();
    let Ok(style) = panels.get(panel_entity) else {
        return;
    };
    drag_start.propagate(false);
    drag.panel = Some(panel_entity);
    drag.cursor = drag_start.pointer_location.position;
    drag.panel_pos = panel_position(style);
}

pub fn panel_dragged(
    mut drag_event: On<Pointer<Drag>>,
    title_bars: Query<(), With<PanelTitleBar>>,
    ui_scale: Res<UiScale>,
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
    let next = drag.panel_pos
        + screen_to_ui_delta(drag_event.pointer_location.position - drag.cursor, ui_scale.0);
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

fn panel_close_return_mode(ui_runtime: &UiRuntime) -> Option<GameMode> {
    ui_runtime
        .active()
        .and_then(|session| match session.context {
            UiPanelContext::ReturnTo(mode) => Some(mode),
            _ => None,
        })
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

fn panel_position(style: &Node) -> Vec2 {
    Vec2::new(px_or(style.left, 10.0), px_or(style.top, 10.0))
}

fn px_or(value: Val, fallback: f32) -> f32 {
    match value {
        Val::Px(value) => value,
        _ => fallback,
    }
}

fn screen_to_ui_delta(delta: Vec2, ui_scale: f32) -> Vec2 {
    delta / ui_scale.max(0.01)
}

pub fn center_new_panels(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut panels: Query<
        (
            &mut Node,
            &ComputedNode,
            &mut PanelPosition,
            &mut Visibility,
        ),
        With<PanelWindow>,
    >,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let viewport = Vec2::new(window.width(), window.height());

    for (mut style, node, mut position, mut visibility) in &mut panels {
        if style.display == Display::None || position.dragged {
            continue;
        }
        let size = node.size();
        if size.x <= 0.0 || size.y <= 0.0 {
            continue;
        }
        let left = ((viewport.x - size.x) * 0.5).max(10.0);
        let top = ((viewport.y - size.y) * 0.5).max(10.0);
        style.left = Val::Px(left);
        style.top = Val::Px(top);
        style.right = Val::Auto;
        style.bottom = Val::Auto;
        style.margin = UiRect::ZERO;
        position.centered = true;
        *visibility = Visibility::Visible;
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
