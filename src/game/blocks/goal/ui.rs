use bevy::picking::prelude::{Click, Pointer};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::GoalBlock;

use crate::game::block_editing::OpenBlockPanelDropdown;
use crate::game::block_editing::widgets::{
    MaterialIconSlotBlocked, click_material_slot, spawn_facing_radio_row, spawn_material_icon_list,
    spawn_material_icon_toggle, sync_dropdown_overlay, sync_facing_radio_buttons,
    update_material_icon, update_slot_icon,
};
use crate::game::block_editing::world_refresh::apply_block_settings_edit;
use crate::game::blocks::panels::BlockPanelHooks;
use crate::game::blocks::traits::BlockUi;
use crate::game::blocks::{
    BlockKind, MaterialBlockId, PaintMaterialId, StampMaterialId, material_catalog, paint_catalog,
    stamp_catalog,
};
use crate::game::edit_history::EditHistory;
use crate::game::session::PlayingWorldParams;
use crate::game::state::{SolutionState, UiPanelId};
use crate::game::ui::access::{UiMainThread, i18n};
use crate::game::ui::components::{
    PanelOptions, default_button_size, localized_text, spawn_panel as spawn_ui_panel, text,
    transparent_node,
};
use crate::game::ui::core::host::UiHost;
use crate::game::ui::core::runtime::UiRuntime;
use crate::game::ui::core::text_input::primary_click;
use crate::game::ui::features::block_panels::BlockPanelSystems;
use crate::game::ui::types::{CarriedItem, UiActionLabel, UiPanelBinding};
use crate::game::world::direction::Facing;
use crate::game::world::grid::{GoalSettings, WorldBlocks};
use crate::game::world::rendering::BlockIconAssets;

const MATERIAL_SLOT: u8 = 0;
const STAMP_SLOT_BASE: u8 = 1;
const PAINT_SLOT_BASE: u8 = 5;
const ATTACHMENT_SLOT_COUNT: u8 = 4;

/// 验收附着槽对应的世界法线（北→东→南→西，与预览一致）
const ATTACHMENT_FACES: [IVec3; 4] = [
    IVec3::new(0, 0, -1),
    IVec3::X,
    IVec3::new(0, 0, 1),
    IVec3::NEG_X,
];

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum GoalAction {
    ToggleMaterial,
    SetMaterial(MaterialBlockId),
    SetFacing(Facing),
    ToggleStamp(u8),
    /// `None` = 清空该槽
    SetStamp(Option<StampMaterialId>),
    TogglePaint(u8),
    /// `None` = 清空该槽
    SetPaint(Option<PaintMaterialId>),
}

#[derive(Component, Clone, Copy)]
struct GoalMaterialSlot;

#[derive(Component, Clone, Copy)]
struct GoalMaterialList;

#[derive(Component, Clone, Copy)]
struct GoalMaterialOption(MaterialBlockId);

#[derive(Component, Clone, Copy)]
struct GoalAcceptorIdText;

#[derive(Component, Clone, Copy)]
struct GoalFacingRow;

#[derive(Component, Clone, Copy)]
struct GoalStampSlot(u8);

#[derive(Component, Clone, Copy)]
struct GoalStampList;

#[derive(Component, Clone, Copy)]
struct GoalStampOption(Option<StampMaterialId>);

#[derive(Component, Clone, Copy)]
struct GoalPaintSlot(u8);

#[derive(Component, Clone, Copy)]
struct GoalPaintList;

#[derive(Component, Clone, Copy)]
struct GoalPaintOption(Option<PaintMaterialId>);

#[derive(Component, Clone, Copy)]
struct GoalFaceTooltip;

#[derive(Component, Clone, Copy)]
struct GoalFaceTooltipText;

impl UiActionLabel for GoalAction {
    fn label_key(self) -> &'static str {
        match self {
            Self::ToggleMaterial | Self::SetMaterial(_) | Self::SetFacing(_) => {
                "button.material_next"
            }
            Self::ToggleStamp(_) | Self::SetStamp(_) => "button.next_color",
            Self::TogglePaint(_) | Self::SetPaint(_) => "button.next_color",
        }
    }
}

impl BlockUi for GoalBlock {
    fn ui_panel(&self) -> Option<UiPanelId> {
        Some(UiPanelId::Goal)
    }
}

pub fn spawn_panel(root: &mut ChildSpawnerCommands) {
    spawn_ui_panel(
        root,
        PanelOptions::new(520.0, "goal.title").closable(),
        UiPanelBinding(UiPanelId::Goal),
        |panel| {
            spawn_row(panel, "panel.acceptor_id", |row| {
                row.spawn((text("-", 18.0, Color::WHITE), GoalAcceptorIdText));
            });
            spawn_row(panel, "panel.material", |row| {
                spawn_material_icon_toggle(row, GoalMaterialSlot, GoalAction::ToggleMaterial);
            });
            spawn_facing_radio_row(panel, GoalFacingRow, GoalAction::SetFacing);
            spawn_row(panel, "panel.stamp", |row| {
                for index in 0..ATTACHMENT_SLOT_COUNT {
                    spawn_material_icon_toggle(
                        row,
                        GoalStampSlot(index),
                        GoalAction::ToggleStamp(index),
                    );
                }
            });
            spawn_row(panel, "panel.paint", |row| {
                for index in 0..ATTACHMENT_SLOT_COUNT {
                    spawn_material_icon_toggle(
                        row,
                        GoalPaintSlot(index),
                        GoalAction::TogglePaint(index),
                    );
                }
            });
        },
    );
}

pub fn spawn_overlays(root: &mut ChildSpawnerCommands) {
    spawn_material_icon_list(
        root,
        GoalMaterialList,
        material_catalog()
            .iter()
            .map(|(id, _)| (id, GoalAction::SetMaterial(id))),
        GoalMaterialOption,
    );
    // 首项空槽 = 清空；其后为各印花
    spawn_material_icon_list(
        root,
        GoalStampList,
        std::iter::once((None, GoalAction::SetStamp(None))).chain(
            stamp_catalog()
                .iter()
                .map(|(id, _)| (Some(id), GoalAction::SetStamp(Some(id)))),
        ),
        GoalStampOption,
    );
    spawn_material_icon_list(
        root,
        GoalPaintList,
        std::iter::once((None, GoalAction::SetPaint(None))).chain(
            paint_catalog()
                .iter()
                .map(|(id, _)| (Some(id), GoalAction::SetPaint(Some(id)))),
        ),
        GoalPaintOption,
    );
    spawn_face_tooltip(root);
}

pub fn register(app: &mut App) {
    app.add_observer(on_click).add_systems(
        Update,
        (
            update_panel,
            update_dropdown_overlays,
            update_slot_icons,
            sync_attachment_slot_blocked,
            update_face_tooltip,
        )
            .chain()
            .in_set(BlockPanelSystems),
    );
}

inventory::submit! {
    BlockPanelHooks {
        panel: UiPanelId::Goal,
        spawn_panel: spawn_panel,
        spawn_overlays: spawn_overlays,
        register: register,
    }
}

fn spawn_row(
    panel: &mut ChildSpawnerCommands,
    label_key: &'static str,
    controls: impl FnOnce(&mut ChildSpawnerCommands),
) {
    panel
        .spawn(transparent_node(Node {
            width: Val::Percent(100.0),
            height: Val::Px(default_button_size(54.0)),
            display: Display::Flex,
            align_items: AlignItems::Center,
            column_gap: Val::Px(10.0),
            ..default()
        }))
        .with_children(|row| {
            row.spawn((
                localized_text(label_key, 16.0, Color::srgb(0.86, 0.88, 0.86)),
                Node {
                    width: Val::Px(110.0),
                    ..default()
                },
            ));
            controls(row);
        });
}

fn spawn_face_tooltip(root: &mut ChildSpawnerCommands) {
    const MAX_WIDTH: f32 = 252.0;
    root.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            display: Display::None,
            flex_direction: FlexDirection::Column,
            max_width: Val::Px(MAX_WIDTH),
            padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
            border: UiRect::all(Val::Px(1.0)),
            ..default()
        },
        BorderColor::all(Color::srgba(0.72, 0.82, 0.88, 0.75)),
        BackgroundColor(Color::srgba(0.05, 0.06, 0.07, 0.92)),
        GlobalZIndex(30_000),
        Visibility::Hidden,
        Pickable::IGNORE,
        GoalFaceTooltip,
    ))
    .with_children(|tooltip| {
        tooltip.spawn((
            text("", 14.0, Color::WHITE),
            GoalFaceTooltipText,
            Pickable::IGNORE,
            Node {
                max_width: Val::Percent(100.0),
                ..default()
            },
        ));
    });
}

/// 材料该面是否可挂印花/滚刷
fn face_attachable(settings: &GoalSettings, index: u8) -> bool {
    let Some(face) = ATTACHMENT_FACES.get(index as usize).copied() else {
        return false;
    };
    BlockKind::Material(settings.material).material_face_connectable(settings.facing, face)
}

/// 清掉材料当前朝向下不可附着面上的印花/漆设定
fn clear_unsupported_attachments(settings: &mut GoalSettings) -> bool {
    let mut changed = false;
    for index in 0..ATTACHMENT_SLOT_COUNT {
        if face_attachable(settings, index) {
            continue;
        }
        if settings.stamps[index as usize].take().is_some() {
            changed = true;
        }
        if settings.paints[index as usize].take().is_some() {
            changed = true;
        }
    }
    changed
}

fn on_click(
    mut click: On<Pointer<Click>>,
    ui_host: Res<UiHost>,
    ui_runtime: Res<UiRuntime>,
    mut open_dropdown: ResMut<OpenBlockPanelDropdown>,
    mut carried: ResMut<CarriedItem>,
    mut solution_state: ResMut<SolutionState>,
    mut edit_history: ResMut<EditHistory>,
    mut world: PlayingWorldParams,
    actions: Query<&GoalAction>,
) {
    if ui_host.modal_open() || !primary_click(&mut click) {
        return;
    }
    if ui_runtime.active_panel() != Some(UiPanelId::Goal) {
        return;
    }
    let Ok(action) = actions.get(click.entity).copied() else {
        return;
    };
    click.propagate(false);
    let Some(pos) = ui_runtime.active_block_pos() else {
        return;
    };

    let mut settings = world.world.goal_settings(pos);

    let changed = match action {
        GoalAction::ToggleMaterial => {
            if let Some(material) = click_material_slot(
                UiPanelId::Goal,
                MATERIAL_SLOT,
                &mut carried,
                &mut open_dropdown,
            ) {
                settings.material = material;
                clear_unsupported_attachments(&mut settings);
                true
            } else {
                return;
            }
        }
        GoalAction::SetMaterial(material) => {
            settings.material = material;
            clear_unsupported_attachments(&mut settings);
            open_dropdown.close();
            true
        }
        GoalAction::SetFacing(facing) => {
            settings.facing = facing;
            clear_unsupported_attachments(&mut settings);
            true
        }
        GoalAction::ToggleStamp(index) => {
            if !face_attachable(&settings, index) {
                return;
            }
            open_dropdown.toggle(UiPanelId::Goal, STAMP_SLOT_BASE + index);
            return;
        }
        GoalAction::SetStamp(stamp) => {
            let Some(index) = open_stamp_index(&open_dropdown) else {
                return;
            };
            if !face_attachable(&settings, index as u8) {
                open_dropdown.close();
                return;
            }
            // 再选同一印花 → 清空；空槽 / 其它印花按所选写入
            settings.stamps[index] = match stamp {
                Some(id) if settings.stamps[index] == Some(id) => None,
                other => other,
            };
            open_dropdown.close();
            true
        }
        GoalAction::TogglePaint(index) => {
            if !face_attachable(&settings, index) {
                return;
            }
            open_dropdown.toggle(UiPanelId::Goal, PAINT_SLOT_BASE + index);
            return;
        }
        GoalAction::SetPaint(paint) => {
            let Some(index) = open_paint_index(&open_dropdown) else {
                return;
            };
            if !face_attachable(&settings, index as u8) {
                open_dropdown.close();
                return;
            }
            settings.paints[index] = match paint {
                Some(id) if settings.paints[index] == Some(id) => None,
                other => other,
            };
            open_dropdown.close();
            true
        }
    };

    if changed {
        apply_block_settings_edit(&mut edit_history, &mut world, pos, |blocks| {
            blocks.set_goal_settings(pos, settings);
        });
        solution_state.dirty = true;
    }
}

fn open_stamp_index(open_dropdown: &OpenBlockPanelDropdown) -> Option<usize> {
    let (panel, slot) = open_dropdown.0?;
    if panel != UiPanelId::Goal {
        return None;
    }
    let index = slot.checked_sub(STAMP_SLOT_BASE)?;
    (index < ATTACHMENT_SLOT_COUNT).then_some(index as usize)
}

fn open_paint_index(open_dropdown: &OpenBlockPanelDropdown) -> Option<usize> {
    let (panel, slot) = open_dropdown.0?;
    if panel != UiPanelId::Goal {
        return None;
    }
    let index = slot.checked_sub(PAINT_SLOT_BASE)?;
    (index < ATTACHMENT_SLOT_COUNT).then_some(index as usize)
}

fn update_panel(
    _ui_thread: UiMainThread,
    ui_runtime: Res<UiRuntime>,
    world: Res<WorldBlocks>,
    mut id_text: Query<&mut Text, With<GoalAcceptorIdText>>,
    mut facing_rows: Query<&mut Node, With<GoalFacingRow>>,
    mut facing_buttons: Query<(&GoalAction, &mut BackgroundColor, &mut BorderColor), With<Button>>,
) {
    let Some(pos) = ui_runtime.active_block_pos() else {
        return;
    };
    if ui_runtime.active_panel() != Some(UiPanelId::Goal) {
        return;
    }
    let settings = world.goal_settings(pos);
    let show_facing = BlockKind::Material(settings.material).is_directional();
    for mut node in &mut facing_rows {
        node.display = if show_facing {
            Display::Flex
        } else {
            Display::None
        };
    }
    if show_facing {
        sync_facing_radio_buttons(
            &mut facing_buttons,
            settings.facing,
            |action| match action {
                GoalAction::SetFacing(facing) => Some(*facing),
                _ => None,
            },
        );
    }

    let label = world
        .acceptor_id_at(pos)
        .map(|id| format!("#{}", id.0))
        .unwrap_or_else(|| "-".to_string());
    for mut text in &mut id_text {
        if text.0 != label {
            text.0 = label.clone();
        }
    }
}

fn update_dropdown_overlays(
    _ui_thread: UiMainThread,
    ui_runtime: Res<UiRuntime>,
    open_dropdown: Res<OpenBlockPanelDropdown>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut lists: ParamSet<(
        Query<(&GoalMaterialList, &mut Node, &ComputedNode)>,
        Query<(&GoalStampList, &mut Node, &ComputedNode)>,
        Query<(&GoalPaintList, &mut Node, &ComputedNode)>,
    )>,
    triggers: Query<(&GoalAction, &ComputedNode, &UiGlobalTransform), With<Button>>,
) {
    let panel = UiPanelId::Goal;
    let panel_active = ui_runtime.active_panel() == Some(panel);
    let material_open = panel_active && open_dropdown.is_open(panel, MATERIAL_SLOT);
    let stamp_open = panel_active && open_stamp_index(&open_dropdown).is_some();
    let paint_open = panel_active && open_paint_index(&open_dropdown).is_some();

    let window = windows.single().ok();
    let viewport = window
        .map(|w| Vec2::new(w.width(), w.height()))
        .unwrap_or(Vec2::ZERO);

    let material_trigger = triggers.iter().find_map(|(action, node, transform)| {
        (*action == GoalAction::ToggleMaterial && !node.is_empty()).then_some((node, transform))
    });
    for (_, mut style, list_node) in &mut lists.p0() {
        sync_dropdown_overlay(
            material_open,
            &mut style,
            list_node,
            material_trigger,
            viewport,
        );
    }

    let open_stamp = open_stamp_index(&open_dropdown);
    let stamp_trigger =
        triggers
            .iter()
            .find_map(|(action, node, transform)| match (*action, open_stamp) {
                (GoalAction::ToggleStamp(index), Some(open))
                    if index as usize == open && !node.is_empty() =>
                {
                    Some((node, transform))
                }
                _ => None,
            });
    for (_, mut style, list_node) in &mut lists.p1() {
        sync_dropdown_overlay(stamp_open, &mut style, list_node, stamp_trigger, viewport);
    }

    let open_paint = open_paint_index(&open_dropdown);
    let paint_trigger =
        triggers
            .iter()
            .find_map(|(action, node, transform)| match (*action, open_paint) {
                (GoalAction::TogglePaint(index), Some(open))
                    if index as usize == open && !node.is_empty() =>
                {
                    Some((node, transform))
                }
                _ => None,
            });
    for (_, mut style, list_node) in &mut lists.p2() {
        sync_dropdown_overlay(paint_open, &mut style, list_node, paint_trigger, viewport);
    }
}

fn update_slot_icons(
    _ui_thread: UiMainThread,
    ui_runtime: Res<UiRuntime>,
    world: Res<WorldBlocks>,
    block_icons: Option<Res<BlockIconAssets>>,
    mut material_slots: Query<(&GoalMaterialSlot, &Children)>,
    mut material_options: Query<(&GoalMaterialOption, &Children)>,
    mut stamp_slots: Query<(&GoalStampSlot, &Children)>,
    mut stamp_options: Query<(&GoalStampOption, &Children)>,
    mut paint_slots: Query<(&GoalPaintSlot, &Children)>,
    mut paint_options: Query<(&GoalPaintOption, &Children)>,
    mut material_icons: Query<&mut ImageNode>,
) {
    if ui_runtime.active_panel() != Some(UiPanelId::Goal) {
        return;
    }

    let Some(icons) = block_icons.as_ref() else {
        return;
    };
    let block_icons = icons.as_ref();
    for (option, children) in &mut material_options {
        update_material_icon(children, Some(option.0), block_icons, &mut material_icons);
    }
    for (option, children) in &mut stamp_options {
        update_slot_icon(
            children,
            option
                .0
                .and_then(|id| block_icons.get(BlockKind::stamp_block_kind(id))),
            &mut material_icons,
        );
    }
    for (option, children) in &mut paint_options {
        update_slot_icon(
            children,
            option.0.and_then(|id| block_icons.paint(id)),
            &mut material_icons,
        );
    }

    let settings = ui_runtime
        .active_block_pos()
        .map(|pos| world.goal_settings(pos));
    for (_, children) in &mut material_slots {
        update_material_icon(
            children,
            settings.map(|s| s.material),
            block_icons,
            &mut material_icons,
        );
    }
    for (slot, children) in &mut stamp_slots {
        let stamp = settings.and_then(|s| s.stamps[slot.0 as usize]);
        update_slot_icon(
            children,
            stamp.and_then(|id| block_icons.get(BlockKind::stamp_block_kind(id))),
            &mut material_icons,
        );
    }
    for (slot, children) in &mut paint_slots {
        let paint = settings.and_then(|s| s.paints[slot.0 as usize]);
        update_slot_icon(
            children,
            paint.and_then(|id| block_icons.paint(id)),
            &mut material_icons,
        );
    }
}

/// 不可附着面：打 Blocked 标记并关掉已打开的下拉；顺带清掉无效设定
fn sync_attachment_slot_blocked(
    mut commands: Commands,
    ui_runtime: Res<UiRuntime>,
    mut open_dropdown: ResMut<OpenBlockPanelDropdown>,
    mut solution_state: ResMut<SolutionState>,
    mut edit_history: ResMut<EditHistory>,
    mut playing: PlayingWorldParams,
    stamp_slots: Query<(Entity, &GoalStampSlot, Option<&MaterialIconSlotBlocked>)>,
    paint_slots: Query<(Entity, &GoalPaintSlot, Option<&MaterialIconSlotBlocked>)>,
) {
    if ui_runtime.active_panel() != Some(UiPanelId::Goal) {
        return;
    }
    let Some(pos) = ui_runtime.active_block_pos() else {
        return;
    };
    let mut settings = playing.world.goal_settings(pos);
    if clear_unsupported_attachments(&mut settings) {
        apply_block_settings_edit(&mut edit_history, &mut playing, pos, |blocks| {
            blocks.set_goal_settings(pos, settings);
        });
        solution_state.dirty = true;
    }

    for (entity, slot, blocked) in &stamp_slots {
        let should_block = !face_attachable(&settings, slot.0);
        if should_block != blocked.is_some() {
            if should_block {
                commands.entity(entity).insert(MaterialIconSlotBlocked);
            } else {
                commands.entity(entity).remove::<MaterialIconSlotBlocked>();
            }
        }
        if should_block && open_dropdown.is_open(UiPanelId::Goal, STAMP_SLOT_BASE + slot.0) {
            open_dropdown.close();
        }
    }
    for (entity, slot, blocked) in &paint_slots {
        let should_block = !face_attachable(&settings, slot.0);
        if should_block != blocked.is_some() {
            if should_block {
                commands.entity(entity).insert(MaterialIconSlotBlocked);
            } else {
                commands.entity(entity).remove::<MaterialIconSlotBlocked>();
            }
        }
        if should_block && open_dropdown.is_open(UiPanelId::Goal, PAINT_SLOT_BASE + slot.0) {
            open_dropdown.close();
        }
    }
}

/// 不可附着槽悬停：跟随光标的提示（风格同背包 tooltip）
fn update_face_tooltip(
    _ui_thread: UiMainThread,
    ui_runtime: Res<UiRuntime>,
    stamp_slots: Query<(&GoalStampSlot, &Interaction), With<MaterialIconSlotBlocked>>,
    paint_slots: Query<(&GoalPaintSlot, &Interaction), With<MaterialIconSlotBlocked>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut tooltip: Query<(&mut Node, &mut Visibility), (With<GoalFaceTooltip>, Without<Button>)>,
    mut tooltip_text: Query<&mut Text, With<GoalFaceTooltipText>>,
) {
    let Ok((mut tooltip_node, mut tooltip_visibility)) = tooltip.single_mut() else {
        return;
    };

    let show = ui_runtime.active_panel() == Some(UiPanelId::Goal)
        && (stamp_slots
            .iter()
            .any(|(_, interaction)| *interaction == Interaction::Hovered)
            || paint_slots
                .iter()
                .any(|(_, interaction)| *interaction == Interaction::Hovered));

    if !show {
        if tooltip_node.display != Display::None {
            tooltip_node.display = Display::None;
        }
        tooltip_visibility.set_if_neq(Visibility::Hidden);
        return;
    }
    let Ok(window) = windows.single() else {
        tooltip_node.display = Display::None;
        tooltip_visibility.set_if_neq(Visibility::Hidden);
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        tooltip_node.display = Display::None;
        tooltip_visibility.set_if_neq(Visibility::Hidden);
        return;
    };

    tooltip_visibility.set_if_neq(Visibility::Visible);
    tooltip_node.display = Display::Flex;
    tooltip_node.left = Val::Px(cursor.x + 16.0);
    tooltip_node.top = Val::Px(cursor.y + 16.0);
    if let Ok(mut text) = tooltip_text.single_mut() {
        let label = i18n.t("goal.face_not_attachable");
        if text.0 != label {
            text.0 = label;
        }
    }
}
