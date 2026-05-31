#[derive(SystemParam)]
pub struct BlockPanelDropdownParams<'w, 's> {
    pub labels: Query<'w, 's, (&'static BlockPanelDropdownLabel, &'static mut Text)>,
    pub lists: Query<'w, 's, (&'static BlockPanelDropdownList, &'static mut Node)>,
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
                .map(|pos| world.labeler_settings(pos).color)
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

    for (list, mut style) in &mut dropdowns.lists {
        style.display = if open_dropdown.0 == Some(list.0) {
            Display::Flex
        } else {
            Display::None
        };
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
