//! 印花机/滚筒共用的单色图标选择槽与下拉

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::game::block_editing::OpenBlockPanelDropdown;
use crate::game::block_editing::widgets::{
    spawn_labeled_control_row, spawn_material_icon_list, spawn_material_icon_toggle,
    sync_dropdown_overlay, update_slot_icon,
};
use crate::game::blocks::{BlockKind, PaintMaterialId, StampMaterialId};
use crate::game::state::UiPanelId;
use crate::game::ui::access::UiMainThread;
use crate::game::ui::core::runtime::UiRuntime;
use crate::game::world::grid::WorldBlocks;
use crate::game::world::rendering::BlockIconAssets;

/// 颜色下拉槽位编号（印花机/滚筒面板各只有这一格）
pub const COLOR_SLOT: u8 = 0;

/// 颜色选择格（与 Toggle 动作同实体）
#[derive(Component, Clone, Copy)]
pub struct ColorSelectSlot;

/// 颜色选项悬浮列表
#[derive(Component, Clone, Copy)]
pub struct ColorSelectList;

/// 下拉选项上的图标来源
#[derive(Component, Clone, Copy)]
pub enum ColorSelectOption {
    Stamp(StampMaterialId),
    Paint(PaintMaterialId),
}

impl ColorSelectOption {
    /// 解析选项对应的图标句柄
    fn icon(self, block_icons: &BlockIconAssets) -> Option<Handle<Image>> {
        match self {
            Self::Stamp(id) => block_icons.get(BlockKind::stamp_block_kind(id)),
            Self::Paint(id) => block_icons.paint(id),
        }
    }
}

/// 生成「颜色」标签行与图标开关
pub fn spawn_color_select_row<A: Component + Copy>(
    panel: &mut ChildSpawnerCommands,
    toggle_action: A,
) {
    spawn_labeled_control_row(panel, "panel.color", 40.0, |row| {
        spawn_material_icon_toggle(row, ColorSelectSlot, toggle_action);
    });
}

/// 生成颜色选项悬浮列表
pub fn spawn_color_select_list<A, Id>(
    root: &mut ChildSpawnerCommands,
    options: impl IntoIterator<Item = (Id, A)>,
    to_option: fn(Id) -> ColorSelectOption,
) where
    A: Component + Copy,
    Id: Copy,
{
    spawn_material_icon_list(root, ColorSelectList, options, to_option);
}

/// 同步颜色下拉显隐与槽位/选项图标
pub(crate) fn update_color_select_dropdowns(
    _ui_thread: UiMainThread,
    ui_runtime: Res<UiRuntime>,
    open_dropdown: Res<OpenBlockPanelDropdown>,
    world: Res<WorldBlocks>,
    block_icons: Option<Res<BlockIconAssets>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut slots: Query<(&ColorSelectSlot, &Children)>,
    mut options: Query<(&ColorSelectOption, &Children)>,
    mut icons: Query<&mut ImageNode>,
    mut lists: Query<(&ColorSelectList, &mut Node, &ComputedNode)>,
    triggers: Query<(&ColorSelectSlot, &ComputedNode, &UiGlobalTransform), With<Button>>,
) {
    let panel = ui_runtime.active_panel();
    let color_panel = matches!(panel, Some(UiPanelId::Stamper) | Some(UiPanelId::Roller));
    let open = color_panel && panel.is_some_and(|p| open_dropdown.is_open(p, COLOR_SLOT));

    let window = windows.single().ok();
    let viewport = window
        .map(|w| Vec2::new(w.width(), w.height()))
        .unwrap_or(Vec2::ZERO);
    let trigger = triggers
        .iter()
        .find_map(|(_, node, transform)| (!node.is_empty()).then_some((node, transform)));
    for (_, mut style, list_node) in &mut lists {
        sync_dropdown_overlay(open, &mut style, list_node, trigger, viewport);
    }

    if !color_panel {
        return;
    }

    // 不缓存「已填充」：关面板时本系统被 run_if 跳过，Local 清不掉，二次打开会跳过刷新
    let Some(block_icons_res) = block_icons.as_ref() else {
        return;
    };
    let block_icons = block_icons_res.as_ref();
    for (option, children) in &mut options {
        update_slot_icon(children, option.icon(block_icons), &mut icons);
    }

    let selected = ui_runtime.active_block_pos().and_then(|pos| match panel {
        Some(UiPanelId::Stamper) => {
            Some(ColorSelectOption::Stamp(world.stamper_settings(pos).stamp))
        }
        Some(UiPanelId::Roller) => {
            Some(ColorSelectOption::Paint(world.roller_settings(pos).paint))
        }
        _ => None,
    });
    for (_, children) in &mut slots {
        update_slot_icon(
            children,
            selected.and_then(|opt| opt.icon(block_icons)),
            &mut icons,
        );
    }
}
