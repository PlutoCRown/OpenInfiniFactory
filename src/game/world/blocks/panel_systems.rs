use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_scene::{bsn, prelude::EntityCommandsSceneExt};

use crate::game::state::TeleportRenameState;
use crate::game::ui::components::{
    default_button_size, default_font_size, raised_border, HoverButton, BUTTON_BG,
};
use crate::game::ui::types::{
    BlockEditAction, BlockMaterialIcon, BlockMaterialIconSlot, BlockPanelDropdown,
    BlockPanelDropdownLabel, BlockPanelDropdownList, BlockPanelText, BlockPanelTextKind,
    BlockSettingsChanged, ConverterInputRow, LanguageChanged, OpenBlockPanelDropdown,
    TeleportAction, UiPanelContextChanged, UiPanelKey, UiPanelOpened, UiRuntime,
};
use crate::game::world::blocks::{
    converter_settings, generator_settings, goal_settings, labeler_color, teleport_settings,
    BlockKind, MaterialKind,
};
use crate::game::world::grid::WorldBlocks;
use crate::game::world::rendering::BlockIconAssets;
use crate::shared::i18n::{I18n, Language};

const BLOCK_PANEL_KEYS: [UiPanelKey; 5] = [
    UiPanelKey::GENERATOR,
    UiPanelKey::GOAL,
    UiPanelKey::LABELER,
    UiPanelKey::CONVERTER,
    UiPanelKey::TELEPORT,
];

#[derive(SystemParam)]
pub struct BlockPanelLifecycle<'w, 's> {
    opened: MessageReader<'w, 's, UiPanelOpened>,
    context_changed: MessageReader<'w, 's, UiPanelContextChanged>,
    block_settings_changed: MessageReader<'w, 's, BlockSettingsChanged>,
    language_changed: MessageReader<'w, 's, LanguageChanged>,
}

impl BlockPanelLifecycle<'_, '_> {
    fn dirty(&mut self, key: UiPanelKey, active_pos: Option<IVec3>) -> bool {
        self.opened.read().any(|message| message.key == key)
            || self
                .context_changed
                .read()
                .any(|message| message.key == key)
            || self
                .block_settings_changed
                .read()
                .any(|message| Some(message.pos) == active_pos)
            || self.language_changed.read().next().is_some()
    }

    fn any_block_panel_dirty(&mut self) -> bool {
        self.opened
            .read()
            .any(|message| BLOCK_PANEL_KEYS.contains(&message.key))
            || self
                .context_changed
                .read()
                .any(|message| BLOCK_PANEL_KEYS.contains(&message.key))
            || self.block_settings_changed.read().next().is_some()
            || self.language_changed.read().next().is_some()
    }
}

#[derive(SystemParam)]
pub struct BlockPanelDropdownParams<'w, 's> {
    pub labels: Query<'w, 's, (&'static BlockPanelDropdownLabel, &'static mut Text)>,
    pub added_labels: Query<'w, 's, (), Added<BlockPanelDropdownLabel>>,
    pub material_slots: Query<'w, 's, (&'static BlockMaterialIconSlot, &'static Children)>,
    pub added_material_slots: Query<'w, 's, (), Added<BlockMaterialIconSlot>>,
    pub material_options: Query<'w, 's, (&'static BlockMaterialIcon, &'static Children)>,
    pub added_material_options: Query<'w, 's, (), Added<BlockMaterialIcon>>,
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
    pub added_lists: Query<'w, 's, (), Added<BlockPanelDropdownList>>,
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
    pub added_teleport_pair_options: Query<'w, 's, (), Added<TeleportAction>>,
}

pub fn update_generator_ui(
    ui_runtime: Res<UiRuntime>,
    world: Res<WorldBlocks>,
    mut panel_texts: Query<(&BlockPanelText, &mut Text)>,
    added_panel_texts: Query<(), Added<BlockPanelText>>,
    mut lifecycle: BlockPanelLifecycle,
) {
    let active_pos = ui_runtime.active_block_pos();
    if !world.is_changed()
        && added_panel_texts.is_empty()
        && !lifecycle.dirty(UiPanelKey::GENERATOR, active_pos)
    {
        return;
    }

    let Some(pos) = active_pos else {
        return;
    };

    let generator_settings = generator_settings(&world, pos);
    for (panel_text, mut text) in &mut panel_texts {
        if panel_text.kind == BlockPanelTextKind::GeneratorPeriod {
            text.0 = generator_settings.period.to_string();
        }
    }
}

pub fn update_labeler_ui(
    ui_runtime: Res<UiRuntime>,
    world: Res<WorldBlocks>,
    i18n: Res<I18n>,
    mut panel_texts: Query<(&BlockPanelText, &mut Text)>,
    added_panel_texts: Query<(), Added<BlockPanelText>>,
    mut lifecycle: BlockPanelLifecycle,
) {
    let active_pos = ui_runtime.active_block_pos();
    if !world.is_changed()
        && added_panel_texts.is_empty()
        && !lifecycle.dirty(UiPanelKey::LABELER, active_pos)
    {
        return;
    }

    let Some(pos) = active_pos else {
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
    for (panel_text, mut text) in &mut panel_texts {
        if panel_text.kind == BlockPanelTextKind::LabelerTitle {
            text.0 = i18n.text(key);
        }
    }
}

pub fn update_converter_ui(
    mut converter_input_row: Query<&mut Node, With<ConverterInputRow>>,
    added_rows: Query<(), Added<ConverterInputRow>>,
    mut lifecycle: BlockPanelLifecycle,
) {
    if added_rows.is_empty() && !lifecycle.dirty(UiPanelKey::CONVERTER, None) {
        return;
    }

    for mut style in &mut converter_input_row {
        style.display = Display::Flex;
    }
}

pub fn update_teleport_ui(
    ui_runtime: Res<UiRuntime>,
    rename_state: Res<TeleportRenameState>,
    world: Res<WorldBlocks>,
    mut panel_texts: Query<(&BlockPanelText, &mut Text)>,
    added_panel_texts: Query<(), Added<BlockPanelText>>,
    mut lifecycle: BlockPanelLifecycle,
) {
    let active_pos = ui_runtime.active_block_pos();
    if !rename_state.is_changed()
        && !world.is_changed()
        && added_panel_texts.is_empty()
        && !lifecycle.dirty(UiPanelKey::TELEPORT, active_pos)
    {
        return;
    }

    let Some(pos) = active_pos else {
        return;
    };

    let settings = teleport_settings(&world, pos);
    for (panel_text, mut text) in &mut panel_texts {
        if panel_text.kind == BlockPanelTextKind::TeleportName {
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
    mut lifecycle: BlockPanelLifecycle,
) {
    let active_pos = ui_runtime.active_block_pos();
    let lifecycle_dirty = lifecycle.any_block_panel_dirty();
    let pair_dropdown_open = open_dropdown.0 == Some(BlockPanelDropdown::TeleportPair);
    let any_dropdown_open = open_dropdown.0.is_some();
    let refresh_labels =
        world.is_changed() || !dropdowns.added_labels.is_empty() || lifecycle_dirty;
    let refresh_icons = world.is_changed()
        || block_icons.as_ref().is_some_and(|icons| icons.is_changed())
        || !dropdowns.added_material_slots.is_empty()
        || !dropdowns.added_material_options.is_empty()
        || lifecycle_dirty;
    let refresh_layout = any_dropdown_open
        || open_dropdown.is_changed()
        || ui_scale.is_changed()
        || !dropdowns.added_lists.is_empty()
        || lifecycle_dirty;
    let refresh_pairs = open_dropdown.is_changed()
        || world.is_changed()
        || !dropdowns.added_lists.is_empty()
        || !dropdowns.added_teleport_pair_options.is_empty()
        || lifecycle_dirty;

    if !refresh_labels && !refresh_icons && !refresh_layout && !refresh_pairs {
        return;
    }

    if refresh_labels {
        for (label, mut text) in &mut dropdowns.labels {
            text.0 = match label.0 {
                BlockPanelDropdown::GeneratorMaterial => active_pos
                    .map(|pos| generator_settings(&world, pos).material)
                    .map(|material| i18n.text(material.name_key()))
                    .unwrap_or_default(),
                BlockPanelDropdown::GoalMaterial => active_pos
                    .map(|pos| goal_settings(&world, pos).material)
                    .map(|material| i18n.text(material.name_key()))
                    .unwrap_or_default(),
                BlockPanelDropdown::LabelerColor => active_pos
                    .map(|pos| labeler_color(&world, pos))
                    .map(|color| i18n.text(color.name_key()))
                    .unwrap_or_default(),
                BlockPanelDropdown::ConverterInput => active_pos
                    .map(|pos| converter_settings(&world, pos).input)
                    .map(|material| i18n.text(material.name_key()))
                    .unwrap_or_default(),
                BlockPanelDropdown::ConverterOutput => active_pos
                    .map(|pos| converter_settings(&world, pos).output)
                    .map(|material| i18n.text(material.name_key()))
                    .unwrap_or_default(),
                BlockPanelDropdown::TeleportPair => active_pos
                    .and_then(|pos| teleport_settings(&world, pos).pair)
                    .map(|pair| teleport_settings(&world, pair).name)
                    .unwrap_or_else(|| i18n.text("teleport.none")),
            };
        }
    }

    if refresh_icons {
        if let Some(block_icons) = block_icons.as_deref() {
            for (slot, children) in &dropdowns.material_slots {
                let material = selected_material(slot.dropdown, active_pos, &world);
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
    }

    if refresh_layout {
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
    }

    let pair_cache_key = (
        active_pos,
        world.topology_revision,
        i18n.language(),
        pair_dropdown_open,
    );
    let rebuild_pair_options = refresh_pairs && *teleport_pair_cache != Some(pair_cache_key);
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
                        teleport_settings(&world, pair).name,
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
        BlockPanelDropdown::GeneratorMaterial => Some(generator_settings(&world, pos).material),
        BlockPanelDropdown::GoalMaterial => Some(goal_settings(&world, pos).material),
        BlockPanelDropdown::ConverterInput => Some(converter_settings(&world, pos).input),
        BlockPanelDropdown::ConverterOutput => Some(converter_settings(&world, pos).output),
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
        .spawn((Button, HoverButton, TeleportAction::SetPair(pair)))
        .queue_apply_scene(teleport_pair_option_button_scene())
        .queue_spawn_related_scenes::<Children>(teleport_pair_option_label_scene(label));
}

fn teleport_pair_option_button_scene() -> impl bevy_scene::Scene {
    bsn! {
        Node {
            height: Val::Px(default_button_size(32.0)),
            border: UiRect {
                left: Val::Px(3.0),
                right: Val::Px(3.0),
                top: Val::Px(4.0),
                bottom: Val::Px(5.0),
            },
            padding: UiRect::horizontal(Val::Px(14.0)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
        }
        BorderColor {
            top: {raised_border().top},
            right: {raised_border().right},
            bottom: {raised_border().bottom},
            left: {raised_border().left},
        }
        BackgroundColor(BUTTON_BG)
        BoxShadow::new(
            Color::srgba(0.0, 0.0, 0.0, 0.62),
            Val::Px(0.0),
            Val::Px(0.0),
            Val::Px(0.0),
            Val::Px(4.0),
        )
    }
}

fn teleport_pair_option_label_scene(label: String) -> impl bevy_scene::SceneList {
    bsn! {
        (
            Text({label})
            TextFont {
                font_size: {default_font_size(13.0)}
            }
            TextColor(Color::WHITE)
        )
    }
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
    candidates.sort_by_key(|candidate| teleport_settings(&world, *candidate).name);
    candidates
}
