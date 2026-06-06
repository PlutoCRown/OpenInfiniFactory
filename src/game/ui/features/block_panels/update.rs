use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::game::block_editing::{
    BlockMaterialIcon, BlockMaterialIconSlot, BlockPanelAction, BlockPanelDropdown,
    BlockPanelDropdownLabel, BlockPanelDropdownList, BlockPanelText, BlockPanelTextKind,
    BlockPanelTitle, OpenBlockPanelDropdown,
};
use crate::game::ui::components::{default_font_size, menu_button};
use crate::game::ui::core::runtime::UiRuntime;
use crate::game::ui::core::text_input::InlineTextEditState;
use crate::game::ui::types::ConverterInputRow;
use crate::game::world::blocks::{BlockKind, MaterialKind};
use crate::game::world::grid::WorldBlocks;
use crate::game::world::rendering::BlockIconAssets;
use crate::game::ui::access::{i18n, UiMainThread};
use crate::shared::i18n::Language;

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
            &'static BlockPanelAction,
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
    pub teleport_pair_options: Query<'w, 's, Entity, With<BlockPanelAction>>,
}

pub fn update_active_block_panel(
    _ui_thread: UiMainThread,
    ui_runtime: Res<UiRuntime>,
    inline_edit: Res<InlineTextEditState>,
    world: Res<WorldBlocks>,
    mut texts: ParamSet<(
        Query<(&BlockPanelText, &mut Text)>,
        Query<&mut Text, With<BlockPanelTitle>>,
    )>,
    mut converter_input_row: Query<&mut Node, With<ConverterInputRow>>,
) {
    let Some(pos) = ui_runtime.active_block_pos() else {
        return;
    };

    let generator_settings = world.generator_settings(pos);
    for (panel_text, mut text) in &mut texts.p0() {
        match panel_text.0 {
            BlockPanelTextKind::GeneratorPeriod => {
                text.0 = generator_settings.period.to_string();
            }
            BlockPanelTextKind::TeleportName => {
                let settings = world.teleport_settings(pos);
                text.0 = if inline_edit.is_active()
                    && inline_edit.pos == Some(pos)
                    && inline_edit.field == Some("teleport_name")
                {
                    format!("{}_", inline_edit.buffer)
                } else {
                    settings.name.clone()
                };
            }
        }
    }

    if let Some(block) = world.system_blocks.get(&pos) {
        let key = match block.kind {
            BlockKind::Stamper => "stamper.title",
            BlockKind::Roller => "roller.title",
            _ => "labeler.title",
        };
        for mut text in &mut texts.p1() {
            text.0 = i18n.t(key);
        }
    }

    for mut style in &mut converter_input_row {
        style.display = Display::Flex;
    }
}

pub fn update_block_panel_dropdowns(
    _ui_thread: UiMainThread,
    mut commands: Commands,
    ui_runtime: Res<UiRuntime>,
    open_dropdown: Res<OpenBlockPanelDropdown>,
    world: Res<WorldBlocks>,
    block_icons: Option<Res<BlockIconAssets>>,
    ui_scale: Res<UiScale>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut teleport_pair_cache: Local<Option<(Option<IVec3>, u64, Language, bool)>>,
    mut dropdowns: BlockPanelDropdownParams,
) {
    let active_pos = ui_runtime.active_block_pos();

    for (label, mut text) in &mut dropdowns.labels {
        text.0 = label.0.selected_label(active_pos, &world);
    }

    if let Some(block_icons) = block_icons.as_deref() {
        for (slot, children) in &dropdowns.material_slots {
            if !slot.0.uses_material_icon() {
                continue;
            }
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
        if !list.0.is_dynamic() {
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
                spawn_teleport_pair_option(parent, i18n.t("teleport.none"), None);
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
    dropdown.selected_material(active_pos, world)
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
    triggers: &Query<(&BlockPanelAction, &ComputedNode, &UiGlobalTransform), With<Button>>,
    list_size: Vec2,
    viewport: Vec2,
    scale: f32,
) -> Option<(f32, f32)> {
    let target = dropdown.toggle_action();
    let trigger = triggers
        .iter()
        .find(|(action, node, _)| **action == target && !node.is_empty())
        .map(|(_, node, transform)| (node, transform))?;
    dropdown_position_from_trigger(trigger.0, trigger.1, list_size, viewport, scale)
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
        .spawn((menu_button(32.0), BlockPanelAction::SetTeleportPair(pair)))
        .with_children(|button| {
            button.spawn((
                Text::new(label),
                TextFont {
                    font_size: default_font_size(13.0),
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
