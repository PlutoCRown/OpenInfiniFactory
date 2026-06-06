use bevy::ecs::system::SystemParam;

pub fn register_legacy_panels(mut registry: ResMut<UiPanelRegistry>) {
    registry.register(crate::game::ui::types::UiPanelDescriptor::new(
        crate::game::ui::types::UiPanelKey::MAIN_MENU,
        "main.title",
        true,
        crate::game::ui::screens::spawn_main_menu,
    ));
    registry.register(crate::game::ui::types::UiPanelDescriptor::new(
        crate::game::ui::types::UiPanelKey::SAVE_LIST_EDIT,
        "save.title.edit_puzzle",
        true,
        crate::game::ui::screens::spawn_edit_save_list,
    ));
    registry.register(crate::game::ui::types::UiPanelDescriptor::new(
        crate::game::ui::types::UiPanelKey::SAVE_LIST_PLAY,
        "save.title.play_solution",
        true,
        crate::game::ui::screens::spawn_play_save_list,
    ));
    registry.register(crate::game::ui::types::UiPanelDescriptor::new(
        crate::game::ui::types::UiPanelKey::PAUSE_MENU,
        "state.paused",
        true,
        crate::game::ui::screens::spawn_pause_panel,
    ));
    registry.register(crate::game::ui::types::UiPanelDescriptor::new(
        crate::game::ui::types::UiPanelKey::INVENTORY,
        "inventory.title",
        true,
        crate::game::ui::screens::spawn_inventory_panel,
    ));
    registry.register(crate::game::ui::types::UiPanelDescriptor::new(
        crate::game::ui::types::UiPanelKey::SETTINGS,
        "settings.title",
        true,
        crate::game::ui::screens::spawn_settings_panel,
    ));
    registry.register(crate::game::ui::types::UiPanelDescriptor::new(
        crate::game::ui::types::UiPanelKey::GENERATOR,
        "generator.title",
        true,
        crate::game::world::blocks::generator::ui::spawn_panel,
    ));
    registry.register(crate::game::ui::types::UiPanelDescriptor::new(
        crate::game::ui::types::UiPanelKey::GOAL,
        "goal.title",
        true,
        crate::game::world::blocks::goal::ui::spawn_panel,
    ));
    registry.register(crate::game::ui::types::UiPanelDescriptor::new(
        crate::game::ui::types::UiPanelKey::LABELER,
        "labeler.title",
        true,
        crate::game::world::blocks::labeler::ui::spawn_panel,
    ));
    registry.register(crate::game::ui::types::UiPanelDescriptor::new(
        crate::game::ui::types::UiPanelKey::CONVERTER,
        "converter.title",
        true,
        crate::game::world::blocks::converter::ui::spawn_panel,
    ));
    registry.register(crate::game::ui::types::UiPanelDescriptor::new(
        crate::game::ui::types::UiPanelKey::TELEPORT,
        "teleport.title",
        true,
        crate::game::world::blocks::teleport_entrance::ui::spawn_panel,
    ));
}

pub fn open_panel_messages(
    mut commands: Commands,
    i18n: Res<I18n>,
    registry: Res<UiPanelRegistry>,
    mut host: ResMut<UiPanelHost>,
    mut ui_runtime: ResMut<UiRuntime>,
    mut messages: MessageReader<OpenUiPanel>,
    mut opened: MessageWriter<UiPanelOpened>,
    mut context_changed: MessageWriter<UiPanelContextChanged>,
    roots: Query<Entity, With<UiRoot>>,
) {
    let Ok(root) = roots.single() else {
        return;
    };
    for message in messages.read() {
        let Some(descriptor) = registry.get(message.key) else {
            warn!("Ignoring unregistered UI panel {:?}", message.key);
            continue;
        };
        if host.get(descriptor.key).is_none() {
            let mut spawned = None;
            commands.entity(root).with_children(|root| {
                spawned = Some((descriptor.spawn)(root, &i18n));
            });
            if let Some(spawned) = spawned {
                host.insert(descriptor.key, spawned);
            }
        }
        let old_context = ui_runtime.open_key(
            descriptor.key,
            message.context,
            descriptor.blocks_gameplay,
        );
        if let Some(old) = old_context {
            if old != message.context {
                context_changed.write(UiPanelContextChanged {
                    key: descriptor.key,
                    old,
                    new: message.context,
                });
            }
        } else {
            opened.write(UiPanelOpened {
                key: descriptor.key,
                context: message.context,
            });
        }
    }
}

pub fn open_initial_panel(mut open: MessageWriter<OpenUiPanel>) {
    open.write(OpenUiPanel::new(
        UiPanelKey::MAIN_MENU,
        UiPanelContext::None,
    ));
}

pub fn sync_mode_panels(
    mode: Res<GameMode>,
    solution_state: Res<SolutionState>,
    mut open: MessageWriter<OpenUiPanel>,
) {
    if !mode.is_changed() {
        return;
    }
    let key = match *mode {
        GameMode::MainMenu => UiPanelKey::MAIN_MENU,
        GameMode::SaveListMain => match solution_state.save_list_entry {
            WorldEntryMode::EditPuzzle => UiPanelKey::SAVE_LIST_EDIT,
            WorldEntryMode::PlaySolution => UiPanelKey::SAVE_LIST_PLAY,
        },
        GameMode::Inventory => UiPanelKey::INVENTORY,
        GameMode::Paused => UiPanelKey::PAUSE_MENU,
        GameMode::Playing => return,
    };
    open.write(OpenUiPanel::new(key, UiPanelContext::None));
}

pub fn close_panel_messages(
    registry: Res<UiPanelRegistry>,
    mut ui_runtime: ResMut<UiRuntime>,
    mut messages: MessageReader<CloseUiPanel>,
    mut closed: MessageWriter<UiPanelClosed>,
) {
    for message in messages.read() {
        let Some(key) = message.key else {
            if let Some(session) = ui_runtime.close_active() {
                closed.write(UiPanelClosed { key: session.panel });
            }
            continue;
        };
        let Some(descriptor) = registry.get(key) else {
            continue;
        };
        if let Some(session) = ui_runtime.close_panel(descriptor.key) {
            closed.write(UiPanelClosed { key: session.panel });
        }
    }
}

pub fn modal_messages(
    mut commands: Commands,
    mut ui_runtime: ResMut<UiRuntime>,
    mut confirm: MessageReader<OpenConfirmDialog>,
    mut prompt: MessageReader<OpenTextPrompt>,
    mut close: MessageReader<CloseUiModal>,
    mut opened: MessageWriter<UiModalOpened>,
    mut closed: MessageWriter<UiModalClosed>,
    roots: Query<Entity, With<UiRoot>>,
    confirm_roots: Query<(), With<ConfirmDialogRoot>>,
) {
    for message in close.read() {
        let _ = message;
        if let Some(kind) = ui_runtime.modal_kind() {
            closed.write(UiModalClosed { kind });
        }
        ui_runtime.close_modal();
    }
    for message in confirm.read() {
        if confirm_roots.is_empty() {
            if let Ok(root) = roots.single() {
                commands.entity(root).with_children(|root| {
                    crate::game::ui::screens::spawn_confirm_dialog(root);
                });
            }
        }
        if let Some(kind) = ui_runtime.modal_kind() {
            closed.write(UiModalClosed { kind });
        }
        ui_runtime.open_confirm_dialog(message.0.clone());
        opened.write(UiModalOpened {
            kind: UiModalKind::ConfirmDialog,
        });
    }
    for message in prompt.read() {
        if let Some(kind) = ui_runtime.modal_kind() {
            closed.write(UiModalClosed { kind });
        }
        ui_runtime.open_text_prompt(message.kind.clone(), &message.value);
        opened.write(UiModalOpened {
            kind: UiModalKind::TextPrompt,
        });
    }
}

#[derive(SystemParam)]
pub struct PanelLifecycleTriggers<'w, 's> {
    added_panel_visibility: Query<'w, 's, (), Added<PanelVisibility>>,
    added_panel_windows: Query<'w, 's, (), Added<PanelWindow>>,
    opened: MessageReader<'w, 's, UiPanelOpened>,
    closed: MessageReader<'w, 's, UiPanelClosed>,
    context_changed: MessageReader<'w, 's, UiPanelContextChanged>,
}

impl PanelLifecycleTriggers<'_, '_> {
    fn dirty(&mut self) -> bool {
        !self.added_panel_visibility.is_empty()
            || !self.added_panel_windows.is_empty()
            || self.opened.read().next().is_some()
            || self.closed.read().next().is_some()
            || self.context_changed.read().next().is_some()
    }
}

pub fn open_panel_button_clicked(
    mut click: On<Pointer<Click>>,
    buttons: Query<&OpensPanel>,
    mut open: MessageWriter<OpenUiPanel>,
) {
    if click.event.button != PointerButton::Primary {
        return;
    }
    let Ok(target) = buttons.get(click.entity).copied() else {
        return;
    };
    click.propagate(false);
    open.write(OpenUiPanel::new(target.key, target.context));
}

pub fn update_panel_visibility(
    mode: Res<GameMode>,
    save_state: Res<SaveState>,
    solution_state: Res<SolutionState>,
    settings_tab: Res<SettingsTab>,
    ui_runtime: Res<UiRuntime>,
    world: Res<WorldBlocks>,
    mut open_block_dropdown: ResMut<OpenBlockPanelDropdown>,
    mut lifecycle: PanelLifecycleTriggers,
    mut nodes: ParamSet<(
        Query<
            (&PanelVisibility, Option<&UiPanelBinding>, &mut Node),
            Without<PauseMenuAction>,
        >,
        Query<
            (&PauseMenuAction, &mut Node),
            (With<Button>, Without<PanelVisibility>, Without<UiPanelBinding>),
        >,
        Query<
            (&UiPanelBinding, &mut Node),
            (Without<PanelVisibility>, Without<PauseMenuAction>),
        >,
        Query<(&Node, &mut PanelPosition), (With<PanelWindow>, Without<TextPromptRoot>)>,
    )>,
) {
    let _ = lifecycle.dirty();

    let active_panel = ui_runtime.active_key();
    for (visibility, binding, mut style) in &mut nodes.p0() {
        let stack_visible = binding.is_none_or(|binding| active_panel == Some(binding.0));
        let visible = stack_visible && panel_visible(*visibility, *mode, *settings_tab, &ui_runtime);
        style.display = display_for(visible);
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

    for (style, mut position) in &mut nodes.p3() {
        if style.display == Display::None {
            position.centered = false;
            position.dragged = false;
        }
    }
}

pub fn panel_close_clicked(
    mut click: On<Pointer<Click>>,
    mut close_panel: MessageWriter<CloseUiPanel>,
    close_buttons: Query<(), With<PanelCloseButton>>,
) {
    if click.event.button != PointerButton::Primary || close_buttons.get(click.entity).is_err() {
        return;
    }
    click.propagate(false);

    close_panel.write(CloseUiPanel { key: None });
}

pub fn cleanup_closed_panel_state(
    mut closed: MessageReader<UiPanelClosed>,
    mut open_block_dropdown: ResMut<OpenBlockPanelDropdown>,
    mut teleport_rename: ResMut<TeleportRenameState>,
    mut drag: ResMut<PanelDragState>,
) {
    if closed.read().next().is_none() {
        return;
    }
    open_block_dropdown.0 = None;
    teleport_rename.editing = None;
    drag.clear();
}

pub fn apply_game_mode_lifecycle(
    mode: Res<GameMode>,
    mut ui_runtime: ResMut<UiRuntime>,
    mut open_block_dropdown: ResMut<OpenBlockPanelDropdown>,
    mut teleport_rename: ResMut<TeleportRenameState>,
    mut drag: ResMut<PanelDragState>,
    mut hover: ResMut<UiHoverState>,
) {
    if !mode.is_changed() {
        return;
    }

    match *mode {
        GameMode::MainMenu => ui_runtime.show_main_menu(),
        GameMode::Playing => ui_runtime.clear_runtime_panels(),
        GameMode::SaveListMain | GameMode::Inventory | GameMode::Paused => {}
    }
    open_block_dropdown.0 = None;
    teleport_rename.editing = None;
    drag.clear();
    hover.entity = None;
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
        + screen_to_ui_delta(
            drag_event.pointer_location.position - drag.cursor,
            ui_scale.0,
        );
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

fn active_block_has_panel(
    ui_runtime: &UiRuntime,
    world: &WorldBlocks,
    active_panel: Option<UiPanelKey>,
) -> bool {
    let Some(pos) = ui_runtime.active_block_pos() else {
        return false;
    };
    world
        .system_blocks
        .get(&pos)
        .and_then(|block| block.kind.ui_panel_key())
        == active_panel
}

fn panel_visible(
    visibility: PanelVisibility,
    mode: GameMode,
    settings_tab: SettingsTab,
    ui_runtime: &UiRuntime,
) -> bool {
    match visibility {
        PanelVisibility::GameMode(target_mode) => mode == target_mode,
        PanelVisibility::SettingsTab(tab) => ui_runtime.is_settings_open() && settings_tab == tab,
        PanelVisibility::ConfirmDialog => ui_runtime.confirm_dialog().is_some(),
        PanelVisibility::ModalScrim => ui_runtime.has_modal_panel() || ui_runtime.has_modal(),
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
        (&mut Node, &ComputedNode, &mut PanelPosition),
        With<PanelWindow>,
    >,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let viewport = Vec2::new(window.width(), window.height());

    for (mut style, node, mut position) in &mut panels {
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
    }
}

pub fn update_ui_layers(
    ui_runtime: Res<UiRuntime>,
    mut lifecycle: PanelLifecycleTriggers,
    mut layered_nodes: Query<(
        &mut GlobalZIndex,
        Option<&UiPanelBinding>,
        Option<&PanelVisibility>,
    )>,
) {
    if !ui_runtime.is_changed() && !lifecycle.dirty() {
        return;
    }

    const BASE_LAYER: i32 = 100;

    let top_panel_z = ui_runtime
        .top_modal_layer()
        .map(panel_layer_z)
        .unwrap_or(PANEL_LAYER_BASE);
    let confirm_z = if ui_runtime.confirm_dialog().is_some() {
        top_panel_z + CONFIRM_LAYER_STEP
    } else {
        PANEL_LAYER_BASE
    };
    let scrim_z = if ui_runtime.confirm_dialog().is_some() {
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
