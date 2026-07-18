use crate::game::blocks::BlockPresent;
pub fn dismiss_active_panel(
    ui_runtime: &mut UiRuntime,
    ui_host: &mut UiHost,
    open_block_dropdown: &mut OpenBlockPanelDropdown,
    open_settings_dropdown: &mut OpenSettingsDropdown,
    pending_key_bind: &mut PendingKeyBind,
    inline_edit: &mut InlineTextEditState,
    drag: &mut PanelDragState,
    commands: &mut Commands,
) -> bool {
    let Some(panel) = ui_runtime.active_panel() else {
        return false;
    };

    if ui_runtime.is_settings_open() {
        open_settings_dropdown.0 = None;
        pending_key_bind.0 = None;
    }
    open_block_dropdown.0 = None;
    inline_edit.clear();
    ui_host.unmount_panel(panel, ui_runtime, Some(commands));
    drag.clear();
    true
}

pub fn close_active_closable_panel(
    ui_runtime: &mut UiRuntime,
    ui_host: &mut UiHost,
    open_block_dropdown: &mut OpenBlockPanelDropdown,
    open_settings_dropdown: &mut OpenSettingsDropdown,
    pending_key_bind: &mut PendingKeyBind,
    inline_edit: &mut InlineTextEditState,
    drag: &mut PanelDragState,
    commands: &mut Commands,
) -> bool {
    if !ui_runtime
        .active_panel()
        .is_some_and(UiPanelId::is_closable)
    {
        return false;
    }
    dismiss_active_panel(
        ui_runtime,
        ui_host,
        open_block_dropdown,
        open_settings_dropdown,
        pending_key_bind,
        inline_edit,
        drag,
        commands,
    )
}

/// Closes the topmost in-game overlay: block panel → inventory → pause menu.
pub fn dismiss_playing_overlay(
    playing_ui: &mut PlayingUiState,
    carried: &mut CarriedItem,
    ui_runtime: &mut UiRuntime,
    ui_host: &mut UiHost,
    open_block_dropdown: &mut OpenBlockPanelDropdown,
    open_settings_dropdown: &mut OpenSettingsDropdown,
    pending_key_bind: &mut PendingKeyBind,
    inline_edit: &mut InlineTextEditState,
    drag: &mut PanelDragState,
    commands: &mut Commands,
) -> bool {
    if close_active_closable_panel(
        ui_runtime,
        ui_host,
        open_block_dropdown,
        open_settings_dropdown,
        pending_key_bind,
        inline_edit,
        drag,
        commands,
    ) {
        return true;
    }
    if playing_ui.inventory_open {
        playing_ui.inventory_open = false;
        carried.clear();
        return true;
    }
    if playing_ui.paused {
        playing_ui.paused = false;
        return true;
    }
    false
}

pub fn update_panel_visibility(
    mode: Res<State<GameMode>>,
    start_menu_screen: Res<StartMenuScreen>,
    playing_ui: Res<PlayingUiState>,
    settings_tab: Res<SettingsTab>,
    ui_runtime: Res<UiRuntime>,
    ui_host: Res<UiHost>,
    world: Res<WorldBlocks>,
    mut open_block_dropdown: ResMut<OpenBlockPanelDropdown>,
    mut primed: Local<bool>,
    added_visibility: Query<(), Added<PanelVisibility>>,
    added_binding: Query<(), Added<UiPanelBinding>>,
    added_window: Query<(), Added<PanelWindow>>,
    mut nodes: ParamSet<(
        Query<(&PanelVisibility, &mut Node)>,
        Query<(&UiPanelBinding, &mut Node)>,
        Query<
            (
                &mut Node,
                &mut Visibility,
                &mut PanelPosition,
                Option<&PanelFlowLayout>,
            ),
            (With<PanelWindow>, Without<TextPromptRoot>),
        >,
    )>,
) {
    let active_panel = ui_runtime.active_panel();
    if open_block_dropdown.0.is_some() && !active_block_has_panel(&ui_runtime, &world, active_panel)
    {
        open_block_dropdown.0 = None;
    }

    let dirty = !*primed
        || mode.is_changed()
        || start_menu_screen.is_changed()
        || playing_ui.is_changed()
        || settings_tab.is_changed()
        || ui_runtime.is_changed()
        || ui_host.is_changed()
        || !added_visibility.is_empty()
        || !added_binding.is_empty()
        || !added_window.is_empty();
    if !dirty {
        return;
    }
    *primed = true;

    let mode = mode.get();
    for (visibility, mut style) in &mut nodes.p0() {
        let next = display_for(panel_visible(
            *visibility,
            *mode,
            *start_menu_screen,
            &playing_ui,
            *settings_tab,
            &ui_runtime,
            &ui_host,
        ));
        if style.display != next {
            style.display = next;
        }
    }

    for (binding, mut style) in &mut nodes.p1() {
        let next = display_for(active_panel == Some(binding.0));
        if style.display != next {
            style.display = next;
        }
    }

    for (mut style, mut visibility, mut position, flow) in &mut nodes.p2() {
        let flow = flow.is_some();
        if style.display == Display::None {
            position.dragged = false;
            reset_panel_layout(&mut style, flow);
            visibility.set_if_neq(Visibility::Hidden);
        } else if position.dragged {
            visibility.set_if_neq(Visibility::Visible);
        } else {
            reset_panel_layout(&mut style, flow);
            visibility.set_if_neq(Visibility::Visible);
        }
    }
}

pub fn panel_close_clicked(
    mut click: On<Pointer<Click>>,
    mut ui_runtime: ResMut<UiRuntime>,
    mut ui_host: ResMut<UiHost>,
    mut open_block_dropdown: ResMut<OpenBlockPanelDropdown>,
    mut open_settings_dropdown: ResMut<OpenSettingsDropdown>,
    mut pending_key_bind: ResMut<PendingKeyBind>,
    mut inline_edit: ResMut<InlineTextEditState>,
    mut drag: ResMut<PanelDragState>,
    mut commands: Commands,
    close_buttons: Query<(), With<PanelCloseButton>>,
) {
    if click.event.button != PointerButton::Primary || close_buttons.get(click.entity).is_err() {
        return;
    }
    click.propagate(false);
    dismiss_active_panel(
        &mut ui_runtime,
        &mut ui_host,
        &mut open_block_dropdown,
        &mut open_settings_dropdown,
        &mut pending_key_bind,
        &mut inline_edit,
        &mut drag,
        &mut commands,
    );
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
    // 统一切到 Absolute：流式面板的 left/top 是偏移量，屏幕坐标只能写给 Absolute
    style.position_type = PositionType::Absolute;
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
    ui_host: &UiHost,
) -> bool {
    match visibility {
        PanelVisibility::StartMenuScreen(screen) => {
            mode == GameMode::StartMenu && start_menu_screen == screen
        }
        PanelVisibility::PauseMenu => mode == GameMode::Playing && playing_ui.paused,
        PanelVisibility::Inventory => mode == GameMode::Playing && playing_ui.inventory_open,
        PanelVisibility::SettingsTab(tab) => ui_runtime.is_settings_open() && settings_tab == tab,
        PanelVisibility::ConfirmDialog => ui_host.confirm_open(),
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
        .or_else(|| world.blocks.get(&pos))
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
    ui_logical_bounds(computed, transform).min
}

fn reset_panel_layout(style: &mut Node, flow: bool) {
    if style.left != Val::Auto {
        style.left = Val::Auto;
    }
    if style.right != Val::Auto {
        style.right = Val::Auto;
    }
    if style.top != Val::Auto {
        style.top = Val::Auto;
    }
    if style.bottom != Val::Auto {
        style.bottom = Val::Auto;
    }
    if flow {
        if style.position_type != PositionType::Relative {
            style.position_type = PositionType::Relative;
        }
        if style.margin != UiRect::all(Val::Px(0.0)) {
            style.margin = UiRect::all(Val::Px(0.0));
        }
    } else {
        if style.position_type != PositionType::Absolute {
            style.position_type = PositionType::Absolute;
        }
        if style.margin != UiRect::all(Val::Auto) {
            style.margin = UiRect::all(Val::Auto);
        }
    }
}

pub fn update_ui_layers(
    ui_runtime: Res<UiRuntime>,
    ui_host: Res<UiHost>,
    editor_open: Option<Res<crate::game::ui::features::virtual_remote::VirtualLayoutEditorOpen>>,
    mut primed: Local<bool>,
    mut last_editor_open: Local<bool>,
    // 不能用 Added<GlobalZIndex>：会与下面的 &mut GlobalZIndex 触发 B0001
    added: Query<(), Or<(Added<UiPanelBinding>, Added<PanelVisibility>)>>,
    mut layered_nodes: Query<(
        &mut GlobalZIndex,
        Option<&UiPanelBinding>,
        Option<&PanelVisibility>,
    )>,
) {
    const BASE_LAYER: i32 = 100;
    const LAYOUT_EDITOR_CONFIRM_Z: i32 = 60_000;

    let editor_flag = editor_open.as_ref().is_some_and(|open| open.0);
    let dirty = !*primed
        || ui_runtime.is_changed()
        || ui_host.is_changed()
        || editor_flag != *last_editor_open
        || editor_open.as_ref().is_some_and(|open| open.is_changed())
        || !added.is_empty();
    *last_editor_open = editor_flag;
    if !dirty {
        return;
    }
    *primed = true;

    let top_panel_z = ui_runtime
        .top_modal_layer()
        .map(panel_layer_z)
        .unwrap_or(PANEL_LAYER_BASE);
    let confirm_z = if ui_host.confirm_open() {
        if editor_flag {
            LAYOUT_EDITOR_CONFIRM_Z
        } else {
            top_panel_z + CONFIRM_LAYER_STEP
        }
    } else {
        PANEL_LAYER_BASE
    };
    for (mut z, binding, visibility) in &mut layered_nodes {
        let next = if visibility == Some(&PanelVisibility::ConfirmDialog) {
            confirm_z
        } else if let Some(binding) = binding {
            ui_runtime
                .panel_layer(binding.0)
                .map(panel_layer_z)
                .unwrap_or(PANEL_LAYER_BASE)
        } else if visibility.is_some() {
            BASE_LAYER
        } else {
            continue;
        };
        if z.0 != next {
            z.0 = next;
        }
    }
}

const PANEL_LAYER_BASE: i32 = 1_000;
const PANEL_LAYER_STEP: i32 = 20;
const CONFIRM_LAYER_STEP: i32 = 20;

fn panel_layer_z(layer: usize) -> i32 {
    PANEL_LAYER_BASE + layer as i32 * PANEL_LAYER_STEP
}
