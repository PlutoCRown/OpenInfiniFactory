#[derive(SystemParam)]
pub struct BlockPanelDropdownParams<'w, 's> {
    pub labels: Query<'w, 's, (&'static BlockPanelDropdownLabel, &'static mut Text)>,
    pub material_slots: Query<'w, 's, (&'static BlockMaterialIconSlot, &'static Children)>,
    pub material_options: Query<'w, 's, (&'static BlockMaterialIcon, &'static Children)>,
    pub material_icons: Query<'w, 's, &'static mut ImageNode>,
    pub lists: Query<
        'w,
        's,
        (
            &'static BlockPanelDropdownList,
            &'static mut Node,
            &'static ComputedNode,
        ),
    >,
    pub triggers: Query<
        'w,
        's,
        (
            &'static BlockEditAction,
            &'static ComputedNode,
            &'static UiGlobalTransform,
        ),
        With<Button>,
    >,
    pub teleport_triggers: Query<
        'w,
        's,
        (
            &'static TeleportAction,
            &'static ComputedNode,
            &'static UiGlobalTransform,
        ),
        With<Button>,
    >,
    pub teleport_pair_list: Query<
        'w,
        's,
        (
            Entity,
            &'static BlockPanelDropdownList,
            Option<&'static Children>,
        ),
    >,
    pub teleport_pair_options: Query<'w, 's, Entity, With<TeleportAction>>,
}

pub fn update_generator_ui(
    ui_runtime: Res<UiRuntime>,
    world: Res<WorldBlocks>,
    mut panel_texts: Query<(&BlockPanelText, &mut Text)>,
) {
    let Some(pos) = ui_runtime.active_block_pos() else {
        return;
    };

    let generator_settings = world.generator_settings(pos);
    for (panel_text, mut text) in &mut panel_texts {
        if panel_text.0 == BlockPanelTextKind::GeneratorPeriod {
            text.0 = generator_settings.period.to_string();
        }
    }
}

pub fn update_labeler_ui(
    ui_runtime: Res<UiRuntime>,
    world: Res<WorldBlocks>,
    i18n: Res<I18n>,
    mut title_text: Query<&mut Text, With<super::types::LocalizedText>>,
) {
    let Some(pos) = ui_runtime.active_block_pos() else {
        return;
    };

    let Some(block) = world.system_blocks.get(&pos) else {
        return;
    };
    let key = match block.kind {
        BlockKind::Stamper => "stamper.title",
        BlockKind::Roller => "roller.title",
        _ => "labeler.title",
    };
    for mut text in &mut title_text {
        if text.0 == i18n.text("labeler.title")
            || text.0 == i18n.text("stamper.title")
            || text.0 == i18n.text("roller.title")
        {
            text.0 = i18n.text(key);
        }
    }
}

pub fn update_converter_ui(mut converter_input_row: Query<&mut Node, With<ConverterInputRow>>) {
    for mut style in &mut converter_input_row {
        style.display = Display::Flex;
    }
}

pub fn update_teleport_ui(
    ui_runtime: Res<UiRuntime>,
    rename_state: Res<TeleportRenameState>,
    world: Res<WorldBlocks>,
    mut panel_texts: Query<(&BlockPanelText, &mut Text)>,
) {
    let Some(pos) = ui_runtime.active_block_pos() else {
        return;
    };

    let settings = world.teleport_settings(pos);
    for (panel_text, mut text) in &mut panel_texts {
        if panel_text.0 == BlockPanelTextKind::TeleportName {
            text.0 = if rename_state.editing == Some(pos) {
                format!("{}_", rename_state.buffer)
            } else {
                settings.name.clone()
            };
        }
    }
}

pub fn update_block_panel_dropdowns_ui(
    mut commands: Commands,
    ui_runtime: Res<UiRuntime>,
    open_dropdown: Res<OpenBlockPanelDropdown>,
    world: Res<WorldBlocks>,
    i18n: Res<I18n>,
    block_icons: Option<Res<BlockIconAssets>>,
    ui_scale: Res<UiScale>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut teleport_pair_cache: Local<Option<(Option<IVec3>, u64, Language, bool)>>,
    mut dropdowns: BlockPanelDropdownParams,
) {
    let active_pos = ui_runtime.active_block_pos();

    for (label, mut text) in &mut dropdowns.labels {
        text.0 = match label.0 {
            BlockPanelDropdown::GeneratorMaterial => active_pos
                .map(|pos| world.generator_settings(pos).material)
                .map(|material| i18n.text(material.name_key()))
                .unwrap_or_default(),
            BlockPanelDropdown::GoalMaterial => active_pos
                .map(|pos| world.goal_settings(pos).material)
                .map(|material| i18n.text(material.name_key()))
                .unwrap_or_default(),
            BlockPanelDropdown::LabelerColor => active_pos
                .map(|pos| world.labeler_color(pos))
                .map(|color| i18n.text(color.name_key()))
                .unwrap_or_default(),
            BlockPanelDropdown::ConverterInput => active_pos
                .map(|pos| world.converter_settings(pos).input)
                .map(|material| i18n.text(material.name_key()))
                .unwrap_or_default(),
            BlockPanelDropdown::ConverterOutput => active_pos
                .map(|pos| world.converter_settings(pos).output)
                .map(|material| i18n.text(material.name_key()))
                .unwrap_or_default(),
            BlockPanelDropdown::TeleportPair => active_pos
                .and_then(|pos| world.teleport_settings(pos).pair)
                .map(|pair| world.teleport_settings(pair).name)
                .unwrap_or_else(|| i18n.text("teleport.none")),
        };
    }

    if let Some(block_icons) = block_icons.as_deref() {
        for (slot, children) in &dropdowns.material_slots {
            let material = selected_material(slot.0, active_pos, &world);
            update_material_children(
                children,
                material,
                block_icons,
                &mut dropdowns.material_icons,
            );
        }
        for (option, children) in &dropdowns.material_options {
            update_material_children(
                children,
                Some(option.0),
                block_icons,
                &mut dropdowns.material_icons,
            );
        }
    }

    let window = windows.single().ok();
    let viewport = window
        .map(|window| Vec2::new(window.width(), window.height()))
        .unwrap_or(Vec2::ZERO);
    let scale = ui_transform_scale(window, ui_scale.0);
    for (list, mut style, list_node) in &mut dropdowns.lists {
        let open = open_dropdown.0 == Some(list.0);
        style.display = if open { Display::Flex } else { Display::None };
        if !open {
            continue;
        }
        if let Some((left, top)) = block_dropdown_position(
            list.0,
            &dropdowns.triggers,
            &dropdowns.teleport_triggers,
            list_node.size(),
            viewport,
            scale,
        ) {
            style.left = Val::Px(left);
            style.top = Val::Px(top);
        }
    }

    let pair_dropdown_open = open_dropdown.0 == Some(BlockPanelDropdown::TeleportPair);
    let pair_cache_key = (
        active_pos,
        world.topology_revision,
        i18n.language(),
        pair_dropdown_open,
    );
    let rebuild_pair_options = *teleport_pair_cache != Some(pair_cache_key);
    if rebuild_pair_options {
        *teleport_pair_cache = Some(pair_cache_key);
    }

    for (entity, list, children) in &mut dropdowns.teleport_pair_list {
        if list.0 != BlockPanelDropdown::TeleportPair {
            continue;
        }
        if !rebuild_pair_options {
            continue;
        }
        if let Some(children) = children {
            for child in children {
                if dropdowns.teleport_pair_options.get(*child).is_ok() {
                    commands.entity(*child).despawn();
                }
            }
        }
        if pair_dropdown_open {
            let Some(pos) = active_pos else {
                continue;
            };
            commands.entity(entity).with_children(|parent| {
                spawn_teleport_pair_option(parent, i18n.text("teleport.none"), None);
                for pair in teleport_pair_candidates(&world, pos) {
                    spawn_teleport_pair_option(
                        parent,
                        world.teleport_settings(pair).name,
                        Some(pair),
                    );
                }
            });
        }
    }
}

fn selected_material(
    dropdown: BlockPanelDropdown,
    active_pos: Option<IVec3>,
    world: &WorldBlocks,
) -> Option<MaterialKind> {
    let pos = active_pos?;
    match dropdown {
        BlockPanelDropdown::GeneratorMaterial => Some(world.generator_settings(pos).material),
        BlockPanelDropdown::GoalMaterial => Some(world.goal_settings(pos).material),
        BlockPanelDropdown::ConverterInput => Some(world.converter_settings(pos).input),
        BlockPanelDropdown::ConverterOutput => Some(world.converter_settings(pos).output),
        BlockPanelDropdown::LabelerColor | BlockPanelDropdown::TeleportPair => None,
    }
}

fn update_material_children(
    children: &Children,
    material: Option<MaterialKind>,
    block_icons: &BlockIconAssets,
    icon_query: &mut Query<&mut ImageNode>,
) {
    let icon = material
        .and_then(BlockKind::material_block_kind)
        .and_then(|kind| block_icons.get(kind));
    for child in children.iter() {
        if let Ok(mut image) = icon_query.get_mut(child) {
            *image = icon.clone().map(ImageNode::new).unwrap_or_default();
        }
    }
}

fn block_dropdown_position(
    dropdown: BlockPanelDropdown,
    triggers: &Query<(&BlockEditAction, &ComputedNode, &UiGlobalTransform), With<Button>>,
    teleport_triggers: &Query<(&TeleportAction, &ComputedNode, &UiGlobalTransform), With<Button>>,
    list_size: Vec2,
    viewport: Vec2,
    scale: f32,
) -> Option<(f32, f32)> {
    let target = block_dropdown_toggle_action(dropdown);
    let trigger = target
        .and_then(|target| {
            triggers
                .iter()
                .find(|(action, node, _)| **action == target && !node.is_empty())
                .map(|(_, node, transform)| (node, transform))
        })
        .or_else(|| {
            (dropdown == BlockPanelDropdown::TeleportPair).then(|| {
                teleport_triggers
                    .iter()
                    .find(|(action, node, _)| {
                        **action == TeleportAction::TogglePairDropdown && !node.is_empty()
                    })
                    .map(|(_, node, transform)| (node, transform))
            })?
        })?;
    dropdown_position_from_trigger(trigger.0, trigger.1, list_size, viewport, scale)
}

fn block_dropdown_toggle_action(dropdown: BlockPanelDropdown) -> Option<BlockEditAction> {
    match dropdown {
        BlockPanelDropdown::GeneratorMaterial | BlockPanelDropdown::GoalMaterial => {
            Some(BlockEditAction::ToggleMaterialDropdown)
        }
        BlockPanelDropdown::LabelerColor => Some(BlockEditAction::ToggleColorDropdown),
        BlockPanelDropdown::ConverterInput => Some(BlockEditAction::ToggleInputDropdown),
        BlockPanelDropdown::ConverterOutput => Some(BlockEditAction::ToggleOutputDropdown),
        BlockPanelDropdown::TeleportPair => None,
    }
}

fn dropdown_position_from_trigger(
    trigger_node: &ComputedNode,
    transform: &UiGlobalTransform,
    list_size: Vec2,
    viewport: Vec2,
    scale: f32,
) -> Option<(f32, f32)> {
    let trigger_size = trigger_node.size();
    let center = (*transform * Vec2::ZERO) * scale;
    let trigger_left = center.x - trigger_size.x * 0.5;
    let trigger_top = center.y - trigger_size.y * 0.5;
    let below = trigger_top + trigger_size.y + 4.0;
    let above = trigger_top - list_size.y - 4.0;
    let top = if below + list_size.y <= viewport.y - 10.0 || above < 10.0 {
        below
    } else {
        above.max(10.0)
    };
    let left = trigger_left.clamp(10.0, (viewport.x - list_size.x - 10.0).max(10.0));
    Some((left, top))
}

fn ui_transform_scale(window: Option<&Window>, ui_scale: f32) -> f32 {
    window.map(Window::scale_factor).unwrap_or(1.0) / ui_scale.max(0.01)
}

fn spawn_teleport_pair_option(
    parent: &mut ChildSpawnerCommands,
    label: String,
    pair: Option<IVec3>,
) {
    parent
        .spawn((menu_button(32.0), TeleportAction::SetPair(pair)))
        .with_children(|button| {
            button.spawn((
                Text::new(label),
                TextFont {
                    font_size: super::components::default_font_size(13.0),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

fn teleport_pair_candidates(world: &WorldBlocks, pos: IVec3) -> Vec<IVec3> {
    let Some(block) = world.system_blocks.get(&pos) else {
        return Vec::new();
    };
    let target_kind = match block.kind {
        BlockKind::TeleportEntrance => BlockKind::TeleportExit,
        BlockKind::TeleportExit => BlockKind::TeleportEntrance,
        _ => return Vec::new(),
    };
    let mut candidates: Vec<IVec3> = world
        .system_blocks
        .iter()
        .filter_map(|(candidate_pos, candidate)| {
            (candidate.kind == target_kind).then_some(*candidate_pos)
        })
        .collect();
    candidates.sort_by_key(|candidate| world.teleport_settings(*candidate).name);
    candidates
}

fn builder_mode_name(mode: BuilderMode, i18n: &I18n) -> String {
    match mode {
        BuilderMode::Edit => i18n.text("mode.edit"),
        BuilderMode::Play => i18n.text("mode.play"),
    }
}
