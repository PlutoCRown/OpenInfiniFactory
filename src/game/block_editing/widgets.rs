use bevy::prelude::*;

use crate::game::blocks::{BlockKind, MaterialBlockId};
use crate::game::state::UiPanelId;
use crate::game::ui::access::UiMainThread;
use crate::game::ui::components::{
    BUTTON_BG, BUTTON_PRESSED_BG, default_button_size, default_font_size, hover_border,
    inset_border, localized_text, menu_button, raised_border, styled_button, text,
    ui_logical_bounds,
};
use crate::game::ui::types::{CarriedItem, UiActionLabel};
use crate::game::world::direction::Facing;
use crate::game::world::rendering::BlockIconAssets;

use super::OpenBlockPanelDropdown;

/// 材料图标凹槽：面板当前格与悬浮选项共用，靠 Interaction 刷 hover，不挂 HoverButton
#[derive(Component, Clone, Copy)]
pub struct MaterialIconSlot;

/// 凹槽不可点选（如验收器该面不可附着）：悬停不变亮，由业务刷暗色
#[derive(Component, Clone, Copy)]
pub struct MaterialIconSlotBlocked;

pub fn spawn_labeled_panel_button<A>(parent: &mut ChildSpawnerCommands, action: A)
where
    A: Component + Copy + UiActionLabel,
{
    parent
        .spawn((menu_button(36.0), action))
        .with_children(|button| {
            button.spawn(localized_text(action.label_key(), 14.0, Color::WHITE));
        });
}

/// 材料朝向：0/90/180/270 四个选项
const FACING_RADIO_OPTIONS: [(Facing, &'static str); 4] = [
    (Facing::North, "0"),
    (Facing::East, "90"),
    (Facing::South, "180"),
    (Facing::West, "270"),
];

/// 生成朝向 radio 行（有向材料时显示）
pub fn spawn_facing_radio_row<A, T>(
    panel: &mut ChildSpawnerCommands,
    row_tag: T,
    action_for: impl Fn(Facing) -> A,
) where
    A: Component + Copy,
    T: Component + Copy,
{
    panel
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(default_button_size(40.0)),
                display: Display::Flex,
                align_items: AlignItems::Center,
                column_gap: Val::Px(10.0),
                ..default()
            },
            BackgroundColor(Color::NONE),
            row_tag,
        ))
        .with_children(|row| {
            row.spawn((
                localized_text("panel.rotation", 16.0, Color::srgb(0.86, 0.88, 0.86)),
                Node {
                    width: Val::Px(110.0),
                    ..default()
                },
            ));
            for (facing, label) in FACING_RADIO_OPTIONS {
                // 不用 HoverButton：选中态由 sync_facing_radio_buttons 每帧刷
                row.spawn((
                    Button,
                    Node {
                        width: Val::Auto,
                        height: Val::Px(default_button_size(32.0)),
                        flex_shrink: 0.0,
                        border: UiRect {
                            left: Val::Px(3.0),
                            right: Val::Px(3.0),
                            top: Val::Px(4.0),
                            bottom: Val::Px(5.0),
                        },
                        padding: UiRect::horizontal(Val::Px(10.0)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    raised_border(),
                    BackgroundColor(BUTTON_BG),
                    action_for(facing),
                ))
                .with_children(|button| {
                    button.spawn(text(label, 14.0, Color::WHITE));
                });
            }
        });
}

/// 按当前朝向刷新 radio 选中态
pub fn sync_facing_radio_buttons<A: Component>(
    buttons: &mut Query<(&A, &mut BackgroundColor, &mut BorderColor), With<Button>>,
    selected: Facing,
    facing_of: impl Fn(&A) -> Option<Facing>,
) {
    for (action, mut bg, mut border) in buttons.iter_mut() {
        let Some(facing) = facing_of(action) else {
            continue;
        };
        if facing == selected {
            *bg = BUTTON_PRESSED_BG.into();
            *border = inset_border();
        } else {
            *bg = BUTTON_BG.into();
            *border = raised_border();
        }
    }
}

pub fn spawn_text_dropdown_toggle<A, L>(
    parent: &mut ChildSpawnerCommands,
    toggle_action: A,
    label_marker: L,
) where
    A: Component + Copy,
    L: Component + Copy,
{
    parent
        .spawn((
            Node {
                width: Val::Px(230.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                position_type: PositionType::Relative,
                ..default()
            },
            BackgroundColor(Color::NONE),
            ZIndex(300),
        ))
        .with_children(|container| {
            container
                .spawn((
                    styled_button(
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(default_button_size(34.0)),
                            padding: UiRect::horizontal(Val::Px(12.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::SpaceBetween,
                            ..default()
                        },
                        Color::srgb(0.38, 0.39, 0.40),
                        Color::srgba(0.18, 0.20, 0.22, 0.96),
                    ),
                    toggle_action,
                ))
                .with_children(|button| {
                    button.spawn((
                        Text::new(""),
                        TextFont {
                            font_size: default_font_size(14.0),
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        label_marker,
                    ));
                    button.spawn((
                        Text::new("v"),
                        TextFont {
                            font_size: default_font_size(12.0),
                            ..default()
                        },
                        TextColor(Color::srgb(0.72, 0.80, 0.84)),
                    ));
                });
        });
}

pub fn spawn_text_dropdown_list<A, L>(
    parent: &mut ChildSpawnerCommands,
    list_marker: L,
    options: impl IntoIterator<Item = (String, A)>,
) where
    A: Component + Copy,
    L: Component + Copy,
{
    parent
        .spawn((dropdown_list_node(230.0), GlobalZIndex(20_000), list_marker))
        .with_children(|list| {
            for (label, action) in options {
                spawn_text_option(list, label, action);
            }
        });
}

pub fn spawn_material_icon_toggle<A, S>(
    parent: &mut ChildSpawnerCommands,
    slot_marker: S,
    toggle_action: A,
) where
    A: Component + Copy,
    S: Component + Copy,
{
    parent
        .spawn((material_slot_button(), slot_marker, toggle_action))
        .with_children(|slot| {
            slot.spawn(material_icon_node());
        });
}

pub fn spawn_material_icon_list<A, O, L, Id>(
    parent: &mut ChildSpawnerCommands,
    list_marker: L,
    options: impl IntoIterator<Item = (Id, A)>,
    option_marker: fn(Id) -> O,
) where
    A: Component + Copy,
    O: Component + Copy,
    L: Component + Copy,
    Id: Copy,
{
    parent
        .spawn((icon_dropdown_list_node(), GlobalZIndex(20_000), list_marker))
        .with_children(|list| {
            for (material, action) in options {
                list.spawn((material_slot_button(), option_marker(material), action))
                    .with_children(|slot| {
                        slot.spawn(material_icon_node());
                    });
            }
        });
}

pub fn update_material_icon(
    children: &Children,
    material: Option<MaterialBlockId>,
    block_icons: &BlockIconAssets,
    icon_query: &mut Query<&mut ImageNode>,
) {
    let icon = material
        .map(BlockKind::material_block_kind)
        .and_then(|kind| block_icons.get(kind));
    update_slot_icon(children, icon, icon_query);
}

/// 更新格子子节点上的图标贴图
pub fn update_slot_icon(
    children: &Children,
    icon: Option<Handle<Image>>,
    icon_query: &mut Query<&mut ImageNode>,
) {
    let next = icon.map(ImageNode::new).unwrap_or_default();
    for child in children.iter() {
        if let Ok(mut image) = icon_query.get_mut(child) {
            *image = next.clone();
        }
    }
}

/// 材料选择格点击：与背包/物品栏统一手持逻辑，并处理悬浮菜单。
/// - 悬浮菜单开着：无论手上有没有东西都关闭，不写入
/// - 手持材料：写入该材料并清空手持
/// - 手持非材料：取消手里的东西
/// - 空手：打开悬浮菜单
/// 返回 Some 表示要把该材料写入方块设置。
pub fn click_material_slot(
    panel: UiPanelId,
    slot: u8,
    carried: &mut CarriedItem,
    open_dropdown: &mut OpenBlockPanelDropdown,
) -> Option<MaterialBlockId> {
    if open_dropdown.is_open(panel, slot) {
        open_dropdown.close();
        return None;
    }

    let hand_material = carried
        .item()
        .and_then(|item| item.block())
        .and_then(BlockKind::material_id);

    if let Some(id) = hand_material {
        carried.clear();
        return Some(id);
    }

    if carried.item().is_some() {
        carried.clear();
        return None;
    }

    open_dropdown.toggle(panel, slot);
    None
}

pub fn position_dropdown_from_trigger(
    trigger_node: &ComputedNode,
    transform: &UiGlobalTransform,
    list_node: &ComputedNode,
    viewport: Vec2,
) -> Option<(f32, f32)> {
    let trigger = ui_logical_bounds(trigger_node, transform);
    let list_size = list_node.size() * list_node.inverse_scale_factor();
    let below = trigger.max.y + 4.0;
    let above = trigger.min.y - list_size.y - 4.0;
    let top = if below + list_size.y <= viewport.y - 10.0 || above < 10.0 {
        below
    } else {
        above.max(10.0)
    };
    let top = top.clamp(10.0, (viewport.y - list_size.y - 10.0).max(10.0));
    let left = trigger
        .min
        .x
        .clamp(10.0, (viewport.x - list_size.x - 10.0).max(10.0));
    Some((left, top))
}

/// 同步下拉层显隐与锚点；Display 有变化才写，打开时再跟 trigger 定位
pub fn sync_dropdown_overlay(
    open: bool,
    style: &mut Node,
    list_node: &ComputedNode,
    trigger: Option<(&ComputedNode, &UiGlobalTransform)>,
    viewport: Vec2,
) {
    let next = if open { Display::Flex } else { Display::None };
    if style.display != next {
        style.display = next;
    }
    if !open {
        return;
    }
    let Some((trigger_node, transform)) = trigger else {
        return;
    };
    if let Some((left, top)) =
        position_dropdown_from_trigger(trigger_node, transform, list_node, viewport)
    {
        let left = Val::Px(left);
        let top = Val::Px(top);
        if style.left != left {
            style.left = left;
        }
        if style.top != top {
            style.top = top;
        }
    }
}

fn material_slot_button() -> impl Bundle {
    // 与背包物品槽同款凹槽样式；不挂 HoverButton，避免移出后被恢复成凸起按钮边框
    const SLOT_BORDER: f32 = 3.0;
    (
        Button,
        MaterialIconSlot,
        Node {
            width: Val::Px(default_button_size(54.0)),
            height: Val::Px(default_button_size(54.0)),
            border: UiRect::all(Val::Px(SLOT_BORDER)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        inset_border(),
        BackgroundColor(Color::srgb(0.255, 0.251, 0.251)),
        BoxShadow::new(
            Color::srgba(0.0, 0.0, 0.0, 0.50),
            Val::Px(0.0),
            Val::Px(0.0),
            Val::Px(0.0),
            Val::Px(3.0),
        ),
    )
}

fn material_icon_node() -> impl Bundle {
    const ICON_INSET: f32 = 4.0;
    (
        ImageNode::default(),
        Pickable::IGNORE,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(ICON_INSET),
            right: Val::Px(ICON_INSET),
            top: Val::Px(ICON_INSET),
            bottom: Val::Px(ICON_INSET),
            ..default()
        },
    )
}

/// 材料凹槽悬停：亮一点背景 + hover 边框，移出恢复凹槽；Blocked 保持暗色
pub fn update_material_slot_hover(
    _ui_thread: UiMainThread,
    mut slots: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (
            With<MaterialIconSlot>,
            Without<MaterialIconSlotBlocked>,
            Changed<Interaction>,
        ),
    >,
    mut blocked: Query<
        (&mut BackgroundColor, &mut BorderColor),
        (With<MaterialIconSlot>, With<MaterialIconSlotBlocked>),
    >,
) {
    for (interaction, mut background, mut border) in &mut slots {
        let hovered = *interaction == Interaction::Hovered;
        *background = if hovered {
            Color::srgb(0.32, 0.31, 0.31).into()
        } else {
            Color::srgb(0.255, 0.251, 0.251).into()
        };
        *border = if hovered {
            hover_border()
        } else {
            inset_border()
        };
    }
    for (mut background, mut border) in &mut blocked {
        *background = Color::srgb(0.14, 0.14, 0.14).into();
        *border = inset_border();
    }
}

fn dropdown_list_node(width: f32) -> impl Bundle {
    (
        Node {
            width: Val::Px(width),
            display: Display::None,
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(3.0),
            padding: UiRect::all(Val::Px(4.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.10, 0.11, 0.12, 0.98)),
    )
}

fn icon_dropdown_list_node() -> impl Bundle {
    // 一行 5 格：5×槽宽 + 4×间距 + 两侧 padding
    const SLOTS_PER_ROW: f32 = 5.0;
    const SLOT_GAP: f32 = 4.0;
    const LIST_PADDING: f32 = 4.0;
    let slot = default_button_size(54.0);
    let width = SLOTS_PER_ROW * slot + (SLOTS_PER_ROW - 1.0) * SLOT_GAP + LIST_PADDING * 2.0;
    (
        Node {
            width: Val::Px(width),
            display: Display::None,
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            row_gap: Val::Px(SLOT_GAP),
            column_gap: Val::Px(SLOT_GAP),
            padding: UiRect::all(Val::Px(LIST_PADDING)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.10, 0.11, 0.12, 0.98)),
    )
}

fn spawn_text_option<A>(parent: &mut ChildSpawnerCommands, label: String, action: A)
where
    A: Component + Copy,
{
    parent
        .spawn((menu_button(32.0), action))
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
